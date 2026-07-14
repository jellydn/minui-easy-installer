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
    // so we cannot verify the fork directly. Accept it only when no
    // specific fork is expected, or when the canonical "MinUI" is
    // expected — legacy official installs only ever wrote this file
    // for the canonical MinUI. Custom forks (e.g. MinUI-Zero) always
    // write minui.txt via the installer, so non-canonical forks skip
    // this file because we cannot prove it belongs to a fork we
    // don't recognise.
    if expected_prefix.is_none() || expected_prefix == Some("MinUI") {
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

    // Single run of 6+ characters: date-only format (e.g. MinUI-Zero
    // "20250525") or tagged builds like "20240212b-1". Hyphens are only
    // accepted when the string also contains a letter, so plain
    // dash-separated dates such as "2024-12-25" are still rejected.
    if s.len() >= 6
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        && (s.chars().any(|c| c.is_ascii_alphabetic()) || !s.contains('-'))
    {
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
mod tests;
