use std::fs;
use std::path::Path;

use crate::download;
use crate::extract;

#[derive(Debug, Clone, serde::Serialize)]
pub struct InstallResult {
    pub success: bool,
    pub error: Option<String>,
    pub base_files_copied: u32,
    pub extras_files_copied: u32,
    pub extras_warning: Option<String>,
}

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

/// Copies a directory tree from src to dst, skipping preserved folders.
/// Returns the number of files copied.
fn copy_dir_recursive(src: &Path, dst: &Path, _sd_root: &Path) -> Result<u32, String> {
    let mut files_copied = 0u32;

    let entries = fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory {}: {}", src.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        // Skip preserved folders (ROMS, Saves, etc.)
        if is_preserved_path(&src_path, src) {
            continue;
        }

        if src_path.is_dir() {
            fs::create_dir_all(&dst_path)
                .map_err(|e| format!("Failed to create directory {}: {}", dst_path.display(), e))?;
            files_copied += copy_dir_recursive(&src_path, &dst_path, _sd_root)?;
        } else {
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent directory: {}", e))?;
            }
            fs::copy(&src_path, &dst_path).map_err(|e| {
                format!(
                    "Failed to copy {} to {}: {}",
                    src_path.display(),
                    dst_path.display(),
                    e
                )
            })?;
            files_copied += 1;
        }
    }

    Ok(files_copied)
}

/// Copies all files from extracted MinUI base archive to SD card root.
///
/// The MinUI base archive has all platform folders at the root level
/// (trimui/, miyoo/, rg35xxplus/, etc.) plus shared files (MinUI.zip,
/// README.txt, em_ui.sh). Per MinUI docs: "Copy all the folders from
/// this zip file to the root of your primary card." The device selects
/// the appropriate platform folder on first boot.
pub fn copy_base_files(
    extracted_base_path: &str,
    sd_mount: &str,
    _platform: &str,
) -> Result<u32, String> {
    let base_dir = Path::new(extracted_base_path);
    let sd_root = Path::new(sd_mount);
    copy_dir_recursive(base_dir, sd_root, sd_root)
}

/// Copies Extras files to the SD card extras directory.
pub fn copy_extras_files(
    extracted_extras_path: &str,
    sd_mount: &str,
    extras_dir: &str,
) -> Result<u32, String> {
    let extras_src = Path::new(extracted_extras_path);
    let sd_root = Path::new(sd_mount);
    let extras_dst = sd_root.join(extras_dir.trim_start_matches('/'));

    if !extras_src.exists() {
        return Err("Extras source directory does not exist".to_string());
    }

    fs::create_dir_all(&extras_dst)
        .map_err(|e| format!("Failed to create extras directory: {}", e))?;

    copy_dir_recursive(extras_src, &extras_dst, sd_root)
}

/// Runs extras download → extract → copy, returning the number of files copied.
/// Errors are propagated via `Result` — the caller decides how to handle failures.
async fn try_install_extras(
    url: &str,
    checksum: Option<&str>,
    sd_mount: &str,
    extras_dir: &str,
) -> Result<u32, String> {
    let (result, _temp) = download::download_archive(url, checksum)
        .await
        .map_err(|e| format!("Extras download failed: {}", e))?;
    let path = result.file_path.ok_or("No extras download path")?;
    let (extraction, _temp) = extract::extract_archive(&path, None)
        .map_err(|e| format!("Extras extraction failed: {}", e))?;
    let extracted = extraction.output_path.ok_or("No extras extraction path")?;
    copy_extras_files(&extracted, sd_mount, extras_dir)
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
    extras_dir: &str,
    version: &str,
) -> Result<InstallResult, String> {
    // Step 1: Download and extract base
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

    // Step 2: Copy base files
    let base_files_copied =
        copy_base_files(&base_extracted, sd_mount, platform)?;

    // Step 3: Download and extract extras (if available) — non-fatal on failure
    let mut extras_files_copied = 0u32;
    let mut extras_warning: Option<String> = None;

    if let Some(url) = extras_url {
        match try_install_extras(url, extras_checksum, sd_mount, extras_dir).await {
            Ok(copied) => extras_files_copied = copied,
            Err(e) => extras_warning = Some(e),
        }
    }

    // Write version metadata after successful install
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

        let copied = copy_dir_recursive(&src, &dst, &sd_root).unwrap();
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

        let copied = copy_dir_recursive(&src, &dst, &sd_root).unwrap();
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
    fn test_copy_extras_files() {
        let temp = tempfile::tempdir().unwrap();
        let extras_src = temp.path().join("extras_extracted");
        let sd_root = temp.path().join("sdcard");

        fs::create_dir_all(&extras_src).unwrap();
        fs::create_dir_all(sd_root.join("Tools")).unwrap();

        fs::write(extras_src.join("wifi.pak"), "wifi").unwrap();

        let copied = copy_extras_files(
            extras_src.to_str().unwrap(),
            sd_root.to_str().unwrap(),
            "/Tools",
        )
        .unwrap();

        assert_eq!(copied, 1);
        assert!(sd_root.join("Tools/wifi.pak").exists());
    }
}
