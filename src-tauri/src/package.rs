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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstalledPackage {
    pub name: String,
    pub version: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackageUpdateInfo {
    pub name: String,
    pub installed_version: Option<String>,
    pub latest_version: String,
    pub update_available: bool,
}

/// Resolves the install path for a package based on rules and SD card mount.
///
/// Returns an error if the resolved path escapes the SD card root (path traversal).
pub fn resolve_package_install_path(
    sd_mount: &str,
    rules: &PackageInstallPathRules,
) -> Result<String, String> {
    let sd_root = Path::new(sd_mount);

    if rules.extract_to_root {
        return Ok(sd_mount.to_string());
    }

    // Reject any path component that could be used for traversal
    let target_dir = rules.target_dir.trim_start_matches('/');
    if target_dir.split('/').any(|c| c == "..") {
        return Err(format!(
            "Path traversal detected in targetDir: '{}'",
            rules.target_dir
        ));
    }

    let target = sd_root.join(target_dir);

    // If the target path exists, verify it stays within SD root via canonicalization
    if let Ok(canonical_target) = target.canonicalize() {
        if let Ok(canonical_root) = sd_root.canonicalize() {
            if !canonical_target.starts_with(&canonical_root) {
                return Err(format!(
                    "Security violation: resolved path '{}' escapes SD card root",
                    target.display()
                ));
            }
        }
    }

    Ok(target.to_string_lossy().to_string())
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

/// Detect installed packages on the SD card.
///
/// Looks for .pak directories in the Tools folder and checks for version files.
pub fn detect_installed_packages(sd_mount: &str) -> Vec<InstalledPackage> {
    let sd_root = Path::new(sd_mount);
    let tools_dir = sd_root.join("Tools");

    if !tools_dir.exists() || !tools_dir.is_dir() {
        return Vec::new();
    }

    let mut packages = Vec::new();

    if let Ok(entries) = fs::read_dir(&tools_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();

                // Look for version file
                let version_file = path.join("version.txt");
                let version = if version_file.exists() {
                    fs::read_to_string(&version_file)
                        .ok()
                        .map(|v| v.trim().to_string())
                        .filter(|v| !v.is_empty())
                } else {
                    None
                };

                packages.push(InstalledPackage {
                    name,
                    version,
                    source: "Tools".to_string(),
                });
            }
        }
    }

    packages
}

/// Check for package updates by comparing installed versions with registry.
pub fn check_package_updates(
    sd_mount: &str,
    registry_packages: &[(String, String)], // (name, version) pairs
) -> Vec<PackageUpdateInfo> {
    let installed = detect_installed_packages(sd_mount);
    let mut updates = Vec::new();

    for (name, latest_version) in registry_packages {
        let installed_pkg = installed.iter().find(|p| &p.name == name);

        let update_available = match installed_pkg {
            Some(pkg) => match &pkg.version {
                Some(installed_ver) => latest_version > installed_ver,
                None => true, // Unknown version - assume update available
            },
            None => false, // Not installed
        };

        if installed_pkg.is_some() {
            updates.push(PackageUpdateInfo {
                name: name.clone(),
                installed_version: installed_pkg.and_then(|p| p.version.clone()),
                latest_version: latest_version.clone(),
                update_available,
            });
        }
    }

    updates
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
    // Step 1: Download the artifact (checksum verification is optional — not all registries provide them)
    let (download_result, _artifact_temp) = download::download_archive(artifact_url, checksum)
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
    let (extraction_result, _pkg_extract_temp) = extract::extract_archive(&artifact_path, None)
        .map_err(|e| format!("Package extraction failed: {}", e))?;

    if !extraction_result.success {
        return Ok(PackageInstallResult {
            success: false,
            error: Some(
                extraction_result
                    .error
                    .unwrap_or("Package extraction failed".to_string()),
            ),
            files_copied: 0,
        });
    }

    let extracted_path = extraction_result
        .output_path
        .ok_or("No extraction path returned")?;

    // Step 3: Copy files to target directory
    let target_dir = resolve_package_install_path(sd_mount, rules)
        .map_err(|e| format!("Failed to resolve install path: {}", e))?;
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
        let temp = tempfile::tempdir().unwrap();
        let sd_mount = temp.path().to_str().unwrap();
        let rules = PackageInstallPathRules {
            target_dir: "/Tools".to_string(),
            extract_to_root: false,
        };

        let result = resolve_package_install_path(sd_mount, &rules).unwrap();
        assert!(result.ends_with("/Tools"));
        assert!(result.starts_with(sd_mount));
    }

    #[test]
    fn test_resolve_package_install_path_to_root() {
        let temp = tempfile::tempdir().unwrap();
        let sd_mount = temp.path().to_str().unwrap();
        let rules = PackageInstallPathRules {
            target_dir: "/".to_string(),
            extract_to_root: true,
        };

        let result = resolve_package_install_path(sd_mount, &rules).unwrap();
        assert_eq!(result, sd_mount);
    }

    #[test]
    fn test_resolve_package_install_path_nested() {
        let temp = tempfile::tempdir().unwrap();
        let sd_mount = temp.path().to_str().unwrap();
        let rules = PackageInstallPathRules {
            target_dir: "/Apps/Emulators".to_string(),
            extract_to_root: false,
        };

        // Create the target dir so canonicalize works
        std::fs::create_dir_all(temp.path().join("Apps/Emulators")).unwrap();
        let result = resolve_package_install_path(sd_mount, &rules).unwrap();
        assert!(result.ends_with("/Apps/Emulators"));
        assert!(result.starts_with(sd_mount));
    }

    #[test]
    fn test_resolve_package_install_path_rejects_traversal() {
        let temp = tempfile::tempdir().unwrap();
        let sd_mount = temp.path().to_str().unwrap();
        let rules = PackageInstallPathRules {
            target_dir: "../escape".to_string(),
            extract_to_root: false,
        };

        let result = resolve_package_install_path(sd_mount, &rules);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("traversal"));
    }

    #[test]
    fn test_resolve_package_install_path_rejects_deep_traversal() {
        let temp = tempfile::tempdir().unwrap();
        let sd_mount = temp.path().to_str().unwrap();
        let rules = PackageInstallPathRules {
            target_dir: "/Tools/../../etc".to_string(),
            extract_to_root: false,
        };

        let result = resolve_package_install_path(sd_mount, &rules);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("traversal"));
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

    #[test]
    fn test_detect_installed_packages_empty() {
        let temp = tempfile::tempdir().unwrap();
        let packages = detect_installed_packages(temp.path().to_str().unwrap());
        assert!(packages.is_empty());
    }

    #[test]
    fn test_detect_installed_packages_with_tools() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();
        let tools_dir = sd_root.join("Tools");

        fs::create_dir_all(tools_dir.join("wifi.pak")).unwrap();
        fs::create_dir_all(tools_dir.join("ssh.pak")).unwrap();
        fs::write(tools_dir.join("wifi.pak/version.txt"), "1.0.0").unwrap();

        let packages = detect_installed_packages(sd_root.to_str().unwrap());
        assert_eq!(packages.len(), 2);

        let wifi = packages.iter().find(|p| p.name == "wifi.pak").unwrap();
        assert_eq!(wifi.version, Some("1.0.0".to_string()));

        let ssh = packages.iter().find(|p| p.name == "ssh.pak").unwrap();
        assert_eq!(ssh.version, None);
    }

    #[test]
    fn test_check_package_updates() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();
        let tools_dir = sd_root.join("Tools");

        fs::create_dir_all(tools_dir.join("wifi.pak")).unwrap();
        fs::write(tools_dir.join("wifi.pak/version.txt"), "1.0.0").unwrap();

        let registry = vec![
            ("wifi.pak".to_string(), "1.1.0".to_string()),
            ("ssh.pak".to_string(), "1.0.0".to_string()),
        ];

        let updates = check_package_updates(sd_root.to_str().unwrap(), &registry);
        assert_eq!(updates.len(), 1); // Only wifi.pak is installed

        let wifi_update = &updates[0];
        assert_eq!(wifi_update.name, "wifi.pak");
        assert!(wifi_update.update_available);
        assert_eq!(wifi_update.installed_version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_check_package_updates_unknown_version() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();
        let tools_dir = sd_root.join("Tools");

        fs::create_dir_all(tools_dir.join("wifi.pak")).unwrap();
        // No version file

        let registry = vec![("wifi.pak".to_string(), "1.1.0".to_string())];

        let updates = check_package_updates(sd_root.to_str().unwrap(), &registry);
        assert_eq!(updates.len(), 1);

        let wifi_update = &updates[0];
        assert!(wifi_update.update_available); // Unknown version = update available
        assert_eq!(wifi_update.installed_version, None);
    }
}
