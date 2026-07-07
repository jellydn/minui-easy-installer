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
///
/// When `expected_prefix` is provided (e.g. "MinUI", "MinUI-Zero"),
/// the prefix in minui.txt must match — otherwise the installation is
/// treated as belonging to a different fork and `None` is returned.
pub fn detect_installed_version(
    sd_mount: &str,
    expected_prefix: Option<&str>,
) -> Option<InstalledVersion> {
    let sd_root = Path::new(sd_mount);

    // Check for minui.txt in root
    let minui_txt = sd_root.join("minui.txt");
    if minui_txt.exists() {
        if let Ok(content) = fs::read_to_string(&minui_txt) {
            if let Some((prefix, version)) = parse_minui_version_with_prefix(&content) {
                // If the caller expects a specific fork, verify the prefix matches.
                // This prevents showing "MinUI-Zero: v2024.12.25" when minui.txt
                // was written by official MinUI (or vice versa).
                if let Some(expected) = expected_prefix {
                    if prefix.as_deref() != Some(expected) {
                        return None;
                    }
                }
                return Some(InstalledVersion {
                    version,
                    source: "minui.txt".to_string(),
                });
            }
        }
    }

    // .minui/version is the legacy fallback. It carries no fork prefix,
    // so we cannot verify the fork directly. The rules:
    //   - minui.txt present (any case) — already returned above, never
    //     reach this branch.
    //   - minui.txt missing + canonical "MinUI" expected — accept
    //     .minui/version (legacy official installs only ever wrote this
    //     file, and they always wrote it for the canonical MinUI).
    //   - minui.txt missing + non-canonical fork expected — skip. The
    //     file cannot be proven to belong to a fork we don't recognise.
    //   - minui.txt missing + no prefix expected — accept as before.
    let minui_txt_present = minui_txt.exists();
    if matches!(
        (minui_txt_present, expected_prefix),
        (false, None) | (false, Some("MinUI"))
    ) {
        if let Some(installed) = read_dot_minui_version(sd_root) {
            return Some(installed);
        }
    }

    None
}

fn read_dot_minui_version(sd_root: &Path) -> Option<InstalledVersion> {
    let path = sd_root.join(".minui").join("version");
    let content = fs::read_to_string(&path).ok()?;
    let version = content.trim().to_string();
    if version.is_empty() {
        return None;
    }
    Some(InstalledVersion {
        version,
        source: ".minui/version".to_string(),
    })
}

/// Returns true if `s` looks like a release version string.
///
/// Accepts:
/// - A single numeric run of 6+ digits: "20250525" (date-only, e.g. MinUI-Zero)
/// - 2 or 3 dot-separated numeric segments: "2024.12.25", "0.12.0", "1.2.3", "2024.12", "1.0"
///
/// Rejects free-form text like "Created by MinUI Team 2024" and
/// dash-separated versions like "v2024-12-25".
fn looks_like_version(s: &str) -> bool {
    let s = s.trim().trim_start_matches(['v', 'V']).trim();
    if s.is_empty() {
        return false;
    }

    // Single numeric run of 6+ digits: date-only format (e.g. MinUI-Zero "20250525")
    if s.len() >= 6 && s.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }

    let segments: Vec<&str> = s.split('.').collect();
    if segments.len() < 2 || segments.len() > 3 {
        return false;
    }
    segments
        .iter()
        .all(|seg| !seg.is_empty() && seg.chars().all(|c| c.is_ascii_digit()))
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
///
/// Returns `(prefix, version)` where prefix is the detected fork name
/// (e.g. "MinUI", "MinUI-Zero") or `None` if no prefix was present.
fn parse_minui_version_with_prefix(content: &str) -> Option<(Option<String>, String)> {
    let first_line = content.lines().next()?.trim();

    // Split into prefix and version candidate if a space is present.
    // Handles custom forks/prefixes (e.g., "MinUI v2024.12.25", "MinUI-Zero 20250525")
    let (prefix, version_raw) = if let Some((p, v)) = first_line.rsplit_once(' ') {
        (Some(p.trim().to_string()), v)
    } else {
        (None, first_line)
    };

    // Normalize the version by stripping leading v/V prefix and wrapping spaces
    let version = version_raw
        .trim()
        .trim_start_matches(['v', 'V'])
        .trim()
        .to_string();

    if looks_like_version(&version) {
        Some((prefix, version))
    } else {
        None
    }
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
    if let (Some(inst_ver), Some(lat_ver)) = (try_parse_semver(installed), try_parse_semver(latest))
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
#[allow(dead_code)]
pub fn check_for_updates(sd_mount: &str, latest_version: Option<&str>) -> VersionCheckResult {
    // Backwards compat: no prefix filter by default.
    check_for_updates_with_prefix(sd_mount, latest_version, None)
}

/// Check for version updates, only accepting installations where the
/// minui.txt prefix matches `expected_prefix` (e.g. "MinUI", "MinUI-Zero").
pub fn check_for_updates_with_prefix(
    sd_mount: &str,
    latest_version: Option<&str>,
    expected_prefix: Option<&str>,
) -> VersionCheckResult {
    let installed = detect_installed_version(sd_mount, expected_prefix);

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
        assert_eq!(parse_minui_version_with_prefix(content).map(|(_, v)| v), Some("2024.12.25".to_string()));
    }

    #[test]
    fn test_parse_minui_version_fork_prefix_minui_zero() {
        let content = "MinUI-Zero 20250525\n";
        assert_eq!(parse_minui_version_with_prefix(content).map(|(_, v)| v), Some("20250525".to_string()));
    }

    #[test]
    fn test_parse_minui_version_preserves_trailing_v_in_prefix() {
        // A fork name ending in 'v' (e.g. "MyRev") must not have its
        // trailing 'v' stripped — that was a bug in trim_end_matches('v').
        // The trailing 'v' here is on the *prefix* side of rsplit_once(' ').
        // The 'v' that gets stripped is on the *version* side ("v1.2.3" → "1.2.3").
        let content = "MyRev 1.2.3\n";
        let (prefix, version) = parse_minui_version_with_prefix(content).unwrap();
        assert_eq!(prefix, Some("MyRev".to_string()));
        assert_eq!(version, "1.2.3");
    }

    #[test]
    fn test_parse_minui_version_fork_prefix_with_v() {
        let content = "MinUI-Zero v20250525\n";
        assert_eq!(parse_minui_version_with_prefix(content).map(|(_, v)| v), Some("20250525".to_string()));
    }

    #[test]
    fn test_parse_minui_version_arbitrary_prefix_two_segment() {
        let content = "MyCustomFork 2025.01\n";
        assert_eq!(parse_minui_version_with_prefix(content).map(|(_, v)| v), Some("2025.01".to_string()));
    }

    #[test]
    fn test_parse_minui_version_prefix_rejects_non_version_after_space() {
        // "MyFork not.a.version" — rsplit_once gives "not.a.version" after space
        // but looks_like_version rejects it because "not" and "a" are non-numeric
        let content = "MyFork not.a.version\n";
        assert_eq!(parse_minui_version_with_prefix(content).map(|(_, v)| v), None);
    }

    #[test]
    fn test_parse_minui_version_without_prefix() {
        let content = "2024.12.25\nSome other content";
        assert_eq!(parse_minui_version_with_prefix(content).map(|(_, v)| v), Some("2024.12.25".to_string()));
    }

    #[test]
    fn test_parse_minui_version_with_v_prefix() {
        let content = "v2024.12.25";
        assert_eq!(parse_minui_version_with_prefix(content).map(|(_, v)| v), Some("2024.12.25".to_string()));
    }

    #[test]
    fn test_parse_minui_version_empty() {
        assert_eq!(parse_minui_version_with_prefix("").map(|(_, v)| v), None);
    }

    #[test]
    fn test_looks_like_version_accepts_three_segments() {
        assert!(looks_like_version("2024.12.25"));
        assert!(looks_like_version("0.12.0"));
        assert!(looks_like_version("1.2.3"));
    }

    #[test]
    fn test_looks_like_version_accepts_two_segments() {
        assert!(looks_like_version("2024.12"));
        assert!(looks_like_version("1.0"));
    }

    #[test]
    fn test_looks_like_version_accepts_optional_v_prefix() {
        assert!(looks_like_version("v2024.12.25"));
        assert!(looks_like_version("V2024.12.25"));
    }

    #[test]
    fn test_looks_like_version_accepts_date_only() {
        // MinUI-Zero uses date-only versions like "20250525"
        assert!(looks_like_version("20250525"));
    }

    #[test]
    fn test_looks_like_version_rejects_short_numeric() {
        // 5 digits is not a date (YYYYMMDD needs 8) and not a semver segment
        assert!(!looks_like_version("12345"));
    }

    #[test]
    fn test_looks_like_version_rejects_free_form_text() {
        assert!(!looks_like_version("Created by MinUI Team 2024"));
        assert!(!looks_like_version("MinUI"));
        assert!(!looks_like_version(""));
        assert!(!looks_like_version("v2024-12-25"));
        assert!(!looks_like_version("2024"));
        assert!(!looks_like_version("2024.12.25.1"));
    }

    #[test]
    fn test_parse_minui_version_rejects_free_form_text() {
        // Regression for the "Created by MinUI Team 2024" failure mode.
        assert_eq!(parse_minui_version_with_prefix("Created by MinUI Team 2024\n").map(|(_, v)| v), None);
        assert_eq!(
            parse_minui_version_with_prefix("Released 2024-12-25 by the team\n").map(|(_, v)| v),
            None
        );
    }

    #[test]
    fn test_parse_minui_version_accepts_strict_raw_version() {
        // The raw fallback should still accept a clean version-only line.
        assert_eq!(
            parse_minui_version_with_prefix("2024.12.25\n").map(|(_, v)| v),
            Some("2024.12.25".to_string())
        );
        assert_eq!(
            parse_minui_version_with_prefix("v2024.12.25\n").map(|(_, v)| v),
            Some("2024.12.25".to_string())
        );
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

        let result = detect_installed_version(sd_root.to_str().unwrap(), None);
        assert!(result.is_some());
        let version = result.unwrap();
        assert_eq!(version.version, "2024.12.25");
        assert_eq!(version.source, "minui.txt");
    }

    #[test]
    fn test_detect_installed_version_no_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let result = detect_installed_version(temp.path().to_str().unwrap(), None);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_installed_version_ignores_wrong_fork_prefix() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        // Write minui.txt with official MinUI prefix
        let mut f = fs::File::create(sd_root.join("minui.txt")).unwrap();
        f.write_all(b"MinUI v2024.12.25").unwrap();
        drop(f);

        // When expecting MinUI-Zero, the official prefix should NOT match
        let result = detect_installed_version(sd_root.to_str().unwrap(), Some("MinUI-Zero"));
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_installed_version_matches_correct_fork_prefix() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        let mut f = fs::File::create(sd_root.join("minui.txt")).unwrap();
        f.write_all(b"MinUI-Zero 20250525").unwrap();
        drop(f);

        let result = detect_installed_version(sd_root.to_str().unwrap(), Some("MinUI-Zero"));
        assert!(result.is_some());
        let version = result.unwrap();
        assert_eq!(version.version, "20250525");
    }

    #[test]
    fn test_detect_installed_version_skips_dot_minui_when_prefix_expected() {
        // .minui/version has no fork prefix, so when a prefix is expected
        // the install should NOT be attributed to the selected fork.
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        fs::create_dir_all(sd_root.join(".minui")).unwrap();
        let mut f = fs::File::create(sd_root.join(".minui").join("version")).unwrap();
        f.write_all(b"2024.12.25").unwrap();
        drop(f);

        let result = detect_installed_version(sd_root.to_str().unwrap(), Some("MinUI-Zero"));
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_installed_version_accepts_dot_minui_when_no_prefix() {
        // Backwards-compat: without an expected prefix, .minui/version
        // is still used as the fallback detection path.
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        fs::create_dir_all(sd_root.join(".minui")).unwrap();
        let mut f = fs::File::create(sd_root.join(".minui").join("version")).unwrap();
        f.write_all(b"2024.12.25").unwrap();
        drop(f);

        let result = detect_installed_version(sd_root.to_str().unwrap(), None);
        assert!(result.is_some());
        let version = result.unwrap();
        assert_eq!(version.version, "2024.12.25");
        assert_eq!(version.source, ".minui/version");
    }

    #[test]
    fn test_detect_installed_version_accepts_dot_minui_when_canonical_prefix() {
        // Legacy official MinUI installs only write .minui/version, no
        // minui.txt. The canonical-prefix rule accepts the legacy file
        // when the expected prefix is "MinUI".
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        fs::create_dir_all(sd_root.join(".minui")).unwrap();
        let mut f = fs::File::create(sd_root.join(".minui").join("version")).unwrap();
        f.write_all(b"2024.12.25").unwrap();
        drop(f);

        let result = detect_installed_version(sd_root.to_str().unwrap(), Some("MinUI"));
        assert!(result.is_some());
        let version = result.unwrap();
        assert_eq!(version.version, "2024.12.25");
        assert_eq!(version.source, ".minui/version");
    }

    #[test]
    fn test_detect_installed_version_skips_dot_minui_when_minui_txt_present() {
        // When both files exist, minui.txt takes precedence and the
        // fallback is not used.
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        let mut a = fs::File::create(sd_root.join("minui.txt")).unwrap();
        a.write_all(b"MinUI v2025.01.01").unwrap();
        drop(a);
        fs::create_dir_all(sd_root.join(".minui")).unwrap();
        let mut b = fs::File::create(sd_root.join(".minui").join("version")).unwrap();
        b.write_all(b"2024.12.25").unwrap();
        drop(b);

        let result = detect_installed_version(sd_root.to_str().unwrap(), Some("MinUI"));
        assert!(result.is_some());
        let version = result.unwrap();
        // minui.txt version wins
        assert_eq!(version.version, "2025.01.01");
        assert_eq!(version.source, "minui.txt");
    }

    #[test]
    fn test_detect_installed_version_no_prefix_filter() {
        let temp = tempfile::tempdir().unwrap();
        let sd_root = temp.path();

        let mut f = fs::File::create(sd_root.join("minui.txt")).unwrap();
        f.write_all(b"MinUI-Zero 20250525").unwrap();
        drop(f);

        // No expected_prefix → accepts any fork's minui.txt
        let result = detect_installed_version(sd_root.to_str().unwrap(), None);
        assert!(result.is_some());
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
