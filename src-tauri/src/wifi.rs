use std::fs;
use std::path::Path;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

/// WiFi configuration options, received from the frontend via Tauri IPC.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WifiConfig {
    pub sd_mount: String,
    pub ssid: String,
    pub password: String,
}

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

    // If wifi.txt exists (or is a symlink), remove it to break any potential symlink escapes.
    if let Ok(meta) = fs::symlink_metadata(&wifi_path) {
        if meta.is_file() || meta.file_type().is_symlink() {
            fs::remove_file(&wifi_path)
                .map_err(|e| format!("Failed to remove existing wifi.txt file/symlink: {}", e))?;
        }
    }

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
        macos::current_ssid()
    }

    #[cfg(target_os = "linux")]
    {
        linux::current_ssid()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

/// Scan for available WiFi networks.
///
/// Returns a list of SSIDs found nearby. Uses platform-specific commands.
/// Falls back to the currently connected SSID if scanning is unavailable.
pub fn scan_wifi_networks() -> Vec<String> {
    #[cfg(target_os = "macos")]
    {
        macos::scan()
    }

    #[cfg(target_os = "linux")]
    {
        linux::scan()
    }

    #[cfg(target_os = "windows")]
    {
        windows::scan()
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Vec::new()
    }
}

#[cfg(test)]
#[path = "wifi_tests.rs"]
mod tests;
