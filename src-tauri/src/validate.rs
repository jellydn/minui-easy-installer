use std::fs;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationResult {
    pub success: bool,
    pub checks: Vec<ValidationCheck>,
    pub passed_count: u32,
    pub failed_count: u32,
    pub free_space_bytes: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthCheckResult {
    pub checks: Vec<ValidationCheck>,
    pub passed_count: u32,
    pub failed_count: u32,
    pub free_space_bytes: Option<u64>,
    pub filesystem: Option<String>,
    pub support_report: String,
}

const ESSENTIAL_BASE_PATHS: &[&str] = &["minui.pak", "boot.sh", "DMG.png"];

fn check_path_exists(sd_root: &Path, relative_path: &str) -> ValidationCheck {
    let full_path = sd_root.join(relative_path.trim_start_matches('/'));
    let exists = full_path.exists();

    ValidationCheck {
        name: format!("Check: {}", relative_path),
        passed: exists,
        message: if exists {
            format!("Found: {}", relative_path)
        } else {
            format!("Missing: {}", relative_path)
        },
    }
}

fn check_directory_exists(sd_root: &Path, relative_path: &str) -> ValidationCheck {
    let full_path = sd_root.join(relative_path.trim_start_matches('/'));
    let exists = full_path.exists() && full_path.is_dir();

    ValidationCheck {
        name: format!("Directory: {}", relative_path),
        passed: exists,
        message: if exists {
            format!("Found directory: {}", relative_path)
        } else {
            format!("Missing directory: {}", relative_path)
        },
    }
}

fn check_pak_files(sd_root: &Path, tools_dir: &str) -> Vec<ValidationCheck> {
    let mut checks = Vec::new();
    let tools_path = sd_root.join(tools_dir.trim_start_matches('/'));

    if !tools_path.exists() {
        checks.push(ValidationCheck {
            name: "Tools directory".to_string(),
            passed: false,
            message: format!("Missing Tools directory at {}", tools_dir),
        });
        return checks;
    }

    // Check for .pak files in Tools directory
    if let Ok(entries) = fs::read_dir(&tools_path) {
        let mut pak_count = 0u32;
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("pak") {
                pak_count += 1;
            }
        }

        checks.push(ValidationCheck {
            name: "PAK files count".to_string(),
            passed: pak_count > 0,
            message: if pak_count > 0 {
                format!("Found {} PAK file(s) in {}", pak_count, tools_dir)
            } else {
                format!("No PAK files found in {}", tools_dir)
            },
        });
    }

    checks
}

fn check_free_space(sd_root: &Path) -> Option<u64> {
    get_free_space(sd_root.to_str()?)
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

pub fn validate_installation(
    sd_mount: &str,
    has_extras: bool,
    extras_dir: &str,
) -> Result<ValidationResult, String> {
    let sd_root = Path::new(sd_mount);

    if !sd_root.exists() {
        return Err(format!("SD card mount path does not exist: {}", sd_mount));
    }

    let mut checks = Vec::new();

    // Check essential base paths
    for path in ESSENTIAL_BASE_PATHS {
        checks.push(check_path_exists(sd_root, path));
    }

    // Check Tools directory exists
    checks.push(check_directory_exists(sd_root, extras_dir));

    // Check PAK files if extras were installed
    if has_extras {
        let pak_checks = check_pak_files(sd_root, extras_dir);
        checks.extend(pak_checks);
    }

    // Check free space
    let free_space = check_free_space(sd_root);
    if let Some(space) = free_space {
        checks.push(ValidationCheck {
            name: "Free space".to_string(),
            passed: space > 100 * 1024 * 1024, // At least 100MB free
            message: format!("Available: {}", format_bytes(space)),
        });
    }

    let passed_count = checks.iter().filter(|c| c.passed).count() as u32;
    let failed_count = checks.iter().filter(|c| !c.passed).count() as u32;

    Ok(ValidationResult {
        success: failed_count == 0,
        checks,
        passed_count,
        failed_count,
        free_space_bytes: free_space,
    })
}

pub fn format_validation_report(result: &ValidationResult) -> String {
    let mut report = String::new();
    report.push_str("MinUI Installation Validation Report\n");
    report.push_str("=====================================\n\n");

    if result.success {
        report.push_str("Status: PASSED\n\n");
    } else {
        report.push_str("Status: FAILED\n\n");
    }

    report.push_str(&format!(
        "Checks: {} passed, {} failed\n\n",
        result.passed_count, result.failed_count
    ));

    report.push_str("Details:\n");
    for check in &result.checks {
        let status = if check.passed { "✓" } else { "✗" };
        report.push_str(&format!("  {} {}\n", status, check.message));
    }

    if let Some(space) = result.free_space_bytes {
        report.push_str(&format!("\nFree Space: {}\n", format_bytes(space)));
    }

    report
}

/// Perform a comprehensive health check on the SD card.
///
/// Checks filesystem, free space, MinUI folders, and package PAK files.
pub fn check_sd_card_health(
    sd_mount: &str,
    _device_platform: Option<&str>,
) -> Result<HealthCheckResult, String> {
    let sd_root = Path::new(sd_mount);

    if !sd_root.exists() {
        return Err("SD card mount point does not exist".to_string());
    }

    let mut checks = Vec::new();
    let mut passed_count = 0u32;
    let mut failed_count = 0u32;

    // Check filesystem (warn if not FAT32)
    let filesystem = detect_filesystem(sd_mount);
    if let Some(ref fs_type) = filesystem {
        let is_fat32 =
            fs_type.to_lowercase().contains("fat32") || fs_type.to_lowercase().contains("ms-dos");
        checks.push(ValidationCheck {
            name: "filesystem".to_string(),
            passed: is_fat32,
            message: if is_fat32 {
                format!("Filesystem: {} (FAT32 - recommended)", fs_type)
            } else {
                format!("Filesystem: {} (not FAT32 - may cause issues)", fs_type)
            },
        });
        if is_fat32 {
            passed_count += 1;
        } else {
            failed_count += 1;
        }
    }

    // Check free space
    let free_space = get_free_space(sd_mount);
    if let Some(space) = free_space {
        let has_space = space > 100 * 1024 * 1024; // 100MB minimum
        checks.push(ValidationCheck {
            name: "free_space".to_string(),
            passed: has_space,
            message: if has_space {
                format!("Free space: {} (sufficient)", format_bytes(space))
            } else {
                format!(
                    "Free space: {} (low - may need more space)",
                    format_bytes(space)
                )
            },
        });
        if has_space {
            passed_count += 1;
        } else {
            failed_count += 1;
        }
    }

    // Check for MinUI folders
    let minui_folders = ["Apps", "Tools", "Emus"];
    for folder in &minui_folders {
        let folder_path = sd_root.join(folder);
        let exists = folder_path.exists() && folder_path.is_dir();
        checks.push(ValidationCheck {
            name: format!("folder_{}", folder.to_lowercase()),
            passed: exists,
            message: if exists {
                format!("{}: Found", folder)
            } else {
                format!("{}: Missing", folder)
            },
        });
        if exists {
            passed_count += 1;
        } else {
            failed_count += 1;
        }
    }

    // Check for PAK files in Tools
    let tools_dir = sd_root.join("Tools");
    if tools_dir.exists() {
        let expected_paks = ["wifi.pak", "ssh.pak"];
        for pak in &expected_paks {
            let pak_path = tools_dir.join(pak);
            let exists = pak_path.exists() && pak_path.is_dir();
            if exists {
                checks.push(ValidationCheck {
                    name: format!("pak_{}", pak.replace(".pak", "")),
                    passed: true,
                    message: format!("{}: Installed", pak),
                });
                passed_count += 1;
            }
            // Don't flag missing optional PAKs as failures
        }
    }

    let support_report = generate_support_report(&checks, free_space, filesystem.as_deref());

    Ok(HealthCheckResult {
        checks,
        passed_count,
        failed_count,
        free_space_bytes: free_space,
        filesystem,
        support_report,
    })
}

fn detect_filesystem(sd_mount: &str) -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("diskutil")
            .arg("info")
            .arg(sd_mount)
            .output()
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("File System Personality:") {
                    return Some(line.split(':').nth(1)?.trim().to_string());
                }
            }
        }
    }

    None
}

fn get_free_space(sd_mount: &str) -> Option<u64> {
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::mem;

        let path = CString::new(sd_mount).ok()?;
        let mut stat: libc::statvfs = unsafe { mem::zeroed() };

        let result = unsafe { libc::statvfs(path.as_ptr(), &mut stat) };
        if result == 0 {
            Some((stat.f_bavail as u64) * (stat.f_frsize as u64))
        } else {
            None
        }
    }

    #[cfg(not(unix))]
    {
        None
    }
}

fn generate_support_report(
    checks: &[ValidationCheck],
    free_space: Option<u64>,
    filesystem: Option<&str>,
) -> String {
    let mut report = String::new();
    report.push_str("SD Card Health Report\n");
    report.push_str("====================\n\n");

    for check in checks {
        let status = if check.passed { "✓" } else { "✗" };
        report.push_str(&format!("{} {}\n", status, check.message));
    }

    if let Some(space) = free_space {
        report.push_str(&format!("\nFree Space: {}\n", format_bytes(space)));
    }

    if let Some(fs) = filesystem {
        report.push_str(&format!("Filesystem: {}\n", fs));
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_missing_sd_card() {
        let result = validate_installation("/nonexistent/path", false, "/Tools");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_sd_card() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        let result = validate_installation(sd_root.to_str().unwrap(), false, "/Tools").unwrap();

        assert!(!result.success);
        assert!(result.failed_count > 0);
        // Free space check may pass on real filesystems
        assert!(result.passed_count <= 1);
    }

    #[test]
    fn test_validate_with_base_files() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        // Create essential files
        fs::write(sd_root.join("minui.pak"), "test").unwrap();
        fs::write(sd_root.join("boot.sh"), "#!/bin/sh").unwrap();
        fs::write(sd_root.join("DMG.png"), "png").unwrap();
        fs::create_dir_all(sd_root.join("Tools")).unwrap();

        let result = validate_installation(sd_root.to_str().unwrap(), false, "/Tools").unwrap();

        assert!(result.success);
        assert_eq!(result.failed_count, 0);
        assert!(result.passed_count >= 4); // 3 base + Tools dir
    }

    #[test]
    fn test_validate_with_extras() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        // Create essential files
        fs::write(sd_root.join("minui.pak"), "test").unwrap();
        fs::write(sd_root.join("boot.sh"), "#!/bin/sh").unwrap();
        fs::write(sd_root.join("DMG.png"), "png").unwrap();

        // Create Tools with pak files
        fs::create_dir_all(sd_root.join("Tools")).unwrap();
        fs::write(sd_root.join("Tools/wifi.pak"), "wifi").unwrap();
        fs::write(sd_root.join("Tools/ssh.pak"), "ssh").unwrap();

        let result = validate_installation(sd_root.to_str().unwrap(), true, "/Tools").unwrap();

        assert!(result.success);
        assert_eq!(result.failed_count, 0);
        assert!(result.passed_count >= 5); // 3 base + Tools dir + pak count
    }

    #[test]
    fn test_validate_missing_tools_dir() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        // Create only some base files
        fs::write(sd_root.join("minui.pak"), "test").unwrap();

        let result = validate_installation(sd_root.to_str().unwrap(), false, "/Tools").unwrap();

        assert!(!result.success);
        let tools_check = result.checks.iter().find(|c| c.name.contains("Tools"));
        assert!(tools_check.is_some());
        assert!(!tools_check.unwrap().passed);
    }

    #[test]
    fn test_format_validation_report() {
        let result = ValidationResult {
            success: true,
            checks: vec![
                ValidationCheck {
                    name: "Check 1".to_string(),
                    passed: true,
                    message: "File exists".to_string(),
                },
                ValidationCheck {
                    name: "Check 2".to_string(),
                    passed: false,
                    message: "File missing".to_string(),
                },
            ],
            passed_count: 1,
            failed_count: 1,
            free_space_bytes: Some(1024 * 1024 * 500),
        };

        let report = format_validation_report(&result);
        assert!(report.contains("PASSED"));
        assert!(report.contains("File exists"));
        assert!(report.contains("File missing"));
        assert!(report.contains("500.00 MB"));
    }

    #[test]
    fn test_check_sd_card_health_nonexistent() {
        let result = check_sd_card_health("/nonexistent/path", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_sd_card_health_empty_card() {
        let temp = tempfile::tempdir().unwrap();
        let result = check_sd_card_health(temp.path().to_str().unwrap(), None);
        assert!(result.is_ok());

        let health = result.unwrap();
        // Should have checks for missing folders
        assert!(health
            .checks
            .iter()
            .any(|c| c.name == "folder_apps" && !c.passed));
        assert!(health
            .checks
            .iter()
            .any(|c| c.name == "folder_tools" && !c.passed));
        assert!(!health.support_report.is_empty());
    }

    #[test]
    fn test_check_sd_card_health_with_folders() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        std::fs::create_dir_all(sd_root.join("Apps")).unwrap();
        std::fs::create_dir_all(sd_root.join("Tools")).unwrap();
        std::fs::create_dir_all(sd_root.join("Emus")).unwrap();

        let result = check_sd_card_health(sd_root.to_str().unwrap(), None);
        assert!(result.is_ok());

        let health = result.unwrap();
        assert!(health
            .checks
            .iter()
            .any(|c| c.name == "folder_apps" && c.passed));
        assert!(health
            .checks
            .iter()
            .any(|c| c.name == "folder_tools" && c.passed));
        assert!(health
            .checks
            .iter()
            .any(|c| c.name == "folder_emus" && c.passed));
    }
}
