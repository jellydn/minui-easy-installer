use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::download;
use crate::extract;
use crate::install::{InstallProgressEvent, ProgressCallback};

/// Owns temp directories for the duration of an install pipeline.
///
/// TempDirs are created by download/extract operations and kept alive
/// here so the file paths remain valid throughout the install. When
/// the InstallSession drops, all temp dirs are cleaned up atomically.
pub struct InstallSession {
    _base_archive: Option<TempDir>,
    _base_extracted: Option<TempDir>,
    _extras_archive: Option<TempDir>,
    _extras_extracted: Option<TempDir>,
    _package_archive: Option<TempDir>,
    _package_extracted: Option<TempDir>,
}

impl InstallSession {
    pub fn new() -> Self {
        Self {
            _base_archive: None,
            _base_extracted: None,
            _extras_archive: None,
            _extras_extracted: None,
            _package_archive: None,
            _package_extracted: None,
        }
    }

    /// Slot identifiers for Pipeline::run
    pub(crate) fn slot_archive(&mut self, label: &str) -> &mut Option<TempDir> {
        match label {
            "base" => &mut self._base_archive,
            "extras" => &mut self._extras_archive,
            "package" => &mut self._package_archive,
            _ => panic!("unknown pipeline label: {label}"),
        }
    }

    pub(crate) fn slot_extracted(&mut self, label: &str) -> &mut Option<TempDir> {
        match label {
            "base" => &mut self._base_extracted,
            "extras" => &mut self._extras_extracted,
            "package" => &mut self._package_extracted,
            _ => panic!("unknown pipeline label: {label}"),
        }
    }
}

/// Orchestrates the download → extract → copy pipeline for any archive type.
pub struct Pipeline;

impl Pipeline {
    /// Run download → extract → copy. Returns the number of files copied.
    pub async fn run<Cp>(
        label: &str,
        url: &str,
        checksum: Option<&str>,
        copy: Cp,
        progress: ProgressCallback,
        session: &mut InstallSession,
    ) -> Result<u32, String>
    where
        Cp: FnOnce(PathBuf) -> Result<u32, String>,
    {
        let extracted = Self::run_to_extracted(label, url, checksum, progress.clone(), session).await?;
        progress(InstallProgressEvent {
            step: "copy".to_string(),
            details: format!("Copying {} files", label),
        });
        copy(extracted)
    }

    /// Run download → extract, returning the extracted path without copying.
    /// Caller handles the copy step separately (needed by packages).
    pub async fn run_to_extracted(
        label: &str,
        url: &str,
        checksum: Option<&str>,
        progress: ProgressCallback,
        session: &mut InstallSession,
    ) -> Result<PathBuf, String> {
        progress(InstallProgressEvent {
            step: "download".to_string(),
            details: format!("Downloading {} archive", label),
        });
        let archive_path: PathBuf = download::download_archive_into(
            session.slot_archive(label),
            url,
            checksum,
        )
        .await?;

        progress(InstallProgressEvent {
            step: "extract".to_string(),
            details: format!("Extracting {} archive", label),
        });
        extract::extract_archive_into(
            session.slot_extracted(label),
            &archive_path,
            None,
        )
    }
}

/// Create a validated target directory within the SD card mount.
///
/// Resolves the final path, validates that it stays inside the SD card,
/// creates it, and re-validates the canonical form (catches symlink races).
/// Returns the canonical path.
///
/// Security note: validation happens on the parent directory BEFORE any
/// `create_dir_all` call. This prevents a malicious or malformed
/// `target_dir`/`platform`/`pak_name` from creating directories outside
/// the SD card mount, even briefly.
pub fn create_target_within(
    sd_mount: &Path,
    target_dir: &str,
    platform: &str,
    pak_name: &str,
) -> Result<PathBuf, String> {
    let canonical_sd = sd_mount
        .canonicalize()
        .map_err(|e| format!("Failed to resolve SD card path: {}", e))?;

    let target = sd_mount
        .join(target_dir.trim_start_matches('/'))
        .join(platform)
        .join(format!("{}.pak", pak_name));

    // Validate the target is inside canonical_sd BEFORE creating anything.
    // We walk up to the first existing ancestor and canonicalize that, then
    // check it stays within the SD card root. Walking up is required because
    // on a fresh install the parent directory tree may not exist yet --
    // a plain `parent.canonicalize()` would fail with NotFound before
    // create_dir_all has a chance to run. If a symlink at any non-existing
    // intermediate path resolves to outside the SD card, the canonical
    // ancestor check will still catch it.
    let parent = target
        .parent()
        .ok_or_else(|| "Target path has no parent directory".to_string())?;
    let canonical_parent = canonicalize_existing_ancestor(parent)
        .map_err(|e| format!("Failed to resolve target parent: {}", e))?;

    if !canonical_parent.starts_with(&canonical_sd) {
        return Err(format!(
            "Security violation: target escapes SD card: {}",
            target.display()
        ));
    }

    // Safe to create now that the parent is validated.
    let created_now = !target.exists();
    fs::create_dir_all(&target)
        .map_err(|e| format!("Failed to create package directory: {}", e))?;

    let canonical = target
        .canonicalize()
        .map_err(|e| format!("Failed to resolve package path: {}", e))?;

    // Re-validate after creation to catch symlink races (e.g. someone
    // swapped a directory for a symlink between our create and canonicalize).
    if !canonical.starts_with(&canonical_sd) {
        // Best-effort cleanup. Only remove what we *know* we created —
        // never touch a directory that pre-existed this call. And always
        // operate on the canonical path so we can't accidentally follow a
        // symlink to an unrelated directory.
        if created_now {
            if let Err(cleanup_err) = fs::remove_dir(&canonical) {
                eprintln!(
                    "create_target_within: cleanup failed for escaped path {}: {}",
                    canonical.display(),
                    cleanup_err
                );
            }
        }
        return Err(format!(
            "Security violation: target escapes SD card after creation: {}",
            target.display()
        ));
    }

    Ok(canonical)
}

/// Walk up `path` until we find an existing ancestor, then canonicalize it.
///
/// `Path::canonicalize` requires every component to exist. On a fresh
/// install, the target parent directory tree may not exist yet (e.g. SD
/// card just formatted). This helper finds the highest existing ancestor
/// and canonicalizes that, so the caller can still reason about the path's
/// location relative to the SD card root.
///
/// Symlink-safety: if any existing ancestor is a symlink pointing outside
/// the SD card, `canonicalize` resolves through the symlink and the caller's
/// `starts_with(canonical_sd)` check will reject it. So this helper does
/// not weaken the security boundary -- it just makes the fresh-install
/// case work.
fn canonicalize_existing_ancestor(path: &Path) -> std::io::Result<PathBuf> {
    let mut current: &Path = path;
    loop {
        match current.canonicalize() {
            Ok(canonical) => return Ok(canonical),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                match current.parent() {
                    Some(parent) => current = parent,
                    None => return Err(e),
                }
            }
            Err(e) => return Err(e),
        }
    }
}
