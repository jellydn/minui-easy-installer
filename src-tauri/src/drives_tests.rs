use std::path::Path;
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
fn test_classify_volume_internal_sd_card_reader() {
    // MacBook built-in SD card reader: the reader is internal
    // but the SD card media is removable. Must be classified as
    // External so list_removable_drives() includes it.
    let info = "   Device Location:        Internal\n\
                Removable Media:        Removable\n\
                Protocol:               Secure Digital\n";
    assert_eq!(classify_volume(info), VolumeKind::External);
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
    assert_eq!(find_field_value(info, "Device Location"), Some("External"));
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
    assert_eq!(find_field_value(info, "Device Location"), Some("External"));
}

#[cfg(target_os = "macos")]
#[test]
fn test_parse_filesystem_fat32() {
    let info = "   File System Personality:  MS-DOS FAT32\n";
    assert_eq!(
        parse_filesystem_from_info(info),
        Some("MS-DOS FAT32".to_string())
    );
}

#[cfg(target_os = "macos")]
#[test]
fn test_parse_filesystem_apfs() {
    let info = "   File System Personality:    APFS\n";
    assert_eq!(parse_filesystem_from_info(info), Some("APFS".to_string()));
}

#[cfg(target_os = "macos")]
#[test]
fn test_parse_filesystem_missing() {
    let info = "   Volume Name:        MinUI\n";
    assert_eq!(parse_filesystem_from_info(info), None);
}

#[cfg(target_os = "macos")]
#[test]
fn test_parse_filesystem_from_real_sd_card_output() {
    // Real `diskutil info` output snippet from /Volumes/MinUI.
    let info = "   File System Personality:  MS-DOS FAT32\n\
                Type (Bundle):            msdos\n\
                Name (User Visible):      MS-DOS (FAT32)\n";
    assert_eq!(
        parse_filesystem_from_info(info),
        Some("MS-DOS FAT32".to_string())
    );
}

/// Integration test against a real mounted SD card.
///
/// This test calls the production `list_removable_drives()` function —
/// which shells out to `df` and `diskutil` — and asserts that the SD
/// card mounted at the configured mount path is returned with the
/// expected fields. It guards against regressions of the bug where
/// the detection logic excluded real SD cards because `diskutil info`
/// output does not contain an `Internal:` field for removable media.
///
/// Marked `#[ignore]` because it requires physical hardware (an SD card
/// inserted into the Mac) and a specific mount path. To run:
///
/// ```sh
/// cargo test --lib drives -- --ignored list_removable_drives_finds_real_sd_card
/// ```
///
/// If the configured volume is not currently mounted, the test returns
/// early with a printed message instead of failing — making it safe to
/// run on a developer machine that doesn't have the SD card inserted
/// at the moment.
#[cfg(target_os = "macos")]
#[test]
#[ignore = "requires a real SD card mounted at the configured path; run with `cargo test -- --ignored`"]
fn test_list_removable_drives_finds_real_sd_card() {
    // The SD card volume name configuration.
    // Compared case-insensitively because macOS HFS+/APFS is
    // case-insensitive, so `Path::exists` check is case-insensitive.
    let sd_card_name = std::env::var("SD_CARD_NAME").unwrap_or_else(|_| "KNULLI".to_string());
    let sd_card_probe_path = std::env::var("SD_CARD_PROBE_PATH")
        .unwrap_or_else(|_| format!("/Volumes/{}", sd_card_name));

    if !Path::new(&sd_card_probe_path).exists() {
        eprintln!(
            "test_list_removable_drives_finds_real_sd_card: {sd_card_probe_path} \
             is not mounted on this machine — skipping assertion. \
             Insert the SD card and re-run to verify the fix end-to-end."
        );
        return;
    }

    let drives = list_removable_drives().unwrap_or_else(|err| {
        panic!(
            "list_removable_drives() failed with \"{err}\" even though \
             {sd_card_probe_path} is mounted. This is the regression we \
             are guarding against: the SD card should be detected."
        )
    });

    eprintln!(
        "list_removable_drives() returned {} drive(s):",
        drives.len()
    );
    for drive in &drives {
        eprintln!(
            "  - name={:?} mount={:?} fs={:?} size={:?} avail={:?}",
            drive.name, drive.mount_path, drive.filesystem, drive.size_bytes, drive.available_bytes
        );
    }

    // Match by volume name case-insensitively. `d.name` is derived
    // from the leaf of `d.mount_path`, so a single check is enough.
    let sd_card = drives
        .iter()
        .find(|d| d.name.eq_ignore_ascii_case(&sd_card_name))
        .unwrap_or_else(|| {
            panic!(
                "SD card with volume name {sd_card_name:?} (case-insensitive) \
                 was not returned by list_removable_drives(). This is the \
                 bug the test guards against: real SD cards were \
                 incorrectly excluded because their `diskutil info` \
                 output lacks an `Internal:` field."
            )
        });

    // The function's other contract: internal drives like
    // "Macintosh HD" must NOT appear in the results. Catches a
    // regression where someone "fixes" inclusion by dropping the
    // internal-drive exclusion entirely.
    for drive in &drives {
        assert!(
            !drive.name.starts_with("Macintosh HD"),
            "internal drive {:?} was incorrectly returned by \
             list_removable_drives() (mount={:?})",
            drive.name,
            drive.mount_path
        );
    }

    // The MinUI SD card is formatted as FAT32. The function reports
    // it as `MS-DOS FAT32` (matching the diskutil personality string).
    let fs = sd_card
        .filesystem
        .as_deref()
        .unwrap_or_else(|| panic!("filesystem should be populated for the SD card"));
    let fs_upper = fs.to_uppercase();
    assert!(
        fs_upper.contains("FAT32") || fs_upper.contains("MS-DOS"),
        "expected FAT32/MS-DOS filesystem for the SD card, got: {fs:?}"
    );

    // The `df`-derived free space should be populated and smaller than
    // the total size.
    let size = sd_card
        .size_bytes
        .unwrap_or_else(|| panic!("size_bytes should be populated for the SD card"));
    let avail = sd_card
        .available_bytes
        .unwrap_or_else(|| panic!("available_bytes should be populated for the SD card"));
    assert!(
        avail <= size,
        "available_bytes ({avail}) should be <= size_bytes ({size})"
    );
    assert!(size > 0, "size_bytes should be > 0 for a real SD card");
}
