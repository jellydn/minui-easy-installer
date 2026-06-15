use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

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
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_removable_drives,
            format_drive,
            download_and_verify_archive,
            verify_archive_checksum,
            extract_archive_to_directory,
            install_minui,
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

    // ---- validate::validate_installation ----

    #[test]
    fn test_validate_installation_errors_on_nonexistent_mount() {
        let result = validate::validate_installation(
            "/nonexistent/path/that/cannot/exist",
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
            false,
            "/Tools",
        );
        assert!(result.is_ok());
        let v = result.unwrap();
        // Empty dir = no MinUI files = failures expected
        assert!(!v.success);
        assert!(v.failed_count > 0);
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
        let result =
            version::check_for_updates(temp.path().to_str().unwrap(), Some("2025.01.01"));
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

    // ---- download::download_archive ----

    #[tokio::test]
    async fn test_download_and_verify_archive_errors_on_unreachable() {
        let result = download::download_archive("http://127.0.0.1:1/never.zip", None).await;
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
        let result =
            download::verify_checksum(file_path.to_str().unwrap(), "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
        assert_eq!(result, Ok(true));

        // Wrong checksum returns Ok(false)
        let result =
            download::verify_checksum(file_path.to_str().unwrap(), "0000000000000000000000000000000000000000000000000000000000000000");
        assert_eq!(result, Ok(false));
    }

    // ---- extract::extract_archive_to_directory ----
    // Already covered by `extract.rs` tests. Contract test in lib.rs would
    // duplicate that work; skip and document.
}
