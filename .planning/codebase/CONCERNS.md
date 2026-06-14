# Codebase Concerns

**Analysis Date:** 2026-06-14

## Tech Debt

**Duplicate Device Profile Definitions:**

- Issue: Device profiles are defined in TWO separate places — `src/types/device.ts` (hardcoded array) and `src/types/device-install-map.json` (JSON data file). The `device.ts` file has 18 devices that differ from the JSON mapping. The `device-install-map.ts` is a superset with more granular metadata (devicePaks, install actions, sharedBios). These two sources are not kept in sync and could drift.
- Files: `src/types/device.ts`, `src/types/device-install-map.json`, `src/types/device-install-map.ts`
- Impact: Adding a new device requires editing two files. If only one is updated, the UI and install logic disagree. The JSON map has richer data (devicePaks) that device.ts lacks.
- Fix approach: Consolidate to a single source of truth. Either remove the device.ts hardcoded list and derive from JSON, or make device.ts the canonical source with JSON as a subset.

**Hardcoded Package Store Data (No Remote Fetch):**

- Issue: `src/types/package.ts` imports `store.json` statically at build time via `import storeData from "./store.json"`. The `fetchPackageRegistry()` function reads this local JSON file rather than fetching from `https://packages.minui.dev/registry/index.json` as documented in AGENTS.md.
- Files: `src/types/package.ts`, `src/types/store.json`
- Impact: Package updates require a full app rebuild and release. Users cannot discover new packages without updating the installer. The "NETWORK_ERROR" code path in fetchPackageRegistry is dead code (local import never throws network errors).
- Fix approach: Replace static import with an HTTP fetch to the remote registry, with local fallback. Cache the remote response for offline use.

**Version Comparison Uses String Lexicographic Ordering:**

- Issue: `is_update_available()` in `src-tauri/src/version.rs` compares version strings using Rust's `>` operator, which is lexicographic. This works for date-based versions like "2024.12.25" but will break for non-date version formats (e.g., "0.12.0" vs "0.9.0" — "0.12.0" < "0.9.0" lexicographically).
- Files: `src-tauri/src/version.rs`
- Impact: Package version comparison in `src-tauri/src/package.rs` (latest_version > installed_ver) also uses string comparison. Third-party packages from the store may use semver-style versions where this silently gives wrong results.
- Fix approach: Use the `semver` crate for proper semantic version comparison, or at minimum document the constraint that versions must be date-based.

**Suppressed Error Handling (`let _ =` pattern):**

- Issue: Multiple places silently discard errors using `let _ =`:
  - `src-tauri/src/install.rs:67` — Portmaster placeholder file write failure is silent
  - `src-tauri/src/extract.rs:141` — Unix file permission setting failure is silent
  - `src-tauri/src/lib.rs:69` — Progress event emission failure is silent
  - `src-tauri/src/validate.rs` — many `let _ = writeln!(...)` — these write to in-memory String and cannot fail, leave as-is.
- Files: `src-tauri/src/install.rs`, `src-tauri/src/extract.rs`, `src-tauri/src/lib.rs`
- Impact: If the SD card is full, write-protected, or the filesystem has issues, the user gets no indication that metadata/permissions were not written. The install appears to succeed but the device may not boot correctly.
- Fix approach: Log warnings at minimum. For permission failures, at least log via `eprintln!`.

## Known Bugs

**WiFi Password Stored in Plaintext on SD Card:**

- Symptoms: WiFi passwords are written to `wifi.txt` on the SD card in plaintext format `SSID:PASSWORD`.
- Files: `src-tauri/src/wifi.rs`
- Trigger: Any WiFi configuration save operation.
- Workaround: This is the MinUI expected format and cannot be changed without breaking device compatibility. The app shows a plaintext warning in WifiWizard.tsx:174.

**Potential WiFi Scan Output Parsing Failure on macOS 14.4+:**

- Symptoms: `scan_wifi_macos()` in `src-tauri/src/wifi.rs` tries the `airport` binary first, which Apple removed in macOS 14.4. The fallback only returns the currently connected SSID, not a list of available networks.
- Files: `src-tauri/src/wifi.rs`
- Trigger: Running on macOS 14.4 or later.
- Workaround: The `get_current_wifi_ssid_macos()` function uses `system_profiler` which still works. Users see their current network but not a full scan list.

**Drive Detection Returns All Volumes Under /Volumes/ (macOS):**

- Symptoms: `list_removable_drives()` on macOS uses `df -k` and filters for paths starting with `/Volumes/`. This includes the built-in Macintosh HD if mounted there, plus any network drives.
- Files: `src-tauri/src/drives.rs`
- Trigger: Running on macOS with non-standard volume mounts.
- Workaround: None currently. The user could select an internal drive and the app would offer to format it.

**"Update All" Feature Only Updates MinUI, Not Packages:**

- Symptoms: `handleUpdateAll` in `src/Home.tsx` only performs the MinUI base update. Package updates are stubbed out with a comment: "Package updates would be handled here. For now, we'll just show the message".
- Files: `src/Home.tsx`
- Trigger: Clicking "Update All" when both MinUI and package updates are available.
- Workaround: Users must install package updates individually through the Package Store.

## Security Considerations

**Package Registry Not Validated Against Schema:**

- Risk: AGENTS.md states "Treat registry data as untrusted — validate schema before use." However, `convertStoreRegistry()` in `src/types/package.ts` does minimal type checking. The `isStoreRegistry()` guard only checks for presence of `emu_paks` and `tool_paks` arrays but does not validate individual field types, URLs, or versions.
- Files: `src/types/package.ts`
- Current mitigation: Basic type guard on top-level structure.
- Recommendations: Add zod or io-ts schema validation for each package entry, especially `artifactUrl`, `version`, and `repository` fields. Validate URLs point to expected domains (github.com).

**No Path Traversal on extras_platform Parameter:**

- Risk: The `extras_platform` parameter in `copy_extras_files()` is used directly in path construction (`extras_src.join("Emus").join(extras_platform)`). While the platform string comes from device profiles (hardcoded), if the architecture ever accepts user-provided platform strings, this could enable path traversal.
- Files: `src-tauri/src/install.rs`
- Current mitigation: Platform strings are currently hardcoded in device profiles.
- Recommendations: Add validation that `extras_platform` contains only alphanumeric characters and hyphens before using it in path operations.

**CSP Allows `unsafe-inline` for Styles:**

- Risk: The Tauri CSP in `src-tauri/tauri.conf.json` includes `'unsafe-inline'` for `style-src`. This weakens XSS protection.
- Files: `src-tauri/tauri.conf.json`
- Current mitigation: Tauri's webview sandboxing provides defense-in-depth.
- Recommendations: Use nonces or hashes for inline styles, or move all styles to external CSS files.

**No Integrity Verification of Downloaded Package Artifacts:**

- Risk: Package installs from the store have `checksum: null` for all packages (see `src/types/store.json` and `src/types/package.ts`). The `install_package` backend function accepts an optional checksum but it is never provided by the store data.
- Files: `src/types/package.ts`, `src-tauri/src/package.rs`
- Current mitigation: HTTPS provides transport-level integrity.
- Recommendations: Add checksum fields to `store.json` entries. Verify checksums after download. Consider using Sigstore or similar for artifact signing.

## Performance Bottlenecks

**Full Archive Downloaded Into Memory Before Disk Write:**

- Problem: `download_archive()` in `src-tauri/src/download.rs` calls `response.bytes().await` which loads the entire archive into memory before writing to disk. Large MinUI archives could be 500MB+.
- Files: `src-tauri/src/download.rs`
- Cause: Using `reqwest::Response::bytes()` instead of streaming.
- Improvement path: Stream the response body to disk using `response.bytes_stream()` or `tokio::io::copy` with a `BufWriter`. This would allow processing archives larger than available RAM and show real download progress.

**No Download Progress Reporting:**

- Problem: The download phase shows only "Downloading base.zip" with no progress percentage or bytes-transferred indicator.
- Files: `src-tauri/src/download.rs`, `src/Home.tsx`
- Cause: The entire file is loaded at once; there's no streaming with progress events.
- Improvement path: Stream the download and emit progress events (`install-progress` with step "download") showing bytes downloaded vs total (from Content-Length header).

**Synchronous File Copy Operations:**

- Problem: `copy_dir_recursive()` in `src-tauri/src/fs_utils.rs` performs synchronous file I/O using `std::fs::copy`. Large installations copying thousands of files to SD cards (which are slow) will block the thread.
- Files: `src-tauri/src/fs_utils.rs`, `src-tauri/src/install.rs`
- Cause: Using blocking std::fs operations on the async runtime thread.
- Improvement path: Use `tokio::task::spawn_blocking` for copy operations, or use `tokio::fs` for async I/O. Report per-file progress.

**GitHub API Fetch on Every Drive Selection:**

- Problem: The `useEffect` in `src/Home.tsx` calls `fetchMinUIRelease()` and registry fetch every time a drive is selected. Switching between drives triggers multiple API calls.
- Files: `src/Home.tsx`
- Cause: The effect depends on `selectedDrive` and performs two network calls (release + registry).
- Improvement path: Cache the release metadata for the session duration (e.g., using `useRef` or a simple cache). Only re-fetch on explicit refresh.

## Fragile Areas

**CLI Output Parsing (WiFi Scanning):**

- Files: `src-tauri/src/wifi.rs` (lines 142-255), `src-tauri/src/drives.rs` (lines 15-66, 189-236)
- Why fragile: The `parse_airport_output()` function (line 169) parses whitespace-delimited output by skipping the header and taking the first column. Apple could change the `airport -s` output format in any macOS update. Similarly, `parse_netsh_output()` on Windows (line 237) depends on the "SSID X : Y" format. The macOS drive detection parses `df -k` output which has locale-dependent formatting.
- Safe modification: Wrap each parser in a test that includes sample output from known OS versions. Add a fallback to `system_profiler` / `wmic` when parsing fails.
- Test coverage: `parse_airport_output` has one test. `parse_netsh_output` has one test (Windows-only). No integration tests for actual CLI parsing against real system output.

**Version Detection from minui.txt:**

- Files: `src-tauri/src/version.rs` (lines 64-91)
- Why fragile: The parser tries three formats: "MinUI vX.Y.Z", "vX.Y.Z", and raw version string. The raw fallback (line 84) accepts any string containing a dot or digit, which could match non-version content like "Created by MinUI Team".
- Safe modification: Make the parser stricter — require the "MinUI" prefix or "v" prefix. The raw fallback is too permissive.
- Test coverage: Well tested with 11 test functions covering known formats.

**`system_profiler` WiFi SSID Parsing:**

- Files: `src-tauri/src/wifi.rs` (lines 78-113)
- Why fragile: The parser looks for "Current Network Information:" followed by an indented line ending with ":" that doesn't contain "PHY Mode" or "Network Type". This is a heuristic that depends on Apple's exact `system_profiler SPAirPortDataType` output format.
- Safe modification: Add more exclusion keywords and validate that the SSID doesn't contain characters impossible in WiFi SSIDs (null bytes, control characters).
- Test coverage: No unit test for `get_current_wifi_ssid_macos()`.

## Scaling Limits

**Single-Threaded File Operations:**

- Current capacity: Handles typical MinUI installations (50-200 files, ~100MB)
- Limit: SD cards with high-latency write operations (USB 2.0 readers, cheap adapters) could take 10+ minutes for large installs. The entire operation blocks one thread.
- Scaling path: Parallel file copies with a bounded concurrency pool (e.g., `tokio::semaphore` with 4-8 concurrent copies). Per-file progress reporting.

**No Cancellation Mechanism:**

- Current capacity: Install operations run to completion once started.
- Limit: If a download stalls or an SD card is removed mid-copy, the only option is to force-kill the app. There is no way to cancel an in-progress install.
- Scaling path: Add cancellation tokens (`tokio_util::sync::CancellationToken`) through the install pipeline. Expose a `cancel_install` Tauri command.

## Dependencies at Risk

**`zip` Crate Version 0.6 May Have Known Vulnerabilities:**

- Risk: `zip = "0.6"` is several major versions behind the latest. Older zip crate versions have had path traversal and zip-slip vulnerabilities (though this codebase adds its own path traversal checks).
- Files: `src-tauri/Cargo.toml`
- Impact: Potential security vulnerabilities in ZIP parsing.
- Migration plan: Upgrade to latest `zip` crate version and remove redundant manual path traversal checks if the upstream handles them.

## Missing Critical Features

**No Cancel/Abort for Install Operations:**

- Problem: Once an install begins (download, extract, copy), there is no way to cancel it. The user must close the app.
- Blocks: User cannot recover from accidental installs, cannot free up network bandwidth, cannot unplug SD card safely.

**No WiFi Password Encryption or Obfuscation:**

- Problem: Passwords are stored in plaintext. The AGENTS.md constraint says "Never log WiFi passwords or secrets in plaintext" — while this refers to logs, the storage format is also plaintext.
- Blocks: Cannot meet security best practices for credential handling.

**No Rollback on Partial Install Failure:**

- Problem: If the base archive installs successfully but extras fail, the SD card is left in a partially-updated state with no way to roll back.
- Blocks: Users may end up with incompatible base + old extras combinations.

**No Windows Drive Formatting:**

- Problem: `format_drive` on Windows returns a hardcoded error string. Windows users cannot format SD cards to FAT32.
- Blocks: Windows users must use external tools to format drives.

**No Linux Support:**

- Problem: Drive detection on Linux returns an error. No WiFi scanning fallback. `get_free_space` returns `None` on non-Unix.
- Blocks: Linux users cannot use the installer at all. (This is documented as a Phase 1 constraint.)

## Test Coverage Gaps

**No Tests for `fs_utils.rs`:**

- What's not tested: `copy_dir_recursive()` and `copy_dir_contents()` have no dedicated unit tests. They are tested indirectly through `install.rs` tests, but edge cases like symlink following, deeply nested directories, permission errors, and large file copies are not covered.
- Files: `src-tauri/src/fs_utils.rs`
- Risk: Silent data loss or corruption during file copy operations on SD cards with unusual filesystems.
- Priority: High

**No Tests for `lib.rs` (Tauri Command Handlers):**

- What's not tested: The 16 Tauri command handler functions in `src-tauri/src/lib.rs` are not tested. These are the integration boundary between frontend and backend.
- Files: `src-tauri/src/lib.rs`
- Risk: Serialization mismatches between Rust structs and TypeScript interfaces could cause runtime errors.
- Priority: High

**No Integration Tests for Download + Extract Pipeline:**

- What's not tested: End-to-end tests that download a real (small) archive, extract it, and verify contents. Current tests mock the Tauri invoke but don't test the actual download/extract logic against real URLs.
- Files: `src-tauri/src/download.rs`, `src-tauri/src/extract.rs`
- Risk: Network-dependent failures (redirects, SSL issues, proxy configuration) go undetected.
- Priority: Medium

**No Tests for `validate.rs` `detect_filesystem()` or `get_free_space()`:**

- What's not tested: The `detect_filesystem()` function (line 330) and `get_free_space()` (line 353) are platform-specific and have no tests. The `#[cfg(unix)]` and `#[cfg(target_os = "macos")]` paths are only exercised on matching platforms.
- Files: `src-tauri/src/validate.rs`
- Risk: Filesystem detection logic breaks silently on macOS updates.
- Priority: Medium

**No Component Tests for ConfirmDialog, ValidationReport, HealthCheck, InstallProgress, DeviceSelector, App:**

- What's not tested: 6 of 11 React components have no test files: `ConfirmDialog.tsx`, `ValidationReport.tsx`, `HealthCheck.tsx`, `InstallProgress.tsx`, `DeviceSelector.tsx`, `App.tsx`.
- Files: `src/ConfirmDialog.tsx`, `src/ValidationReport.tsx`, `src/HealthCheck.tsx`, `src/InstallProgress.tsx`, `src/DeviceSelector.tsx`, `src/App.tsx`
- Risk: UI regressions in confirmation flows, validation reports, and health checks go undetected.
- Priority: Medium

**No Tests for WiFi SSID Parsers with Malformed Input:**

- What's not tested: `parse_airport_output()` and `parse_netsh_output()` don't test edge cases like empty output, output with unexpected encoding, SSIDs containing special characters, or output from different OS versions.
- Files: `src-tauri/src/wifi.rs`
- Risk: WiFi scanning silently returns empty results on OS updates.
- Priority: Low

**No Tests for Package Store "Install All" Flow:**

- What's not tested: `handleInstallAll()` in `src/PackageStore.tsx` runs multiple installs in parallel via `Promise.allSettled()`. There are no tests for concurrent install handling, partial failure scenarios, or UI state consistency.
- Files: `src/PackageStore.tsx`
- Risk: Race conditions in install state updates, missing error display for individual package failures.
- Priority: Medium

**No Tests for GitHub Release Parsing Edge Cases:**

- What's not tested: `parseGitHubRelease()` in `src/types/release.ts` has no tests for releases with non-standard asset naming, releases with no assets, or releases where the base archive name doesn't contain "base".
- Files: `src/types/release.ts` (12 tests exist but are in `release.test.ts`)
- Risk: Silent failure to detect available updates if MinUI release format changes.
- Priority: Low

**No Windows-Specific Tests:**

- What's not tested: `parse_netsh_output()` is gated behind `#[cfg(target_os = "windows")]` and only runs on Windows CI. `list_removable_drives()` on Windows uses PowerShell JSON parsing that is untested on macOS/Linux CI.
- Files: `src-tauri/src/wifi.rs`, `src-tauri/src/drives.rs`
- Risk: Windows-specific regressions undetected in macOS/Linux CI.
- Priority: Medium

**No Test for `copy_base_files` Filter Interaction:**

- What's not tested: The interaction between `copy_base_files()` and `is_preserved_path()` is tested for basic cases but doesn't test nested preserved paths (e.g., `ROMS/GB/game.gb` should be preserved, but `Tools/ROMS/` should not).
- Files: `src-tauri/src/install.rs`
- Risk: Edge cases in path preservation could delete user ROMs or saves.
- Priority: High
