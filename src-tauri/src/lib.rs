use std::sync::{Arc, OnceLock};
use tauri::{AppHandle, Emitter, Manager};

mod bios;
mod download;
mod drives;
mod extract;
mod fs_utils;
mod health;
mod install;
mod install_manager;
mod package;
mod pipeline;
mod platform;
mod validate;
mod version;
mod wifi;

use install_manager::{EventDispatcher, InstallManager};

/// Thin adapter: bridges Tauri's `AppHandle` to the `EventDispatcher`
/// trait so `InstallManager` can emit events without knowing about Tauri.
struct TauriAppDispatcher {
    handle: AppHandle,
}

impl EventDispatcher for TauriAppDispatcher {
    fn emit_progress(&self, event: install::InstallProgressEvent) {
        if let Err(e) = self.handle.emit("install-progress", event) {
            eprintln!("Warning: failed to emit install progress event: {}", e);
        }
    }

    fn emit_complete(&self, result: install::InstallResult) {
        let _ = self.handle.emit("install-complete", result);
    }

    fn emit_error(&self, error: String) {
        let _ = self.handle.emit("install-error", error);
    }
}

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
    manager: tauri::State<'_, Arc<InstallManager>>,
    options: install::InstallOptions,
) -> Result<String, String> {
    let dispatcher = Arc::new(TauriAppDispatcher { handle: app_handle });
    manager.inner().start(dispatcher, options)
}

/// Cancel an in-flight install. No-op if no install is running.
#[tauri::command]
fn cancel_install(manager: tauri::State<'_, Arc<InstallManager>>) -> Result<(), String> {
    manager.cancel()
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

/// Lazily-initialised HTTP client with connection pooling.
/// Built once and reused across all `fetch_url` calls.
fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client")
    })
}

#[tauri::command]
async fn fetch_url(url: String) -> Result<String, String> {
    if !ALLOWED_URLS.iter().any(|allowed| url == *allowed) {
        return Err(format!("URL not allowed: {}", url));
    }

    let response = http_client()
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
    let manager = Arc::new(InstallManager::new());
    tauri::Builder::default()
        .manage(manager)
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
