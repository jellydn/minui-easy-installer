use serde::Serialize;
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
    let removable_or_external =
        find_field_value(info, "Removable Media Or External Device");

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
    match device_location {
        Some("External") => return VolumeKind::External,
        Some("Internal") => return VolumeKind::Internal,
        _ => {}
    }

    // Legacy fallback: explicit `Internal: Yes` / `Internal: true`.
    if is_yes(internal) {
        return VolumeKind::Internal;
    }

    // Secondary positive signals. The modern format is
    // `Removable Media: Removable` / `Fixed` / `Not Removable` (Catalina+).
    // Older versions used `Removable Media: Yes` / `No`.
    if removable_media == Some("Removable")
        || is_yes(removable_media)
        || is_yes(removable_or_external)
    {
        return VolumeKind::External;
    }

    VolumeKind::Unknown
}

#[cfg(target_os = "macos")]
fn is_removable_volume(mount_path: &str) -> bool {
    use std::process::Command;

    let output = match Command::new("diskutil")
        .args(["info", mount_path])
        .output()
    {
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
        // Unmount the partition first
        let unmount = Command::new("diskutil")
            .args(["unmount", &device])
            .output()
            .map_err(|e| format!("Failed to unmount: {}", e))?;

        if !unmount.status.success() {
            let stderr = String::from_utf8_lossy(&unmount.stderr);
            return Err(format!("Failed to unmount partition: {}", stderr));
        }

        parent.clone()
    } else {
        device
    };

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
fn get_filesystem(mount_path: &str) -> Option<String> {
    let output = Command::new("diskutil")
        .args(["info", mount_path])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("File System Personality:") {
            return line.split(':').nth(1).map(|s| s.trim().to_string());
        }
    }
    None
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

    #[cfg(target_os = "macos")]
    #[test]
    fn test_classify_volume_external_sd_card() {
        // Real `diskutil info` output for a FAT32 SD card mounted at
        // /Volumes/MinUI on macOS. Note the absence of `Internal:`,
        // `Virtual:`, `Disk Image:`, `Network Volume:`, and
        // `Removable Media Or External Device:` lines — the previous
        // implementation incorrectly defaulted missing keys to "internal".
        let info = "   Device Identifier:        disk6s1\n\
                    Device Node:              /dev/disk6s1\n\
                    Whole:                    No\n\
                    Part of Whole:            disk6\n\
                    \n\
                    Volume Name:              MinUI\n\
                    Mounted:                  Yes\n\
                    Mount Point:              /Volumes/MinUI\n\
                    \n\
                    Partition Type:           Windows_FAT_32\n\
                    File System Personality:  MS-DOS FAT32\n\
                    Type (Bundle):            msdos\n\
                    \n\
                    Protocol:                 USB\n\
                    \n\
                    Device Location:          External\n\
                    Removable Media:          Removable\n\
                    Media Removal:            Software-Activated\n\
                    Solid State:              Info not available\n";
        assert_eq!(classify_volume(info), VolumeKind::External);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_classify_volume_internal_drive() {
        let info = "   Device Location:        Internal\n\
                    Removable Media:        Fixed\n\
                    Internal:               Yes\n";
        assert_eq!(classify_volume(info), VolumeKind::Internal);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_classify_volume_legacy_internal_no_device_location() {
        // Older macOS or non-physical outputs may omit `Device Location:`.
        let info = "   Internal:               Yes\n\
                    Removable Media:        Fixed\n";
        assert_eq!(classify_volume(info), VolumeKind::Internal);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_classify_volume_disk_image() {
        let info = "   Device Location:        Internal\n\
                    Disk Image:             Yes\n\
                    Removable Media:        Fixed\n";
        assert_eq!(classify_volume(info), VolumeKind::DiskImage);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_classify_volume_network_mount() {
        let info = "   Network Volume:         Yes\n\
                    Mount Point:            /Volumes/SomeShare\n";
        assert_eq!(classify_volume(info), VolumeKind::Network);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_classify_volume_legacy_removable_yes() {
        // Pre-Catalina format used `Removable Media: Yes` / `No`.
        let info = "   Internal:               No\n\
                    Removable Media:        Yes\n";
        assert_eq!(classify_volume(info), VolumeKind::External);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_classify_volume_unknown_when_no_signals() {
        // Empty / unrecognized output should not be classified as external.
        let info = "";
        assert_eq!(classify_volume(info), VolumeKind::Unknown);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_find_field_value_column_aligned() {
        let info = "   File System Personality:  MS-DOS FAT32\n\
                    Volume Name:              MinUI\n\
                    Device Location:          External\n";
        assert_eq!(
            find_field_value(info, "File System Personality"),
            Some("MS-DOS FAT32")
        );
        assert_eq!(find_field_value(info, "Volume Name"), Some("MinUI"));
        assert_eq!(
            find_field_value(info, "Device Location"),
            Some("External")
        );
        assert_eq!(find_field_value(info, "Missing Field"), None);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_find_field_value_empty_input() {
        assert_eq!(find_field_value("", "Device Location"), None);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_find_field_value_no_colon() {
        // Lines without a colon should be skipped, not cause a panic.
        let info = "Some header line\n   Device Location: External\n";
        assert_eq!(find_field_value(info, "Device Location"), Some("External"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_find_field_value_substring_field_does_not_match() {
        // `Internal` should not match a field named `Internal Foo` via
        // `contains`, because we use `==` on the trimmed key.
        let info = "   Internal Foo:          Bar\n   Internal:               No\n";
        assert_eq!(find_field_value(info, "Internal"), Some("No"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_find_field_value_substring_prefix_does_not_match_for_filesystem() {
        // The same `==` invariant should hold for the filesystem field: a
        // hypothetical `File System Personality Or Other:` must not match a
        // lookup of `File System Personality`. Guards against regressions to
        // the previous `contains("Field:")` behavior in `get_filesystem`.
        let info = "   File System Personality Or Other:  X\n\
                    File System Personality:            MS-DOS FAT32\n";
        assert_eq!(
            find_field_value(info, "File System Personality"),
            Some("MS-DOS FAT32")
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_find_field_value_empty_value_returns_empty_string() {
        // A line like `   Field:   ` has an empty value after trim. The
        // helper should return `Some("")` (not `None`) so callers can
        // distinguish "field present, value empty" from "field absent".
        let info = "   Field:   \n   Other:  X\n";
        assert_eq!(find_field_value(info, "Field"), Some(""));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_find_field_value_first_match_wins_on_duplicate_field() {
        // If a field is repeated in the output, the first occurrence wins.
        // Documents the behavior so a future refactor doesn't silently
        // flip to last-wins.
        let info = "   Device Location:  External\n   Other:  Y\n\
                    Device Location:  Internal\n";
        assert_eq!(
            find_field_value(info, "Device Location"),
            Some("External")
        );
    }
}
