mod download;
mod drives;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_removable_drives,
            download_and_verify_archive,
            verify_archive_checksum
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
