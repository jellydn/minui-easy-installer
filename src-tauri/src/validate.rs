use std::fmt::Write;
use std::fs;
use std::path::Path;

use crate::fs_utils;
use crate::platform::{device_base_item, KNOWN_DEVICE_BASE_ITEMS};

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
    /// The device-specific path that was validated (e.g. "miyoo", "rg35xxplus", "em_ui.sh").
    pub device_path: String,
    /// Warning message when multiple device folders are present on the SD card.
    pub multiple_device_folders_warning: Option<String>,
}


/// Detect device-specific folders/files present at the SD card root.
fn detect_device_base_items(sd_root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    for item in KNOWN_DEVICE_BASE_ITEMS {
        let path = sd_root.join(item);
        if path.exists() {
            found.push((*item).to_string());
        }
    }
    found
}

/// Returns a warning if more than one device-specific folder/file is present
/// at the SD card root.
fn check_multiple_device_folders(sd_root: &Path) -> Option<String> {
    let detected_device_items = detect_device_base_items(sd_root);
    if detected_device_items.len() > 1 {
        Some(format!(
            "Multiple device folders found: {}. Only the folder for the selected device should be on the SD card.",
            detected_device_items.join(", ")
        ))
    } else {
        None
    }
}

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

/// Recursively count .pak directories under a given path.
fn count_pak_dirs(path: &Path) -> u32 {
    let mut count = 0u32;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if entry_path.is_dir() {
                if name.ends_with(".pak") {
                    count += 1;
                } else {
                    count += count_pak_dirs(&entry_path);
                }
            }
        }
    }
    count
}

fn check_pak_files(sd_root: &Path, dir: &str) -> Vec<ValidationCheck> {
    let mut checks = Vec::new();
    let dir_path = sd_root.join(dir.trim_start_matches('/'));

    if !dir_path.exists() {
        checks.push(ValidationCheck {
            name: format!("{} directory", dir),
            passed: false,
            message: format!("Missing {} directory", dir),
        });
        return checks;
    }

    let pak_count = count_pak_dirs(&dir_path);

    checks.push(ValidationCheck {
        name: format!("PAK packages in {}", dir),
        passed: pak_count > 0,
        message: if pak_count > 0 {
            format!("Found {} PAK package(s) in {}", pak_count, dir)
        } else {
            format!("No PAK packages found in {}", dir)
        },
    });

    checks
}

pub fn format_bytes(bytes: u64) -> String {
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
    platform: &str,
    has_extras: bool,
    extras_dir: &str,
) -> Result<ValidationResult, String> {
    let sd_root = Path::new(sd_mount);

    if !sd_root.exists() {
        return Err(format!("SD card mount path does not exist: {}", sd_mount));
    }

    let device_item = device_base_item(platform);

    // Check essential base paths
    let mut checks = vec![
        check_path_exists(sd_root, "minui.txt"),
        check_path_exists(sd_root, "MinUI.zip"),
        check_path_exists(sd_root, device_item),
    ];

    // Check Tools directory exists
    checks.push(check_directory_exists(sd_root, extras_dir));

    // Check PAK files if extras were installed (PAKs are inside Tools/<platform>/ or Emus/<platform>/)
    if has_extras {
        let tools_pak = check_pak_files(sd_root, "Tools");
        let emus_pak = check_pak_files(sd_root, "Emus");
        checks.extend(tools_pak);
        checks.extend(emus_pak);
    }

    // Check free space
    let free_space = sd_root.to_str().and_then(fs_utils::get_free_space);
    if let Some(space) = free_space {
        checks.push(ValidationCheck {
            name: "Free space".to_string(),
            passed: space > 100 * 1024 * 1024, // At least 100MB free
            message: format!("Available: {}", format_bytes(space)),
        });
    }

    let passed_count = checks.iter().filter(|c| c.passed).count() as u32;
    let failed_count = checks.iter().filter(|c| !c.passed).count() as u32;

    // Warn if multiple device folders are present on the SD card.
    let multiple_device_folders_warning = check_multiple_device_folders(sd_root);

    Ok(ValidationResult {
        success: failed_count == 0,
        checks,
        passed_count,
        failed_count,
        free_space_bytes: free_space,
        device_path: device_item.to_string(),
        multiple_device_folders_warning,
    })
}

pub fn format_validation_report(result: &ValidationResult) -> String {
    let mut report = String::new();
    let _ = writeln!(report, "MinUI Installation Validation Report");
    let _ = writeln!(report, "=====================================");
    let _ = writeln!(report);

    if result.success {
        let _ = writeln!(report, "Status: PASSED");
    } else {
        let _ = writeln!(report, "Status: FAILED");
    }
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "Checks: {} passed, {} failed",
        result.passed_count, result.failed_count
    );
    let _ = writeln!(report);

    let _ = writeln!(report, "Details:");
    for check in &result.checks {
        let status = if check.passed { "✓" } else { "✗" };
        let _ = writeln!(report, "  {} {}", status, check.message);
    }

    if let Some(space) = result.free_space_bytes {
        let _ = writeln!(report);
        let _ = write!(report, "Free Space: {}", format_bytes(space));
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_missing_sd_card() {
        let result = validate_installation("/nonexistent/path", "miyoo", false, "/Tools");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_empty_sd_card() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        let result = validate_installation(sd_root.to_str().unwrap(), "miyoo", false, "/Tools").unwrap();

        assert!(!result.success);
        assert!(result.failed_count > 0);
        // Free space check may pass on real filesystems
        assert!(result.passed_count <= 1);
    }

    #[test]
    fn test_validate_with_base_files() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        // Create essential files (MinUI install creates MinUI.zip + minui.txt + device folder at root)
        fs::write(sd_root.join("MinUI.zip"), "archive").unwrap();
        fs::write(sd_root.join("minui.txt"), "MinUI 2025.01.01").unwrap();
        fs::create_dir_all(sd_root.join("miyoo")).unwrap();
        fs::create_dir_all(sd_root.join("Tools")).unwrap();

        let result = validate_installation(sd_root.to_str().unwrap(), "miyoo", false, "/Tools").unwrap();

        assert!(result.success);
        assert_eq!(result.failed_count, 0);
        assert!(result.passed_count >= 4); // minui.txt + MinUI.zip + miyoo + Tools dir
    }

    #[test]
    fn test_validate_with_extras() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        // Create essential files
        fs::write(sd_root.join("MinUI.zip"), "archive").unwrap();
        fs::write(sd_root.join("minui.txt"), "MinUI 2025.01.01").unwrap();
        fs::create_dir_all(sd_root.join("rg35xxplus")).unwrap();

        // Create Tools with pak directories (MinUI uses .pak dirs per platform)
        fs::create_dir_all(sd_root.join("Tools/rg35xxplus/wifi.pak")).unwrap();
        fs::create_dir_all(sd_root.join("Emus/rg35xxplus/mgba.pak")).unwrap();

        let result = validate_installation(sd_root.to_str().unwrap(), "rg35xxplus", true, "/Tools").unwrap();

        assert!(result.success);
        assert_eq!(result.failed_count, 0);
        assert!(result.passed_count >= 5); // minui.txt + MinUI.zip + rg35xxplus + Tools dir + pak count
    }

    #[test]
    fn test_validate_missing_tools_dir() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        // Create only some base files
        fs::write(sd_root.join("MinUI.zip"), "archive").unwrap();
        fs::write(sd_root.join("minui.txt"), "MinUI 2025.01.01").unwrap();
        fs::create_dir_all(sd_root.join("miyoo")).unwrap();

        let result = validate_installation(sd_root.to_str().unwrap(), "miyoo", false, "/Tools").unwrap();

        assert!(!result.success);
        let tools_check = result.checks.iter().find(|c| c.name.contains("Tools"));
        assert!(tools_check.is_some());
        assert!(!tools_check.unwrap().passed);
    }

    #[test]
    fn test_validate_m17_uses_em_ui_sh() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        fs::write(sd_root.join("MinUI.zip"), "archive").unwrap();
        fs::write(sd_root.join("minui.txt"), "MinUI 2025.01.01").unwrap();
        fs::write(sd_root.join("em_ui.sh"), "#!/bin/sh").unwrap();
        fs::create_dir_all(sd_root.join("Tools")).unwrap();

        let result = validate_installation(sd_root.to_str().unwrap(), "m17", false, "/Tools").unwrap();

        assert!(result.success);
        let em_ui_check = result.checks.iter().find(|c| c.name.contains("em_ui.sh"));
        assert!(em_ui_check.is_some());
        assert!(em_ui_check.unwrap().passed);
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
            device_path: "miyoo".to_string(),
            multiple_device_folders_warning: None,
        };

        let report = format_validation_report(&result);
        assert!(report.contains("PASSED"));
        assert!(report.contains("File exists"));
        assert!(report.contains("File missing"));
        assert!(report.contains("500.00 MB"));
    }

    #[test]
    fn test_detect_multiple_device_folders_warning() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        fs::write(sd_root.join("MinUI.zip"), "archive").unwrap();
        fs::write(sd_root.join("minui.txt"), "MinUI 2025.01.01").unwrap();
        fs::create_dir_all(sd_root.join("miyoo")).unwrap();
        fs::create_dir_all(sd_root.join("trimui")).unwrap();
        fs::create_dir_all(sd_root.join("Tools")).unwrap();

        let result = validate_installation(sd_root.to_str().unwrap(), "miyoo", false, "/Tools").unwrap();

        assert!(result.success);
        assert!(result.multiple_device_folders_warning.is_some());
        let warning = result.multiple_device_folders_warning.unwrap();
        assert!(warning.contains("miyoo"));
        assert!(warning.contains("trimui"));
    }

    #[test]
    fn test_no_warning_with_single_device_folder() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        fs::write(sd_root.join("MinUI.zip"), "archive").unwrap();
        fs::write(sd_root.join("minui.txt"), "MinUI 2025.01.01").unwrap();
        fs::create_dir_all(sd_root.join("miyoo")).unwrap();
        fs::create_dir_all(sd_root.join("Tools")).unwrap();

        let result = validate_installation(sd_root.to_str().unwrap(), "miyoo", false, "/Tools").unwrap();

        assert!(result.success);
        assert!(result.multiple_device_folders_warning.is_none());
        assert_eq!(result.device_path, "miyoo");
    }
}
