use std::fs;
use std::path::Path;
use std::process::Command;

/// Write WiFi configuration to SD card.
///
/// Creates wifi.txt in the root of the SD card with the format:
/// ```text
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

/// Scan for available WiFi networks.
///
/// Returns a list of SSIDs found nearby. Uses platform-specific commands:
/// - macOS: `airport -s`
/// - Linux: `nmcli -t -f SSID dev wifi`
/// - Windows: `netsh wlan show networks mode=bssid`
///
/// Returns an empty list if scanning is not supported or fails.
pub fn scan_wifi_networks() -> Vec<String> {
    #[cfg(target_os = "macos")]
    {
        scan_wifi_macos()
    }

    #[cfg(target_os = "linux")]
    {
        scan_wifi_linux()
    }

    #[cfg(target_os = "windows")]
    {
        scan_wifi_windows()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Vec::new()
    }
}

#[cfg(target_os = "macos")]
fn scan_wifi_macos() -> Vec<String> {
    // Try airport command first
    let output = Command::new(
        "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport",
    )
    .arg("-s")
    .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return parse_airport_output(&stdout);
        }
    }

    // Fallback to networksetup (less reliable on macOS 14.4+ where airport is removed)
    let output = Command::new("networksetup")
        .arg("-listallhardwareports")
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(ssid) = parse_networksetup_output(&stdout) {
                return vec![ssid];
            }
        }
    }

    Vec::new()
}

#[cfg(target_os = "macos")]
fn parse_airport_output(output: &str) -> Vec<String> {
    let mut ssids = Vec::new();

    for line in output.lines().skip(1) {
        // Skip header line
        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() {
            let ssid = parts[0].trim();
            if !ssid.is_empty() && !ssid.contains(':') {
                // Skip BSSIDs (contain colons)
                ssids.push(ssid.to_string());
            }
        }
    }

    ssids.sort();
    ssids.dedup();
    ssids
}

#[cfg(target_os = "macos")]
fn parse_networksetup_output(_output: &str) -> Option<String> {
    // This is a simplified parser - in reality we'd need to find WiFi interface
    // and get its SSID
    None
}

#[cfg(target_os = "linux")]
fn scan_wifi_linux() -> Vec<String> {
    let output = Command::new("nmcli")
        .arg("-t")
        .arg("-f")
        .arg("SSID")
        .arg("dev")
        .arg("wifi")
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut ssids: Vec<String> = stdout
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            ssids.sort();
            ssids.dedup();
            return ssids;
        }
    }

    Vec::new()
}

#[cfg(target_os = "windows")]
fn scan_wifi_windows() -> Vec<String> {
    let output = Command::new("netsh")
        .arg("wlan")
        .arg("show")
        .arg("networks")
        .arg("mode=bssid")
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return parse_netsh_output(&stdout);
        }
    }

    Vec::new()
}

#[cfg(target_os = "windows")]
fn parse_netsh_output(output: &str) -> Vec<String> {
    let mut ssids = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("SSID") && trimmed.contains(':') {
            if let Some(ssid_part) = trimmed.split(':').nth(1) {
                let ssid = ssid_part.trim();
                if !ssid.is_empty() {
                    ssids.push(ssid.to_string());
                }
            }
        }
    }

    ssids.sort();
    ssids.dedup();
    ssids
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

    #[test]
    fn test_scan_wifi_networks_returns_vec() {
        // This test just verifies the function returns without panicking
        let _networks = scan_wifi_networks();
        // We can't assert specific networks since it depends on the environment
        // Just check it runs without panic
    }

    #[test]
    fn test_parse_airport_output() {
        let output =
            "                            SSID BSSID             RSSI CHANNEL HT CC SECURITY\n\
                       MyNetwork 00:11:22:33:44:55 -50  6       Y  -- WPA2\n\
                       OtherNet  66:77:88:99:AA:BB -60  11      Y  -- WPA2\n";

        let ssids = parse_airport_output(output);
        assert_eq!(ssids, vec!["MyNetwork", "OtherNet"]);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_parse_netsh_output() {
        let output = "SSID 1 : MyNetwork\n\
                       Network type            : Infrastructure\n\
                       Authentication          : WPA2-Personal\n\
                       SSID 2 : OtherNet\n";

        let ssids = parse_netsh_output(output);
        assert_eq!(ssids, vec!["MyNetwork", "OtherNet"]);
    }
}
