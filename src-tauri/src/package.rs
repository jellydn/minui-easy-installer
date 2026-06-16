use std::fs;
use std::path::Path;
use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::fs_utils;
use crate::pipeline::{
    create_target_within, DownloadProgressCallback, InstallSession, Pipeline,
};
use crate::version;

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
    pub pak_name: String,
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
                Some(installed_ver) => version::compare_versions(installed_ver, latest_version),
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
    platform: &str,
) -> Result<PackageInstallResult, String> {
    install_package_with_cancel(
        artifact_url,
        checksum,
        sd_mount,
        rules,
        platform,
        Arc::new(|_, _| {}),
        CancellationToken::new(),
    )
    .await
}

/// Package install with cancellation and download progress support.
pub async fn install_package_with_cancel(
    artifact_url: &str,
    checksum: Option<&str>,
    sd_mount: &str,
    rules: &PackageInstallPathRules,
    platform: &str,
    download_progress: DownloadProgressCallback,
    cancel: CancellationToken,
) -> Result<PackageInstallResult, String> {
    let mut session = InstallSession::new();

    // Pipeline handles download + extract, returns extracted path
    let extracted = Pipeline::run_to_extracted(
        "package",
        artifact_url,
        checksum,
        Arc::new(|_| {}),
        download_progress,
        cancel,
        &mut session,
    )
    .await?;

    // Copy files to target directory with path-traversal protection
    let pak_root = create_target_within(
        Path::new(sd_mount),
        &rules.target_dir,
        platform,
        &rules.pak_name,
    )?;

    let files_copied =
        fs_utils::copy_dir_recursive(&extracted, &pak_root, &|_s, _d| false, &|| false)?;

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
    fn test_copy_dir_recursive_with_subdirs() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");

        fs::create_dir_all(src.join("subdir")).unwrap();
        fs::write(src.join("file1.txt"), "content1").unwrap();
        fs::write(src.join("subdir/file2.txt"), "content2").unwrap();

        let copied =
            fs_utils::copy_dir_recursive(&src, &dst, &|_s, _d| false, &|| false).unwrap();
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
