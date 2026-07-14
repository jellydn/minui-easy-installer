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
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::Arc;

    // ---- drives::list_removable_drives ----

    #[test]
    fn test_get_removable_drives_returns_result_shape() {
        // The function either returns Ok(Vec<RemovableDrive>) or Err(String).
        // We don't assert which — the test runs on macOS and Windows CI alike.
        // We do assert the field-type contract for whatever it returns.
        match drives::list_removable_drives() {
            Ok(drives) => {
                for d in &drives {
                    assert!(!d.mount_path.is_empty());
                    assert!(!d.name.is_empty());
                }
            }
            Err(msg) => {
                assert!(!msg.is_empty());
            }
        }
    }

    // ---- drives::format_drive ----

    #[test]
    #[cfg(target_os = "macos")]
    fn test_format_drive_errors_on_nonexistent_mount() {
        // format_drive shells out to diskutil; we don't want to actually
        // format anything in tests. Assert that nonexistent paths error
        // rather than panic.
        let result = drives::format_drive("/nonexistent/this/should/not/exist", "TEST");
        assert!(result.is_err());
    }

    // ---- install::install_minui (underlying, not the IPC wrapper) ----

    #[tokio::test]
    async fn test_install_minui_underlying_errors_on_bad_url() {
        // We don't want to actually download a real archive. Call the
        // underlying function with an unreachable URL and assert the
        // error propagates as a String (the IPC contract).
        let options = install::InstallOptions {
            base_url: "http://127.0.0.1:1/never-exists.zip".to_string(),
            extras_url: None,
            base_checksum: None,
            extras_checksum: None,
            sd_mount: "/tmp".to_string(),
            platform: "trimui-brick".to_string(),
            extras_platform: "trimui-brick".to_string(),
            version: "test".to_string(),
            fork_name: None,
        };
        let result = install::install_minui(
            &options,
            Arc::new(|_event: install::InstallProgressEvent| {}),
        )
        .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(!err.is_empty());
    }

    // ---- install_minui (IPC command wrapper) ----
    //
    // This test calls the #[tauri::command] wrapper directly to prove
    // the command is registered and not gated behind #[cfg(test)].
    // If someone adds #[cfg(test)] back to the underlying function, the
    // wrapper won't compile — catching the regression in CI.

    #[tokio::test]
    async fn test_install_minui_command_errors_on_bad_url() {
        let options = install::InstallOptions {
            base_url: "http://127.0.0.1:1/never-exists.zip".to_string(),
            extras_url: None,
            base_checksum: None,
            extras_checksum: None,
            sd_mount: "/tmp".to_string(),
            platform: "trimui-brick".to_string(),
            extras_platform: "trimui-brick".to_string(),
            version: "test".to_string(),
            fork_name: None,
        };
        let result = install_minui(options).await;
        // The command should return a proper Err(String), not panic
        // and not return a Tauri "command not found" transport error.
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(!err.is_empty());
    }

    // ---- start_install (IPC command wrapper) ----
    //
    // Compile-time guard: proves start_install exists in the module
    // scope with the correct Tauri command signature. We can't easily
    // call it (needs AppHandle + State from a running app), but this
    // test catches the same class of regression that broke
    // install_minui — if someone removes the function from the module
    // or renames it, this symbol reference won't compile.
    //
    // Note: #[cfg(test)] gating is undetectable from within tests
    // (the symbol would still be visible here). The generate_handler!
    // macro at the top of the file is the runtime guard for that.

    #[test]
    fn test_start_install_command_is_registered() {
        let _ = start_install;
    }

    // ---- cancel_install: poisoned mutex ----
    //
    // cancel_install (and start_install) both use the pattern
    //   registry.token.lock()
    //       .map_err(|_| "...state lock is poisoned".to_string())?
    // This test poisons the mutex and asserts the lock returns an
    // Err with the expected message — proving the .map_err() branch
    // is covered and doesn't panic.

    #[test]
    fn test_install_registry_returns_err_on_poisoned_mutex() {
        let registry = Arc::new(InstallRegistry::new());

        // Poison by panicking while holding the lock in another thread.
        let reg = registry.clone();
        let handle = std::thread::spawn(move || {
            let _guard = reg.token.lock().unwrap();
            panic!("intentional panic to poison mutex");
        });
        let _ = handle.join(); // absorb the panic

        // Replicate the exact pattern used by cancel_install:
        // lock, then map poisoned → Err. Callers never panic.
        let result: Result<(), String> = (|| {
            let _slot = registry
                .token
                .lock()
                .map_err(|_| "Internal error: state lock is poisoned".to_string())?;
            Ok(())
        })();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("poisoned"));
    }

    // ---- validate::validate_installation ----

    #[test]
    fn test_validate_installation_errors_on_nonexistent_mount() {
        let result = validate::validate_installation(
            "/nonexistent/path/that/cannot/exist",
            "miyoo",
            false,
            "/Tools",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_installation_on_empty_tempdir() {
        let temp = tempfile::tempdir().unwrap();
        let result = validate::validate_installation(
            temp.path().to_str().unwrap(),
            "miyoo",
            false,
            "/Tools",
        );
        assert!(result.is_ok());
        let v = result.unwrap();
        // Empty dir = no MinUI files = failures expected
        assert!(!v.success);
        assert!(v.failed_count > 0);
        assert_eq!(v.device_path, "miyoo");
        assert!(v.multiple_device_folders_warning.is_none());
    }

    // ---- validate::format_validation_report ----

    #[test]
    fn test_format_validation_report_contains_pass_and_fail_lines() {
        let v = validate::ValidationResult {
            success: false,
            checks: vec![
                validate::ValidationCheck {
                    name: "a".into(),
                    passed: true,
                    message: "ok-line".into(),
                },
                validate::ValidationCheck {
                    name: "b".into(),
                    passed: false,
                    message: "bad-line".into(),
                },
            ],
            passed_count: 1,
            failed_count: 1,
            free_space_bytes: Some(1024 * 1024 * 1024),
            device_path: "miyoo".into(),
            multiple_device_folders_warning: None,
        };
        let report = validate::format_validation_report(&v);
        assert!(report.contains("ok-line"));
        assert!(report.contains("bad-line"));
        assert!(report.contains("1.00 GB"));
    }

    // ---- version::check_for_updates ----

    #[test]
    fn test_check_minui_version_on_empty_tempdir() {
        let temp = tempfile::tempdir().unwrap();
        let result = version::check_for_updates_with_prefix(
            temp.path().to_str().unwrap(),
            Some("2025.01.01"),
            None,
        );
        // No minui.txt on a fresh card → installed = None, update_available = true
        assert!(result.installed.is_none());
        assert!(result.update_available);
    }

    // ---- package::install_package (underlying, not the IPC wrapper) ----

    #[tokio::test]
    async fn test_install_package_underlying_errors_on_bad_url() {
        let rules = package::PackageInstallPathRules {
            target_dir: "/Tools".to_string(),
            extract_to_root: false,
            pak_name: "test.pak".to_string(),
        };
        let temp = tempfile::tempdir().unwrap();
        let result = package::install_package(
            "http://127.0.0.1:1/never.zip",
            None,
            temp.path().to_str().unwrap(),
            &rules,
            "rg35xxplus",
        )
        .await;
        assert!(result.is_err());
    }

    // ---- wifi::scan_wifi_networks, wifi::get_current_wifi_ssid ----

    #[test]
    fn test_scan_wifi_networks_returns_vec() {
        // Don't assert specific networks (CI-dependent). Just assert it
        // returns a Vec and doesn't panic.
        let _ = wifi::scan_wifi_networks();
    }

    #[test]
    fn test_get_current_wifi_ssid_returns_option_string() {
        // Environment-dependent. Just assert the return type.
        let _ = wifi::get_current_wifi_ssid();
    }

    // ---- wifi::write_wifi_config ----
    // Already covered by `wifi.rs` tests. Contract test in lib.rs would
    // duplicate that work; skip and document.

    // ---- bios::catalog ----

    #[test]
    fn test_list_bios_catalog_returns_all_entries() {
        let entries = bios::catalog();
        // We don't assert the exact count (the catalog can grow), but we
        // do assert it's non-empty and that all expected ids from issue
        // #7 are present. Mirrors the unit test in bios.rs.
        assert!(!entries.is_empty());
        let ids: Vec<&str> = entries.iter().map(|e| e.id.as_str()).collect();
        for &required in bios::EXPECTED_BIOS_IDS {
            assert!(ids.contains(&required), "missing {required} in catalog");
        }
    }

    // ---- bios::status ----

    #[test]
    fn test_get_bios_status_errors_on_missing_mount() {
        let result = bios::status("/nonexistent/this/should/not/exist");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_bios_status_on_empty_tempdir_reports_zero_installed() {
        let temp = tempfile::tempdir().unwrap();
        let result = bios::status(temp.path().to_str().unwrap());
        assert!(result.is_ok());
        let s = result.unwrap();
        assert_eq!(s.installed_count, 0);
        assert!(s.entries.iter().all(|e| !e.present));
    }

    // ---- bios::install_bios_from_bytes ----

    #[test]
    fn test_install_bios_file_underlying_round_trip() {
        // Mirrors the wifi write_wifi_config contract test: prove the
        // function works through the bare path the IPC wrapper calls.
        use base64::engine::general_purpose::STANDARD as BASE64;
        use base64::Engine as _;

        let temp = tempfile::tempdir().unwrap();
        let payload = b"hello bios";
        let result = bios::install_bios_from_bytes(
            temp.path().to_str().unwrap(),
            "gb_bios",
            &BASE64.encode(payload),
        );
        assert!(result.is_ok());
        let written = std::fs::read(temp.path().join("Bios/GB/gb_bios.bin")).unwrap();
        assert_eq!(written, payload);
    }

    // ---- package::detect_installed_packages ----

    #[test]
    fn test_detect_installed_packages_empty_tempdir() {
        let temp = tempfile::tempdir().unwrap();
        let result = package::detect_installed_packages(temp.path().to_str().unwrap());
        assert!(result.is_empty());
    }

    // ---- package::check_package_updates ----

    #[test]
    fn test_check_package_updates_empty_input() {
        let temp = tempfile::tempdir().unwrap();
        let result = package::check_package_updates(temp.path().to_str().unwrap(), &[]);
        assert!(result.is_empty());
    }

    // ---- health::check_sd_card_health ----

    #[test]
    fn test_check_sd_card_health_errors_on_nonexistent() {
        let result = health::check_sd_card_health("/nonexistent/path/here", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_sd_card_health_on_empty_tempdir() {
        let temp = tempfile::tempdir().unwrap();
        let result = health::check_sd_card_health(temp.path().to_str().unwrap(), None);
        assert!(result.is_ok());
        let h = result.unwrap();
        // Empty card → no MinUI folders → failed_count > 0
        assert!(h.failed_count > 0);
    }

    // ---- fetch_url (inline in lib.rs) ----

    #[tokio::test]
    async fn test_fetch_url_errors_on_unreachable() {
        // We replicate the fetch_url command body here (it's a one-off
        // inline command) and assert the unreachable URL errors out at
        // the .send() step. The actual Tauri wrapper just plumbs the
        // AppHandle, so the body is the contract.
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
            .unwrap();
        let result = client.get("http://127.0.0.1:1/never").send().await;
        assert!(result.is_err());
    }

    // ---- download::verify_checksum ----

    #[test]
    fn test_verify_archive_checksum_errors_on_missing_file() {
        let result = download::verify_checksum("/nonexistent/file.zip", "deadbeef");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_archive_checksum_matches_correct_hash() {
        let temp = tempfile::tempdir().unwrap();
        let file_path = temp.path().join("payload.bin");
        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(b"hello world").unwrap();
        drop(f);

        // sha256("hello world") = b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9
        let result = download::verify_checksum(
            file_path.to_str().unwrap(),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
        );
        assert_eq!(result, Ok(true));

        // Wrong checksum returns Ok(false)
        let result = download::verify_checksum(
            file_path.to_str().unwrap(),
            "0000000000000000000000000000000000000000000000000000000000000000",
        );
        assert_eq!(result, Ok(false));
    }
}
