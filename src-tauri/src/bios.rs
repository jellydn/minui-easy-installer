//! BIOS file management.
//!
//! MinUI itself does not bundle BIOS files (they are copyrighted). The
//! installer is responsible only for copying BIOS files that the user
//! already owns into the right location on the SD card.
//!
//! The expected layout on the SD card is `Bios/<subdir>/<filename>`
//! (or `Bios/<filename>` for files that sit at the root, e.g. `sgb.bios`).
//! The catalog below is the source of truth for which entries we know
//! about and where each one goes.

use std::fs;
use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD as BASE64;

use crate::fs_utils;
use base64::Engine as _;

/// One BIOS file the user might want to install.
///
/// `subdir` is the directory under `Bios/`. Empty string means the file
/// lives at `Bios/<filename>` (e.g. `Bios/sgb.bios`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct BiosEntry {
    /// Stable id used by the frontend to refer to a specific entry.
    pub id: String,
    /// Subdirectory under `Bios/` on the SD card. Empty string = root.
    pub subdir: String,
    /// Filename on the SD card (must match the MinUI spec exactly).
    pub filename: String,
    /// Short, human-friendly description shown in the UI.
    pub description: String,
    /// Which emulator / system uses this BIOS file.
    pub system: String,
}

/// Catalog of known BIOS entries. Order matches the order shown in the UI.
pub fn catalog() -> Vec<BiosEntry> {
    vec![
        BiosEntry {
            id: "gb_bios".to_string(),
            subdir: "GB".to_string(),
            filename: "gb_bios.bin".to_string(),
            description: "Game Boy boot logo".to_string(),
            system: "Game Boy".to_string(),
        },
        BiosEntry {
            id: "gbc_bios".to_string(),
            subdir: "GBC".to_string(),
            filename: "gbc_bios.bin".to_string(),
            description: "Game Boy Color boot logo".to_string(),
            system: "Game Boy Color".to_string(),
        },
        BiosEntry {
            id: "gba_bios".to_string(),
            subdir: "GBA".to_string(),
            filename: "gba_bios.bin".to_string(),
            description: "Game Boy Advance boot logo".to_string(),
            system: "Game Boy Advance".to_string(),
        },
        BiosEntry {
            id: "md_cd_e".to_string(),
            subdir: "MD".to_string(),
            filename: "bios_CD_E.bin".to_string(),
            description: "Sega CD (Europe)".to_string(),
            system: "Sega CD".to_string(),
        },
        BiosEntry {
            id: "md_cd_j".to_string(),
            subdir: "MD".to_string(),
            filename: "bios_CD_J.bin".to_string(),
            description: "Sega CD (Japan)".to_string(),
            system: "Sega CD".to_string(),
        },
        BiosEntry {
            id: "md_cd_u".to_string(),
            subdir: "MD".to_string(),
            filename: "bios_CD_U.bin".to_string(),
            description: "Sega CD (USA)".to_string(),
            system: "Sega CD".to_string(),
        },
        BiosEntry {
            id: "ps_bios".to_string(),
            subdir: "PS".to_string(),
            filename: "psxonpsp660.bin".to_string(),
            description: "Sony PlayStation".to_string(),
            system: "PlayStation".to_string(),
        },
        BiosEntry {
            id: "pce_bios".to_string(),
            subdir: "PCE".to_string(),
            filename: "syscard3.pce".to_string(),
            description: "TurboGrafx CD system card".to_string(),
            system: "TurboGrafx CD".to_string(),
        },
        BiosEntry {
            id: "fc_disksys".to_string(),
            subdir: "FC".to_string(),
            filename: "disksys.rom".to_string(),
            description: "Famicom Disk System".to_string(),
            system: "Famicom".to_string(),
        },
        BiosEntry {
            id: "pkm_bios".to_string(),
            subdir: "PKM".to_string(),
            filename: "bios.min".to_string(),
            description: "Pokemon Mini".to_string(),
            system: "Pokemon Mini".to_string(),
        },
        BiosEntry {
            id: "sgb_bios".to_string(),
            subdir: "".to_string(),
            filename: "sgb.bios".to_string(),
            description: "Super Game Boy".to_string(),
            system: "Super Game Boy".to_string(),
        },
        BiosEntry {
            id: "dc_boot".to_string(),
            subdir: "DC".to_string(),
            filename: "dc_boot.bin".to_string(),
            description: "Dreamcast BIOS".to_string(),
            system: "Dreamcast".to_string(),
        },
        BiosEntry {
            id: "dc_naomi".to_string(),
            subdir: "DC".to_string(),
            filename: "naomi.zip".to_string(),
            description: "Naomi arcade BIOS".to_string(),
            system: "Dreamcast / Naomi".to_string(),
        },
        BiosEntry {
            id: "nds_bios7".to_string(),
            subdir: "NDS".to_string(),
            filename: "bios7.bin".to_string(),
            description: "Nintendo DS ARM7 BIOS".to_string(),
            system: "Nintendo DS".to_string(),
        },
        BiosEntry {
            id: "nds_bios9".to_string(),
            subdir: "NDS".to_string(),
            filename: "bios9.bin".to_string(),
            description: "Nintendo DS ARM9 BIOS".to_string(),
            system: "Nintendo DS".to_string(),
        },
        BiosEntry {
            id: "nds_firmware".to_string(),
            subdir: "NDS".to_string(),
            filename: "firmware.bin".to_string(),
            description: "Nintendo DS firmware".to_string(),
            system: "Nintendo DS".to_string(),
        },
    ]
}

fn find_entry(id: &str) -> Option<BiosEntry> {
    catalog().into_iter().find(|e| e.id == id)
}

/// Resolve the absolute target path for a catalog entry on the SD card.
fn target_path(sd_mount: &Path, entry: &BiosEntry) -> PathBuf {
    if entry.subdir.is_empty() {
        sd_mount.join("Bios").join(&entry.filename)
    } else {
        sd_mount
            .join("Bios")
            .join(&entry.subdir)
            .join(&entry.filename)
    }
}

/// Sanitize a subdir/filename pair. Returns Err if the components are
/// unsafe (absolute paths, traversal, NUL, or empty after stripping).
fn safe_component(name: &str, kind: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err(format!("{kind} cannot be empty"));
    }
    if name.contains('\0') {
        return Err(format!("{kind} contains NUL byte"));
    }
    if name.contains('/') || name.contains('\\') {
        return Err(format!("{kind} cannot contain path separators"));
    }
    if name == "." || name == ".." {
        return Err(format!("{kind} cannot be '.' or '..'"));
    }
    Ok(name.to_string())
}

/// Check whether a single catalog entry is present on the SD card.
///
/// Returns `Ok(true)` if the file exists (regular file or symlink to one),
/// `Ok(false)` if it does not, and `Err` for actual I/O errors.
pub fn entry_is_present(sd_mount: &Path, entry: &BiosEntry) -> Result<bool, String> {
    let path = target_path(sd_mount, entry);
    match fs::symlink_metadata(&path) {
        Ok(meta) => Ok(meta.is_file() || meta.file_type().is_symlink()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("Failed to check {}: {}", path.display(), e)),
    }
}

/// Status for the whole catalog, used by the UI to show which entries
/// are already installed.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BiosStatus {
    pub entries: Vec<BiosStatusEntry>,
    /// Number of entries currently present.
    pub installed_count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BiosStatusEntry {
    pub entry: BiosEntry,
    pub present: bool,
}

/// Build a status snapshot for the whole catalog.
pub fn status(sd_mount: &str) -> Result<BiosStatus, String> {
    let mount = Path::new(sd_mount);
    if !mount.exists() {
        return Err("SD card mount point does not exist".to_string());
    }

    let catalog = catalog();
    let mut entries = Vec::with_capacity(catalog.len());
    let mut installed_count = 0usize;
    for entry in catalog {
        let present = entry_is_present(mount, &entry)?;
        if present {
            installed_count += 1;
        }
        entries.push(BiosStatusEntry {
            entry: entry.clone(),
            present,
        });
    }

    Ok(BiosStatus {
        entries,
        installed_count,
    })
}

/// Install one BIOS entry from a base64-encoded payload.
///
/// The frontend reads the user's selected file as bytes and base64-encodes
/// it (so we can pass binary through Tauri's JSON invoke layer). We
/// decode, write to the validated target path, and re-validate the
/// canonical path to ensure the write did not escape the SD card.
pub fn install_bios_from_bytes(
    sd_mount: &str,
    entry_id: &str,
    base64_payload: &str,
) -> Result<String, String> {
    let entry = find_entry(entry_id).ok_or_else(|| format!("Unknown BIOS entry: {entry_id}"))?;

    // Validate the catalog entry itself before using it to build a path.
    // This is defense in depth: the catalog is hard-coded, but a future
    // refactor (e.g. loading the catalog from a file) must not weaken
    // this guarantee. The subdir is allowed to be empty (meaning the
    // file lives at Bios/<filename>) — only the filename is required.
    let safe_subdir = if entry.subdir.is_empty() {
        String::new()
    } else {
        safe_component(&entry.subdir, "subdir")?
    };
    let safe_filename = safe_component(&entry.filename, "filename")?;

    let mount = Path::new(sd_mount);
    if !mount.exists() {
        return Err("SD card mount point does not exist".to_string());
    }

    let canonical_mount = mount
        .canonicalize()
        .map_err(|e| format!("Failed to resolve SD card path: {}", e))?;

    // Build the target path. We re-derive it from the sanitized parts so
    // the write is provably constrained by what the sanitizer accepted.
    let target = if entry.subdir.is_empty() {
        mount.join("Bios").join(&safe_filename)
    } else {
        mount.join("Bios").join(&safe_subdir).join(&safe_filename)
    };

    // Validate the target stays inside the SD card BEFORE writing.
    let parent = target
        .parent()
        .ok_or_else(|| "Target path has no parent directory".to_string())?;
    let canonical_parent = fs_utils::canonicalize_existing_ancestor(parent)
        .map_err(|e| format!("Failed to resolve target parent: {}", e))?;
    if !canonical_parent.starts_with(&canonical_mount) {
        return Err(format!(
            "Security violation: target escapes SD card: {}",
            target.display()
        ));
    }

    // Decode the payload.
    let bytes = BASE64
        .decode(base64_payload)
        .map_err(|e| format!("Invalid payload encoding: {e}"))?;
    if bytes.is_empty() {
        return Err("Empty file payload".to_string());
    }

    // Create the parent dir (e.g. Bios/GB) and write.
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
    }

    // If target exists (or is a symlink), remove it to break any potential symlink escapes.
    if let Ok(meta) = fs::symlink_metadata(&target) {
        if meta.is_file() || meta.file_type().is_symlink() {
            fs::remove_file(&target).map_err(|e| {
                format!(
                    "Failed to remove existing file/symlink at target {}: {}",
                    target.display(),
                    e
                )
            })?;
        }
    }

    fs::write(&target, &bytes)
        .map_err(|e| format!("Failed to write {}: {}", target.display(), e))?;

    // Re-validate the canonical path after writing (symlink-race guard).
    let canonical = target
        .canonicalize()
        .map_err(|e| format!("Failed to resolve written path: {}", e))?;
    if !canonical.starts_with(&canonical_mount) {
        // Best-effort cleanup. The write happened, so we remove what we
        // just wrote to avoid leaving a stray file outside the SD card.
        let _ = fs::remove_file(&canonical);
        return Err(format!(
            "Security violation: written file escapes SD card: {}",
            target.display()
        ));
    }

    Ok(target.display().to_string())
}

#[cfg(test)]
pub(crate) const EXPECTED_BIOS_IDS: &[&str] = &[
    "gb_bios",
    "gbc_bios",
    "gba_bios",
    "md_cd_e",
    "md_cd_j",
    "md_cd_u",
    "ps_bios",
    "pce_bios",
    "fc_disksys",
    "pkm_bios",
    "sgb_bios",
    "dc_boot",
    "dc_naomi",
    "nds_bios7",
    "nds_bios9",
    "nds_firmware",
];

#[cfg(test)]
mod tests {
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

        let path =
            install_bios_from_bytes(temp.path().to_str().unwrap(), "gb_bios", &encoded).unwrap();

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
}
