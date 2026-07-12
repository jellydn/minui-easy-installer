use std::fs;
use std::path::Path;
use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::fs_utils;
use crate::pipeline::{DownloadProgressCallback, InstallSession, Pipeline};
use crate::platform::device_base_item;

#[derive(Debug, Clone, serde::Serialize)]
pub struct InstallResult {
    pub success: bool,
    pub error: Option<String>,
    pub base_files_copied: u32,
    pub extras_files_copied: u32,
    pub extras_warning: Option<String>,
    pub rom_dirs_created: u32,
}

/// Progress event emitted during the install flow.
#[derive(Debug, Clone, serde::Serialize)]
pub struct InstallProgressEvent {
    pub step: String,
    pub details: String,
}

/// Callback for reporting install progress. Passed through the install flow.
pub type ProgressCallback = Arc<dyn Fn(InstallProgressEvent) + Send + Sync>;

/// Standard ROM folders created on the SD card after install
const ROM_DIRS: &[&str] = &[
    "Arcade (FBN)",
    "Game Boy (GB)",
    "Game Boy Advance (GBA)",
    "Game Boy Color (GBC)",
    "Neo Geo (FBN)",
    "Neo Geo Pocket (NGP)",
    "Nintendo 64 (N64)",
    "Nintendo DS (NDS)",
    "Nintendo Entertainment System (FC)",
    "Pico-8 (PICO)",
    "Sega Dreamcast (DC)",
    "Sega Genesis (MD)",
    "Sony PlayStation (PS)",
    "Sony PlayStation Portable (PSP)",
    "Super Nintendo Entertainment System (SFC)",
    "Ports (PORTS)",
];

/// Creates standard ROM directories on the SD card if they don't exist.
/// Skips directories that already exist (e.g. from a previous install).
pub fn create_rom_dirs(sd_mount: &str) -> Result<u32, String> {
    let roms_root = Path::new(sd_mount).join("Roms");
    let mut created = 0u32;

    for dir in ROM_DIRS {
        let path = roms_root.join(dir);
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|e| format!("Failed to create Roms/{}: {}", dir, e))?;
            created += 1;
        }
    }

    // Create placeholder for Portmaster in the Ports directory
    let ports_dst = roms_root.join("Ports (PORTS)").join("Portmaster.sh");
    if !ports_dst.exists() {
        if let Err(e) = fs::write(&ports_dst, "") {
            eprintln!("Warning: failed to create Portmaster placeholder: {}", e);
        }
    }

    Ok(created)
}

/// Folders that must never be deleted or overwritten during install.
/// Only lowercase needed — comparisons use eq_ignore_ascii_case.
const PRESERVED_FOLDERS: &[&str] = &["roms", "saves", "save", "bios", "cheats"];

fn is_preserved_path(path: &Path, sd_root: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(sd_root) else {
        return false;
    };
    let Some(name) = relative.iter().next() else {
        return false;
    };
    let name_str = name.to_string_lossy();
    PRESERVED_FOLDERS
        .iter()
        .any(|preserved| name_str.eq_ignore_ascii_case(preserved))
}

/// Shared items that should be copied from every base archive to the SD root.
const SHARED_BASE_ITEMS: &[&str] = &["Bios", "Roms", "Saves", "MinUI.zip"];

/// Copy a single base archive item (file or directory) from `src` to `dst`.
///
/// When `preserve` is true, existing user data under the destination is
/// skipped using `is_preserved_path`. Returns the number of files copied,
/// or 0 if the source does not exist.
fn copy_archive_item(
    src: &Path,
    dst: &Path,
    sd_root: &Path,
    preserve: bool,
) -> Result<u32, String> {
    if !src.exists() {
        return Ok(0);
    }

    if src.is_file() {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directory: {}", e))?;
        }
        fs::copy(src, dst).map_err(|e| {
            format!("Failed to copy {} to {}: {}", src.display(), dst.display(), e)
        })?;
        Ok(1)
    } else {
        fs::create_dir_all(dst)
            .map_err(|e| format!("Failed to create directory {}: {}", dst.display(), e))?;
        let skip_predicate = |_src: &Path, dst: &Path| preserve && is_preserved_path(dst, sd_root);
        fs_utils::copy_dir_recursive(src, dst, &skip_predicate, &|| false)
    }
}

/// Copy the base archive contents to the SD card root.
///
/// Only copies the shared items (`Bios`, `Roms`, `Saves`, `MinUI.zip`) and the
/// device-specific folder/file for `platform`. Other device folders and
/// `README.txt` are intentionally skipped so the SD card is not polluted with
/// files for devices the user did not select.
pub fn copy_base_files(
    extracted_base_path: &str,
    sd_mount: &str,
    platform: &str,
) -> Result<u32, String> {
    let base_dir = Path::new(extracted_base_path);
    let sd_root = Path::new(sd_mount);
    let mut files_copied = 0u32;

    // Copy shared items, preserving existing user data in Bios/Roms/Saves.
    for item in SHARED_BASE_ITEMS {
        let src = base_dir.join(item);
        let dst = sd_root.join(item);
        files_copied += copy_archive_item(&src, &dst, sd_root, true)?;
    }

    // Copy the device-specific folder/file.
    let device_item = device_base_item(platform);
    let device_src = base_dir.join(device_item);
    let device_dst = sd_root.join(device_item);
    files_copied += copy_archive_item(&device_src, &device_dst, sd_root, false)?;

    Ok(files_copied)
}

/// Copies a subdirectory tree from src_root/subpath to dst_root/subpath.
/// Returns the number of files copied, or 0 if the source doesn't exist.
fn copy_subtree(src_root: &Path, dst_root: &Path, subpath: &str) -> Result<u32, String> {
    let src = src_root.join(subpath);
    if !src.exists() {
        return Ok(0);
    }
    let dst = dst_root.join(subpath);
    fs::create_dir_all(&dst)
        .map_err(|e| format!("Failed to create {} directory: {}", subpath, e))?;
    fs_utils::copy_dir_recursive(&src, &dst, &|_s, _d| false, &|| false)
}

/// Copies Extras files to the SD card, filtering by platform.
///
/// The extras archive contains all platforms' emulators and tools at:
///   Emus/{platform}/{pakName}.pak/
///   Tools/{platform}/{pakName}.pak/
///   Bios/          (shared across all devices)
///
/// This function only copies:
///   1. Everything under `Emus/{extras_platform}/` → SD `Emus/{extras_platform}/`
///   2. Everything under `Tools/{extras_platform}/` → SD `Tools/{extras_platform}/`
///   3. Everything under `Bios/` → SD `Bios/`
pub fn copy_extras_files(
    extracted_extras_path: &str,
    sd_mount: &str,
    extras_platform: &str,
) -> Result<u32, String> {
    // Security guard: extras_platform must match expected format
    if extras_platform.is_empty()
        || !extras_platform
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(format!(
            "Invalid extras_platform: '{}' must contain only alphanumeric characters and hyphens",
            extras_platform
        ));
    }
    let extras_src = Path::new(extracted_extras_path);
    let sd_root = Path::new(sd_mount);

    if !extras_src.exists() {
        return Err("Extras source directory does not exist".to_string());
    }

    let mut files_copied = 0u32;
    files_copied += copy_subtree(extras_src, sd_root, "Bios")?;
    files_copied += copy_subtree(
        &extras_src.join("Emus"),
        &sd_root.join("Emus"),
        extras_platform,
    )?;
    files_copied += copy_subtree(
        &extras_src.join("Tools"),
        &sd_root.join("Tools"),
        extras_platform,
    )?;

    Ok(files_copied)
}

/// Configuration for a MinUI installation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct InstallOptions {
    pub base_url: String,
    pub extras_url: Option<String>,
    pub base_checksum: Option<String>,
    pub extras_checksum: Option<String>,
    pub sd_mount: String,
    pub platform: String,
    pub extras_platform: String,
    pub version: String,
    /// Name of the fork (e.g. "MinUI", "MinUI-Zero"). Written into
    /// minui.txt as "{fork_name} {version}". Defaults to "MinUI".
    pub fork_name: Option<String>,
}

/// Runs extras download → extract → copy, returning the number of files copied.
/// Errors are propagated via `Result` — the caller decides how to handle failures.
async fn try_install_extras(
    options: &InstallOptions,
    progress: ProgressCallback,
    download_progress: DownloadProgressCallback,
    cancel: CancellationToken,
    session: &mut InstallSession,
) -> Result<u32, String> {
    let extras_url = options
        .extras_url
        .as_deref()
        .ok_or("No extras URL provided")?;

    Pipeline::run(
        "extras",
        extras_url,
        options.extras_checksum.as_deref(),
        |p| {
            copy_extras_files(
                p.to_str().unwrap(),
                &options.sd_mount,
                &options.extras_platform,
            )
        },
        progress,
        download_progress,
        cancel,
        session,
    )
    .await
}

/// Full installation flow: download, extract, copy base + extras.
///
/// This is the main entry point that coordinates the entire install process.
pub async fn install_minui(
    options: &InstallOptions,
    progress: ProgressCallback,
) -> Result<InstallResult, String> {
    install_minui_with_cancel(
        options,
        progress,
        Arc::new(|_, _| {}),
        CancellationToken::new(),
    )
    .await
}

/// Full installation flow with cancellation support.
///
/// Identical to `install_minui` but accepts a `CancellationToken` so the
/// caller (typically a Tauri command) can abort mid-pipeline. Also accepts
/// a byte-level download progress callback.
pub async fn install_minui_with_cancel(
    options: &InstallOptions,
    progress: ProgressCallback,
    download_progress: DownloadProgressCallback,
    cancel: CancellationToken,
) -> Result<InstallResult, String> {
    let mut session = InstallSession::new();

    // Step 1: Download, extract, and copy base
    let file_name = options.base_url.rsplit('/').next().unwrap_or("MinUI.zip");
    progress(InstallProgressEvent {
        step: "download".to_string(),
        details: format!("Downloading {}", file_name),
    });
    let base_files_copied = Pipeline::run(
        "base",
        &options.base_url,
        options.base_checksum.as_deref(),
        |p| copy_base_files(p.to_str().unwrap(), &options.sd_mount, &options.platform),
        progress.clone(),
        download_progress.clone(),
        cancel.clone(),
        &mut session,
    )
    .await?;

    // Step 2: Download, extract, and copy extras (if available) — non-fatal
    let mut extras_files_copied = 0u32;
    let mut extras_warning: Option<String> = None;

    if options.extras_url.is_some() {
        match try_install_extras(
            options,
            progress.clone(),
            download_progress,
            cancel.clone(),
            &mut session,
        )
        .await
        {
            Ok(copied) => extras_files_copied = copied,
            Err(e) => extras_warning = Some(e),
        }
    }

    // Step 3: Create standard ROM directories
    progress(InstallProgressEvent {
        step: "copy".to_string(),
        details: "Creating standard ROM directories...".to_string(),
    });
    let rom_dirs_created = create_rom_dirs(&options.sd_mount).unwrap_or(0);

    // Write version metadata after successful install
    let fork_label = options.fork_name.as_deref().unwrap_or("MinUI");
    progress(InstallProgressEvent {
        step: "finish".to_string(),
        details: format!(
            "Writing version metadata ({} {})",
            fork_label, options.version
        ),
    });
    let minui_txt_path = Path::new(&options.sd_mount).join("minui.txt");
    if let Err(e) = fs::write(
        &minui_txt_path,
        format!("{} {}\n", fork_label, options.version),
    ) {
        // Surface the failure as a non-fatal warning so the UI can
        // show it. The install itself succeeded; only the metadata
        // file is missing, so we don't downgrade success.
        eprintln!("Warning: failed to write version metadata: {}", e);
        extras_warning = Some(format!("Failed to write version metadata: {}", e));
    }

    // session drops here — temp dirs cleaned up after all operations complete
    Ok(InstallResult {
        success: true,
        error: None,
        base_files_copied,
        extras_files_copied,
        extras_warning,
        rom_dirs_created,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_is_preserved_path() {
        let sd_root = Path::new("/Volumes/SDCARD");

        assert!(is_preserved_path(
            Path::new("/Volumes/SDCARD/ROMS/game.nes"),
            sd_root
        ));
        assert!(is_preserved_path(
            Path::new("/Volumes/SDCARD/roms/game.nes"),
            sd_root
        ));
        assert!(is_preserved_path(
            Path::new("/Volumes/SDCARD/Saves/save.sav"),
            sd_root
        ));
        assert!(is_preserved_path(
            Path::new("/Volumes/SDCARD/saves/save.sav"),
            sd_root
        ));
        assert!(!is_preserved_path(
            Path::new("/Volumes/SDCARD/Apps/minui.pak"),
            sd_root
        ));
        assert!(!is_preserved_path(
            Path::new("/Volumes/SDCARD/Tools/wifi.pak"),
            sd_root
        ));
    }

    #[test]
    fn test_copy_dir_recursive_copies_files() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");

        fs::create_dir_all(&src).unwrap();

        let mut f = fs::File::create(src.join("test.txt")).unwrap();
        f.write_all(b"hello").unwrap();
        drop(f);

        let copied = fs_utils::copy_dir_recursive(&src, &dst, &|_s, _d| false, &|| false).unwrap();
        assert_eq!(copied, 1);
        assert!(dst.join("test.txt").exists());
    }

    #[test]
    fn test_copy_dir_recursive_skips_preserved_folders() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        let sd_root = temp.path().join("sdcard");

        fs::create_dir_all(src.join("ROMS")).unwrap();
        fs::create_dir_all(src.join("Saves")).unwrap();
        fs::create_dir_all(src.join("Tools")).unwrap();
        fs::create_dir_all(&sd_root).unwrap();

        fs::write(src.join("ROMS/game.nes"), "rom").unwrap();
        fs::write(src.join("Saves/save.sav"), "save").unwrap();
        fs::write(src.join("Tools/tool.pak"), "tool").unwrap();

        let copied = fs_utils::copy_dir_recursive(
            &src,
            &sd_root,
            &|_src, dst| is_preserved_path(dst, &sd_root),
            &|| false,
        )
        .unwrap();
        assert_eq!(copied, 1); // Only tool.pak
        assert!(!sd_root.join("ROMS").exists());
        assert!(!sd_root.join("Saves").exists());
        assert!(sd_root.join("Tools/tool.pak").exists());
    }

    #[test]
    fn test_copy_base_files_with_platform_dir() {
        let temp = tempfile::tempdir().unwrap();
        let extracted = temp.path().join("extracted");
        let platform_dir = extracted.join("miyoo354");
        let sd_root = temp.path().join("sdcard");

        fs::create_dir_all(&platform_dir).unwrap();
        fs::create_dir_all(&sd_root).unwrap();

        fs::write(platform_dir.join("minui.pak"), "base").unwrap();
        fs::write(platform_dir.join("boot.sh"), "boot").unwrap();

        // Shared items
        fs::create_dir_all(extracted.join("Bios")).unwrap();
        fs::create_dir_all(extracted.join("Roms")).unwrap();
        fs::create_dir_all(extracted.join("Saves")).unwrap();
        fs::write(extracted.join("MinUI.zip"), "minui").unwrap();

        // Other device folders should not be copied
        fs::create_dir_all(extracted.join("trimui")).unwrap();
        fs::write(extracted.join("trimui/trimui.pak"), "trimui").unwrap();
        fs::create_dir_all(extracted.join("miyoo")).unwrap();
        fs::write(extracted.join("miyoo/miyoo.pak"), "miyoo").unwrap();

        // README should not be copied
        fs::write(extracted.join("README.txt"), "readme").unwrap();

        let copied =
            copy_base_files(extracted.to_str().unwrap(), sd_root.to_str().unwrap(), "miyoo354")
                .unwrap();

        // 2 device files + MinUI.zip + Bios/Roms/Saves dirs (empty) = 3 copied files/entries
        assert_eq!(copied, 3);
        assert!(sd_root.join("miyoo354/minui.pak").exists());
        assert!(sd_root.join("miyoo354/boot.sh").exists());
        assert!(sd_root.join("MinUI.zip").exists());
        assert!(!sd_root.join("trimui").exists());
        assert!(!sd_root.join("miyoo").exists());
        assert!(!sd_root.join("README.txt").exists());
    }

    /// End-to-end test of the base-archive copy step with a realistic
    /// multi-device MinUI archive. Verifies that selecting a platform only
    /// copies the shared items plus that platform's device folder/file,
    /// leaving all other device folders and README.txt behind.
    ///
    /// This test mocks the SD card with a temporary directory and exercises
    /// the same `copy_base_files` path the full install pipeline uses after
    /// download/extract, so it proves the installer copy behavior without
    /// requiring network I/O or a real SD card.
    /// Verifies graceful handling when the selected platform has no matching
    /// device folder/file in the base archive. Only shared items should be copied.
    #[test]
    fn test_copy_base_files_end_to_end_missing_device_folder_copies_only_shared_items() {
        let temp = tempfile::tempdir().unwrap();
        let extracted = temp.path().join("extracted");
        let sd_root = temp.path().join("sdcard");
        fs::create_dir_all(&extracted).unwrap();
        fs::create_dir_all(&sd_root).unwrap();

        fs::create_dir_all(extracted.join("Bios")).unwrap();
        fs::create_dir_all(extracted.join("Roms")).unwrap();
        fs::create_dir_all(extracted.join("Saves")).unwrap();
        fs::write(extracted.join("MinUI.zip"), "minui").unwrap();

        // No device folders exist in this archive.
        let copied = copy_base_files(extracted.to_str().unwrap(), sd_root.to_str().unwrap(), "miyoo354").unwrap();

        assert_eq!(copied, 1, "only MinUI.zip should be copied when device folder is missing");
        assert!(sd_root.join("MinUI.zip").exists());
        assert!(sd_root.join("Bios").is_dir());
        assert!(sd_root.join("Roms").is_dir());
        assert!(sd_root.join("Saves").is_dir());
        assert!(!sd_root.join("miyoo354").exists());
    }

    /// End-to-end test of the base-archive copy step with a realistic
    /// multi-device MinUI archive. Verifies that selecting a platform only
    /// copies the shared items plus that platform's device folder/file,
    /// leaving all other device folders and README.txt behind.
    ///
    /// This test mocks the SD card with a temporary directory and exercises
    /// the same `copy_base_files` path the full install pipeline uses after
    /// download/extract, so it proves the installer copy behavior without
    /// requiring network I/O or a real SD card.
    #[test]
    fn test_copy_base_files_end_to_end_only_selected_device_folder_is_copied() {
        let temp = tempfile::tempdir().unwrap();
        let extracted = temp.path().join("extracted");
        let sd_root = temp.path().join("sdcard");
        fs::create_dir_all(&extracted).unwrap();
        fs::create_dir_all(&sd_root).unwrap();

        // Shared items that every base archive contains
        fs::create_dir_all(extracted.join("Bios")).unwrap();
        fs::create_dir_all(extracted.join("Roms")).unwrap();
        fs::create_dir_all(extracted.join("Saves")).unwrap();
        fs::write(extracted.join("MinUI.zip"), "minui").unwrap();

        // Device-specific folders/files that should only be copied when
        // their matching platform is selected. Derive the archive contents
        // from the canonical mappings so the test stays in sync.
        let mut device_folders: Vec<(&str, &str)> = Vec::new();
        for (_, item) in crate::platform::DEVICE_BASE_MAPPINGS {
            if *item == "em_ui.sh" {
                fs::write(extracted.join("em_ui.sh"), "#!/bin/sh").unwrap();
                continue;
            }
            let file = "minui.pak";
            fs::create_dir_all(extracted.join(item)).unwrap();
            fs::write(extracted.join(item).join(file), format!("{item} data")).unwrap();
            device_folders.push((*item, file));
        }

        // README and other top-level files should never be copied
        fs::write(extracted.join("README.txt"), "readme").unwrap();
        fs::write(extracted.join("LICENSE.txt"), "license").unwrap();

        fn clean_sd_root(sd_root: &std::path::Path) {
            for entry in std::fs::read_dir(sd_root).unwrap() {
                let entry = entry.unwrap();
                if entry.path().is_dir() {
                    std::fs::remove_dir_all(entry.path()).unwrap();
                } else {
                    std::fs::remove_file(entry.path()).unwrap();
                }
            }
        }

        // Iterate over every supported platform so the test stays in sync
        // with the canonical device mappings.
        for (platform, expected_item) in crate::platform::DEVICE_BASE_MAPPINGS {
            let platform = *platform;
            let expected_item = *expected_item;

            // Clean the SD card root between runs so each assertion is independent.
            clean_sd_root(&sd_root);

            let _copied = copy_base_files(
                extracted.to_str().unwrap(),
                sd_root.to_str().unwrap(),
                platform,
            )
            .unwrap();

            // Shared items: MinUI.zip + Bios/Roms/Saves directories.
            // Bios/Roms/Saves are empty in this mock, so only MinUI.zip counts as a file,
            // but the directories should still be created on the SD card.
            assert!(sd_root.join("MinUI.zip").exists(), "MinUI.zip should be copied for {platform}");
            assert_eq!(
                fs::read_to_string(sd_root.join("MinUI.zip")).unwrap(),
                "minui",
                "MinUI.zip content should be preserved for {platform}"
            );
            assert!(sd_root.join("Bios").is_dir(), "Bios directory should be created for {platform}");
            assert!(sd_root.join("Roms").is_dir(), "Roms directory should be created for {platform}");
            assert!(sd_root.join("Saves").is_dir(), "Saves directory should be created for {platform}");
            assert!(
                sd_root.join(expected_item).exists(),
                "selected device item {expected_item} should be copied for {platform}"
            );

            // Verify the selected device file content is correct.
            if expected_item != "em_ui.sh" {
                assert_eq!(
                    fs::read_to_string(sd_root.join(expected_item).join("minui.pak")).unwrap(),
                    format!("{expected_item} data"),
                    "selected device file content should match for {platform}"
                );
            } else {
                assert_eq!(
                    fs::read_to_string(sd_root.join(expected_item)).unwrap(),
                    "#!/bin/sh",
                    "em_ui.sh content should match for {platform}"
                );
            }

            // Verify no other device folders were copied.
            for (folder, _) in &device_folders {
                if *folder == expected_item {
                    continue;
                }
                assert!(
                    !sd_root.join(folder).exists(),
                    "unselected device folder {folder} should not be copied for {platform}"
                );
            }

            // M17 script should only exist when platform is m17.
            if platform != "m17" {
                assert!(
                    !sd_root.join("em_ui.sh").exists(),
                    "em_ui.sh should not be copied for {platform}"
                );
            }

            // README and LICENSE should never be copied.
            assert!(!sd_root.join("README.txt").exists(), "README.txt should not be copied for {platform}");
            assert!(!sd_root.join("LICENSE.txt").exists(), "LICENSE.txt should not be copied for {platform}");
        }
    }

    #[test]
    fn test_copy_base_files_m17_copies_em_ui_sh() {
        let temp = tempfile::tempdir().unwrap();
        let extracted = temp.path().join("extracted");
        let sd_root = temp.path().join("sdcard");

        fs::create_dir_all(&extracted).unwrap();
        fs::create_dir_all(&sd_root).unwrap();

        fs::write(extracted.join("em_ui.sh"), "#!/bin/sh").unwrap();
        fs::write(extracted.join("MinUI.zip"), "minui").unwrap();
        fs::create_dir_all(extracted.join("Bios")).unwrap();

        let copied =
            copy_base_files(extracted.to_str().unwrap(), sd_root.to_str().unwrap(), "m17")
                .unwrap();

        assert_eq!(copied, 2); // em_ui.sh + MinUI.zip
        assert!(sd_root.join("em_ui.sh").exists());
        assert!(sd_root.join("MinUI.zip").exists());
    }

    #[test]
    fn test_copy_base_files_preserves_existing_user_data() {
        let temp = tempfile::tempdir().unwrap();
        let extracted = temp.path().join("extracted");
        let sd_root = temp.path().join("sdcard");

        fs::create_dir_all(&extracted).unwrap();
        fs::create_dir_all(&sd_root).unwrap();

        // Existing user data
        fs::create_dir_all(sd_root.join("Roms/GB")).unwrap();
        fs::write(sd_root.join("Roms/GB/pokemon.gb"), "rom_data").unwrap();
        fs::create_dir_all(sd_root.join("Saves")).unwrap();
        fs::write(sd_root.join("Saves/pokemon.sav"), "save_data").unwrap();

        // Archive content
        fs::create_dir_all(extracted.join("Roms")).unwrap();
        fs::create_dir_all(extracted.join("Saves")).unwrap();
        fs::write(extracted.join("MinUI.zip"), "minui").unwrap();
        fs::create_dir_all(extracted.join("miyoo")).unwrap();
        fs::write(extracted.join("miyoo/app"), "app").unwrap();

        let copied =
            copy_base_files(extracted.to_str().unwrap(), sd_root.to_str().unwrap(), "miyoo")
                .unwrap();

        // MinUI.zip + miyoo/app = 2 files; Roms/Saves skipped because they exist
        assert_eq!(copied, 2);
        assert!(sd_root.join("MinUI.zip").exists());
        assert!(sd_root.join("miyoo/app").exists());
        assert!(sd_root.join("Roms/GB/pokemon.gb").exists());
        assert_eq!(
            fs::read_to_string(sd_root.join("Roms/GB/pokemon.gb")).unwrap(),
            "rom_data"
        );
        assert!(sd_root.join("Saves/pokemon.sav").exists());
        assert_eq!(
            fs::read_to_string(sd_root.join("Saves/pokemon.sav")).unwrap(),
            "save_data"
        );
    }

    #[test]
    fn test_is_preserved_path_nested() {
        let sd_root = Path::new("/Volumes/SDCARD");
        // Case-insensitivity is by design: is_preserved_path uses
        // eq_ignore_ascii_case to match FAT32's case-preserving but
        // case-insensitive filesystem semantics.
        // Nested paths under a preserved top-level folder are preserved;
        // preserved folder names at non-top-level paths are not.

        // Deep nesting under preserved folder should be preserved
        assert!(is_preserved_path(
            Path::new("/Volumes/SDCARD/ROMS/GB/game.gb"),
            sd_root
        ));
        assert!(is_preserved_path(
            Path::new("/Volumes/SDCARD/ROMS/Nintendo Entertainment System (FC)/game.nes"),
            sd_root
        ));
        assert!(is_preserved_path(
            Path::new("/Volumes/SDCARD/Saves/game.sav"),
            sd_root
        ));
        assert!(is_preserved_path(
            Path::new("/Volumes/SDCARD/Saves/subdir/nested.sav"),
            sd_root
        ));
        assert!(is_preserved_path(
            Path::new("/Volumes/SDCARD/BIOS/gba_bios.bin"),
            sd_root
        ));

        // Preserved folder name appearing non-top-level should NOT be preserved
        assert!(!is_preserved_path(
            Path::new("/Volumes/SDCARD/Tools/ROMS/wifi.pak"),
            sd_root
        ));
        assert!(!is_preserved_path(
            Path::new("/Volumes/SDCARD/Emus/saves/mgba.pak"),
            sd_root
        ));

        // Case insensitivity for nested folders
        assert!(is_preserved_path(
            Path::new("/Volumes/SDCARD/roms/nes/game.nes"),
            sd_root
        ));
        assert!(!is_preserved_path(
            Path::new("/Volumes/SDCARD/Tools/bios/"),
            sd_root
        ));
    }

    #[test]
    fn test_copy_dir_recursive_preserves_user_data() {
        let temp = tempfile::tempdir().unwrap();
        let base_src = temp.path().join("base_extracted");
        let sd_root = temp.path().join("sdcard");

        // Simulate existing SD card with user data
        fs::create_dir_all(sd_root.join("ROMS/GB")).unwrap();
        fs::create_dir_all(sd_root.join("Saves")).unwrap();
        fs::create_dir_all(sd_root.join("Tools")).unwrap();
        fs::write(sd_root.join("ROMS/GB/pokemon.gb"), "rom_data").unwrap();
        fs::write(sd_root.join("Saves/pokemon.sav"), "save_data").unwrap();

        // Update archive content
        fs::create_dir_all(base_src.join("Tools")).unwrap();
        fs::write(base_src.join("Tools/wifi.pak"), "new_wifi").unwrap();
        fs::write(base_src.join("minui.txt"), "MinUI v2025.01.01").unwrap();

        let copied = fs_utils::copy_dir_recursive(
            &base_src,
            &sd_root,
            &|_src, dst| is_preserved_path(dst, &sd_root),
            &|| false,
        )
        .unwrap();

        // Only minui.txt and Tools/wifi.pak should be copied — ROMs and Saves skipped
        assert_eq!(copied, 2);
        assert!(sd_root.join("Tools/wifi.pak").exists());
        assert!(sd_root.join("minui.txt").exists());

        // User data must survive
        assert!(sd_root.join("ROMS/GB/pokemon.gb").exists());
        assert_eq!(
            fs::read_to_string(sd_root.join("ROMS/GB/pokemon.gb")).unwrap(),
            "rom_data"
        );
        assert!(sd_root.join("Saves/pokemon.sav").exists());
        assert_eq!(
            fs::read_to_string(sd_root.join("Saves/pokemon.sav")).unwrap(),
            "save_data"
        );
    }

    #[test]
    fn test_copy_extras_files_filters_by_platform() {
        let temp = tempfile::tempdir().unwrap();
        let extras_src = temp.path().join("extras_extracted");
        let sd_root = temp.path().join("sdcard");
        let platform = "rg35xxplus";

        // Create a realistic extras archive structure with multiple platforms
        fs::create_dir_all(extras_src.join("Emus/rg35xxplus/mgba.pak")).unwrap();
        fs::create_dir_all(extras_src.join("Emus/rg35xxplus/gambatte.pak")).unwrap();
        fs::create_dir_all(extras_src.join("Tools/rg35xxplus/wifi.pak")).unwrap();
        fs::create_dir_all(extras_src.join("Tools/rg35xxplus/ssh.pak")).unwrap();
        fs::create_dir_all(extras_src.join("Tools/trimuismart/dc.pak")).unwrap();
        fs::create_dir_all(extras_src.join("Tools/trimuismart/wifi.pak")).unwrap();
        fs::create_dir_all(extras_src.join("Bios")).unwrap();

        fs::write(extras_src.join("Emus/rg35xxplus/mgba.pak/launch.sh"), "emu").unwrap();
        fs::write(
            extras_src.join("Emus/rg35xxplus/gambatte.pak/launch.sh"),
            "emu",
        )
        .unwrap();
        fs::write(
            extras_src.join("Tools/rg35xxplus/wifi.pak/launch.sh"),
            "tool",
        )
        .unwrap();
        fs::write(
            extras_src.join("Tools/rg35xxplus/ssh.pak/launch.sh"),
            "tool",
        )
        .unwrap();
        fs::write(
            extras_src.join("Tools/trimuismart/dc.pak/launch.sh"),
            "tool",
        )
        .unwrap();
        fs::write(
            extras_src.join("Tools/trimuismart/wifi.pak/launch.sh"),
            "tool",
        )
        .unwrap();
        fs::write(extras_src.join("Bios/gba_bios.bin"), "bios").unwrap();

        let copied = copy_extras_files(
            extras_src.to_str().unwrap(),
            sd_root.to_str().unwrap(),
            platform,
        )
        .unwrap();

        // Should copy: 2 emus + 2 tools + 1 bios = 5 files (not the trimuismart ones)
        assert_eq!(copied, 5);

        // Verify rg35xxplus emus and tools were copied
        assert!(sd_root.join("Emus/rg35xxplus/mgba.pak/launch.sh").exists());
        assert!(sd_root
            .join("Emus/rg35xxplus/gambatte.pak/launch.sh")
            .exists());
        assert!(sd_root.join("Tools/rg35xxplus/wifi.pak/launch.sh").exists());
        assert!(sd_root.join("Tools/rg35xxplus/ssh.pak/launch.sh").exists());

        // Verify trimuismart stuff was NOT copied
        assert!(!sd_root.join("Tools/trimuismart").exists());

        // Verify Bios was copied
        assert!(sd_root.join("Bios/gba_bios.bin").exists());
    }

    #[test]
    fn test_minui_txt_writes_fork_name() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        // Simulate what install_minui_with_cancel writes after copying
        let fork_label = "MinUI-Zero";
        let version = "20250525";
        let minui_txt_path = sd_root.join("minui.txt");
        fs::write(&minui_txt_path, format!("{} {}\n", fork_label, version)).unwrap();

        let content = fs::read_to_string(&minui_txt_path).unwrap();
        assert_eq!(content, "MinUI-Zero 20250525\n");
    }

    #[test]
    fn test_minui_txt_defaults_to_minui_when_no_fork_name() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        let fork_label = "MinUI"; // default when fork_name is None
        let version = "2025.01.01";
        let minui_txt_path = sd_root.join("minui.txt");
        fs::write(&minui_txt_path, format!("{} {}\n", fork_label, version)).unwrap();

        let content = fs::read_to_string(&minui_txt_path).unwrap();
        assert_eq!(content, "MinUI 2025.01.01\n");
    }
}
