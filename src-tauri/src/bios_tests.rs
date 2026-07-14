use super::*;
use std::fs;
use std::os::unix::fs::symlink;

fn sgb_entry() -> BiosEntry {
    // "sgb_bios" lives at Bios/sgb.bios (no subdir).
    catalog().into_iter().find(|e| e.id == "sgb_bios").unwrap()
}

fn gb_entry() -> BiosEntry {
    catalog().into_iter().find(|e| e.id == "gb_bios").unwrap()
}

#[test]
fn test_catalog_contains_expected_ids() {
    let ids: Vec<String> = catalog().into_iter().map(|e| e.id).collect();
    for &required in EXPECTED_BIOS_IDS {
        assert!(
            ids.iter().any(|id| id == required),
            "missing {required} in catalog"
        );
    }
}

#[test]
fn test_catalog_filenames_match_issue_spec() {
    // The exact filenames in the issue body — regression guard so a
    // rename in the catalog doesn't silently break MinUI.
    let by_id = |id: &str| -> BiosEntry { catalog().into_iter().find(|e| e.id == id).unwrap() };
    assert_eq!(by_id("gb_bios").filename, "gb_bios.bin");
    assert_eq!(by_id("gbc_bios").filename, "gbc_bios.bin");
    assert_eq!(by_id("gba_bios").filename, "gba_bios.bin");
    assert_eq!(by_id("md_cd_e").filename, "bios_CD_E.bin");
    assert_eq!(by_id("md_cd_j").filename, "bios_CD_J.bin");
    assert_eq!(by_id("md_cd_u").filename, "bios_CD_U.bin");
    assert_eq!(by_id("ps_bios").filename, "psxonpsp660.bin");
    assert_eq!(by_id("pce_bios").filename, "syscard3.pce");
    assert_eq!(by_id("fc_disksys").filename, "disksys.rom");
    assert_eq!(by_id("pkm_bios").filename, "bios.min");
    assert_eq!(by_id("sgb_bios").filename, "sgb.bios");
    assert_eq!(by_id("dc_boot").filename, "dc_boot.bin");
    assert_eq!(by_id("dc_naomi").filename, "naomi.zip");
    assert_eq!(by_id("nds_bios7").filename, "bios7.bin");
    assert_eq!(by_id("nds_bios9").filename, "bios9.bin");
    assert_eq!(by_id("nds_firmware").filename, "firmware.bin");
}

#[test]
fn test_catalog_subdirs_match_issue_spec() {
    let by_id = |id: &str| -> BiosEntry { catalog().into_iter().find(|e| e.id == id).unwrap() };
    assert_eq!(by_id("gb_bios").subdir, "GB");
    assert_eq!(by_id("gbc_bios").subdir, "GBC");
    assert_eq!(by_id("gba_bios").subdir, "GBA");
    assert_eq!(by_id("md_cd_e").subdir, "MD");
    assert_eq!(by_id("ps_bios").subdir, "PS");
    assert_eq!(by_id("pce_bios").subdir, "PCE");
    assert_eq!(by_id("fc_disksys").subdir, "FC");
    assert_eq!(by_id("pkm_bios").subdir, "PKM");
    assert_eq!(by_id("sgb_bios").subdir, ""); // root
    assert_eq!(by_id("dc_boot").subdir, "DC");
    assert_eq!(by_id("dc_naomi").subdir, "DC");
    assert_eq!(by_id("nds_bios7").subdir, "NDS");
    assert_eq!(by_id("nds_bios9").subdir, "NDS");
    assert_eq!(by_id("nds_firmware").subdir, "NDS");
}

#[test]
fn test_target_path_for_subdir_entry() {
    let temp = tempfile::tempdir().unwrap();
    let path = target_path(temp.path(), &gb_entry());
    assert_eq!(
        path,
        temp.path().join("Bios").join("GB").join("gb_bios.bin")
    );
}

#[test]
fn test_target_path_for_root_entry() {
    let temp = tempfile::tempdir().unwrap();
    let path = target_path(temp.path(), &sgb_entry());
    assert_eq!(path, temp.path().join("Bios").join("sgb.bios"));
}

#[test]
fn test_safe_component_rejects_traversal_and_separators() {
    for bad in ["../etc", "..", ".", "a/b", "a\\b", "", "\0bad"] {
        assert!(
            safe_component(bad, "x").is_err(),
            "expected {bad:?} to be rejected"
        );
    }
    for good in ["a", "GB", "gba_bios.bin", "syscard3.pce"] {
        assert_eq!(safe_component(good, "x").unwrap(), good);
    }
}

#[test]
fn test_status_reports_missing_entries() {
    let temp = tempfile::tempdir().unwrap();
    let status = status(temp.path().to_str().unwrap()).unwrap();
    assert_eq!(status.entries.len(), catalog().len());
    assert_eq!(status.installed_count, 0);
    for entry in &status.entries {
        assert!(!entry.present, "{} should be missing", entry.entry.id);
    }
}

#[test]
fn test_status_reports_installed_entries() {
    let temp = tempfile::tempdir().unwrap();
    // Pretend the user has already dropped a GB BIOS and a SGB BIOS in.
    fs::create_dir_all(temp.path().join("Bios/GB")).unwrap();
    fs::write(temp.path().join("Bios/GB/gb_bios.bin"), b"x").unwrap();
    fs::write(temp.path().join("Bios/sgb.bios"), b"y").unwrap();

    let status = status(temp.path().to_str().unwrap()).unwrap();
    assert_eq!(status.installed_count, 2);
    let gb = status
        .entries
        .iter()
        .find(|e| e.entry.id == "gb_bios")
        .unwrap();
    assert!(gb.present);
    let sgb = status
        .entries
        .iter()
        .find(|e| e.entry.id == "sgb_bios")
        .unwrap();
    assert!(sgb.present);
}

#[test]
fn test_status_errors_on_missing_mount() {
    let result = status("/nonexistent/this/should/not/exist");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_install_writes_payload_to_expected_path() {
    let temp = tempfile::tempdir().unwrap();
    let payload = b"gb boot rom contents";
    let encoded = BASE64.encode(payload);

    let path = install_bios_from_bytes(temp.path().to_str().unwrap(), "gb_bios", &encoded).unwrap();

    let expected = temp.path().join("Bios/GB/gb_bios.bin");
    assert_eq!(path, expected.display().to_string());
    assert!(expected.exists());
    assert_eq!(fs::read(&expected).unwrap(), payload);
}

#[test]
fn test_install_root_subdir_entry_writes_correctly() {
    let temp = tempfile::tempdir().unwrap();
    let payload = b"sgb boot rom";
    let encoded = BASE64.encode(payload);

    let path =
        install_bios_from_bytes(temp.path().to_str().unwrap(), "sgb_bios", &encoded).unwrap();

    let expected = temp.path().join("Bios/sgb.bios");
    assert_eq!(path, expected.display().to_string());
    assert!(expected.exists());
}

#[test]
fn test_install_creates_missing_parent_dirs() {
    let temp = tempfile::tempdir().unwrap();
    let encoded = BASE64.encode(b"ps bios");

    install_bios_from_bytes(temp.path().to_str().unwrap(), "ps_bios", &encoded).unwrap();

    assert!(temp.path().join("Bios/PS").is_dir());
    assert!(temp.path().join("Bios/PS/psxonpsp660.bin").exists());
}

#[test]
fn test_install_overwrites_existing_file() {
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("Bios/GB/gb_bios.bin");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, b"old contents").unwrap();

    let new_bytes = b"new contents";
    install_bios_from_bytes(
        temp.path().to_str().unwrap(),
        "gb_bios",
        &BASE64.encode(new_bytes),
    )
    .unwrap();

    assert_eq!(fs::read(&target).unwrap(), new_bytes);
}

#[test]
fn test_install_errors_on_unknown_entry() {
    let temp = tempfile::tempdir().unwrap();
    let result = install_bios_from_bytes(
        temp.path().to_str().unwrap(),
        "definitely_not_a_real_id",
        &BASE64.encode(b"x"),
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown BIOS entry"));
}

#[test]
fn test_install_errors_on_invalid_base64() {
    let temp = tempfile::tempdir().unwrap();
    let result = install_bios_from_bytes(
        temp.path().to_str().unwrap(),
        "gb_bios",
        "this is not base64 !!!",
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid payload"));
}

#[test]
fn test_install_errors_on_empty_payload() {
    let temp = tempfile::tempdir().unwrap();
    let result = install_bios_from_bytes(
        temp.path().to_str().unwrap(),
        "gb_bios",
        &BASE64.encode(b""),
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Empty file payload"));
}

#[test]
fn test_install_errors_on_missing_mount() {
    let result = install_bios_from_bytes(
        "/nonexistent/this/should/not/exist",
        "gb_bios",
        &BASE64.encode(b"x"),
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
#[cfg(unix)]
fn test_install_rejects_symlink_escape() {
    // If the parent of the target (e.g. Bios/GB) is a symlink pointing
    // outside the SD card, the write must be rejected — we must not
    // follow the symlink and write to a real path outside the card.
    let temp = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let sd = temp.path();

    // The whole Bios/ directory is a symlink to outside.
    symlink(outside.path(), sd.join("Bios")).unwrap();

    let result = install_bios_from_bytes(sd.to_str().unwrap(), "gb_bios", &BASE64.encode(b"x"));
    assert!(result.is_err(), "expected symlink escape to be rejected");
    let err = result.unwrap_err();
    assert!(
        err.contains("Security violation") || err.contains("escapes SD card"),
        "got: {err}"
    );
}

#[test]
#[cfg(unix)]
fn test_install_rejects_leaf_symlink_escape() {
    // If the target file itself is a symlink pointing outside the SD
    // card, the write must not follow it. Removing the symlink before
    // writing ensures the new file is created directly on the SD card.
    let temp = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let sd = temp.path();

    let target_dir = sd.join("Bios/GB");
    fs::create_dir_all(&target_dir).unwrap();

    let target_file = target_dir.join("gb_bios.bin");
    let outside_file = outside.path().join("leak.bin");
    fs::write(&outside_file, b"original").unwrap();

    // Create a symlink at target pointing to outside
    symlink(&outside_file, &target_file).unwrap();

    let result =
        install_bios_from_bytes(sd.to_str().unwrap(), "gb_bios", &BASE64.encode(b"new_data"));

    assert!(result.is_ok());
    // Verify outside file was NOT modified/followed
    assert_eq!(fs::read(&outside_file).unwrap(), b"original");
    // Verify local file was written as a regular file
    assert_eq!(fs::read(&target_file).unwrap(), b"new_data");
    let meta = fs::symlink_metadata(&target_file).unwrap();
    assert!(
        !meta.file_type().is_symlink(),
        "Target file must be a regular file, not a symlink"
    );
}
