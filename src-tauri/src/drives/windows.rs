use std::process::Command;

use super::{DriveDetector, RemovableDrive};

/// Windows implementation of drive detection and formatting.
pub struct WindowsDetector;

impl DriveDetector for WindowsDetector {
    fn list(&self) -> Result<Vec<RemovableDrive>, String> {
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

    fn format(&self, _mount_path: &str, _volume_name: &str) -> Result<(), String> {
        Err("Formatting is not yet supported on Windows".to_string())
    }
}
