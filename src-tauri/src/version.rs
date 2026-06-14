use std::fs;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InstalledVersion {
    pub version: String,
    pub source: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VersionCheckResult {
    pub installed: Option<InstalledVersion>,
    pub latest: Option<String>,
    pub update_available: bool,
}

/// Detect installed MinUI version from SD card metadata.
///
/// MinUI stores version info in `/minui.txt` on the SD card root.
/// The file format is simple text with version on the first line.
pub fn detect_installed_version(sd_mount: &str) -> Option<InstalledVersion> {
    let sd_root = Path::new(sd_mount);

    // Check for minui.txt in root
    let minui_txt = sd_root.join("minui.txt");
    if minui_txt.exists() {
        if let Ok(content) = fs::read_to_string(&minui_txt) {
            if let Some(version) = parse_minui_version(&content) {
                return Some(InstalledVersion {
                    version,
                    source: "minui.txt".to_string(),
                });
            }
        }
    }

    // Check for .minui/version file (alternative location)
    let dot_minui_version = sd_root.join(".minui").join("version");
    if dot_minui_version.exists() {
        if let Ok(content) = fs::read_to_string(&dot_minui_version) {
            let version = content.trim().to_string();
            if !version.is_empty() {
                return Some(InstalledVersion {
                    version,
                    source: ".minui/version".to_string(),
                });
            }
        }
    }

    None
}

/// Parse version from minui.txt content.
///
/// Expected format:
/// ```text
/// MinUI v2024.12.25
/// ```
/// or just:
/// ```text
/// 2024.12.25
/// ```
fn parse_minui_version(content: &str) -> Option<String> {
    let first_line = content.lines().next()?.trim();

    // Try to extract version after "MinUI" prefix
    if let Some(rest) = first_line.strip_prefix("MinUI ") {
        let version = rest.trim().trim_start_matches('v').trim();
        if !version.is_empty() {
            return Some(version.to_string());
        }
    }

    // Try to extract version after "v" prefix
    if let Some(version) = first_line.strip_prefix('v') {
        let version = version.trim();
        if !version.is_empty() {
            return Some(version.to_string());
        }
    }

    // If line looks like a version (contains dots or numbers), use it directly
    if !first_line.is_empty()
        && (first_line.contains('.') || first_line.chars().any(|c| c.is_ascii_digit()))
    {
        return Some(first_line.to_string());
    }

    None
}

/// Compare two version strings.
///
/// Returns true if latest is newer than installed.
/// Uses simple string comparison for now (works for date-based versions like 2024.12.25).
pub fn is_update_available(installed: &str, latest: &str) -> bool {
    // Normalize versions by removing 'v' prefix
    let installed_norm = installed.trim().trim_start_matches('v').trim();
    let latest_norm = latest.trim().trim_start_matches('v').trim();

    // Simple string comparison works for date-based versions
    latest_norm > installed_norm
}

/// Check for version updates by comparing installed version with latest release.
pub fn check_for_updates(sd_mount: &str, latest_version: Option<&str>) -> VersionCheckResult {
    let installed = detect_installed_version(sd_mount);

    let update_available = match (&installed, latest_version) {
        (Some(installed_ver), Some(latest_ver)) => {
            is_update_available(&installed_ver.version, latest_ver)
        }
        (None, Some(_)) => true, // No installed version means update available
        _ => false,
    };

    VersionCheckResult {
        installed,
        latest: latest_version.map(|v| v.to_string()),
        update_available,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_minui_version_with_prefix() {
        let content = "MinUI v2024.12.25\nSome other content";
        assert_eq!(parse_minui_version(content), Some("2024.12.25".to_string()));
    }

    #[test]
    fn test_parse_minui_version_without_prefix() {
        let content = "2024.12.25\nSome other content";
        assert_eq!(parse_minui_version(content), Some("2024.12.25".to_string()));
    }

    #[test]
    fn test_parse_minui_version_with_v_prefix() {
        let content = "v2024.12.25";
        assert_eq!(parse_minui_version(content), Some("2024.12.25".to_string()));
    }

    #[test]
    fn test_parse_minui_version_empty() {
        assert_eq!(parse_minui_version(""), None);
    }

    #[test]
    fn test_is_update_available() {
        assert!(is_update_available("2024.12.25", "2025.01.01"));
        assert!(!is_update_available("2025.01.01", "2024.12.25"));
        assert!(!is_update_available("2025.01.01", "2025.01.01"));
    }

    #[test]
    fn test_is_update_available_with_v_prefix() {
        assert!(is_update_available("v2024.12.25", "v2025.01.01"));
        assert!(!is_update_available("v2025.01.01", "v2024.12.25"));
    }

    #[test]
    fn test_detect_installed_version_from_minui_txt() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        let mut f = fs::File::create(sd_root.join("minui.txt")).unwrap();
        f.write_all(b"MinUI v2024.12.25").unwrap();
        drop(f);

        let result = detect_installed_version(sd_root.to_str().unwrap());
        assert!(result.is_some());
        let version = result.unwrap();
        assert_eq!(version.version, "2024.12.25");
        assert_eq!(version.source, "minui.txt");
    }

    #[test]
    fn test_detect_installed_version_no_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let result = detect_installed_version(temp.path().to_str().unwrap());
        assert!(result.is_none());
    }

    #[test]
    fn test_check_for_updates_with_install() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        let mut f = fs::File::create(sd_root.join("minui.txt")).unwrap();
        f.write_all(b"MinUI v2024.12.25").unwrap();
        drop(f);

        let result = check_for_updates(sd_root.to_str().unwrap(), Some("2025.01.01"));
        assert!(result.update_available);
        assert!(result.installed.is_some());
        assert_eq!(result.latest, Some("2025.01.01".to_string()));
    }

    #[test]
    fn test_check_for_updates_no_install() {
        let temp = tempfile::tempdir().unwrap();
        let result = check_for_updates(temp.path().to_str().unwrap(), Some("2025.01.01"));
        assert!(result.update_available);
        assert!(result.installed.is_none());
    }

    #[test]
    fn test_check_for_updates_up_to_date() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        let mut f = fs::File::create(sd_root.join("minui.txt")).unwrap();
        f.write_all(b"MinUI v2025.01.01").unwrap();
        drop(f);

        let result = check_for_updates(sd_root.to_str().unwrap(), Some("2025.01.01"));
        assert!(!result.update_available);
    }
}
