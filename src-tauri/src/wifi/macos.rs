/// macOS WiFi scanning and current-SSID detection.
///
/// Uses `airport` command for scanning on macOS < 14.4.
/// Falls back to parsing `system_profiler SPAirPortDataType` on macOS 14.4+
/// where airport was removed — extracts networks from both "Current Network
/// Information" and "Other Local Wireless Networks" sections.
use std::process::Command;

/// Base indentation used by `system_profiler` for property sections.
/// Section headings and their data lines all start at 10-space indent.
const SYSTEM_PROFILER_INDENT: &str = "          ";

/// Scan for available WiFi networks on macOS.
///
/// Three-tier fallback:
/// 1. `airport -s` — fast full scan on macOS < 14.4
/// 2. Parse `system_profiler SPAirPortDataType` for all visible networks
///    (works on all versions, slower but reliable on 14.4+)
/// 3. `current_ssid()` — single-network last resort
pub(crate) fn scan() -> Vec<String> {
    // Tier 1: try airport command first (fast, full scan)
    if let Some(ssids) = try_airport_scan() {
        if !ssids.is_empty() {
            return ssids;
        }
    }

    // Tier 2: parse system_profiler for all visible networks
    if let Some(networks) = try_scan_system_profiler() {
        if !networks.is_empty() {
            return networks;
        }
    }

    // Tier 3: single-network last resort
    if let Some(ssid) = current_ssid() {
        return vec![ssid];
    }

    Vec::new()
}

fn try_airport_scan() -> Option<Vec<String>> {
    let output = Command::new(
        "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport",
    )
    .arg("-s")
    .output()
    .ok()?;

    if output.status.success() {
        Some(parse_airport_output(&String::from_utf8_lossy(
            &output.stdout,
        )))
    } else {
        None
    }
}

/// Run `system_profiler SPAirPortDataType` and parse all visible networks
/// from both "Current Network Information" and "Other Local Wireless Networks"
/// sections. Results are sorted and deduplicated.
fn try_scan_system_profiler() -> Option<Vec<String>> {
    let output = Command::new("system_profiler")
        .args(["SPAirPortDataType"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut networks = parse_system_profiler_networks(&stdout);
    networks.sort();
    networks.dedup();
    Some(networks)
}

/// Parse `system_profiler SPAirPortDataType` output for all visible WiFi
/// network names in document order.
///
/// Looks for SSID entries in both the "Current Network Information" and
/// "Other Local Wireless Networks" sections. Results are returned in the
/// order they appear (current network first, then other networks). Callers
/// should sort/deduplicate if needed.
pub(crate) fn parse_system_profiler_networks(output: &str) -> Vec<String> {
    let mut networks = Vec::new();
    let mut in_section = false;

    for line in output.lines() {
        let trimmed = line.trim();

        // Enter a section when we see "Current Network Information:"
        // or "Other Local Wireless Networks:"
        if trimmed == "Current Network Information:" || trimmed == "Other Local Wireless Networks:"
        {
            in_section = true;
            continue;
        }

        if !in_section {
            continue;
        }

        // Exit the section on a de-indented line (back to section heading level
        // or higher — system_profiler uses 10-space base indent for properties)
        if !line.starts_with(SYSTEM_PROFILER_INDENT) {
            in_section = false;
            continue;
        }

        // Network names are indented (10+ spaces), end with ':', and aren't
        // known metadata fields
        if trimmed.ends_with(':')
            && !trimmed.starts_with("PHY Mode")
            && !trimmed.starts_with("BSSID")
            && !trimmed.starts_with("Channel")
            && !trimmed.starts_with("Network Type")
            && !trimmed.starts_with("Security")
            && !trimmed.starts_with("Signal / Noise")
        {
            let name = trimmed.trim_end_matches(':').trim();
            if !name.is_empty() {
                networks.push(name.to_string());
            }
        }
    }

    networks
}

/// Get the currently connected WiFi SSID on macOS.
///
/// Uses `parse_system_profiler_networks` — the current network is always
/// the first entry (listed in "Current Network Information" before
/// "Other Local Wireless Networks"). Works on all macOS versions including
/// 14.4+ where `airport` was removed.
pub(crate) fn current_ssid() -> Option<String> {
    let output = Command::new("system_profiler")
        .args(["SPAirPortDataType"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_system_profiler_networks(&stdout).into_iter().next()
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

    // ── parse_system_profiler_networks tests ──

    #[test]
    fn test_parse_system_profiler_current_and_other_networks() {
        // Simulates real system_profiler output with both sections.
        // Raw string preserves the columnar indentation that the parser
        // relies on (line continuations with \ would strip whitespace).
        let output = r"
      Software Versions:
          CoreWLAN: 18.0
      Interfaces:
        en0:
          Card Type: AirPort Extreme
          MAC Address: aa:bb:cc:dd:ee:ff
          Current Network Information:
            MyNetwork:
              PHY Mode: 802.11ax
              BSSID: 00:11:22:33:44:55
              Channel: 6
              Network Type: Infrastructure
              Security: WPA2 Personal
              Signal / Noise: -50 dBm / -90 dBm
          Other Local Wireless Networks:
            NeighborNet1:
              PHY Mode: 802.11ac
              Channel: 11
            NeighborNet2:
              PHY Mode: 802.11ax
              Channel: 149
";

        let networks = parse_system_profiler_networks(output);
        assert_eq!(networks, vec!["MyNetwork", "NeighborNet1", "NeighborNet2"]);
    }

    #[test]
    fn test_parse_system_profiler_only_current_network() {
        let output = r"
      Interfaces:
        en0:
          Current Network Information:
            HomeWiFi:
              PHY Mode: 802.11ax
";

        let networks = parse_system_profiler_networks(output);
        assert_eq!(networks, vec!["HomeWiFi"]);
    }

    #[test]
    fn test_parse_system_profiler_no_networks() {
        let output = r"
      Software Versions:
          CoreWLAN: 18.0
      Interfaces:
        en0:
          Card Type: AirPort Extreme
";

        let networks = parse_system_profiler_networks(output);
        assert!(networks.is_empty());
    }

    #[test]
    fn test_parse_system_profiler_empty_output() {
        let networks = parse_system_profiler_networks("");
        assert!(networks.is_empty());
    }

    #[test]
    fn test_parse_system_profiler_skips_metadata_fields() {
        // SSID-like metadata field names (ending with ':') should not be
        // treated as networks
        let output = r"
      Interfaces:
        en0:
          Current Network Information:
            RealNetwork:
              PHY Mode:
              BSSID:
              Channel:
              Network Type:
              Security:
              Signal / Noise:
";

        let networks = parse_system_profiler_networks(output);
        assert_eq!(networks, vec!["RealNetwork"]);
    }

    #[test]
    fn test_parse_system_profiler_deduplicates() {
        // The same network in both sections appears twice in document order.
        // Dedup is the caller's responsibility (try_scan_system_profiler).
        let output = r"
      Interfaces:
        en0:
          Current Network Information:
            SameNet:
              PHY Mode: 802.11ax
          Other Local Wireless Networks:
            SameNet:
              PHY Mode: 802.11ax
";

        let networks = parse_system_profiler_networks(output);
        assert_eq!(networks, vec!["SameNet", "SameNet"]);
    }
}
