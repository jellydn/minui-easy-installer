mod download;
mod drives;
mod extract;
mod install;

use tauri::Manager;

#[tauri::command]
fn get_removable_drives() -> Result<Vec<drives::RemovableDrive>, String> {
    drives::list_removable_drives()
}

#[tauri::command]
async fn download_and_verify_archive(
    url: String,
    checksum: Option<String>,
) -> Result<download::DownloadResult, String> {
    let checksum_ref = checksum.as_deref();
    download::download_archive(&url, checksum_ref).await
}

#[tauri::command]
fn verify_archive_checksum(file_path: String, expected_checksum: String) -> Result<bool, String> {
    download::verify_checksum(&file_path, &expected_checksum)
}

#[tauri::command]
fn extract_archive_to_directory(
    archive_path: String,
    destination: Option<String>,
) -> Result<extract::ExtractionResult, String> {
    let dest_ref = destination.as_deref();
    extract::extract_archive(&archive_path, dest_ref)
}

#[tauri::command]
async fn install_minui(
    base_url: String,
    extras_url: Option<String>,
    base_checksum: Option<String>,
    extras_checksum: Option<String>,
    sd_mount: String,
    platform: String,
    extras_dir: String,
) -> Result<install::InstallResult, String> {
    install::install_minui(
        &base_url,
        extras_url.as_deref(),
        base_checksum.as_deref(),
        extras_checksum.as_deref(),
        &sd_mount,
        &platform,
        &extras_dir,
    )
    .await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_removable_drives,
            download_and_verify_archive,
            verify_archive_checksum,
            extract_archive_to_directory,
            install_minui
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
