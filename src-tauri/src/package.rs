use std::fs;
use std::path::Path;

use crate::download;
use crate::extract;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackageInstallResult {
    pub success: bool,
    pub error: Option<String>,
    pub files_copied: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackageInstallPathRules {
    pub target_dir: String,
    pub extract_to_root: bool,
}

/// Resolves the install path for a package based on rules and SD card mount.
pub fn resolve_package_install_path(
    sd_mount: &str,
    rules: &PackageInstallPathRules,
) -> String {
    let sd_root = Path::new(sd_mount);

    if rules.extract_to_root {
        return sd_mount.to_string();
    }

    let target = sd_root.join(rules.target_dir.trim_start_matches('/'));
    target.to_string_lossy().to_string()
}

/// Copies package files from extracted directory to target directory.
fn copy_package_files(
    extracted_path: &str,
    target_dir: &str,
    extract_to_root: bool,
) -> Result<u32, String> {
    let src = Path::new(extracted_path);
    let dst = Path::new(target_dir);

    if !src.exists() {
        return Err("Extracted package directory does not exist".to_string());
    }

    if extract_to_root {
        // Copy all files from extracted to root
        copy_dir_contents(src, dst)
    } else {
        // Copy all files to target directory
        fs::create_dir_all(dst)
            .map_err(|e| format!("Failed to create target directory: {}", e))?;
        copy_dir_contents(src, dst)
    }
}

/// Copies contents of src directory to dst directory (non-recursive for top level).
fn copy_dir_contents(src: &Path, dst: &Path) -> Result<u32, String> {
    let mut files_copied = 0u32;

    let entries = fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory {}: {}", src.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            fs::create_dir_all(&dst_path)
                .map_err(|e| format!("Failed to create directory {}: {}", dst_path.display(), e))?;
            files_copied += copy_dir_recursive(&src_path, &dst_path)?;
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

/// Recursively copies directory tree.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<u32, String> {
    let mut files_copied = 0u32;

    let entries = fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory {}: {}", src.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            fs::create_dir_all(&dst_path)
                .map_err(|e| format!("Failed to create directory {}: {}", dst_path.display(), e))?;
            files_copied += copy_dir_recursive(&src_path, &dst_path)?;
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

/// Install a package from a URL to the SD card.
///
/// Downloads the artifact, optionally verifies checksum, extracts, and copies
/// files according to the install path rules.
pub async fn install_package(
    artifact_url: &str,
    checksum: Option<&str>,
    sd_mount: &str,
    rules: &PackageInstallPathRules,
) -> Result<PackageInstallResult, String> {
    // Step 1: Download the artifact
    let download_result = download::download_archive(artifact_url, checksum)
        .await
        .map_err(|e| format!("Package download failed: {}", e))?;

    if !download_result.success {
        return Ok(PackageInstallResult {
            success: false,
            error: Some(
                download_result
                    .error
                    .unwrap_or("Package download failed".to_string()),
            ),
            files_copied: 0,
        });
    }

    let artifact_path = download_result
        .file_path
        .ok_or("No artifact file path returned")?;

    // Step 2: Extract the artifact
    let extraction = extract::extract_archive(&artifact_path, None)
        .map_err(|e| format!("Package extraction failed: {}", e))?;

    if !extraction.success {
        return Ok(PackageInstallResult {
            success: false,
            error: Some(
                extraction
                    .error
                    .unwrap_or("Package extraction failed".to_string()),
            ),
            files_copied: 0,
        });
    }

    let extracted_path = extraction
        .output_path
        .ok_or("No extraction path returned")?;

    // Step 3: Copy files to target directory
    let target_dir = resolve_package_install_path(sd_mount, rules);
    let files_copied = copy_package_files(&extracted_path, &target_dir, rules.extract_to_root)?;

    Ok(PackageInstallResult {
        success: true,
        error: None,
        files_copied,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_package_install_path_to_tools() {
        let rules = PackageInstallPathRules {
            target_dir: "/Tools".to_string(),
            extract_to_root: false,
        };

        let result = resolve_package_install_path("/Volumes/SDCARD", &rules);
        assert_eq!(result, "/Volumes/SDCARD/Tools");
    }

    #[test]
    fn test_resolve_package_install_path_to_root() {
        let rules = PackageInstallPathRules {
            target_dir: "/".to_string(),
            extract_to_root: true,
        };

        let result = resolve_package_install_path("/Volumes/SDCARD", &rules);
        assert_eq!(result, "/Volumes/SDCARD");
    }

    #[test]
    fn test_resolve_package_install_path_nested() {
        let rules = PackageInstallPathRules {
            target_dir: "/Apps/Emulators".to_string(),
            extract_to_root: false,
        };

        let result = resolve_package_install_path("/Volumes/SDCARD", &rules);
        assert_eq!(result, "/Volumes/SDCARD/Apps/Emulators");
    }

    #[test]
    fn test_copy_dir_contents() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");

        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("file1.txt"), "content1").unwrap();
        fs::write(src.join("file2.txt"), "content2").unwrap();

        let copied = copy_dir_contents(&src, &dst).unwrap();
        assert_eq!(copied, 2);
        assert!(dst.join("file1.txt").exists());
        assert!(dst.join("file2.txt").exists());
    }

    #[test]
    fn test_copy_dir_contents_with_subdirs() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");

        fs::create_dir_all(src.join("subdir")).unwrap();
        fs::write(src.join("file1.txt"), "content1").unwrap();
        fs::write(src.join("subdir/file2.txt"), "content2").unwrap();

        let copied = copy_dir_contents(&src, &dst).unwrap();
        assert_eq!(copied, 2);
        assert!(dst.join("file1.txt").exists());
        assert!(dst.join("subdir/file2.txt").exists());
    }
}
