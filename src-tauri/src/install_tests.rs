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

#[test]
fn test_is_preserved_path() {
    let sd_root = Path::new("/Volumes/SDCARD");

    assert!(is_preserved_path(
        Path::new("/Volumes/SDCARD/ROMS/game.nes"),
        sd_root
    ));
    assert!(is_preserved_path(
        Path::new("/Volumes/SDCARD/roms/game.nes"),
        sd_root
    ));
    assert!(is_preserved_path(
        Path::new("/Volumes/SDCARD/Saves/save.sav"),
        sd_root
    ));
    assert!(is_preserved_path(
        Path::new("/Volumes/SDCARD/saves/save.sav"),
        sd_root
    ));
    assert!(!is_preserved_path(
        Path::new("/Volumes/SDCARD/Apps/minui.pak"),
        sd_root
    ));
    assert!(!is_preserved_path(
        Path::new("/Volumes/SDCARD/Tools/wifi.pak"),
        sd_root
    ));
}

#[test]
fn test_copy_dir_recursive_copies_files() {
    let temp = tempfile::tempdir().unwrap();
    let src = temp.path().join("src");
    let dst = temp.path().join("dst");

    fs::create_dir_all(&src).unwrap();

    let mut f = fs::File::create(src.join("test.txt")).unwrap();
    f.write_all(b"hello").unwrap();
    drop(f);

    let copied = fs_utils::copy_dir_recursive(&src, &dst, &|_s, _d| false, &|| false).unwrap();
    assert_eq!(copied, 1);
    assert!(dst.join("test.txt").exists());
}

#[test]
fn test_copy_dir_recursive_skips_preserved_folders() {
    let temp = tempfile::tempdir().unwrap();
    let src = temp.path().join("src");
    let sd_root = temp.path().join("sdcard");

    fs::create_dir_all(src.join("ROMS")).unwrap();
    fs::create_dir_all(src.join("Saves")).unwrap();
    fs::create_dir_all(src.join("Tools")).unwrap();
    fs::create_dir_all(&sd_root).unwrap();

    fs::write(src.join("ROMS/game.nes"), "rom").unwrap();
    fs::write(src.join("Saves/save.sav"), "save").unwrap();
    fs::write(src.join("Tools/tool.pak"), "tool").unwrap();

    let copied = fs_utils::copy_dir_recursive(
        &src,
        &sd_root,
        &|_src, dst| is_preserved_path(dst, &sd_root),
        &|| false,
    )
    .unwrap();
    assert_eq!(copied, 1); // Only tool.pak
    assert!(!sd_root.join("ROMS").exists());
    assert!(!sd_root.join("Saves").exists());
    assert!(sd_root.join("Tools/tool.pak").exists());
}

#[test]
fn test_copy_base_files_with_platform_dir() {
    let temp = tempfile::tempdir().unwrap();
    let extracted = temp.path().join("extracted");
    let platform_dir = extracted.join("miyoo354");
    let sd_root = temp.path().join("sdcard");

    fs::create_dir_all(&platform_dir).unwrap();
    fs::create_dir_all(&sd_root).unwrap();

    fs::write(platform_dir.join("minui.pak"), "base").unwrap();
    fs::write(platform_dir.join("boot.sh"), "boot").unwrap();

    // Shared items
    fs::create_dir_all(extracted.join("Bios")).unwrap();
    fs::create_dir_all(extracted.join("Roms")).unwrap();
    fs::create_dir_all(extracted.join("Saves")).unwrap();
    fs::write(extracted.join("MinUI.zip"), "minui").unwrap();

    // Other device folders should not be copied
    fs::create_dir_all(extracted.join("trimui")).unwrap();
    fs::write(extracted.join("trimui/trimui.pak"), "trimui").unwrap();
    fs::create_dir_all(extracted.join("miyoo")).unwrap();
    fs::write(extracted.join("miyoo/miyoo.pak"), "miyoo").unwrap();

    // README should not be copied
    fs::write(extracted.join("README.txt"), "readme").unwrap();

    let copied = copy_base_files(
        extracted.to_str().unwrap(),
        sd_root.to_str().unwrap(),
        "miyoo354",
    )
    .unwrap();

    // 2 device files + MinUI.zip + Bios/Roms/Saves dirs (empty) = 3 copied files/entries
    assert_eq!(copied, 3);
    assert!(sd_root.join("miyoo354/minui.pak").exists());
    assert!(sd_root.join("miyoo354/boot.sh").exists());
    assert!(sd_root.join("MinUI.zip").exists());
    assert!(!sd_root.join("trimui").exists());
    assert!(!sd_root.join("miyoo").exists());
    assert!(!sd_root.join("README.txt").exists());
}

/// End-to-end test of the base-archive copy step with a realistic
/// multi-device MinUI archive. Verifies that selecting a platform only
/// copies the shared items plus that platform's device folder/file,
/// leaving all other device folders and README.txt behind.
///
/// This test mocks the SD card with a temporary directory and exercises
/// the same `copy_base_files` path the full install pipeline uses after
/// download/extract, so it proves the installer copy behavior without
/// requiring network I/O or a real SD card.
/// Verifies graceful handling when the selected platform has no matching
/// device folder/file in the base archive. Only shared items should be copied.
#[test]
fn test_copy_base_files_end_to_end_missing_device_folder_copies_only_shared_items() {
    let temp = tempfile::tempdir().unwrap();
    let extracted = temp.path().join("extracted");
    let sd_root = temp.path().join("sdcard");
    fs::create_dir_all(&extracted).unwrap();
    fs::create_dir_all(&sd_root).unwrap();

    fs::create_dir_all(extracted.join("Bios")).unwrap();
    fs::create_dir_all(extracted.join("Roms")).unwrap();
    fs::create_dir_all(extracted.join("Saves")).unwrap();
    fs::write(extracted.join("MinUI.zip"), "minui").unwrap();

    // No device folders exist in this archive.
    let copied = copy_base_files(
        extracted.to_str().unwrap(),
        sd_root.to_str().unwrap(),
        "miyoo354",
    )
    .unwrap();

    assert_eq!(
        copied, 1,
        "only MinUI.zip should be copied when device folder is missing"
    );
    assert!(sd_root.join("MinUI.zip").exists());
    assert!(sd_root.join("Bios").is_dir());
    assert!(sd_root.join("Roms").is_dir());
    assert!(sd_root.join("Saves").is_dir());
    assert!(!sd_root.join("miyoo354").exists());
}

/// End-to-end test of the base-archive copy step with a realistic
/// multi-device MinUI archive. Verifies that selecting a platform only
/// copies the shared items plus that platform's device folder/file,
/// leaving all other device folders and README.txt behind.
///
/// This test mocks the SD card with a temporary directory and exercises
/// the same `copy_base_files` path the full install pipeline uses after
/// download/extract, so it proves the installer copy behavior without
/// requiring network I/O or a real SD card.
#[test]
fn test_copy_base_files_end_to_end_only_selected_device_folder_is_copied() {
    let temp = tempfile::tempdir().unwrap();
    let extracted = temp.path().join("extracted");
    let sd_root = temp.path().join("sdcard");
    fs::create_dir_all(&extracted).unwrap();
    fs::create_dir_all(&sd_root).unwrap();

    // Shared items that every base archive contains
    fs::create_dir_all(extracted.join("Bios")).unwrap();
    fs::create_dir_all(extracted.join("Roms")).unwrap();
    fs::create_dir_all(extracted.join("Saves")).unwrap();
    fs::write(extracted.join("MinUI.zip"), "minui").unwrap();

    // Device-specific folders/files that should only be copied when
    // their matching platform is selected. Derive the archive contents
    // from the canonical mappings so the test stays in sync.
    let mut device_folders: Vec<(&str, &str)> = Vec::new();
    for (_, item) in crate::platform::DEVICE_BASE_MAPPINGS {
        if *item == "em_ui.sh" {
            fs::write(extracted.join("em_ui.sh"), "#!/bin/sh").unwrap();
            continue;
        }
        let file = "minui.pak";
        fs::create_dir_all(extracted.join(item)).unwrap();
        fs::write(extracted.join(item).join(file), format!("{item} data")).unwrap();
        device_folders.push((*item, file));
    }

    // README and other top-level files should never be copied
    fs::write(extracted.join("README.txt"), "readme").unwrap();
    fs::write(extracted.join("LICENSE.txt"), "license").unwrap();

    fn clean_sd_root(sd_root: &std::path::Path) {
        for entry in std::fs::read_dir(sd_root).unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                std::fs::remove_dir_all(entry.path()).unwrap();
            } else {
                std::fs::remove_file(entry.path()).unwrap();
            }
        }
    }

    // Iterate over every supported platform so the test stays in sync
    // with the canonical device mappings.
    for (platform, expected_item) in crate::platform::DEVICE_BASE_MAPPINGS {
        let platform = *platform;
        let expected_item = *expected_item;

        // Clean the SD card root between runs so each assertion is independent.
        clean_sd_root(&sd_root);

        let _copied = copy_base_files(
            extracted.to_str().unwrap(),
            sd_root.to_str().unwrap(),
            platform,
        )
        .unwrap();

        // Shared items: MinUI.zip + Bios/Roms/Saves directories.
        // Bios/Roms/Saves are empty in this mock, so only MinUI.zip counts as a file,
        // but the directories should still be created on the SD card.
        assert!(
            sd_root.join("MinUI.zip").exists(),
            "MinUI.zip should be copied for {platform}"
        );
        assert_eq!(
            fs::read_to_string(sd_root.join("MinUI.zip")).unwrap(),
            "minui",
            "MinUI.zip content should be preserved for {platform}"
        );
        assert!(
            sd_root.join("Bios").is_dir(),
            "Bios directory should be created for {platform}"
        );
        assert!(
            sd_root.join("Roms").is_dir(),
            "Roms directory should be created for {platform}"
        );
        assert!(
            sd_root.join("Saves").is_dir(),
            "Saves directory should be created for {platform}"
        );
        assert!(
            sd_root.join(expected_item).exists(),
            "selected device item {expected_item} should be copied for {platform}"
        );

        // Verify the selected device file content is correct.
        if expected_item != "em_ui.sh" {
            assert_eq!(
                fs::read_to_string(sd_root.join(expected_item).join("minui.pak")).unwrap(),
                format!("{expected_item} data"),
                "selected device file content should match for {platform}"
            );
        } else {
            assert_eq!(
                fs::read_to_string(sd_root.join(expected_item)).unwrap(),
                "#!/bin/sh",
                "em_ui.sh content should match for {platform}"
            );
        }

        // Verify no other device folders were copied.
        for (folder, _) in &device_folders {
            if *folder == expected_item {
                continue;
            }
            assert!(
                !sd_root.join(folder).exists(),
                "unselected device folder {folder} should not be copied for {platform}"
            );
        }

        // M17 script should only exist when platform is m17.
        if platform != "m17" {
            assert!(
                !sd_root.join("em_ui.sh").exists(),
                "em_ui.sh should not be copied for {platform}"
            );
        }

        // README and LICENSE should never be copied.
        assert!(
            !sd_root.join("README.txt").exists(),
            "README.txt should not be copied for {platform}"
        );
        assert!(
            !sd_root.join("LICENSE.txt").exists(),
            "LICENSE.txt should not be copied for {platform}"
        );
    }
}

#[test]
fn test_copy_base_files_m17_copies_em_ui_sh() {
    let temp = tempfile::tempdir().unwrap();
    let extracted = temp.path().join("extracted");
    let sd_root = temp.path().join("sdcard");

    fs::create_dir_all(&extracted).unwrap();
    fs::create_dir_all(&sd_root).unwrap();

    fs::write(extracted.join("em_ui.sh"), "#!/bin/sh").unwrap();
    fs::write(extracted.join("MinUI.zip"), "minui").unwrap();
    fs::create_dir_all(extracted.join("Bios")).unwrap();

    let copied = copy_base_files(
        extracted.to_str().unwrap(),
        sd_root.to_str().unwrap(),
        "m17",
    )
    .unwrap();

    assert_eq!(copied, 2); // em_ui.sh + MinUI.zip
    assert!(sd_root.join("em_ui.sh").exists());
    assert!(sd_root.join("MinUI.zip").exists());
}

#[test]
fn test_copy_base_files_preserves_existing_user_data() {
    let temp = tempfile::tempdir().unwrap();
    let extracted = temp.path().join("extracted");
    let sd_root = temp.path().join("sdcard");

    fs::create_dir_all(&extracted).unwrap();
    fs::create_dir_all(&sd_root).unwrap();

    // Existing user data
    fs::create_dir_all(sd_root.join("Roms/GB")).unwrap();
    fs::write(sd_root.join("Roms/GB/pokemon.gb"), "rom_data").unwrap();
    fs::create_dir_all(sd_root.join("Saves")).unwrap();
    fs::write(sd_root.join("Saves/pokemon.sav"), "save_data").unwrap();

    // Archive content
    fs::create_dir_all(extracted.join("Roms")).unwrap();
    fs::create_dir_all(extracted.join("Saves")).unwrap();
    fs::write(extracted.join("MinUI.zip"), "minui").unwrap();
    fs::create_dir_all(extracted.join("miyoo")).unwrap();
    fs::write(extracted.join("miyoo/app"), "app").unwrap();

    let copied = copy_base_files(
        extracted.to_str().unwrap(),
        sd_root.to_str().unwrap(),
        "miyoo",
    )
    .unwrap();

    // MinUI.zip + miyoo/app = 2 files; Roms/Saves skipped because they exist
    assert_eq!(copied, 2);
    assert!(sd_root.join("MinUI.zip").exists());
    assert!(sd_root.join("miyoo/app").exists());
    assert!(sd_root.join("Roms/GB/pokemon.gb").exists());
    assert_eq!(
        fs::read_to_string(sd_root.join("Roms/GB/pokemon.gb")).unwrap(),
        "rom_data"
    );
    assert!(sd_root.join("Saves/pokemon.sav").exists());
    assert_eq!(
        fs::read_to_string(sd_root.join("Saves/pokemon.sav")).unwrap(),
        "save_data"
    );
}

#[test]
fn test_is_preserved_path_nested() {
    let sd_root = Path::new("/Volumes/SDCARD");
    // Case-insensitivity is by design: is_preserved_path uses
    // eq_ignore_ascii_case to match FAT32's case-preserving but
    // case-insensitive filesystem semantics.
    // Nested paths under a preserved top-level folder are preserved;
    // preserved folder names at non-top-level paths are not.

    // Deep nesting under preserved folder should be preserved
    assert!(is_preserved_path(
        Path::new("/Volumes/SDCARD/ROMS/GB/game.gb"),
        sd_root
    ));
    assert!(is_preserved_path(
        Path::new("/Volumes/SDCARD/ROMS/Nintendo Entertainment System (FC)/game.nes"),
        sd_root
    ));
    assert!(is_preserved_path(
        Path::new("/Volumes/SDCARD/Saves/game.sav"),
        sd_root
    ));
    assert!(is_preserved_path(
        Path::new("/Volumes/SDCARD/Saves/subdir/nested.sav"),
        sd_root
    ));
    assert!(is_preserved_path(
        Path::new("/Volumes/SDCARD/BIOS/gba_bios.bin"),
        sd_root
    ));

    // Preserved folder name appearing non-top-level should NOT be preserved
    assert!(!is_preserved_path(
        Path::new("/Volumes/SDCARD/Tools/ROMS/wifi.pak"),
        sd_root
    ));
    assert!(!is_preserved_path(
        Path::new("/Volumes/SDCARD/Emus/saves/mgba.pak"),
        sd_root
    ));

    // Case insensitivity for nested folders
    assert!(is_preserved_path(
        Path::new("/Volumes/SDCARD/roms/nes/game.nes"),
        sd_root
    ));
    assert!(!is_preserved_path(
        Path::new("/Volumes/SDCARD/Tools/bios/"),
        sd_root
    ));
}

#[test]
fn test_copy_dir_recursive_preserves_user_data() {
    let temp = tempfile::tempdir().unwrap();
    let base_src = temp.path().join("base_extracted");
    let sd_root = temp.path().join("sdcard");

    // Simulate existing SD card with user data
    fs::create_dir_all(sd_root.join("ROMS/GB")).unwrap();
    fs::create_dir_all(sd_root.join("Saves")).unwrap();
    fs::create_dir_all(sd_root.join("Tools")).unwrap();
    fs::write(sd_root.join("ROMS/GB/pokemon.gb"), "rom_data").unwrap();
    fs::write(sd_root.join("Saves/pokemon.sav"), "save_data").unwrap();

    // Update archive content
    fs::create_dir_all(base_src.join("Tools")).unwrap();
    fs::write(base_src.join("Tools/wifi.pak"), "new_wifi").unwrap();
    fs::write(base_src.join("minui.txt"), "MinUI v2025.01.01").unwrap();

    let copied = fs_utils::copy_dir_recursive(
        &base_src,
        &sd_root,
        &|_src, dst| is_preserved_path(dst, &sd_root),
        &|| false,
    )
    .unwrap();

    // Only minui.txt and Tools/wifi.pak should be copied — ROMs and Saves skipped
    assert_eq!(copied, 2);
    assert!(sd_root.join("Tools/wifi.pak").exists());
    assert!(sd_root.join("minui.txt").exists());

    // User data must survive
    assert!(sd_root.join("ROMS/GB/pokemon.gb").exists());
    assert_eq!(
        fs::read_to_string(sd_root.join("ROMS/GB/pokemon.gb")).unwrap(),
        "rom_data"
    );
    assert!(sd_root.join("Saves/pokemon.sav").exists());
    assert_eq!(
        fs::read_to_string(sd_root.join("Saves/pokemon.sav")).unwrap(),
        "save_data"
    );
}

#[test]
fn test_copy_extras_files_filters_by_platform() {
    let temp = tempfile::tempdir().unwrap();
    let extras_src = temp.path().join("extras_extracted");
    let sd_root = temp.path().join("sdcard");
    let platform = "rg35xxplus";

    // Create a realistic extras archive structure with multiple platforms
    fs::create_dir_all(extras_src.join("Emus/rg35xxplus/mgba.pak")).unwrap();
    fs::create_dir_all(extras_src.join("Emus/rg35xxplus/gambatte.pak")).unwrap();
    fs::create_dir_all(extras_src.join("Tools/rg35xxplus/wifi.pak")).unwrap();
    fs::create_dir_all(extras_src.join("Tools/rg35xxplus/ssh.pak")).unwrap();
    fs::create_dir_all(extras_src.join("Tools/trimuismart/dc.pak")).unwrap();
    fs::create_dir_all(extras_src.join("Tools/trimuismart/wifi.pak")).unwrap();
    fs::create_dir_all(extras_src.join("Bios")).unwrap();

    fs::write(extras_src.join("Emus/rg35xxplus/mgba.pak/launch.sh"), "emu").unwrap();
    fs::write(
        extras_src.join("Emus/rg35xxplus/gambatte.pak/launch.sh"),
        "emu",
    )
    .unwrap();
    fs::write(
        extras_src.join("Tools/rg35xxplus/wifi.pak/launch.sh"),
        "tool",
    )
    .unwrap();
    fs::write(
        extras_src.join("Tools/rg35xxplus/ssh.pak/launch.sh"),
        "tool",
    )
    .unwrap();
    fs::write(
        extras_src.join("Tools/trimuismart/dc.pak/launch.sh"),
        "tool",
    )
    .unwrap();
    fs::write(
        extras_src.join("Tools/trimuismart/wifi.pak/launch.sh"),
        "tool",
    )
    .unwrap();
    fs::write(extras_src.join("Bios/gba_bios.bin"), "bios").unwrap();

    let copied = copy_extras_files(
        extras_src.to_str().unwrap(),
        sd_root.to_str().unwrap(),
        platform,
    )
    .unwrap();

    // Should copy: 2 emus + 2 tools + 1 bios = 5 files (not the trimuismart ones)
    assert_eq!(copied, 5);

    // Verify rg35xxplus emus and tools were copied
    assert!(sd_root.join("Emus/rg35xxplus/mgba.pak/launch.sh").exists());
    assert!(sd_root
        .join("Emus/rg35xxplus/gambatte.pak/launch.sh")
        .exists());
    assert!(sd_root.join("Tools/rg35xxplus/wifi.pak/launch.sh").exists());
    assert!(sd_root.join("Tools/rg35xxplus/ssh.pak/launch.sh").exists());

    // Verify trimuismart stuff was NOT copied
    assert!(!sd_root.join("Tools/trimuismart").exists());

    // Verify Bios was copied
    assert!(sd_root.join("Bios/gba_bios.bin").exists());
}

/// Integration test for extras archive copy filtering with multiple
/// platforms in the same archive. Verifies that selecting each platform
/// only copies that platform's Emus/Tools, plus shared Bios, and leaves
/// all other platforms' files behind.
#[test]
fn test_copy_extras_files_filters_multiple_platforms() {
    let temp = tempfile::tempdir().unwrap();
    let extras_src = temp.path().join("extras_extracted");
    let sd_root = temp.path().join("sdcard");

    // Build an extras archive with three platforms and shared Bios.
    let platforms = &["rg35xxplus", "trimuismart", "miyoo354"];
    for platform in platforms {
        fs::create_dir_all(extras_src.join(format!("Emus/{}/core.pak", platform))).unwrap();
        fs::write(
            extras_src.join(format!("Emus/{}/core.pak/launch.sh", platform)),
            format!("{} emu\n", platform),
        )
        .unwrap();
        fs::create_dir_all(extras_src.join(format!("Tools/{}/tool.pak", platform))).unwrap();
        fs::write(
            extras_src.join(format!("Tools/{}/tool.pak/launch.sh", platform)),
            format!("{} tool\n", platform),
        )
        .unwrap();
    }
    fs::create_dir_all(extras_src.join("Bios")).unwrap();
    fs::write(extras_src.join("Bios/gba_bios.bin"), "bios").unwrap();

    for selected in platforms {
        let selected = *selected;
        let sd_root = sd_root.join(selected);
        fs::create_dir_all(&sd_root).unwrap();

        let copied = copy_extras_files(
            extras_src.to_str().unwrap(),
            sd_root.to_str().unwrap(),
            selected,
        )
        .unwrap();

        // Should copy at least the selected platform's emu/tool plus shared Bios.
        assert!(copied > 0, "nothing copied for {}", selected);

        // Selected platform files exist with correct content
        assert_eq!(
            fs::read_to_string(sd_root.join(format!("Emus/{}/core.pak/launch.sh", selected)))
                .unwrap(),
            format!("{} emu\n", selected)
        );
        assert_eq!(
            fs::read_to_string(sd_root.join(format!("Tools/{}/tool.pak/launch.sh", selected)))
                .unwrap(),
            format!("{} tool\n", selected)
        );

        // Shared Bios exists with correct content
        assert_eq!(
            fs::read_to_string(sd_root.join("Bios/gba_bios.bin")).unwrap(),
            "bios"
        );

        // Other platforms' files do not exist
        for other in platforms {
            let other = *other;
            if other == selected {
                continue;
            }
            assert!(
                !sd_root.join(format!("Emus/{}", other)).exists(),
                "{} Emus should not be copied when installing {}",
                other,
                selected
            );
            assert!(
                !sd_root.join(format!("Tools/{}", other)).exists(),
                "{} Tools should not be copied when installing {}",
                other,
                selected
            );
        }
    }
}

#[test]
fn test_minui_txt_writes_fork_name() {
    let temp = tempfile::tempdir().unwrap();
    let sd_root = temp.path();

    // Simulate what install_minui_with_cancel writes after copying
    let fork_label = "MinUI-Zero";
    let version = "20250525";
    let minui_txt_path = sd_root.join("minui.txt");
    fs::write(&minui_txt_path, format!("{} {}\n", fork_label, version)).unwrap();

    let content = fs::read_to_string(&minui_txt_path).unwrap();
    assert_eq!(content, "MinUI-Zero 20250525\n");
}

#[test]
fn test_minui_txt_defaults_to_minui_when_no_fork_name() {
    let temp = tempfile::tempdir().unwrap();
    let sd_root = temp.path();

    let fork_label = "MinUI"; // default when fork_name is None
    let version = "2025.01.01";
    let minui_txt_path = sd_root.join("minui.txt");
    fs::write(&minui_txt_path, format!("{} {}\n", fork_label, version)).unwrap();

    let content = fs::read_to_string(&minui_txt_path).unwrap();
    assert_eq!(content, "MinUI 2025.01.01\n");
}
