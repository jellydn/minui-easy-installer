# Plan 005 — Harden `parse_minui_version` raw fallback

| Field        | Value                                  |
| ------------ | -------------------------------------- |
| Slug         | `harden-version-parser`                |
| Status       | pending                                |
| Priority     | Medium                                 |
| Category     | correctness                            |
| Impact       | Medium                                 |
| Effort       | S                                      |
| Risk         | Low                                    |
| Audit commit | `4d6e95a`                              |
| Depends on   | none                                   |
| Blocks       | none                                   |

## Problem

`.planning/codebase/CONCERNS.md` → "Fragile Areas" → "Version
Detection from minui.txt" flags the raw fallback as too permissive:

> The parser tries three formats: "MinUI vX.Y.Z", "vX.Y.Z", and raw
> version string. The raw fallback (line 84) accepts any string
> containing a dot or digit, which could match non-version content
> like "Created by MinUI Team".

`src-tauri/src/version.rs:78-90`:

```rust
// If line looks like a version (contains dots or numbers), use it directly
if !first_line.is_empty()
    && (first_line.contains('.') || first_line.chars().any(|c| c.is_ascii_digit()))
{
    return Some(first_line.to_string());
}

None
```

Concrete failure modes:
1. `minui.txt` contains `"Created by MinUI Team 2024"` — the raw
   fallback returns `"Created by MinUI Team 2024"`, which `try_parse_semver`
   can't parse, so `compare_versions` falls back to string comparison.
   This gives `update_available = "v2024.12.25" > "Created by MinUI Team 2024"`
   which is lexicographically false-ish but a maintenance trap.
2. `minui.txt` contains `"MinUI v2024.12.25\nReleased 2024-12-25\n"` —
   the raw fallback returns the first line, but the **comment line**
   "Released 2024-12-25" (line 2) is ignored; this one is actually
   fine, but if a future release ships `minui.txt` with the version
   on line 2, the raw fallback returns the line 1 comment.

## Goal

Restrict the raw fallback to a narrow shape: a string that matches
the existing semver-or-date-version grammar (digits separated by
dots, optional leading `v`). Anything else returns `None`, and the
caller treats the card as "version unknown" — which is the safe
default (the `check_for_updates` logic already handles "unknown
installed" as "update available").

## Files in scope

- `src-tauri/src/version.rs` — tighten `parse_minui_version`; add
  a small helper `looks_like_version` that the raw fallback uses.
- `src-tauri/src/version.rs::tests` — add tests for the new behavior.

## Files explicitly out of scope

- `compare_versions` / `try_parse_semver` — these are correct; the
  bug is upstream in `parse_minui_version`'s acceptance criterion.

## Current state (`src-tauri/src/version.rs:67-90`)

```rust
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
```

## Step-by-step execution

### Step 1 — Add `looks_like_version`

In `src-tauri/src/version.rs`, add a private helper:

```rust
/// Returns true if `s` looks like a release version string.
///
/// Accepts:
/// - 3 dot-separated numeric segments: "2024.12.25", "0.12.0", "1.2.3"
/// - 2 dot-separated numeric segments: "2024.12", "1.0"
/// - Optional leading "v" prefix (case-insensitive)
///
/// Rejects free-form text like "Created by MinUI Team 2024" or
/// "v2024-12-25" (dashes are not part of the grammar).
fn looks_like_version(s: &str) -> bool {
    let s = s.trim().trim_start_matches('v').trim_start_matches('V').trim();
    if s.is_empty() {
        return false;
    }
    let segments: Vec<&str> = s.split('.').collect();
    if segments.len() < 2 || segments.len() > 3 {
        return false;
    }
    segments
        .iter()
        .all(|seg| !seg.is_empty() && seg.chars().all(|c| c.is_ascii_digit()))
}
```

### Step 2 — Use it in the raw fallback

Replace the existing raw fallback with:

```rust
// Raw fallback: only if the first line is a strict version-shaped
// string. We deliberately do NOT accept free-form text here — see
// CONCERNS.md "Fragile Areas" → "Version Detection from minui.txt".
if looks_like_version(first_line) {
    return Some(first_line.to_string());
}

None
```

### Step 3 — Tests

In `src-tauri/src/version.rs::tests`, add:

```rust
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
fn test_looks_like_version_rejects_free_form_text() {
    assert!(!looks_like_version("Created by MinUI Team 2024"));
    assert!(!looks_like_version("MinUI"));
    assert!(!looks_like_version(""));
    assert!(!looks_like_version("v2024-12-25")); // dashes, not dots
    assert!(!looks_like_version("2024"));
    assert!(!looks_like_version("2024.12.25.1")); // too many segments
}

#[test]
fn test_parse_minui_version_rejects_free_form_text() {
    // Regression for the "Created by MinUI Team 2024" failure mode.
    assert_eq!(
        parse_minui_version("Created by MinUI Team 2024\n"),
        None
    );
    assert_eq!(
        parse_minui_version("Released 2024-12-25 by the team\n"),
        None
    );
}

#[test]
fn test_parse_minui_version_accepts_strict_raw_version() {
    // The raw fallback should still accept a clean version-only line.
    assert_eq!(
        parse_minui_version("2024.12.25\n"),
        Some("2024.12.25".to_string())
    );
    assert_eq!(
        parse_minui_version("v2024.12.25\n"),
        Some("2024.12.25".to_string())
    );
}
```

### Step 4 — Run

```bash
cd src-tauri && cargo test --lib version 2>&1 | tail -20
```

Expected: 11 existing + 6 new = 17 tests, all pass.

### Step 5 — Full suite

```bash
cd src-tauri && cargo test --lib 2>&1 | tail -5
```

Expected: no regressions.

## Done criteria (machine-checkable)

- `cd src-tauri && cargo test --lib version` passes 17 tests
  (11 existing + 6 new).
- `cd src-tauri && cargo test --lib` shows no regressions.
- Manual: with `minui.txt` containing `"Created by MinUI Team 2024"`,
  `detect_installed_version` returns `None` (was: returned the whole
  string). With `minui.txt` containing `"2024.12.25"`, it still
  returns `Some("2024.12.25")`.

## Test plan

The 6 new tests above cover:
- 3-segment versions (date-style and semver-style)
- 2-segment versions
- `v` prefix (both cases)
- Free-form text rejection (the actual bug)
- Empty string
- Dashes (different format)
- Too many segments
- End-to-end `parse_minui_version` regression

## Maintenance note

If a future MinUI release changes the `minui.txt` format (e.g. to
include a build metadata segment like `2024.12.25+sha.abc1234`),
this parser will reject it. That's *intentional* — better to fail
closed than to silently mis-detect. The fix is to update
`looks_like_version` to accept the new shape, add a test, and
bump the `compare_versions` parser to handle it.

## Escape hatches

- **If the executor finds real-world `minui.txt` files that the
  current parser accepts but the new `looks_like_version` rejects:**
  file an issue with the sample content, do NOT loosen the check
  silently. The user-facing impact is "version unknown" which is
  the safe default.
- **If a future `looks_like_version` change requires more than 3
  segments** (e.g. semver build metadata): change the `.len() < 2 ||
  .len() > 3` check, but ONLY after a real-world file is found that
  needs it. Do not preemptively add flexibility.
