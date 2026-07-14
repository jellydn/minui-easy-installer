use super::*;
use std::fs;

#[test]
fn test_write_wifi_config() {
    let temp = tempfile::tempdir().unwrap();
    let sd_root = temp.path();

    write_wifi_config(sd_root.to_str().unwrap(), "MyNetwork", "MyPassword123").unwrap();

    let wifi_path = sd_root.join("wifi.txt");
    assert!(wifi_path.exists());

    let content = fs::read_to_string(wifi_path).unwrap();
    // MinUI format: SSID:PASSWORD on one line
    assert!(content.contains("MyNetwork:MyPassword123"));
}

#[test]
fn test_write_wifi_config_empty_ssid() {
    let temp = tempfile::tempdir().unwrap();
    let result = write_wifi_config(temp.path().to_str().unwrap(), "", "password");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SSID cannot be empty"));
}

#[test]
fn test_write_wifi_config_nonexistent_mount() {
    let result = write_wifi_config("/nonexistent/path", "SSID", "pass");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}

#[test]
fn test_write_wifi_config_overwrites_existing() {
    let temp = tempfile::tempdir().unwrap();
    let sd_root = temp.path();

    // Write first config
    write_wifi_config(sd_root.to_str().unwrap(), "OldSSID", "OldPass").unwrap();

    // Write second config for a different SSID (both should be preserved)
    write_wifi_config(sd_root.to_str().unwrap(), "NewSSID", "NewPass").unwrap();

    let content = fs::read_to_string(sd_root.join("wifi.txt")).unwrap();
    assert!(content.contains("OldSSID:OldPass"));
    assert!(content.contains("NewSSID:NewPass"));
}

#[test]
fn test_write_wifi_config_updates_same_ssid() {
    let temp = tempfile::tempdir().unwrap();
    let sd_root = temp.path();

    write_wifi_config(sd_root.to_str().unwrap(), "MyNetwork", "OldPass").unwrap();
    write_wifi_config(sd_root.to_str().unwrap(), "MyNetwork", "NewPass").unwrap();

    let content = fs::read_to_string(sd_root.join("wifi.txt")).unwrap();
    assert!(content.contains("MyNetwork:NewPass"));
    assert!(!content.contains("OldPass"));
}

#[test]
fn test_write_wifi_config_ssid_with_spaces() {
    let temp = tempfile::tempdir().unwrap();
    let sd_root = temp.path();

    write_wifi_config(sd_root.to_str().unwrap(), "awesome wifi for home", "secret").unwrap();

    let content = fs::read_to_string(sd_root.join("wifi.txt")).unwrap();
    assert!(content.contains("awesome wifi for home:secret"));
}

#[test]
fn test_write_wifi_config_preserves_comments() {
    let temp = tempfile::tempdir().unwrap();
    let sd_root = temp.path();

    // Pre-write a file with comments
    let wifi_path = sd_root.join("wifi.txt");
    fs::write(&wifi_path, "# my home network\n").unwrap();

    write_wifi_config(sd_root.to_str().unwrap(), "MyNetwork", "pass").unwrap();

    let content = fs::read_to_string(wifi_path).unwrap();
    assert!(content.contains("# my home network"));
    assert!(content.contains("MyNetwork:pass"));
}

#[test]
#[cfg(unix)]
fn test_write_wifi_config_rejects_symlink_escape() {
    let temp = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let sd = temp.path();

    let wifi_path = sd.join("wifi.txt");
    let outside_file = outside.path().join("leak.txt");
    fs::write(&outside_file, b"original").unwrap();

    // Create a symlink at target pointing to outside
    std::os::unix::fs::symlink(&outside_file, &wifi_path).unwrap();

    write_wifi_config(sd.to_str().unwrap(), "SSID", "password").unwrap();

    // Verify outside file was NOT modified/followed
    assert_eq!(fs::read(&outside_file).unwrap(), b"original");
    // Verify local file was written as a regular file containing the SSID config
    let content = fs::read_to_string(&wifi_path).unwrap();
    assert!(content.contains("SSID:password"));
    let meta = fs::symlink_metadata(&wifi_path).unwrap();
    assert!(
        !meta.file_type().is_symlink(),
        "wifi.txt must be a regular file, not a symlink"
    );
}

#[test]
fn test_scan_wifi_networks_returns_vec() {
    // This test just verifies the function returns without panicking
    let _networks = scan_wifi_networks();
    // We can't assert specific networks since it depends on the environment
    // Just check it runs without panic
}
