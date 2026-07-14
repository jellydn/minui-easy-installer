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
#[path = "bios_tests.rs"]
mod tests;
