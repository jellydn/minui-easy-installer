/// Windows WiFi scanning.
///
/// Uses `netsh wlan show networks mode=bssid` for scanning.
use std::process::Command;

/// Scan for available WiFi networks on Windows using `netsh`.
pub(crate) fn scan() -> Vec<String> {
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

/// Parse `netsh wlan show networks` output into a deduplicated, sorted list
/// of SSIDs.
///
/// Extracts lines starting with `SSID` and followed by `:`.
pub(crate) fn parse_netsh_output(output: &str) -> Vec<String> {
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
    fn test_parse_netsh_output() {
        let output = "SSID 1 : MyNetwork\n\
                       Network type            : Infrastructure\n\
                       Authentication          : WPA2-Personal\n\
                       SSID 2 : OtherNet\n";

        let ssids = parse_netsh_output(output);
        assert_eq!(ssids, vec!["MyNetwork", "OtherNet"]);
    }
}
