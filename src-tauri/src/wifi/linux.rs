/// Linux WiFi scanning and current-SSID detection.
///
/// Uses `nmcli` for scanning (NetworkManager) and `iwgetid` for
/// current-SSID detection (wireless-tools).
use std::process::Command;

/// Scan for available WiFi networks on Linux using `nmcli`.
pub(crate) fn scan() -> Vec<String> {
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

/// Get the currently connected WiFi SSID on Linux.
///
/// Tries `iwgetid -r` first (WiFi-only, part of wireless-tools).
/// Falls back to `nmcli` filtered to WiFi devices (NetworkManager).
pub(crate) fn current_ssid() -> Option<String> {
    // Try iwgetid first — it only returns WiFi SSIDs, never ethernet.
    if let Ok(output) = Command::new("iwgetid").arg("-r").output() {
        if output.status.success() {
            let ssid = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !ssid.is_empty() {
                return Some(ssid);
            }
        }
    }

    // Fall back to nmcli — filter to WiFi devices only.
    if let Ok(output) = Command::new("nmcli")
        .args([
            "-t",
            "-f",
            "GENERAL.TYPE,GENERAL.CONNECTION",
            "device",
            "show",
        ])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut saw_wifi = false;
            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed == "GENERAL.TYPE:wifi" {
                    saw_wifi = true;
                    continue;
                }
                if saw_wifi && trimmed.starts_with("GENERAL.CONNECTION:") {
                    let ssid = trimmed
                        .strip_prefix("GENERAL.CONNECTION:")
                        .unwrap_or("")
                        .trim();
                    if !ssid.is_empty() {
                        return Some(ssid.to_string());
                    }
                }
                saw_wifi = false;
            }
        }
    }

    None
}
