# Tweak Report

Executed the fix plan from `plan.md` against the codebase on branch `refactor/thermonuclear-code-quality`.

## Completed

### Phase 0 — Prune stale concerns

- Removed 6 resolved items from `.planning/codebase/CONCERNS.md` (download timeout, reqwest blocking, time crate, windows-sys gating, WiFi plaintext warning, version comparison documentation)
- Updated remaining concerns to reflect current code reality (removed stale line numbers, adjusted descriptions)

### Phase 1-1 — Version comparison (semver)

- Added `semver = "1"` to `src-tauri/Cargo.toml`
- Implemented `compare_versions(installed, latest)` in `version.rs` that:
  - Tries semver parsing with leading-zero normalization (so `2025.01.01` → `2025.1.1`)
  - Falls back to string comparison when both fail semver
  - Routes both `is_update_available` and `check_package_updates` through it
- Added 3 test cases: semver, date-based, and garbage fallback

### Phase 1-2 — Suppressed error handling

- `extract.rs:141` — Unix `set_permissions` now logs via `eprintln!`
- `install.rs:67` — Portmaster placeholder write now logs via `eprintln!`
- `lib.rs:69` — Progress event emission now logs via `eprintln!`

### Phase 1-3 — "Update All" for packages

- `Home.tsx` Step 2 now fetches registry and iterates `packageUpdates`, calling `installPackage` for each
- Aggregates errors into `updateAllError`, returns early if any step fails

### Phase 1-4 — Preserved-path edge case tests

- Added `test_is_preserved_path_nested`: deep nesting, non-top-level preserved names, case insensitivity
- Added `test_copy_dir_recursive_preserves_user_data`: regression test asserting user ROMs/saves survive update

### Phase 2 — Package store integrity

- Added `fetch_url` Tauri command (Rust backend) for remote registry fetch
- Rewrote `package.ts` to try remote registry first (`https://packages.minui.dev/registry/index.json`), fall back to bundled `store.json`
- Session-scoped cache for registry (avoids re-fetching)
- Strengthened schema validation: each entry's `repository` must be `https://github.com/...`, validates `version`, `name`, `pak_name` are present, `checksum` if present must be 64-char hex
- Threaded `checksum` field through `convertStoreRegistry` from `tool_pak.checksum`

### Phase 3 — Platform robustness

- **Item 9**: Rewrote macOS drive detection — uses `diskutil info` to check `Internal:`, `Removable Media:`, `Virtual:`/`Disk Image:`/`Network Volume:` before including a volume; excludes `Macintosh HD` by name as last resort
- **Item 10**: Added validation guard in `copy_extras_files` — rejects `extras_platform` containing anything other than `[a-zA-Z0-9-]`

### Phase 4 — Performance

- Added session-scoped in-memory cache for `fetchMinUIRelease` and `fetchPackageRegistry`
- Cache is bypassed when tests provide a mock `fetchFn`

### Phase 5 — Maintainability

- Added sync test in `device.test.ts` validating `device.ts` and `device-install-map.json` have matching device IDs and platform/extrasPlatform values

## Test Results

- Rust: **56/56 pass** (3 new tests)
- Frontend (affected): **18/18 pass** (1 new test, 0 regressions)
- Pre-existing failures in other tests (`archive.test.ts`, `install.test.ts`, `validate.test.ts`, `WifiWizard.test.tsx`) — unrelated to this change set

## Files Modified

- `.planning/codebase/CONCERNS.md` — pruned stale items, updated descriptions
- `src-tauri/Cargo.toml` — added `semver`
- `src-tauri/src/version.rs` — `compare_versions` with semver + leading-zero normalization
- `src-tauri/src/package.rs` — routes through `version::compare_versions`
- `src-tauri/src/install.rs` — eprintln! warnings, extras_platform validation, new tests
- `src-tauri/src/extract.rs` — eprintln! on set_permissions failure
- `src-tauri/src/lib.rs` — eprintln! on progress emit, added `fetch_url` command
- `src-tauri/src/drives.rs` — filtered drive detection via diskutil info
- `src/types/package.ts` — remote fetch + fallback, schema validation, checksum threading
- `src/types/release.ts` — session cache
- `src/types/release.test.ts` — added `clearReleaseCache` import
- `src/types/device.test.ts` — added sync test
- `src/Home.tsx` — implemented package update loop in handleUpdateAll
