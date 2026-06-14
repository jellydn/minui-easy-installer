use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::download;
use crate::extract;
use crate::fs_utils;

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
    if let Ok(relative) = path.strip_prefix(sd_root) {
        let first_component = relative.iter().next();
        if let Some(name) = first_component {
            let name_str = name.to_string_lossy();
            for preserved in PRESERVED_FOLDERS {
                if name_str.eq_ignore_ascii_case(preserved) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn copy_base_files(
    extracted_base_path: &str,
    sd_mount: &str,
    _platform: &str,
) -> Result<u32, String> {
    let base_dir = Path::new(extracted_base_path);
    let sd_root = Path::new(sd_mount);
    fs_utils::copy_dir_recursive(base_dir, sd_root, &|_src, dst| {
        is_preserved_path(dst, sd_root)
    })
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
    fs_utils::copy_dir_recursive(&src, &dst, &|_s, _d| false)
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
    files_copied += copy_subtree(&extras_src.join("Emus"), &sd_root.join("Emus"), extras_platform)?;
    files_copied += copy_subtree(&extras_src.join("Tools"), &sd_root.join("Tools"), extras_platform)?;

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
}

/// Runs extras download → extract → copy, returning the number of files copied.
/// Errors are propagated via `Result` — the caller decides how to handle failures.
async fn try_install_extras(
    options: &InstallOptions,
    progress: ProgressCallback,
) -> Result<u32, String> {
    let extras_url = options
        .extras_url
        .as_deref()
        .ok_or("No extras URL provided")?;

    progress(InstallProgressEvent {
        step: "download".to_string(),
        details: "Downloading extras archive...".to_string(),
    });
    let (result, _temp) = download::download_archive(extras_url, options.extras_checksum.as_deref())
        .await
        .map_err(|e| format!("Extras download failed: {}", e))?;
    let path = result.file_path.ok_or("No extras download path")?;

    progress(InstallProgressEvent {
        step: "extract".to_string(),
        details: "Extracting extras archive...".to_string(),
    });
    let (extraction, _temp) = extract::extract_archive(&path, None)
        .map_err(|e| format!("Extras extraction failed: {}", e))?;
    let extracted = extraction.output_path.ok_or("No extras extraction path")?;

    progress(InstallProgressEvent {
        step: "copy".to_string(),
        details: format!(
            "Copying device extras to /Emus/{}/ and /Tools/{}/...",
            options.extras_platform, options.extras_platform
        ),
    });
    copy_extras_files(&extracted, &options.sd_mount, &options.extras_platform)
        .map_err(|e| format!("Extras copy failed: {}", e))
}

/// Full installation flow: download, extract, copy base + extras.
///
/// This is the main entry point that coordinates the entire install process.
pub async fn install_minui(
    options: &InstallOptions,
    progress: ProgressCallback,
) -> Result<InstallResult, String> {
    // Step 1: Download base archive
    let file_name = options.base_url.rsplit('/').next().unwrap_or("MinUI.zip");
    progress(InstallProgressEvent {
        step: "download".to_string(),
        details: format!("Downloading {}", file_name),
    });
    let (base_result, _base_temp) =
        download::download_archive(&options.base_url, options.base_checksum.as_deref())
            .await
            .map_err(|e| format!("Base download failed: {}", e))?;

    if !base_result.success {
        return Ok(InstallResult {
            success: false,
            error: Some(base_result.error.unwrap_or("Base download failed".to_string())),
            base_files_copied: 0,
            extras_files_copied: 0,
            extras_warning: None,
            rom_dirs_created: 0,
        });
    }

    let base_path = base_result.file_path.ok_or("No base file path returned")?;

    // Step 2: Extract base archive
    progress(InstallProgressEvent {
        step: "extract".to_string(),
        details: "Extracting MinUI base archive...".to_string(),
    });
    let (base_extraction, _base_extract_temp) =
        extract::extract_archive(&base_path, None).map_err(|e| format!("Base extraction failed: {}", e))?;

    if !base_extraction.success {
        return Ok(InstallResult {
            success: false,
            error: Some(
                base_extraction
                    .error
                    .unwrap_or("Base extraction failed".to_string()),
            ),
            base_files_copied: 0,
            extras_files_copied: 0,
            extras_warning: None,
            rom_dirs_created: 0,
        });
    }

    let base_extracted = base_extraction
        .output_path
        .ok_or("No base extraction path returned")?;

    // Step 3: Copy base files to SD card
    progress(InstallProgressEvent {
        step: "copy".to_string(),
        details: "Copying base files to SD card...".to_string(),
    });
    let base_files_copied =
        copy_base_files(&base_extracted, &options.sd_mount, &options.platform)?;

    // Step 4: Download and extract extras (if available) — non-fatal on failure
    let mut extras_files_copied = 0u32;
    let mut extras_warning: Option<String> = None;

    if options.extras_url.is_some() {
        match try_install_extras(options, progress.clone()).await {
            Ok(copied) => extras_files_copied = copied,
            Err(e) => extras_warning = Some(e),
        }
    }

    // Step 5: Create standard ROM directories
    progress(InstallProgressEvent {
        step: "copy".to_string(),
        details: "Creating standard ROM directories...".to_string(),
    });
    let rom_dirs_created = create_rom_dirs(&options.sd_mount).unwrap_or(0);

    // Write version metadata after successful install
    progress(InstallProgressEvent {
        step: "finish".to_string(),
        details: format!("Writing version metadata (MinUI {})", options.version),
    });
    let minui_txt_path = Path::new(&options.sd_mount).join("minui.txt");
    if let Err(e) = fs::write(&minui_txt_path, format!("MinUI {}\n", options.version)) {
        // Non-fatal: install succeeded but version metadata couldn't be written
        eprintln!("Warning: Failed to write version metadata: {}", e);
    }

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

        let copied = fs_utils::copy_dir_recursive(&src, &dst, &|_s, _d| false).unwrap();
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

        let copied = fs_utils::copy_dir_recursive(&src, &sd_root, &|_src, dst| {
            is_preserved_path(dst, &sd_root)
        })
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
        let platform_dir = extracted.join("miyoo-mini-plus");
        let sd_root = temp.path().join("sdcard");

        fs::create_dir_all(&platform_dir).unwrap();
        fs::create_dir_all(&sd_root).unwrap();

        fs::write(platform_dir.join("minui.pak"), "base").unwrap();
        fs::write(platform_dir.join("boot.sh"), "boot").unwrap();

        // copy_base_files now copies ALL contents of extracted to sd_root
        let copied = copy_base_files(
            extracted.to_str().unwrap(),
            sd_root.to_str().unwrap(),
            "any",
        )
        .unwrap();

        assert_eq!(copied, 2);
        assert!(sd_root.join("miyoo-mini-plus/minui.pak").exists());
        assert!(sd_root.join("miyoo-mini-plus/boot.sh").exists());
    }

    #[test]
    fn test_is_preserved_path_nested() {
        let sd_root = Path::new("/Volumes/SDCARD");

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

        let copied = fs_utils::copy_dir_recursive(&base_src, &sd_root, &|_src, dst| {
            is_preserved_path(dst, &sd_root)
        })
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
        fs::write(extras_src.join("Emus/rg35xxplus/gambatte.pak/launch.sh"), "emu").unwrap();
        fs::write(extras_src.join("Tools/rg35xxplus/wifi.pak/launch.sh"), "tool").unwrap();
        fs::write(extras_src.join("Tools/rg35xxplus/ssh.pak/launch.sh"), "tool").unwrap();
        fs::write(extras_src.join("Tools/trimuismart/dc.pak/launch.sh"), "tool").unwrap();
        fs::write(extras_src.join("Tools/trimuismart/wifi.pak/launch.sh"), "tool").unwrap();
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
        assert!(sd_root.join("Emus/rg35xxplus/gambatte.pak/launch.sh").exists());
        assert!(sd_root.join("Tools/rg35xxplus/wifi.pak/launch.sh").exists());
        assert!(sd_root.join("Tools/rg35xxplus/ssh.pak/launch.sh").exists());

        // Verify trimuismart stuff was NOT copied
        assert!(!sd_root.join("Tools/trimuismart").exists());

        // Verify Bios was copied
        assert!(sd_root.join("Bios/gba_bios.bin").exists());
    }
}
