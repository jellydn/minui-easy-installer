use std::fs;
use std::path::Path;
use std::process::Command;

/// Write WiFi configuration to SD card.
///
/// Creates or appends to wifi.txt in the root of the SD card with MinUI's
/// expected format: one `SSID:PASSWORD` per line. Lines starting with `#`
/// are comments and ignored. SSIDs can contain spaces.
///
/// Example:
/// ```text
/// # home
/// My Network:MyPassword123
/// # guest
/// GuestWiFi:guestpass
/// ```
pub fn write_wifi_config(sd_mount: &str, ssid: &str, password: &str) -> Result<(), String> {
    let sd_root = Path::new(sd_mount);

    if !sd_root.exists() {
        return Err("SD card mount point does not exist".to_string());
    }

    let ssid = ssid.trim();
    if ssid.is_empty() {
        return Err("SSID cannot be empty".to_string());
    }

    let wifi_path = sd_root.join("wifi.txt");

    // Read existing entries, filtering out any previous entry for this SSID
    let mut entries = Vec::new();
    if wifi_path.exists() {
        if let Ok(content) = fs::read_to_string(&wifi_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    entries.push(line.to_string());
                } else if let Some(colon_pos) = trimmed.find(':') {
                    let existing_ssid = trimmed[..colon_pos].trim();
                    if existing_ssid != ssid {
                        entries.push(line.to_string());
                    }
                } else {
                    entries.push(line.to_string());
                }
            }
        }
    }

    entries.push(format!("{}:{}", ssid, password));

    let content = format!("{}\n", entries.join("\n"));

    fs::write(&wifi_path, content).map_err(|e| format!("Failed to write wifi.txt: {}", e))?;

    Ok(())
}

/// Get the currently connected WiFi SSID.
///
/// Returns the SSID of the network currently connected, or None if not
/// connected to WiFi or if the platform doesn't support detection.
pub fn get_current_wifi_ssid() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        get_current_wifi_ssid_macos()
    }

    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

#[cfg(target_os = "macos")]
fn get_current_wifi_ssid_macos() -> Option<String> {
    // Use system_profiler — works on all macOS versions including 14.4+
    // where airport was removed and networksetup -getairportnetwork is broken.
    let output = Command::new("system_profiler")
        .args(["SPAirPortDataType"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse: find "Current Network Information:" then the next indented line
    // is the SSID (e.g. "    AirTies4920_97Y9:")
    let mut in_current = false;
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed == "Current Network Information:" {
            in_current = true;
            continue;
        }
        if in_current && line.starts_with("          ") && trimmed.ends_with(':') && !trimmed.contains("PHY Mode") && !trimmed.contains("Network Type") {
            let ssid = trimmed.trim_end_matches(':').trim();
            if !ssid.is_empty() {
                return Some(ssid.to_string());
            }
        }
        if in_current && !line.starts_with("          ") {
            break;
        }
    }

    None
}

/// Scan for available WiFi networks.
///
/// Returns a list of SSIDs found nearby. Uses platform-specific commands.
/// Falls back to the currently connected SSID if scanning is unavailable.
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
            let ssids = parse_airport_output(&stdout);
            if !ssids.is_empty() {
                return ssids;
            }
        }
    }

    // Fallback: detect the currently connected network (works on macOS 14.4+)
    if let Some(ssid) = get_current_wifi_ssid_macos() {
        return vec![ssid];
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
