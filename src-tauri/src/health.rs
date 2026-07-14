use std::path::Path;

use crate::fs_utils;
use crate::validate::{format_bytes, ValidationCheck};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthCheckResult {
    pub checks: Vec<ValidationCheck>,
    pub passed_count: u32,
    pub failed_count: u32,
    pub free_space_bytes: Option<u64>,
    pub filesystem: Option<String>,
    pub support_report: String,
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
    }

    // Check free space
    let free_space = fs_utils::get_free_space(sd_mount);
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
    }

    // Check for MinUI folders (Apps is optional — not all releases create it)
    let minui_folders = ["Tools", "Emus"];
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
            }
            // Don't flag missing optional PAKs as failures
        }
    }

    let passed_count = checks.iter().filter(|c| c.passed).count() as u32;
    let failed_count = checks.iter().filter(|c| !c.passed).count() as u32;

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

#[allow(unused_variables)]
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
            .any(|c| c.name == "folder_tools" && !c.passed));
        assert!(!health.support_report.is_empty());
    }

    #[test]
    fn test_check_sd_card_health_with_folders() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        std::fs::create_dir_all(sd_root.join("Tools")).unwrap();
        std::fs::create_dir_all(sd_root.join("Emus")).unwrap();

        let result = check_sd_card_health(sd_root.to_str().unwrap(), None);
        assert!(result.is_ok());

        let health = result.unwrap();
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
