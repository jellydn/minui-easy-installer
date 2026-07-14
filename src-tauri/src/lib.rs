use std::sync::Arc;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tokio_util::sync::CancellationToken;

mod bios;
mod download;
mod drives;
mod extract;
mod fs_utils;
mod health;
mod install;
mod package;
mod pipeline;
mod platform;
mod validate;
mod version;
mod wifi;

#[tauri::command]
async fn get_removable_drives() -> Result<Vec<drives::RemovableDrive>, String> {
    drives::list_removable_drives()
}

/// Options for formatting a drive, received from the frontend via Tauri IPC.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatDriveOptions {
    pub mount_path: String,
    pub volume_name: String,
}

#[tauri::command]
async fn format_drive(opts: FormatDriveOptions) -> Result<(), String> {
    drives::format_drive(&opts.mount_path, &opts.volume_name)
}

/// Options for verifying an archive checksum, received from the frontend via Tauri IPC.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyChecksumOptions {
    pub file_path: String,
    pub expected_checksum: String,
}

#[tauri::command]
fn verify_archive_checksum(opts: VerifyChecksumOptions) -> Result<bool, String> {
    download::verify_checksum(&opts.file_path, &opts.expected_checksum)
}

/// Registry of in-flight install cancellation tokens.
///
/// The UI never runs concurrent installs in a single window, so we keep
/// at most one token at a time. A new install replaces the previous
/// (cancelling the old one — safer than letting it run orphaned).
#[derive(Default)]
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

/// Synchronous install (deprecated — prefer start_install for progress streaming).
#[tauri::command]
async fn install_minui(options: install::InstallOptions) -> Result<install::InstallResult, String> {
    install::install_minui(&options, Arc::new(|_| {})).await
}

/// Start a cancellable install. Returns immediately with the install id
/// (currently always "current" since we support one install at a time).
/// The actual install runs in a background task; the result is emitted
/// as a `install-complete` or `install-error` event.
#[tauri::command]
async fn start_install(
    app_handle: AppHandle,
    registry: tauri::State<'_, Arc<InstallRegistry>>,
    options: install::InstallOptions,
) -> Result<String, String> {
    let token = CancellationToken::new();
    {
        let mut slot = registry
            .token
            .lock()
            .map_err(|_| "Internal error: state lock is poisoned".to_string())?;
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

    let handle_for_dl = app_handle.clone();
    let download_progress: pipeline::DownloadProgressCallback = Arc::new(move |bytes, total| {
        let event = install::InstallProgressEvent {
            step: "download".to_string(),
            details: String::new(),
            current_bytes: Some(bytes),
            total_bytes: total,
        };
        if let Err(e) = handle_for_dl.emit("install-progress", event) {
            eprintln!("Warning: failed to emit download progress event: {}", e);
        }
    });

    let registry_for_task = registry.inner().clone();
    let result_handle = app_handle.clone();
    tokio::spawn(async move {
        let res =
            install::install_minui_with_cancel(&options, progress, download_progress, token).await;
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
    let slot = registry
        .token
        .lock()
        .map_err(|_| "Internal error: state lock is poisoned".to_string())?;
    if let Some(token) = slot.as_ref() {
        token.cancel();
    }
    Ok(())
}

#[tauri::command]
async fn validate_installation(
    opts: validate::ValidateOptions,
) -> Result<validate::ValidationResult, String> {
    validate::validate_installation(
        &opts.sd_mount,
        &opts.platform,
        opts.has_extras,
        &opts.extras_dir,
    )
}

#[tauri::command]
fn format_validation_report(result: validate::ValidationResult) -> String {
    validate::format_validation_report(&result)
}

#[tauri::command]
async fn check_minui_version(opts: version::VersionCheckOptions) -> version::VersionCheckResult {
    version::check_for_updates_with_prefix(
        &opts.sd_mount,
        opts.latest_version.as_deref(),
        opts.expected_prefix.as_deref(),
    )
}

#[tauri::command]
async fn install_package(
    opts: package::PackageInstallOptions,
) -> Result<package::PackageInstallResult, String> {
    let rules = package::PackageInstallPathRules {
        target_dir: opts.target_dir,
        extract_to_root: opts.extract_to_root,
        pak_name: opts.pak_name,
    };
    package::install_package(
        &opts.artifact_url,
        opts.checksum.as_deref(),
        &opts.sd_mount,
        &rules,
        &opts.platform,
    )
    .await
}

#[tauri::command]
async fn write_wifi_config(opts: wifi::WifiConfig) -> Result<(), String> {
    wifi::write_wifi_config(&opts.sd_mount, &opts.ssid, &opts.password)
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
fn list_bios_catalog() -> Vec<bios::BiosEntry> {
    bios::catalog().to_vec()
}

#[tauri::command]
async fn get_bios_status(sd_mount: String) -> Result<bios::BiosStatus, String> {
    bios::status(&sd_mount)
}

/// Options for installing a BIOS file, received from the frontend via Tauri IPC.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiosInstallOptions {
    pub sd_mount: String,
    pub entry_id: String,
    pub base64_payload: String,
}

#[tauri::command]
async fn install_bios_file(opts: BiosInstallOptions) -> Result<String, String> {
    bios::install_bios_from_bytes(&opts.sd_mount, &opts.entry_id, &opts.base64_payload)
}

#[tauri::command]
async fn detect_installed_packages(sd_mount: String) -> Vec<package::InstalledPackage> {
    package::detect_installed_packages(&sd_mount)
}

/// Options for checking package updates, received from the frontend via Tauri IPC.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckPackageUpdatesOptions {
    pub sd_mount: String,
    pub registry_packages: Vec<(String, String)>,
}

#[tauri::command]
async fn check_package_updates(
    opts: CheckPackageUpdatesOptions,
) -> Vec<package::PackageUpdateInfo> {
    package::check_package_updates(&opts.sd_mount, &opts.registry_packages)
}

#[tauri::command]
async fn check_sd_card_health(
    opts: health::HealthCheckOptions,
) -> Result<health::HealthCheckResult, String> {
    health::check_sd_card_health(&opts.sd_mount, opts.device_platform.as_deref())
}

/// Allowed URLs for fetch_url. Prevents SSRF by restricting HTTP
/// fetches to known endpoints. Add new endpoints here as needed.
const ALLOWED_URLS: &[&str] = &["https://packages.minui.dev/registry/index.json"];

#[tauri::command]
async fn fetch_url(url: String) -> Result<String, String> {
    if !ALLOWED_URLS.iter().any(|allowed| url == *allowed) {
        return Err(format!("URL not allowed: {}", url));
    }

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
            verify_archive_checksum,
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
            list_bios_catalog,
            get_bios_status,
            install_bios_file,
            detect_installed_packages,
            check_package_updates,
            check_sd_card_health,
            fetch_url
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
