/// Classification of a diskutil-reported volume.
#[derive(Debug, PartialEq, Eq)]
pub enum VolumeKind {
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
pub fn find_field_value<'a>(info: &'a str, field: &str) -> Option<&'a str> {
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
pub fn classify_volume(info: &str) -> VolumeKind {
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

/// Parse the filesystem name from `diskutil info` output.
pub fn parse_filesystem_from_info(info: &str) -> Option<String> {
    find_field_value(info, "File System Personality").map(|s| s.to_string())
}

/// Parse a human-readable size string (e.g. "32 GB") into bytes.
#[allow(dead_code)]
pub fn parse_size_str(s: &str) -> Option<u64> {
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
