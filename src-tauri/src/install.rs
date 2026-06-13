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
}

/// Progress event emitted during the install flow.
#[derive(Debug, Clone, serde::Serialize)]
pub struct InstallProgressEvent {
    pub step: String,
    pub details: String,
}

/// Callback for reporting install progress. Passed through the install flow.
pub type ProgressCallback = Arc<dyn Fn(InstallProgressEvent) + Send + Sync>;

/// Folders that must never be deleted or overwritten during install
const PRESERVED_FOLDERS: &[&str] = &[
    "ROMS", "roms", "Saves", "saves", "SAVE", "save", "BIOS", "bios", "CHEATS", "cheats",
];

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
    fs_utils::copy_dir_recursive(base_dir, sd_root, &|path| is_preserved_path(path, base_dir))
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
    let extras_src = Path::new(extracted_extras_path);
    let sd_root = Path::new(sd_mount);

    if !extras_src.exists() {
        return Err("Extras source directory does not exist".to_string());
    }

    let mut files_copied = 0u32;

    // Copy Bios/ (shared across all devices)
    let bios_src = extras_src.join("Bios");
    if bios_src.exists() {
        let bios_dst = sd_root.join("Bios");
        fs::create_dir_all(&bios_dst)
            .map_err(|e| format!("Failed to create Bios directory: {}", e))?;
        files_copied += fs_utils::copy_dir_recursive(&bios_src, &bios_dst, &|_| false)?;
    }

    // Copy Emus/{extras_platform}/
    let emus_platform_src = extras_src.join("Emus").join(extras_platform);
    if emus_platform_src.exists() {
        let emus_platform_dst = sd_root.join("Emus").join(extras_platform);
        fs::create_dir_all(&emus_platform_dst)
            .map_err(|e| format!("Failed to create Emus/{} directory: {}", extras_platform, e))?;
        files_copied +=
            fs_utils::copy_dir_recursive(&emus_platform_src, &emus_platform_dst, &|_| false)?;
    }

    // Copy Tools/{extras_platform}/
    let tools_platform_src = extras_src.join("Tools").join(extras_platform);
    if tools_platform_src.exists() {
        let tools_platform_dst = sd_root.join("Tools").join(extras_platform);
        fs::create_dir_all(&tools_platform_dst)
            .map_err(|e| {
                format!("Failed to create Tools/{} directory: {}", extras_platform, e)
            })?;
        files_copied +=
            fs_utils::copy_dir_recursive(&tools_platform_src, &tools_platform_dst, &|_| false)?;
    }

    Ok(files_copied)
}

/// Runs extras download → extract → copy, returning the number of files copied.
/// Errors are propagated via `Result` — the caller decides how to handle failures.
async fn try_install_extras(
    url: &str,
    checksum: Option<&str>,
    sd_mount: &str,
    extras_platform: &str,
    progress: ProgressCallback,
) -> Result<u32, String> {
    progress(InstallProgressEvent {
        step: "download".to_string(),
        details: "Downloading extras archive...".to_string(),
    });
    let (result, _temp) = download::download_archive(url, checksum)
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
            extras_platform, extras_platform
        ),
    });
    copy_extras_files(&extracted, sd_mount, extras_platform)
        .map_err(|e| format!("Extras copy failed: {}", e))
}

/// Full installation flow: download, extract, copy base + extras.
///
/// This is the main entry point that coordinates the entire install process.
#[allow(clippy::too_many_arguments)]
pub async fn install_minui(
    base_url: &str,
    extras_url: Option<&str>,
    base_checksum: Option<&str>,
    extras_checksum: Option<&str>,
    sd_mount: &str,
    platform: &str,
    extras_platform: &str,
    version: &str,
    progress: ProgressCallback,
) -> Result<InstallResult, String> {
    // Step 1: Download base archive
    let file_name = base_url.rsplit('/').next().unwrap_or("MinUI.zip");
    progress(InstallProgressEvent {
        step: "download".to_string(),
        details: format!("Downloading {}", file_name),
    });
    let (base_result, _base_temp) = download::download_archive(base_url, base_checksum)
        .await
        .map_err(|e| format!("Base download failed: {}", e))?;

    if !base_result.success {
        return Ok(InstallResult {
            success: false,
            error: Some(base_result.error.unwrap_or("Base download failed".to_string())),
            base_files_copied: 0,
            extras_files_copied: 0,
            extras_warning: None,
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
        copy_base_files(&base_extracted, sd_mount, platform)?;

    // Step 4: Download and extract extras (if available) — non-fatal on failure
    let mut extras_files_copied = 0u32;
    let mut extras_warning: Option<String> = None;

    if let Some(url) = extras_url {
        match try_install_extras(url, extras_checksum, sd_mount, extras_platform, progress.clone()).await {
            Ok(copied) => extras_files_copied = copied,
            Err(e) => extras_warning = Some(e),
        }
    }

    // Write version metadata after successful install
    progress(InstallProgressEvent {
        step: "finish".to_string(),
        details: format!("Writing version metadata (MinUI {})", version),
    });
    let minui_txt_path = Path::new(sd_mount).join("minui.txt");
    let _ = fs::write(&minui_txt_path, format!("MinUI {}\n", version));

    Ok(InstallResult {
        success: true,
        error: None,
        base_files_copied,
        extras_files_copied,
        extras_warning,
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
        let sd_root = temp.path().join("sdcard");

        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&sd_root).unwrap();

        let mut f = fs::File::create(src.join("test.txt")).unwrap();
        f.write_all(b"hello").unwrap();
        drop(f);

        let copied = fs_utils::copy_dir_recursive(&src, &dst, &|_| false).unwrap();
        assert_eq!(copied, 1);
        assert!(dst.join("test.txt").exists());
    }

    #[test]
    fn test_copy_dir_recursive_skips_preserved_folders() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");
        let sd_root = temp.path().join("sdcard");

        fs::create_dir_all(src.join("ROMS")).unwrap();
        fs::create_dir_all(src.join("Saves")).unwrap();
        fs::create_dir_all(src.join("Tools")).unwrap();
        fs::create_dir_all(&sd_root).unwrap();

        fs::write(src.join("ROMS/game.nes"), "rom").unwrap();
        fs::write(src.join("Saves/save.sav"), "save").unwrap();
        fs::write(src.join("Tools/tool.pak"), "tool").unwrap();

        let copied = fs_utils::copy_dir_recursive(&src, &dst, &|path| is_preserved_path(path, &src)).unwrap();
        assert_eq!(copied, 1); // Only tool.pak
        assert!(!dst.join("ROMS").exists());
        assert!(!dst.join("Saves").exists());
        assert!(dst.join("Tools/tool.pak").exists());
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
