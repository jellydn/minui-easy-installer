use std::fs;
use std::path::Path;

/// Write WiFi configuration to SD card.
///
/// Creates wifi.txt in the root of the SD card with the format:
/// ```
/// SSID: <network_name>
/// PASS: <password>
/// ```
///
/// This format is compatible with MinUI's Wifi.pak.
pub fn write_wifi_config(sd_mount: &str, ssid: &str, password: &str) -> Result<(), String> {
    let sd_root = Path::new(sd_mount);

    if !sd_root.exists() {
        return Err("SD card mount point does not exist".to_string());
    }

    if ssid.trim().is_empty() {
        return Err("SSID cannot be empty".to_string());
    }

    let wifi_path = sd_root.join("wifi.txt");
    let content = format!("SSID: {}\nPASS: {}\n", ssid.trim(), password);

    fs::write(&wifi_path, content).map_err(|e| format!("Failed to write wifi.txt: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_wifi_config() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        write_wifi_config(sd_root.to_str().unwrap(), "MyNetwork", "MyPassword123").unwrap();

        let wifi_path = sd_root.join("wifi.txt");
        assert!(wifi_path.exists());

        let content = fs::read_to_string(wifi_path).unwrap();
        assert!(content.contains("SSID: MyNetwork"));
        assert!(content.contains("PASS: MyPassword123"));
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

        // Write second config (should overwrite)
        write_wifi_config(sd_root.to_str().unwrap(), "NewSSID", "NewPass").unwrap();

        let content = fs::read_to_string(sd_root.join("wifi.txt")).unwrap();
        assert!(content.contains("SSID: NewSSID"));
        assert!(content.contains("PASS: NewPass"));
        assert!(!content.contains("OldSSID"));
    }
}
