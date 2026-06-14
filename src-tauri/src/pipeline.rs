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
/// Resolves the final path, creates it, and verifies it doesn't escape
/// the SD card via symlinks or canonicalization artifacts. Returns the
/// canonical path.
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

    fs::create_dir_all(&target)
        .map_err(|e| format!("Failed to create package directory: {}", e))?;

    let canonical = target
        .canonicalize()
        .map_err(|e| format!("Failed to resolve package path: {}", e))?;

    if !canonical.starts_with(&canonical_sd) {
        return Err(format!(
            "Security violation: target escapes SD card: {}",
            target.display()
        ));
    }

    Ok(canonical)
}
