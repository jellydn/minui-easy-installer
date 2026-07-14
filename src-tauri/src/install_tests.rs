//! Full-pipeline integration tests that exercise the complete
//! download → extract → copy flow with a real HTTP server.

use super::*;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

/// Build a minimal MinUI base archive zip containing the shared items
/// (Bios, Roms, Saves, MinUI.zip) and a device-specific folder.
fn create_minui_base_zip(output_path: &Path, platform: &str) {
    let file = fs::File::create(output_path)
        .unwrap_or_else(|e| panic!("failed to create zip {}: {}", output_path.display(), e));
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default();

    // Shared items
    zip.add_directory("Bios", options).unwrap();
    zip.add_directory("Roms", options).unwrap();
    zip.add_directory("Saves", options).unwrap();
    zip.start_file("MinUI.zip", options).unwrap();
    zip.write_all(b"minui").unwrap();

    // Device-specific folder
    let device_dir = format!("{}/", platform);
    zip.add_directory(&device_dir, options).unwrap();
    let device_file = format!("{}/minui.pak", platform);
    zip.start_file(&device_file, options).unwrap();
    zip.write_all(b"device data").unwrap();

    zip.finish().unwrap();
}

/// Start a one-shot HTTP server that serves `file_path` once.
/// Returns the base URL and a handle that resolves when the request is done.
fn start_one_shot_file_server(file_path: std::path::PathBuf) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let handle = thread::spawn(move || {
        let mut connection = listener.accept().unwrap().0;

        let mut request = [0u8; 1024];
        let _ = connection.read(&mut request);

        let bytes = fs::read(&file_path).unwrap();
        let headers = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/zip\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            bytes.len()
        );
        let mut response = headers.into_bytes();
        response.extend_from_slice(&bytes);
        connection.write_all(&response).unwrap();
    });

    (format!("http://127.0.0.1:{}/MinUI.zip", port), handle)
}

#[tokio::test]
async fn test_install_minui_with_cancel_full_pipeline() {
    let temp = tempfile::tempdir().unwrap();
    let archive_dir = temp.path().join("archive");
    let sd_root = temp.path().join("sdcard");
    fs::create_dir_all(&archive_dir).unwrap();
    fs::create_dir_all(&sd_root).unwrap();

    let zip_path = archive_dir.join("MinUI.zip");
    create_minui_base_zip(&zip_path, "miyoo354");

    let (url, server_handle) = start_one_shot_file_server(zip_path);

    let options = InstallOptions {
        base_url: url,
        extras_url: None,
        base_checksum: None,
        extras_checksum: None,
        sd_mount: sd_root.to_str().unwrap().to_string(),
        platform: "miyoo354".to_string(),
        extras_platform: "miyoo354".to_string(),
        version: "2025.01.01".to_string(),
        fork_name: None,
    };

    let progress_events: Arc<Mutex<Vec<InstallProgressEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let progress_events_clone = Arc::clone(&progress_events);
    let progress = Arc::new(move |event: InstallProgressEvent| {
        progress_events_clone.lock().unwrap().push(event);
    });

    let result = install_minui_with_cancel(
        &options,
        progress,
        Arc::new(|_, _| {}),
        CancellationToken::new(),
    )
    .await
    .expect("full pipeline install should succeed");

    // Wait for the one-shot server to finish so we know the download completed.
    server_handle.join().unwrap();

    assert!(result.success);
    assert!(result.error.is_none());
    assert!(result.base_files_copied > 0);
    assert_eq!(result.extras_files_copied, 0);
    assert_eq!(result.extras_warning, None);
    assert!(result.rom_dirs_created > 0);

    // Verify SD card contents
    assert!(sd_root.join("MinUI.zip").exists());
    assert_eq!(
        fs::read_to_string(sd_root.join("MinUI.zip")).unwrap(),
        "minui"
    );
    assert!(sd_root.join("miyoo354/minui.pak").exists());
    assert_eq!(
        fs::read_to_string(sd_root.join("miyoo354/minui.pak")).unwrap(),
        "device data"
    );
    assert!(sd_root.join("minui.txt").exists());
    assert_eq!(
        fs::read_to_string(sd_root.join("minui.txt")).unwrap(),
        "MinUI 2025.01.01\n"
    );

    // Verify progress events were emitted for the major phases
    let steps: Vec<String> = progress_events
        .lock()
        .unwrap()
        .iter()
        .map(|e| e.step.clone())
        .collect();
    assert!(steps.contains(&"download".to_string()));
    assert!(steps.contains(&"copy".to_string()));
    assert!(steps.contains(&"finish".to_string()));
}
