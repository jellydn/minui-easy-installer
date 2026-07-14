/// macOS WiFi scanning and current-SSID detection.
///
/// Uses `airport` command for scanning (falls back to `system_profiler`
/// for current-SSID detection on macOS 14.4+ where airport was removed).
use std::process::Command;

/// Scan for available WiFi networks on macOS using the `airport` command.
///
/// Falls back to detecting only the currently connected network if airport
/// scanning is unavailable (e.g. on macOS 14.4+ where airport was removed).
pub(crate) fn scan() -> Vec<String> {
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
    if let Some(ssid) = current_ssid() {
        return vec![ssid];
    }

    Vec::new()
}

/// Get the currently connected WiFi SSID on macOS.
///
/// Uses `system_profiler SPAirPortDataType` — works on all macOS versions
/// including 14.4+ where `airport` was removed and
/// `networksetup -getairportnetwork` is broken.
pub(crate) fn current_ssid() -> Option<String> {
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
        if in_current
            && line.starts_with("          ")
            && trimmed.ends_with(':')
            && !trimmed.contains("PHY Mode")
            && !trimmed.contains("Network Type")
        {
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

/// Parse `airport -s` output into a deduplicated, sorted list of SSIDs.
///
/// Filters out hidden networks (where the BSSID slides into the SSID column)
/// and correctly handles SSIDs that contain colons by checking for strict
/// BSSID format (6 groups of 2 hex digits separated by colons).
pub(crate) fn parse_airport_output(output: &str) -> Vec<String> {
    let mut ssids = Vec::new();

    for line in output.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        let ssid = parts[0].trim();
        if ssid.is_empty() {
            // Hidden network: airport puts the BSSID in the first column.
            // Skip the whole line — we don't have an SSID to report.
            continue;
        }
        // BSSIDs are 6 groups of 2 hex digits separated by colons (17 chars).
        // Require each colon-delimited segment to be exactly 2 hex digits so
        // we don't incorrectly drop user SSIDs that contain colons
        // (e.g. "guest:net:2.4ghz") which would have non-hex characters or
        // segments of the wrong length.
        let is_bssid = ssid.len() == 17
            && ssid.split(':').count() == 6
            && ssid
                .split(':')
                .all(|part| part.len() == 2 && part.chars().all(|c| c.is_ascii_hexdigit()));
        if !is_bssid {
            ssids.push(ssid.to_string());
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
    fn test_parse_airport_output() {
        let output =
            "                            SSID BSSID             RSSI CHANNEL HT CC SECURITY\n\
                       MyNetwork 00:11:22:33:44:55 -50  6       Y  -- WPA2\n\
                       OtherNet  66:77:88:99:AA:BB -60  11      Y  -- WPA2\n";

        let ssids = parse_airport_output(output);
        assert_eq!(ssids, vec!["MyNetwork", "OtherNet"]);
    }

    #[test]
    fn test_parse_airport_output_skips_hidden_ssids() {
        let output =
            "                            SSID BSSID             RSSI CHANNEL HT CC SECURITY\n\
                                 00:11:22:33:44:55 -50  6       Y  -- WPA2\n\
                       Visible 66:77:88:99:AA:BB -60  11      Y  -- WPA2\n";
        let ssids = parse_airport_output(output);
        assert_eq!(ssids, vec!["Visible"]);
    }

    #[test]
    fn test_parse_airport_output_keeps_ssids_with_colons() {
        let output =
            "                            SSID BSSID             RSSI CHANNEL HT CC SECURITY\n\
                       guest:net:2.4ghz 00:11:22:33:44:55 -50  6       Y  -- WPA2\n\
                       ab:cde:f:12:34:56 66:77:88:99:AA:BB -60  11      Y  -- WPA2\n";
        let ssids = parse_airport_output(output);
        assert_eq!(ssids, vec!["ab:cde:f:12:34:56", "guest:net:2.4ghz"]);
    }

    #[test]
    fn test_parse_airport_output_still_drops_strict_bssids() {
        let output =
            "                            SSID BSSID             RSSI CHANNEL HT CC SECURITY\n\
                       00:11:22:33:44:55 -50  6       Y  -- WPA2\n\
                       Visible 66:77:88:99:AA:BB -60  11      Y  -- WPA2\n";
        let ssids = parse_airport_output(output);
        assert_eq!(ssids, vec!["Visible"]);
    }
}
