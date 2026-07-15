use std::fs;
use std::path::Path;
use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::fs_utils;
use crate::pipeline::{DownloadProgressCallback, InstallSession, Pipeline};
use crate::platform::device_base_item;

#[derive(Debug, Clone, serde::Serialize)]
pub struct InstallResult {
    pub success: bool,
    pub error: Option<String>,
    pub base_files_copied: u32,
    pub extras_files_copied: u32,
    pub extras_warning: Option<String>,
    pub rom_dirs_created: u32,
}

/// Progress event emitted during the install flow.
#[derive(Debug, Clone, serde::Serialize)]
pub struct InstallProgressEvent {
    pub step: String,
    pub details: String,
    /// Optional byte-level download progress (set when `step === "download"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_bytes: Option<u64>,
    /// Optional total bytes (None when server didn't send Content-Length).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bytes: Option<u64>,
}

impl InstallProgressEvent {
    /// Create a phase-level progress event (no byte-level download data).
    pub fn phase(step: &str, details: &str) -> Self {
        Self {
            step: step.to_string(),
            details: details.to_string(),
            current_bytes: None,
            total_bytes: None,
        }
    }
}

/// Callback for reporting install progress. Passed through the install flow.
pub type ProgressCallback = Arc<dyn Fn(InstallProgressEvent) + Send + Sync>;

/// Standard ROM folders created on the SD card after install
const ROM_DIRS: &[&str] = &[
    "Arcade (FBN)",
    "Game Boy (GB)",
    "Game Boy Advance (GBA)",
    "Game Boy Color (GBC)",
    "Neo Geo (FBN)",
    "Neo Geo Pocket (NGP)",
    "Nintendo 64 (N64)",
    "Nintendo DS (NDS)",
    "Nintendo Entertainment System (FC)",
    "Pico-8 (PICO)",
    "Sega Dreamcast (DC)",
    "Sega Genesis (MD)",
    "Sony PlayStation (PS)",
    "Sony PlayStation Portable (PSP)",
    "Super Nintendo Entertainment System (SFC)",
    "Ports (PORTS)",
];

/// Creates standard ROM directories on the SD card if they don't exist.
/// Skips directories that already exist (e.g. from a previous install).
pub fn create_rom_dirs(sd_mount: &str) -> Result<u32, String> {
    let roms_root = Path::new(sd_mount).join("Roms");
    let mut created = 0u32;

    for dir in ROM_DIRS {
        let path = roms_root.join(dir);
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|e| format!("Failed to create Roms/{}: {}", dir, e))?;
            created += 1;
        }
    }

    // Create placeholder for Portmaster in the Ports directory
    let ports_dst = roms_root.join("Ports (PORTS)").join("Portmaster.sh");
    if !ports_dst.exists() {
        if let Err(e) = fs::write(&ports_dst, "") {
            eprintln!("Warning: failed to create Portmaster placeholder: {}", e);
        }
    }

    Ok(created)
}

/// Folders that must never be deleted or overwritten during install.
/// Only lowercase needed — comparisons use eq_ignore_ascii_case.
const PRESERVED_FOLDERS: &[&str] = &["roms", "saves", "save", "bios", "cheats"];

fn is_preserved_path(path: &Path, sd_root: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(sd_root) else {
        return false;
    };
    let Some(name) = relative.iter().next() else {
        return false;
    };
    let name_str = name.to_string_lossy();
    PRESERVED_FOLDERS
        .iter()
        .any(|preserved| name_str.eq_ignore_ascii_case(preserved))
}

/// Shared items that should be copied from every base archive to the SD root.
const SHARED_BASE_ITEMS: &[&str] = &["Bios", "Roms", "Saves", "MinUI.zip"];

/// Copy a single base archive item (file or directory) from `src` to `dst`.
///
/// When `preserve` is true, existing user data under the destination is
/// skipped using `is_preserved_path`. Returns the number of files copied,
/// or 0 if the source does not exist.
fn copy_archive_item(
    src: &Path,
    dst: &Path,
    sd_root: &Path,
    preserve: bool,
) -> Result<u32, String> {
    if !src.exists() {
        return Ok(0);
    }

    if src.is_file() {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directory: {}", e))?;
        }
        fs::copy(src, dst).map_err(|e| {
            format!(
                "Failed to copy {} to {}: {}",
                src.display(),
                dst.display(),
                e
            )
        })?;
        Ok(1)
    } else {
        fs::create_dir_all(dst)
            .map_err(|e| format!("Failed to create directory {}: {}", dst.display(), e))?;
        let skip_predicate = |_src: &Path, dst: &Path| preserve && is_preserved_path(dst, sd_root);
        fs_utils::copy_dir_recursive(src, dst, &skip_predicate, &|| false)
    }
}

/// Copy the base archive contents to the SD card root.
///
/// Only copies the shared items (`Bios`, `Roms`, `Saves`, `MinUI.zip`) and the
/// device-specific folder/file for `platform`. Other device folders and
/// `README.txt` are intentionally skipped so the SD card is not polluted with
/// files for devices the user did not select.
pub fn copy_base_files(
    extracted_base_path: &str,
    sd_mount: &str,
    platform: &str,
) -> Result<u32, String> {
    let base_dir = Path::new(extracted_base_path);
    let sd_root = Path::new(sd_mount);
    let mut files_copied = 0u32;

    // Copy shared items, preserving existing user data in Bios/Roms/Saves.
    for item in SHARED_BASE_ITEMS {
        let src = base_dir.join(item);
        let dst = sd_root.join(item);
        files_copied += copy_archive_item(&src, &dst, sd_root, true)?;
    }

    // Copy the device-specific folder/file.
    let device_item = device_base_item(platform);
    let device_src = base_dir.join(device_item);
    let device_dst = sd_root.join(device_item);
    files_copied += copy_archive_item(&device_src, &device_dst, sd_root, false)?;

    Ok(files_copied)
}

/// Copies a subdirectory tree from src_root/subpath to dst_root/subpath.
/// Returns the number of files copied, or 0 if the source doesn't exist.
fn copy_subtree(src_root: &Path, dst_root: &Path, subpath: &str) -> Result<u32, String> {
    let src = src_root.join(subpath);
    if !src.exists() {
        return Ok(0);
    }
    let dst = dst_root.join(subpath);
    fs::create_dir_all(&dst)
        .map_err(|e| format!("Failed to create {} directory: {}", subpath, e))?;
    fs_utils::copy_dir_recursive(&src, &dst, &|_s, _d| false, &|| false)
}

/// Copies Extras files to the SD card, filtering by platform.
///
/// The extras archive contains all platforms' emulators and tools at:
///   Emus/{platform}/{pakName}.pak/
///   Tools/{platform}/{pakName}.pak/
///   Bios/          (shared across all devices)
///
/// This function only copies:
///   1. Everything under `Emus/{extras_platform}/` → SD `Emus/{extras_platform}/`
///   2. Everything under `Tools/{extras_platform}/` → SD `Tools/{extras_platform}/`
///   3. Everything under `Bios/` → SD `Bios/`
pub fn copy_extras_files(
    extracted_extras_path: &str,
    sd_mount: &str,
    extras_platform: &str,
) -> Result<u32, String> {
    // Security guard: extras_platform must match expected format
    if extras_platform.is_empty()
        || !extras_platform
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(format!(
            "Invalid extras_platform: '{}' must contain only alphanumeric characters and hyphens",
            extras_platform
        ));
    }
    let extras_src = Path::new(extracted_extras_path);
    let sd_root = Path::new(sd_mount);

    if !extras_src.exists() {
        return Err("Extras source directory does not exist".to_string());
    }

    let mut files_copied = 0u32;
    files_copied += copy_subtree(extras_src, sd_root, "Bios")?;
    files_copied += copy_subtree(
        &extras_src.join("Emus"),
        &sd_root.join("Emus"),
        extras_platform,
    )?;
    files_copied += copy_subtree(
        &extras_src.join("Tools"),
        &sd_root.join("Tools"),
        extras_platform,
    )?;

    Ok(files_copied)
}

/// Configuration for a MinUI installation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallOptions {
    pub base_url: String,
    pub extras_url: Option<String>,
    pub base_checksum: Option<String>,
    pub extras_checksum: Option<String>,
    pub sd_mount: String,
    pub platform: String,
    pub extras_platform: String,
    pub version: String,
    /// Name of the fork (e.g. "MinUI", "MinUI-Zero"). Written into
    /// minui.txt as "{fork_name} {version}". Defaults to "MinUI".
    pub fork_name: Option<String>,
}

/// Convenience wrapper that delegates to `install_minui_with_cancel`
/// without cancellation support. Used by the synchronous `install_minui`
/// Tauri command for simple installs that don't need progress streaming.
///
/// Callers that need cancellation should use `install_minui_with_cancel`
/// or the `start_install` Tauri command.
pub async fn install_minui(
    options: &InstallOptions,
    progress: ProgressCallback,
) -> Result<InstallResult, String> {
    install_minui_with_cancel(
        options,
        progress,
        Arc::new(|_, _| {}),
        CancellationToken::new(),
    )
    .await
}

/// Write version metadata to `minui.txt` on the SD card.
/// Returns a warning string if the write fails.
fn write_version_metadata(options: &InstallOptions) -> Option<String> {
    let fork_label = options.fork_name.as_deref().unwrap_or("MinUI");
    let minui_txt_path = Path::new(&options.sd_mount).join("minui.txt");
    match fs::write(
        &minui_txt_path,
        format!("{} {}\n", fork_label, options.version),
    ) {
        Ok(()) => None,
        Err(e) => {
            eprintln!("Warning: failed to write version metadata: {}", e);
            Some(format!("Failed to write version metadata: {}", e))
        }
    }
}

/// Full installation flow with cancellation support.
///
/// Runs four steps in sequence:
/// 1. Base archive — download, extract, copy to SD (fatal on error)
/// 2. Extras archive — download, extract, copy (non-fatal on error)
/// 3. ROM directories — create standard folders on SD
/// 4. Version metadata — write minui.txt to SD root
///
/// The `InstallSession` owns temp dirs and cleans them up on drop.
pub async fn install_minui_with_cancel(
    options: &InstallOptions,
    progress: ProgressCallback,
    download_progress: DownloadProgressCallback,
    cancel: CancellationToken,
) -> Result<InstallResult, String> {
    let mut session = InstallSession::new();
    let mut extras_files_copied = 0u32;
    let mut extras_warning: Option<String> = None;

    // ── Step 1: Base archive (fatal on error) ──────────────
    let file_name = options.base_url.rsplit('/').next().unwrap_or("MinUI.zip");
    (progress)(InstallProgressEvent::phase(
        "download",
        &format!("Downloading {}", file_name),
    ));
    let base_files_copied = Pipeline::run(
        "base",
        &options.base_url,
        options.base_checksum.as_deref(),
        |p| copy_base_files(p.to_str().unwrap(), &options.sd_mount, &options.platform),
        progress.clone(),
        download_progress.clone(),
        cancel.clone(),
        &mut session,
    )
    .await?;

    // ── Step 2: Extras archive (non-fatal on error) ─────────
    if let Some(extras_url) = &options.extras_url {
        (progress)(InstallProgressEvent::phase(
            "extract",
            "Downloading Extras...",
        ));
        match Pipeline::run(
            "extras",
            extras_url,
            options.extras_checksum.as_deref(),
            |p| {
                copy_extras_files(
                    p.to_str().unwrap(),
                    &options.sd_mount,
                    &options.extras_platform,
                )
            },
            progress.clone(),
            download_progress.clone(),
            cancel.clone(),
            &mut session,
        )
        .await
        {
            Ok(n) => extras_files_copied = n,
            Err(e) => extras_warning = Some(e),
        }
    }

    // ── Step 3: ROM directories ────────────────────────────
    (progress)(InstallProgressEvent::phase(
        "copy",
        "Creating standard ROM directories...",
    ));
    let rom_dirs_created = create_rom_dirs(&options.sd_mount).unwrap_or(0);

    // ── Step 4: Version metadata ───────────────────────────
    let fork_label = options.fork_name.as_deref().unwrap_or("MinUI");
    (progress)(InstallProgressEvent::phase(
        "finish",
        &format!(
            "Writing version metadata ({} {})",
            fork_label, options.version
        ),
    ));
    if let Some(w) = write_version_metadata(options) {
        extras_warning = Some(w);
    }

    // session drops here — temp dirs cleaned up after all operations complete
    Ok(InstallResult {
        success: true,
        error: None,
        base_files_copied,
        extras_files_copied,
        extras_warning,
        rom_dirs_created,
    })
}

#[cfg(test)]
#[path = "install_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "install_copy_tests.rs"]
mod copy_tests;

#[cfg(test)]
#[path = "install_extras_tests.rs"]
mod extras_tests;
