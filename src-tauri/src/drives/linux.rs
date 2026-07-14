//! Linux (and other non-macOS, non-Windows) implementation of
//! drive detection and formatting. Covers Linux, FreeBSD, and any
//! other target that supports `lsblk` with JSON output.
use std::process::Command;

use super::{DriveDetector, RemovableDrive};
use crate::fs_utils;

/// Linux implementation of drive detection and formatting.
pub struct LinuxDetector;

impl DriveDetector for LinuxDetector {
    fn list(&self) -> Result<Vec<RemovableDrive>, String> {
        let output = Command::new("lsblk")
            .args(["-o", "NAME,SIZE,FSTYPE,MOUNTPOINT,RM", "-ln", "-J"])
            .output()
            .map_err(|e| format!("Failed to run lsblk: {}", e))?;

        if !output.status.success() {
            return Err("lsblk command failed".to_string());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value = serde_json::from_str(&stdout)
            .map_err(|e| format!("Failed to parse lsblk output: {}", e))?;

        let devices = match json["blockdevices"].as_array() {
            Some(d) => d,
            None => return Ok(Vec::new()),
        };

        let mut drives = Vec::new();
        for device in devices {
            let rm = &device["rm"];
            let is_removable = match rm {
                serde_json::Value::String(s) => s == "1" || s == "true",
                serde_json::Value::Number(n) => n.as_u64() == Some(1),
                serde_json::Value::Bool(b) => *b,
                _ => false,
            };
            if !is_removable {
                continue;
            }

            let mountpoint = device["mountpoint"].as_str().unwrap_or("");
            if mountpoint.is_empty() {
                continue;
            }
            let name = device["name"].as_str().unwrap_or(mountpoint).to_string();
            let filesystem = device["fstype"].as_str().map(|s| s.to_string());
            let size_str = device["size"].as_str().unwrap_or("0");
            let size_bytes = size_str.parse::<u64>().ok();
            let available = fs_utils::get_free_space(mountpoint);

            drives.push(RemovableDrive {
                name,
                mount_path: mountpoint.to_string(),
                size_bytes,
                filesystem,
                available_bytes: available,
            });
        }

        if drives.is_empty() {
            return Err("No removable volumes found".to_string());
        }

        Ok(drives)
    }

    fn format(&self, _mount_path: &str, _volume_name: &str) -> Result<(), String> {
        Err("Formatting is not yet supported on this platform".to_string())
    }
}
