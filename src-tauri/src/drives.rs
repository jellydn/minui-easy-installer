use serde::Serialize;
use std::process::Command;

#[derive(Debug, Serialize, Clone)]
pub struct RemovableDrive {
    pub name: String,
    pub mount_path: String,
    pub size_bytes: Option<u64>,
    pub filesystem: Option<String>,
    pub available_bytes: Option<u64>,
}

#[cfg(target_os = "macos")]
pub fn list_removable_drives() -> Result<Vec<RemovableDrive>, String> {
    let output = Command::new("diskutil")
        .args(["list", "external"])
        .output()
        .map_err(|e| format!("Failed to run diskutil: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("diskutil failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut drives = Vec::new();

    let mut current_mount = String::new();
    let mut current_fs = String::new();
    let mut current_size: Option<u64> = None;

    for line in stdout.lines() {
        let trimmed = line.trim();

        if let Some(mount) = trimmed.strip_prefix("*: ") {
            if !current_mount.is_empty() {
                drives.push(RemovableDrive {
                    name: current_mount
                        .rsplit('/')
                        .next()
                        .unwrap_or("Unknown")
                        .to_string(),
                    mount_path: current_mount.clone(),
                    size_bytes: current_size,
                    filesystem: if current_fs.is_empty() {
                        None
                    } else {
                        Some(current_fs.clone())
                    },
                    available_bytes: None,
                });
            }
            current_mount = mount.to_string();
            current_fs.clear();
            current_size = None;
        } else if let Some(fs) = trimmed.strip_prefix("File System: ") {
            current_fs = fs.to_string();
        } else if let Some(size_str) = trimmed.strip_prefix("Volume Size: ") {
            let size_str = size_str.trim();
            if let Some(bytes) = parse_size_str(size_str) {
                current_size = Some(bytes);
            }
        }
    }

    if !current_mount.is_empty() {
        drives.push(RemovableDrive {
            name: current_mount
                .rsplit('/')
                .next()
                .unwrap_or("Unknown")
                .to_string(),
            mount_path: current_mount,
            size_bytes: current_size,
            filesystem: if current_fs.is_empty() {
                None
            } else {
                Some(current_fs)
            },
            available_bytes: None,
        });
    }

    if drives.is_empty() {
        list_removable_drives_df()
    } else {
        Ok(drives)
    }
}

#[cfg(target_os = "macos")]
fn parse_size_str(s: &str) -> Option<u64> {
    let s = s.trim();
    let (num_str, unit) = if let Some(pos) = s.find(char::is_alphabetic) {
        s.split_at(pos)
    } else {
        (s, "")
    };

    let num: f64 = num_str.trim().parse().ok()?;
    let multiplier = match unit.trim().to_lowercase().as_str() {
        "bytes" | "b" => 1.0,
        "kb" | "k" => 1024.0,
        "mb" | "m" => 1024.0 * 1024.0,
        "gb" | "g" => 1024.0 * 1024.0 * 1024.0,
        "tb" | "t" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };

    Some((num * multiplier) as u64)
}

#[cfg(target_os = "macos")]
fn list_removable_drives_df() -> Result<Vec<RemovableDrive>, String> {
    let output = Command::new("df")
        .args(["-k"])
        .output()
        .map_err(|e| format!("Failed to run df: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("df failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut drives = Vec::new();

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 6 {
            let filesystem = parts[0].to_string();
            let mount_path = parts[5].to_string();
            let size_kb: u64 = parts[1].parse().unwrap_or(0);
            let available_kb: u64 = parts[3].parse().unwrap_or(0);

            if mount_path.starts_with("/Volumes/") {
                let name = mount_path
                    .rsplit('/')
                    .next()
                    .unwrap_or("Unknown")
                    .to_string();

                drives.push(RemovableDrive {
                    name,
                    mount_path,
                    size_bytes: Some(size_kb * 1024),
                    filesystem: Some(filesystem),
                    available_bytes: Some(available_kb * 1024),
                });
            }
        }
    }

    Ok(drives)
}

#[cfg(target_os = "windows")]
pub fn list_removable_drives() -> Result<Vec<RemovableDrive>, String> {
    let output = Command::new("powershell")
        .args([
            "-Command",
            "Get-CimInstance -ClassName Win32_LogicalDisk | Where-Object { $_.DriveType -eq 2 } | Select-Object DeviceID, FileSystem, FreeSpace, Size, VolumeName | ConvertTo-Json",
        ])
        .output()
        .map_err(|e| format!("Failed to run powershell: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("powershell failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_val: serde_json::Value =
        serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let mut drives = Vec::new();
    let items = match &json_val {
        serde_json::Value::Array(arr) => arr.clone(),
        serde_json::Value::Object(_) => vec![json_val.clone()],
        _ => vec![],
    };

    for item in &items {
        let mount_path = item
            .get("DeviceID")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let filesystem = item
            .get("FileSystem")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let available_bytes = item.get("FreeSpace").and_then(|v| v.as_u64());
        let size_bytes = item.get("Size").and_then(|v| v.as_u64());
        let name = item
            .get("VolumeName")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if !mount_path.is_empty() {
            drives.push(RemovableDrive {
                name: if name.is_empty() {
                    mount_path.trim_end_matches('\\').to_string()
                } else {
                    name
                },
                mount_path,
                size_bytes,
                filesystem,
                available_bytes,
            });
        }
    }

    Ok(drives)
}

#[cfg(not(target_os = "macos"))]
#[cfg(not(target_os = "windows"))]
pub fn list_removable_drives() -> Result<Vec<RemovableDrive>, String> {
    Err("Unsupported platform for drive detection".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_removable_drive_serialization() {
        let drive = RemovableDrive {
            name: "SD_CARD".to_string(),
            mount_path: "/Volumes/SD_CARD".to_string(),
            size_bytes: Some(32_000_000_000),
            filesystem: Some("FAT32".to_string()),
            available_bytes: Some(28_000_000_000),
        };

        let json = serde_json::to_string(&drive).unwrap();
        assert!(json.contains("SD_CARD"));
        assert!(json.contains("/Volumes/SD_CARD"));
        assert!(json.contains("FAT32"));
    }

    #[test]
    fn test_removable_drive_missing_filesystem() {
        let drive = RemovableDrive {
            name: "UNKNOWN".to_string(),
            mount_path: "/Volumes/UNKNOWN".to_string(),
            size_bytes: None,
            filesystem: None,
            available_bytes: None,
        };

        let json = serde_json::to_string(&drive).unwrap();
        assert!(json.contains("UNKNOWN"));
        assert!(json.contains("null"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_parse_size_str() {
        assert_eq!(parse_size_str("1024 bytes"), Some(1024));
        assert_eq!(parse_size_str("1 KB"), Some(1024));
        assert_eq!(parse_size_str("1.5 MB"), Some(1_572_864));
        assert_eq!(parse_size_str("32 GB"), Some(34_359_738_368));
        assert!(parse_size_str("invalid").is_none());
    }
}
