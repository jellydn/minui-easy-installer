use serde::Serialize;
#[cfg(target_os = "macos")]
use std::path::Path;
use std::process::Command;

use crate::fs_utils;

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
    // Use df to find volumes under /Volumes/, then verify they are removable/external
    // using diskutil info.
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
        if parts.len() < 6 {
            continue;
        }
        let mount_path = parts.last().unwrap_or(&"");
        if !mount_path.starts_with("/Volumes/") {
            continue;
        }

        // Exclude known internal/system volumes by name
        let name = Path::new(mount_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if name == "Macintosh HD" || name.starts_with("Macintosh HD") {
            continue;
        }

        // Use diskutil info to check if this volume is on a physical disk
        // and was not synthesised from a disk image or network mount
        let is_external = is_removable_volume(mount_path);

        // Skip internal drives — only include confirmed removable/external
        if !is_external {
            continue;
        }

        let filesystem = get_filesystem(mount_path);
        let available = parts[3].parse::<u64>().ok().map(|k| k * 1024);
        let size = fs_utils::get_disk_space(mount_path).map(|ds| ds.total);

        drives.push(RemovableDrive {
            name,
            mount_path: mount_path.to_string(),
            size_bytes: size,
            filesystem,
            available_bytes: available,
        });
    }

    if drives.is_empty() {
        return Err("No removable volumes found".to_string());
    }

    Ok(drives)
}

#[cfg(target_os = "macos")]
#[derive(Debug, PartialEq, Eq)]
enum VolumeKind {
    External,
    Internal,
    DiskImage,
    Network,
    Unknown,
}

/// Look up a field's value in `diskutil info` output.
///
/// `diskutil info` produces column-aligned output where the field name and
/// value are separated by a variable run of spaces (e.g.
/// `   File System Personality:  MS-DOS FAT32`). This helper tolerates that
/// layout by splitting each line on the first `:`, trimming the key for
/// comparison, and returning the trimmed value as a borrow into the input.
///
/// Returns `None` if the field is absent.
#[cfg(target_os = "macos")]
fn find_field_value<'a>(info: &'a str, field: &str) -> Option<&'a str> {
    for line in info.lines() {
        if let Some((key, value)) = line.split_once(':') {
            if key.trim() == field {
                return Some(value.trim());
            }
        }
    }
    None
}

/// Classify a `diskutil info` output into a high-level volume kind.
///
/// This is split out from `is_removable_volume` so the parsing logic can be
/// unit-tested against known-good and known-bad samples of `diskutil` output.
#[cfg(target_os = "macos")]
fn classify_volume(info: &str) -> VolumeKind {
    let network = find_field_value(info, "Network Volume");
    let disk_image = find_field_value(info, "Disk Image");
    let virtual_disk = find_field_value(info, "Virtual");
    let device_location = find_field_value(info, "Device Location");
    let internal = find_field_value(info, "Internal");
    let removable_media = find_field_value(info, "Removable Media");
    let removable_or_external = find_field_value(info, "Removable Media Or External Device");

    let is_yes = |v: Option<&str>| v == Some("Yes");

    // Exclusions first: even if other fields suggest external, never treat
    // disk images, virtual disks, or network mounts as removable media.
    if is_yes(network) {
        return VolumeKind::Network;
    }
    if is_yes(disk_image) || is_yes(virtual_disk) {
        return VolumeKind::DiskImage;
    }

    // `Device Location:` is the most reliable signal — `diskutil` writes
    // `External` for SD cards and USB sticks, and `Internal` for the boot
    // disk and built-in SSDs. Absent from some legacy / non-physical outputs.
    if device_location == Some("External") {
        return VolumeKind::External;
    }

    // Removable media takes priority over Device Location.
    // Built-in SD card readers report Device Location: Internal,
    // but Removable Media: Removable — the media IS removable.
    if removable_media == Some("Removable")
        || is_yes(removable_media)
        || is_yes(removable_or_external)
    {
        return VolumeKind::External;
    }

    // Not external and not removable — classify as internal.
    if device_location == Some("Internal") || is_yes(internal) {
        return VolumeKind::Internal;
    }

    VolumeKind::Unknown
}

#[cfg(target_os = "macos")]
fn is_removable_volume(mount_path: &str) -> bool {
    use std::process::Command;

    let output = match Command::new("diskutil").args(["info", mount_path]).output() {
        Ok(o) if o.status.success() => o,
        _ => return false,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    matches!(classify_volume(&stdout), VolumeKind::External)
}

/// Format a drive to FAT32 on macOS using diskutil.
///
/// # Arguments
/// * `mount_path` - Mount path of the volume to format (e.g. `/Volumes/NEXT28`)
/// * `volume_name` - New name for the volume after formatting
///
/// WARNING: This destroys all data on the drive.
#[cfg(target_os = "macos")]
pub fn format_drive(mount_path: &str, volume_name: &str) -> Result<(), String> {
    // Find the disk identifier from the mount path
    let output = Command::new("diskutil")
        .args(["info", mount_path])
        .output()
        .map_err(|e| format!("Failed to get disk info: {}", e))?;

    if !output.status.success() {
        return Err("Unable to find disk information for the selected volume".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut disk_id: Option<String> = None;
    let mut part_of_whole: Option<String> = None;

    for line in stdout.lines() {
        if let Some(device) = line.strip_prefix("   Device Node:") {
            disk_id = Some(device.trim().to_string());
        }
        if let Some(whole) = line.strip_prefix("   Part of Whole:") {
            let val = whole.trim().to_string();
            if val != "No" {
                part_of_whole = Some(val);
            }
        }
    }

    let device = disk_id.ok_or("Could not determine device node")?;

    // If this is a partition, use the parent disk
    let target = if let Some(ref parent) = part_of_whole {
        parent.clone()
    } else {
        device
    };

    // Unmount the entire disk (all its partitions) first
    let unmount = Command::new("diskutil")
        .args(["unmountDisk", &target])
        .output()
        .map_err(|e| format!("Failed to unmount disk: {}", e))?;

    if !unmount.status.success() {
        let stderr = String::from_utf8_lossy(&unmount.stderr);
        return Err(format!("Failed to unmount disk: {}", stderr));
    }

    // Erase the disk with FAT32
    let format_name = volume_name.trim();
    let format_name = if format_name.is_empty() {
        "MINUI"
    } else {
        format_name
    };

    // Truncate to 11 chars (FAT32 volume label limit)
    let format_name: String = format_name.chars().take(11).collect();

    let result = Command::new("diskutil")
        .args(["eraseDisk", "FAT32", &format_name, &target])
        .output()
        .map_err(|e| format!("Failed to run diskutil: {}", e))?;

    if result.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        Err(format!("Format failed: {}", stderr))
    }
}

#[cfg(target_os = "windows")]
pub fn format_drive(_mount_path: &str, _volume_name: &str) -> Result<(), String> {
    Err("Formatting is not yet supported on Windows".to_string())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn format_drive(_mount_path: &str, _volume_name: &str) -> Result<(), String> {
    Err("Formatting is not yet supported on this platform".to_string())
}

#[cfg(target_os = "macos")]
fn parse_filesystem_from_info(info: &str) -> Option<String> {
    find_field_value(info, "File System Personality").map(|s| s.to_string())
}

#[cfg(target_os = "macos")]
fn get_filesystem(mount_path: &str) -> Option<String> {
    let output = Command::new("diskutil")
        .args(["info", mount_path])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_filesystem_from_info(&stdout)
}

#[cfg(target_os = "macos")]
#[allow(dead_code)]
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
    // Use lsblk with the RM (removable) column so we can filter out
    // internal drives. On Linux, internal SSDs/HDDs have RM=0 while SD
    // cards and USB sticks report RM=1.
    let output = Command::new("lsblk")
        .args(["-o", "NAME,SIZE,FSTYPE,MOUNTPOINT,RM", "-ln", "-J"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                if let Some(devices) = json["blockdevices"].as_array() {
                    let mut drives = Vec::new();
                    for device in devices {
                        // Only include removable devices (RM=1 or "true").
                        let rm = device["rm"];
                        let is_removable = match rm {
                            Some(serde_json::Value::String(s)) => s == "1" || s == "true",
                            Some(serde_json::Value::Number(n)) => n.as_u64() == Some(1),
                            Some(serde_json::Value::Bool(b)) => *b,
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
                    if !drives.is_empty() {
                        return Ok(drives);
                    }
                }
            }
        }
    }

    Err("Unsupported platform for drive detection".to_string())
}

#[cfg(test)]
#[path = "drives_tests.rs"]
mod tests;
