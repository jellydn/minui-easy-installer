use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::Instant;

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
    /// Sequential read speed in MB/s, if the benchmark ran.
    pub read_speed_mbs: Option<f64>,
}

/// Health check options, received from the frontend via Tauri IPC.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckOptions {
    pub sd_mount: String,
    pub device_platform: Option<String>,
}

/// Perform a comprehensive health check on the SD card.
///
/// Checks filesystem, free space, sequential read speed,
/// MinUI folders, and all installed PAK files.
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

    // Benchmark sequential read speed
    let read_speed_mbs = benchmark_read_speed(sd_mount, &mut checks);

    // Check for MinUI folders
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

    // Check for PAK files in Tools (recursive)
    let tools_dir = sd_root.join("Tools");
    if tools_dir.exists() {
        let pak_count = scan_pak_dirs(&tools_dir, &mut checks);
        if pak_count == 0 {
            checks.push(ValidationCheck {
                name: "pak_packages".to_string(),
                passed: false,
                message: "No PAK packages found in Tools/".to_string(),
            });
        }
    }

    let passed_count = checks.iter().filter(|c| c.passed).count() as u32;
    let failed_count = checks.iter().filter(|c| !c.passed).count() as u32;

    let support_report =
        generate_support_report(&checks, free_space, filesystem.as_deref(), read_speed_mbs);

    Ok(HealthCheckResult {
        checks,
        passed_count,
        failed_count,
        free_space_bytes: free_space,
        filesystem,
        support_report,
        read_speed_mbs,
    })
}

/// Benchmark sequential read speed by creating a test file and reading
/// it back in 1 MB chunks. Returns MB/s or None if the benchmark failed.
fn benchmark_read_speed(sd_mount: &str, checks: &mut Vec<ValidationCheck>) -> Option<f64> {
    let test_path = Path::new(sd_mount).join(".minui_health_bench.tmp");
    // Use 64 MB — large enough to overcome OS caching on first read,
    // small enough to not disrupt the user's SD card.
    const BENCH_SIZE: u64 = 64 * 1024 * 1024;

    // Write the test file
    let buf = vec![0u8; 1024 * 1024]; // 1 MB zero-filled
    let mut f = match fs::File::create(&test_path) {
        Ok(f) => f,
        Err(e) => {
            checks.push(ValidationCheck {
                name: "read_speed".to_string(),
                passed: false,
                message: format!("Read speed: could not create test file ({})", e),
            });
            return None;
        }
    };

    // Write the test file in 1 MB chunks. No BufWriter needed —
    // chunks are already large enough for efficient syscalls.
    {
        use std::io::Write;
        let mut remaining = BENCH_SIZE;
        while remaining > 0 {
            let chunk = buf.len().min(remaining as usize);
            if f.write_all(&buf[..chunk]).is_err() {
                checks.push(ValidationCheck {
                    name: "read_speed".to_string(),
                    passed: false,
                    message: "Read speed: could not write test file".to_string(),
                });
                let _ = fs::remove_file(&test_path);
                return None;
            }
            remaining -= chunk as u64;
        }
    }

    // Read the file back, measuring wall time.
    let mut file = match fs::File::open(&test_path) {
        Ok(f) => f,
        Err(e) => {
            checks.push(ValidationCheck {
                name: "read_speed".to_string(),
                passed: false,
                message: format!("Read speed: could not read test file ({})", e),
            });
            let _ = fs::remove_file(&test_path);
            return None;
        }
    };

    let start = Instant::now();
    let mut read_buf = vec![0u8; 1024 * 1024];
    let mut total_read = 0u64;
    loop {
        match file.read(&mut read_buf) {
            Ok(0) => break,
            Ok(n) => total_read += n as u64,
            Err(e) => {
                checks.push(ValidationCheck {
                    name: "read_speed".to_string(),
                    passed: false,
                    message: format!("Read speed: read error ({})", e),
                });
                let _ = fs::remove_file(&test_path);
                return None;
            }
        }
    }
    let elapsed = start.elapsed();

    // Clean up the test file regardless of outcome.
    let _ = fs::remove_file(&test_path);

    let seconds = elapsed.as_secs_f64();
    if seconds < 0.001 {
        return None; // too fast to measure reliably
    }
    let speed_mbs = (total_read as f64 / (1024.0 * 1024.0)) / seconds;

    // Flag cards slower than 5 MB/s (counterfeit/failing)
    let passed = speed_mbs >= 5.0;
    checks.push(ValidationCheck {
        name: "read_speed".to_string(),
        passed,
        message: format!(
            "Read speed: {:.1} MB/s ({} - {} than 5 MB/s threshold)",
            speed_mbs,
            if passed { "healthy" } else { "SLOW" },
            if passed { "faster" } else { "slower" },
        ),
    });

    Some(speed_mbs)
}

/// Recursively scan a directory for `.pak` directories (one level only).
/// Returns the count of PAK directories found and appends checks for each.
fn scan_pak_dirs(tools_dir: &Path, checks: &mut Vec<ValidationCheck>) -> usize {
    let entries = match fs::read_dir(tools_dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    let mut count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if path.is_dir() && name.ends_with(".pak") {
            count += 1;
            checks.push(ValidationCheck {
                name: format!("pak_{}", name.trim_end_matches(".pak")),
                passed: true,
                message: format!("{}: Installed", name),
            });
        }
    }
    count
}

#[cfg(target_os = "windows")]
fn detect_filesystem(sd_mount: &str) -> Option<String> {
    use std::process::Command;
    // fsutil reports "NTFS", "FAT32", etc. in the first line of output.
    let output = Command::new("fsutil")
        .arg("fsinfo")
        .arg("volumeinfo")
        .arg(sd_mount)
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("File System Name") {
                return Some(line.split(':').nth(1)?.trim().to_string());
            }
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn detect_filesystem(sd_mount: &str) -> Option<String> {
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

    None
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn detect_filesystem(_sd_mount: &str) -> Option<String> {
    None
}

fn generate_support_report(
    checks: &[ValidationCheck],
    free_space: Option<u64>,
    filesystem: Option<&str>,
    read_speed_mbs: Option<f64>,
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

    if let Some(speed) = read_speed_mbs {
        report.push_str(&format!("Read Speed: {:.1} MB/s\n", speed));
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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
        // Should have checks for missing folders and read speed
        assert!(health
            .checks
            .iter()
            .any(|c| c.name == "folder_tools" && !c.passed));
        assert!(health.checks.iter().any(|c| c.name == "read_speed"));
        assert!(!health.support_report.is_empty());
    }

    #[test]
    fn test_check_sd_card_health_with_folders() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        fs::create_dir_all(sd_root.join("Tools")).unwrap();
        fs::create_dir_all(sd_root.join("Emus")).unwrap();

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

    #[test]
    fn test_benchmark_read_speed_writes_and_cleans_up() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        // Should create and clean up the test file
        let result = check_sd_card_health(sd_root.to_str().unwrap(), None);
        assert!(result.is_ok());

        let health = result.unwrap();
        assert!(health.checks.iter().any(|c| c.name == "read_speed"));

        // The benchmark temp file should be cleaned up
        assert!(!sd_root.join(".minui_health_bench.tmp").exists());
    }

    #[test]
    fn test_scan_pak_dirs_discovers_recurively() {
        let temp = tempfile::tempdir().unwrap();
        let tools = temp.path().join("Tools");
        fs::create_dir_all(tools.join("wifi.pak")).unwrap();
        fs::create_dir_all(tools.join("ssh.pak")).unwrap();
        fs::create_dir_all(tools.join("custom.pak")).unwrap();
        // Not a .pak directory — should be skipped
        fs::create_dir_all(tools.join("readme.txt")).unwrap();

        let mut checks = Vec::new();
        let count = scan_pak_dirs(&tools, &mut checks);
        assert_eq!(count, 3);
        assert!(checks.iter().any(|c| c.name == "pak_wifi"));
        assert!(checks.iter().any(|c| c.name == "pak_ssh"));
        assert!(checks.iter().any(|c| c.name == "pak_custom"));
        // The .txt should not appear
        assert!(!checks.iter().any(|c| c.name == "pak_readme.txt"));
    }
}
