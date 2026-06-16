use std::sync::Arc;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tokio_util::sync::CancellationToken;

mod download;
mod drives;
mod extract;
mod fs_utils;
mod health;
mod install;
mod package;
mod pipeline;
mod validate;
mod version;
mod wifi;


#[tauri::command]
async fn get_removable_drives() -> Result<Vec<drives::RemovableDrive>, String> {
    drives::list_removable_drives()
}

#[tauri::command]
async fn format_drive(mount_path: String, volume_name: String) -> Result<(), String> {
    drives::format_drive(&mount_path, &volume_name)
}

/// Standalone download command — deprecated in favor of the install pipeline.
/// The TempDir is dropped immediately after this returns, so the file_path
/// in the result is no longer valid once this returns. Kept for backward
/// compatibility with frontend archive.ts. Prefer install_minui or install_package.
#[tauri::command]
async fn download_and_verify_archive(
    url: String,
    checksum: Option<String>,
) -> Result<download::DownloadResult, String> {
    let checksum_ref = checksum.as_deref();
    let (result, _temp_dir) = download::download_archive(&url, checksum_ref).await?;
    // _temp_dir drops here — file still exists on disk for the return trip
    // but will be cleaned up shortly after. Not safe to chain with extraction.
    Ok(result)
}

#[tauri::command]
fn verify_archive_checksum(file_path: String, expected_checksum: String) -> Result<bool, String> {
    download::verify_checksum(&file_path, &expected_checksum)
}

#[tauri::command]
async fn extract_archive_to_directory(
    archive_path: String,
    destination: Option<String>,
) -> Result<extract::ExtractionResult, String> {
    let dest_ref = destination.as_deref();
    let (result, _temp_dir) = extract::extract_archive(&archive_path, dest_ref)?;
    // Same caveat as download_and_verify_archive: if no destination is
    // specified, the TempDir drops here and the extracted files vanish.
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
async fn install_minui(
    app_handle: AppHandle,
    base_url: String,
    extras_url: Option<String>,
    base_checksum: Option<String>,
    extras_checksum: Option<String>,
    sd_mount: String,
    platform: String,
    extras_platform: String,
    version: String,
) -> Result<install::InstallResult, String> {
    let handle = app_handle.clone();
    let progress = Arc::new(move |event: install::InstallProgressEvent| {
        if let Err(e) = handle.emit("install-progress", event) {
            eprintln!("Warning: failed to emit install progress event: {}", e);
        }
    });
    let options = install::InstallOptions {
        base_url,
        extras_url,
        base_checksum,
        extras_checksum,
        sd_mount,
        platform,
        extras_platform,
        version,
    };
    install::install_minui(&options, progress).await
}

/// Registry of in-flight install cancellation tokens.
///
/// The UI never runs concurrent installs in a single window, so we keep
/// at most one token at a time. A new install replaces the previous
/// (cancelling the old one — safer than letting it run orphaned).
pub struct InstallRegistry {
    pub token: Mutex<Option<CancellationToken>>,
}

impl InstallRegistry {
    pub fn new() -> Self {
        Self {
            token: Mutex::new(None),
        }
    }
}

/// Start a cancellable install. Returns immediately with the install id
/// (currently always "current" since we support one install at a time).
/// The actual install runs in a background task; the result is emitted
/// as a `install-complete` or `install-error` event.
#[tauri::command]
async fn start_install(
    app_handle: AppHandle,
    registry: tauri::State<'_, Arc<InstallRegistry>>,
    base_url: String,
    extras_url: Option<String>,
    base_checksum: Option<String>,
    extras_checksum: Option<String>,
    sd_mount: String,
    platform: String,
    extras_platform: String,
    version: String,
) -> Result<String, String> {
    let token = CancellationToken::new();
    {
        let mut slot = registry.token.lock().unwrap();
        // Cancel any prior install before replacing.
        if let Some(old) = slot.take() {
            old.cancel();
        }
        *slot = Some(token.clone());
    }

    let handle = app_handle.clone();
    let progress = Arc::new(move |event: install::InstallProgressEvent| {
        if let Err(e) = handle.emit("install-progress", event) {
            eprintln!("Warning: failed to emit install progress event: {}", e);
        }
    });
    let download_progress: pipeline::DownloadProgressCallback =
        Arc::new(move |_bytes, _total| {
            // Byte-level progress events are emitted via the existing
            // install-progress channel with the `step: "download"` label.
            // Future enhancement: extend InstallProgressEvent with
            // currentBytes / totalBytes and emit them here.
        });
    let options = install::InstallOptions {
        base_url,
        extras_url,
        base_checksum,
        extras_checksum,
        sd_mount,
        platform,
        extras_platform,
        version,
    };

    let registry_for_task = registry.inner().clone();
    let result_handle = app_handle.clone();
    tokio::spawn(async move {
        let res = install::install_minui_with_cancel(
            &options,
            progress,
            download_progress,
            token,
        )
        .await;
        // Clear the slot on completion.
        if let Ok(mut slot) = registry_for_task.token.lock() {
            *slot = None;
        }
        match res {
            Ok(r) => {
                let _ = result_handle.emit("install-complete", r);
            }
            Err(e) => {
                let _ = result_handle.emit("install-error", e);
            }
        }
    });

    Ok("current".to_string())
}

/// Cancel an in-flight install. No-op if no install is running.
#[tauri::command]
fn cancel_install(registry: tauri::State<'_, Arc<InstallRegistry>>) -> Result<(), String> {
    if let Some(token) = registry.token.lock().unwrap().as_ref() {
        token.cancel();
    }
    Ok(())
}

#[tauri::command]
async fn validate_installation(
    sd_mount: String,
    has_extras: bool,
    extras_dir: String,
) -> Result<validate::ValidationResult, String> {
    validate::validate_installation(&sd_mount, has_extras, &extras_dir)
}

#[tauri::command]
fn format_validation_report(result: validate::ValidationResult) -> String {
    validate::format_validation_report(&result)
}

#[tauri::command]
async fn check_minui_version(
    sd_mount: String,
    latest_version: Option<String>,
) -> version::VersionCheckResult {
    version::check_for_updates(&sd_mount, latest_version.as_deref())
}

#[tauri::command]
async fn install_package(
    artifact_url: String,
    checksum: Option<String>,
    sd_mount: String,
    target_dir: String,
    extract_to_root: bool,
    pak_name: String,
    platform: String,
) -> Result<package::PackageInstallResult, String> {
    let rules = package::PackageInstallPathRules {
        target_dir,
        extract_to_root,
        pak_name,
    };
    package::install_package(&artifact_url, checksum.as_deref(), &sd_mount, &rules, &platform).await
}

#[tauri::command]
async fn write_wifi_config(
    sd_mount: String,
    ssid: String,
    password: String,
) -> Result<(), String> {
    wifi::write_wifi_config(&sd_mount, &ssid, &password)
}

#[tauri::command]
async fn scan_wifi_networks() -> Vec<String> {
    wifi::scan_wifi_networks()
}

#[tauri::command]
async fn get_current_wifi_ssid() -> Option<String> {
    wifi::get_current_wifi_ssid()
}

#[tauri::command]
async fn detect_installed_packages(sd_mount: String) -> Vec<package::InstalledPackage> {
    package::detect_installed_packages(&sd_mount)
}

#[tauri::command]
async fn check_package_updates(
    sd_mount: String,
    registry_packages: Vec<(String, String)>,
) -> Vec<package::PackageUpdateInfo> {
    package::check_package_updates(&sd_mount, &registry_packages)
}

#[tauri::command]
async fn check_sd_card_health(
    sd_mount: String,
    device_platform: Option<String>,
) -> Result<health::HealthCheckResult, String> {
    health::check_sd_card_health(&sd_mount, device_platform.as_deref())
}

#[tauri::command]
async fn fetch_url(url: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch URL: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let registry = Arc::new(InstallRegistry::new());
    tauri::Builder::default()
        .manage(registry)
        .invoke_handler(tauri::generate_handler![
            get_removable_drives,
            format_drive,
            download_and_verify_archive,
            verify_archive_checksum,
            extract_archive_to_directory,
            install_minui,
            start_install,
            cancel_install,
            validate_installation,
            format_validation_report,
            check_minui_version,
            install_package,
            write_wifi_config,
            scan_wifi_networks,
            get_current_wifi_ssid,
            detect_installed_packages,
            check_package_updates,
            check_sd_card_health,
            fetch_url
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
