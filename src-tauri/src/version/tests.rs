use super::*;
use std::io::Write;

#[test]
fn test_parse_minui_version_with_prefix() {
    let content = "MinUI v2024.12.25\nSome other content";
    assert_eq!(
        parse_minui_version_with_prefix(content).map(|(_, v)| v),
        Some("2024.12.25".to_string())
    );
}

#[test]
fn test_parse_minui_version_fork_prefix_minui_zero() {
    let content = "MinUI-Zero 20250525\n";
    assert_eq!(
        parse_minui_version_with_prefix(content).map(|(_, v)| v),
        Some("20250525".to_string())
    );
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
    assert_eq!(
        parse_minui_version_with_prefix(content).map(|(_, v)| v),
        Some("20250525".to_string())
    );
}

#[test]
fn test_parse_minui_version_arbitrary_prefix_two_segment() {
    let content = "MyCustomFork 2025.01\n";
    assert_eq!(
        parse_minui_version_with_prefix(content).map(|(_, v)| v),
        Some("2025.01".to_string())
    );
}

#[test]
fn test_parse_minui_version_prefix_rejects_non_version_after_space() {
    // "MyFork not.a.version" — rsplit_once gives "not.a.version" after space
    // but looks_like_version rejects it because "not" and "a" are non-numeric
    let content = "MyFork not.a.version\n";
    assert_eq!(
        parse_minui_version_with_prefix(content).map(|(_, v)| v),
        None
    );
}

#[test]
fn test_parse_minui_version_without_prefix() {
    let content = "2024.12.25\nSome other content";
    assert_eq!(
        parse_minui_version_with_prefix(content).map(|(_, v)| v),
        Some("2024.12.25".to_string())
    );
}

#[test]
fn test_parse_minui_version_with_v_prefix() {
    let content = "v2024.12.25";
    assert_eq!(
        parse_minui_version_with_prefix(content).map(|(_, v)| v),
        Some("2024.12.25".to_string())
    );
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
    assert_eq!(
        parse_minui_version_with_prefix("Created by MinUI Team 2024\n").map(|(_, v)| v),
        None
    );
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
