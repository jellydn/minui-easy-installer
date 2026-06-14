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

/// Try to parse a version string as semver, normalizing leading zeros first.
fn try_parse_semver(v: &str) -> Option<semver::Version> {
    // Direct parse first (fast path for clean semver like "0.12.0")
    if let Ok(ver) = semver::Version::parse(v) {
        return Some(ver);
    }

    // Normalize: strip leading zeros from each dot-separated segment
    // This handles date versions like "2025.01.01" -> "2025.1.1"
    let parts: Vec<&str> = v.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let normalized: String = parts
        .iter()
        .map(|p| p.trim_start_matches('0'))
        .map(|p| if p.is_empty() { "0" } else { p })
        .collect::<Vec<_>>()
        .join(".");
    semver::Version::parse(&normalized).ok()
}

/// Compare two version strings intelligently.
///
/// Tries semver parsing first (works for "0.12.0", "1.2.3", etc.
/// as well as date versions like "2025.01.01" after normalization).
/// Falls back to date-based string comparison when neither parses as semver.
pub fn compare_versions(installed: &str, latest: &str) -> bool {
    let installed = installed.trim().trim_start_matches('v').trim();
    let latest = latest.trim().trim_start_matches('v').trim();

    // Try semver for both
    if let (Some(inst_ver), Some(lat_ver)) =
        (try_parse_semver(installed), try_parse_semver(latest))
    {
        return lat_ver > inst_ver;
    }

    // If one is semver and the other isn't, assume the semver one is more recent
    // (this handles the case where a version format changes)
    if try_parse_semver(installed).is_some() != try_parse_semver(latest).is_some() {
        return try_parse_semver(latest).is_some();
    }

    // Neither parses as semver — fall back to string comparison (works for YYYY.MM.DD)
    latest > installed
}

/// Compare two version strings.
///
/// Returns true if latest is newer than installed.
pub fn is_update_available(installed: &str, latest: &str) -> bool {
    compare_versions(installed, latest)
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
    fn test_compare_versions_semver() {
        // These are the real store.json versions that fail lexicographic comparison
        assert!(compare_versions("0.9.0", "0.12.0"));
        assert!(compare_versions("0.6.2", "0.7.0"));
        assert!(!compare_versions("0.12.0", "0.9.0"));
        assert!(!compare_versions("0.7.0", "0.6.2"));
        assert!(!compare_versions("0.12.0", "0.12.0"));
    }

    #[test]
    fn test_compare_versions_date_based() {
        assert!(compare_versions("2024.11.01", "2024.12.25"));
        assert!(!compare_versions("2024.12.25", "2024.11.01"));
    }

    #[test]
    fn test_compare_versions_garbage() {
        // Both unparseable — falls through to string comparison
        assert!(compare_versions("alpha", "beta"));
        assert!(!compare_versions("beta", "alpha"));
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
