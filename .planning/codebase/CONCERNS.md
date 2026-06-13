# Codebase Concerns

**Analysis Date:** 2026-06-13

> Scope: MinUI Easy Installer & Package Store — Tauri v2 desktop app (Rust backend in
> `src-tauri/src/`, React/TS frontend in `src/`). Findings below were verified against the
> current source on 2026-06-13 by reading files and running `cargo test`, `cargo clippy`,
> and `npx vitest run`.
>
> **NOTE on stale hand-off findings:** Several "confirmed" findings handed to this audit
> were **already fixed** or **outdated** in the current tree. They are documented honestly
> under "Corrected / Already-Resolved Findings" at the bottom so they are not re-filed as bugs.

---

## Tech Debt

**Leaked temp directories (`std::mem::forget`) — resource leak:**

- Issue: Both the download and the no-destination extract paths deliberately leak their
  `TempDir` via `std::mem::forget` so the files survive after the function returns. They are
  never cleaned up for the lifetime of the process (and the OS temp dir keeps growing across
  repeated install attempts).
- Files: `src-tauri/src/download.rs:84-87`, `src-tauri/src/extract.rs:62-67`
- Impact: Each install/extract leaves a full copy of the MinUI archive + extracted tree in
  the system temp dir. Multiple installs in one session accumulate hundreds of MB. The code
  comment itself says "In a real implementation, we'd manage this more carefully."
- Fix approach: Return the owned `TempDir` handle up the call stack (or copy out then drop),
  or explicitly remove the temp dir after the SD-card copy completes.

**Duplicated free-space (`statvfs`) implementations:**

- Issue: Two near-identical `unsafe libc::statvfs` helpers exist that compute available bytes.
- Files: `src-tauri/src/validate.rs:98` (`check_free_space`) and `src-tauri/src/validate.rs:365`
  (`get_free_space`)
- Impact: Divergent maintenance; a fix/bug in one (e.g. overflow handling of
  `f_bavail * f_frsize`) won't reach the other.
- Fix approach: Collapse into a single helper used by both `validate_installation` and
  `check_sd_card_health`.

**Coarse, simulated install progress (no real backend progress events):**

- Issue: `install_minui` runs download → extract → copy-base → download-extras →
  extract-extras → copy-extras as a single blocking async call returning one `InstallResult`.
  No Tauri events are emitted mid-flight. The frontend manually sets phase labels
  (`downloading`, then `extracting`) around the one blocking `invoke`, so the displayed phase
  does not reflect the backend's actual step.
- Files: `src-tauri/src/install.rs:164-245`, `src/Home.tsx:130-165` (see the explicit comment
  at `src/Home.tsx:148` "coarse phase — backend is one blocking call")
- Impact: For a multi-hundred-MB download the UI shows a static phase with no byte/percent
  progress; users cannot tell a slow download from a hang. US-007's intent of real download
  progress is not met.
- Fix approach: Stream the HTTP body and emit `tauri::Window::emit` progress events per chunk
  /step; have the frontend `listen` for them instead of setting phases by hand.

**`csp: null` placeholder in Tauri config:**

- Issue: Content-Security-Policy is explicitly disabled.
- Files: `src-tauri/tauri.conf.json` (`app.security.csp: null`)
- Impact: No CSP hardening for a desktop webview that renders untrusted registry-sourced
  strings (package names/descriptions/author). See Security section.
- Fix approach: Set a restrictive CSP before shipping.

**Silent partial-install on Extras failure:**

- Issue: If the Extras archive fails to download or extract, the failure is swallowed and the
  install still returns `success: true` with `extras_files_copied: 0`. The user is told the
  install succeeded.
- Files: `src-tauri/src/install.rs:216-237` (comments: "Extras failure is non-fatal", "Extras
  download failure is non-fatal")
- Impact: Users believe Extras were installed when they were not; no warning surfaced.
- Fix approach: Carry a `extras_warning: Option<String>` in `InstallResult` and surface it.

---

## Known Bugs

**`cargo clippy --all-targets` fails to compile (hard error in test code):**

- Symptoms: `cargo clippy --all-targets` aborts with
  `error: this comparison involving the minimum or maximum element ... always true`
  (`clippy::absurd_extreme_comparisons`, which is `deny`-by-default) plus 6 `needless_borrow`
  warnings. `cargo test`/`cargo build` still pass because the lint only fires under clippy.
- Files: `src-tauri/src/wifi.rs:249` (`assert!(networks.len() >= 0)` — a `usize` is always ≥ 0);
  `src-tauri/src/install.rs:257,261,265,269,273,277` (`&Path::new(...)` needless borrows in
  tests)
- Trigger: `cd src-tauri && cargo clippy --all-targets`
- Workaround: `cargo clippy --lib` is clean (0 warnings). Fix the assert to a meaningful check
  (e.g. assert the function returns without panicking) and drop the redundant `&`.

**Failing/uncompilable Rust doctests:**

- Symptoms: `cargo test` (which includes `--doc`) reports 3 doctest failures: the
  documentation code-fences are plain text, not valid Rust, so rustc tries to compile them and
  errors (`expected one of ! or ::, found ...`).
- Files: `src-tauri/src/version.rs:57-62` (`MinUI v2024.12.25` / `2024.12.25` in a ` ``` `
  block), `src-tauri/src/wifi.rs:8-11` (`SSID: ...` / `PASS: ...` example block)
- Trigger: `cd src-tauri && cargo test` → "test result: FAILED. 0 passed; 3 failed" in the
  Doc-tests run (the 52 lib unit tests all pass).
- Workaround: Mark the fences as `text`/`ignore`/`no_run` (e.g. ` ```text `) so they are
  treated as documentation, not executable doctests.

---

## Security Considerations

**Untrusted registry can point installs at an arbitrary URL (no scheme/host allowlist):**

- Risk: `artifactUrl` is validated only as a non-empty string. There is no check that it is
  `https`, nor that the host is `packages.minui.dev` / a GitHub releases host. A compromised
  registry, DNS/MITM, or a malicious mirror can serve arbitrary content that is downloaded and
  written to the user's SD card (which is then booted by the handheld).
- Files: `src/types/package.ts:56-103` (`validatePackageEntry` — `artifactUrl` only string-checked),
  `src/types/package.ts:422` (`fetchPackageRegistry`), `src-tauri/src/download.rs:37-95`,
  `src-tauri/src/package.rs:224-280` (`install_package`)
- Current mitigation: Registry JSON is schema-validated (`validatePackageRegistry`); archive
  entry paths are checked for traversal in `extract.rs`.
- Recommendations: Enforce `https://` and a host allowlist for `artifactUrl` and `REGISTRY_URL`;
  prefer pinned hosts; consider signature verification of the registry document itself.

**Checksum verification is optional — unverified artifacts are installed when metadata omits it:**

- Risk: In `download_archive`, integrity is only checked `if let Some(expected) = expected_checksum`.
  Registry schema treats `checksum` as optional (`pkg.checksum !== null` → only type-checked).
  A package/release entry with no checksum is downloaded and installed with **zero** integrity
  verification. Combined with the URL gap above, this is the main supply-chain risk for a tool
  whose output is booted firmware/paks on a device.
- Files: `src-tauri/src/download.rs:66-82`, `src/types/package.ts:121-130` (checksum optional)
- Current mitigation: When a checksum _is_ present it is SHA-256 verified (case-insensitive),
  and a mismatch aborts the install (`download.rs:70-77`).
- Recommendations: Treat missing checksum as a hard error for store packages (or at minimum a
  prominent unverified-install warning the user must accept).

**Path traversal via untrusted `targetDir` (install destination not sanitized):**

- Risk: `resolve_package_install_path` only strips a leading `/` from the registry-supplied
  `target_dir` (`trim_start_matches('/')`). It does not reject `..` components, so a malicious
  registry entry with `targetDir: "../../something"` resolves _outside_ the SD-card root. The
  extract step protects archive _entry_ paths, but the destination base itself is attacker-influenced.
- Files: `src-tauri/src/package.rs:36-47` (`resolve_package_install_path`),
  `src-tauri/src/package.rs:272-273`, `src/types/package.ts:149-160` (`targetDir` only
  non-empty-string checked)
- Current mitigation: None for `..` on the target dir.
- Recommendations: After joining, canonicalize and assert the result `starts_with` the SD-card
  root (same pattern already used in `extract.rs:110`); reject `..` in `targetDir` at validation.

**SD-card writes / no formatting (per project constraints) — verified posture:**

- Risk: App writes directly to removable media.
- Files: `src/ConfirmDialog.tsx`, `src/Home.tsx` (confirm gating), `src-tauri/src/install.rs`,
  `src-tauri/src/package.rs`, `src-tauri/src/wifi.rs`
- Current mitigation (GOOD): MinUI base install preserves user data — `PRESERVED_FOLDERS`
  (ROMS/Saves/BIOS/CHEATS, case-insensitive) are skipped during copy
  (`src-tauri/src/install.rs:16-33,48-51`). No drive-format code exists. Writes are gated behind
  a confirmation dialog in the UI.
- Recommendation: Note that **package** installs (`copy_package_files`) do **not** apply the
  preserved-folder skip; this is fine for Tools/Apps targets but compounds the `targetDir`
  traversal risk above.

**WiFi password handling — verified GOOD, with one caveat:**

- Risk: Credentials handling.
- Files: `src-tauri/src/wifi.rs:14-31`, `src/WifiWizard.tsx`
- Current mitigation (GOOD): The password is **never logged**. It is only written to
  `wifi.txt` on the SD card in the `SSID:`/`PASS:` format MinUI's Wifi.pak requires.
- Caveat: `wifi.txt` necessarily stores the password in plaintext on the card (inherent to the
  MinUI format, not a defect of this app) — worth a one-line user notice.

**`unsafe` usage is narrow and FFI-only:**

- Risk: Memory safety.
- Files: `src-tauri/src/validate.rs:103,117,372,374` (`libc::statvfs` /
  `GetDiskFreeSpaceExA` free-space queries)
- Current mitigation: `unsafe` blocks are limited to zeroing a struct and a single syscall each;
  return values are checked. No raw pointer arithmetic on user data.
- Recommendation: Acceptable; just de-duplicate (see Tech Debt) so the audited surface is one
  function.

---

## Performance Bottlenecks

**Whole-archive download buffered in memory:**

- Problem: The HTTP response is read fully into memory with `response.bytes().await` then
  written to disk in one `fs::write`. MinUI base archives plus Extras can be hundreds of MB.
- Files: `src-tauri/src/download.rs:59-64`
- Cause: No streaming; entire body held in RAM before any disk write.
- Improvement path: Stream the body to the temp file in chunks (also enables real progress
  events — see Tech Debt).

**Recursive synchronous SD-card copy on the command thread:**

- Problem: `copy_dir_recursive` / `copy_dir_contents` copy file-by-file synchronously; the whole
  install is one blocking call with no cancellation and no progress.
- Files: `src-tauri/src/install.rs:37-75`, `src-tauri/src/package.rs:73-` (`copy_dir_contents`)
- Cause: Simple synchronous design.
- Improvement path: Emit per-file/percent progress; consider a cancel token.

---

## Fragile Areas

**Drive detection by parsing CLI output (`diskutil`/`df`/PowerShell):**

- Files: `src-tauri/src/drives.rs:14-22` (`diskutil list external`),
  `src-tauri/src/drives.rs:115-116` (`df -k` fallback), `src-tauri/src/drives.rs:159-169`
  (`powershell` on Windows), `src-tauri/src/validate.rs:346` (`diskutil info`)
- Why fragile: Parsers depend on exact human-readable output formatting and English labels
  (e.g. `strip_prefix("File System: ")`, `strip_prefix("Volume Size: ")`,
  `"File System Personality:"`). `diskutil list external` does not emit those `File System:`/
  `Volume Size:` lines in that shape, and macOS output/labels change between releases — so
  size/filesystem can silently come back `None`. Commands are hardcoded paths/names.
- Safe modification: Prefer machine-readable output (`diskutil list -plist`, `df` parsed by
  column index, PowerShell `ConvertTo-Json`) and unit-test against captured real fixtures.
- Test coverage: Only serialization and the `parse_size_str` helper are unit-tested; the actual
  multi-line `diskutil`/`df`/PowerShell parsers have **no** tests against real output.

**WiFi scanning uses the removed macOS `airport` CLI:**

- Files: `src-tauri/src/wifi.rs:63-95` (`scan_wifi_macos`), `:97-116` (`parse_airport_output`),
  `:118-` (`parse_networksetup_output` — a documented "simplified parser" stub)
- Why fragile: The `airport` private-framework binary is deprecated and removed on modern macOS
  (14.4+), so scanning falls through to the stub `networksetup` parser; `parse_airport_output`
  also drops any SSID containing `:`. WiFi scan is best-effort and returns `Vec::new()` on
  failure, so this degrades silently.
- Safe modification: Replace with CoreWLAN/`system_profiler SPAirPortDataType`; keep the
  empty-list fallback. (Scan is optional per US-020, so impact is limited.)

**`to_str().unwrap()` on filesystem paths (non-UTF-8 panic):**

- Files: `src-tauri/src/download.rs:67,86` (production path), `src-tauri/src/extract.rs:160`
  (uses `unwrap_or("")` — safer), `src-tauri/src/lib.rs:156` (`get_webview_window("main").unwrap()`
  in debug-only devtools setup)
- Why fragile: A temp path that is not valid UTF-8 would panic the install command. Low
  likelihood on the target OSes but it is an un-handled panic in a write path.
- Safe modification: Propagate an error instead of `unwrap()` on `to_str()`.

---

## Scaling Limits

**Registry is a single static JSON loaded and filtered client-side:**

- Current capacity: Entire `index.json` is fetched and all filtering/search happens in-memory in
  `PackageStore.tsx` (`useMemo` over `registry.packages`).
- Limit: Fine for tens/low-hundreds of packages; a very large registry would bloat the initial
  fetch and client filtering.
- Scaling path: Paginate/segment the registry or add server-side indices if the catalog grows.
- Files: `src/PackageStore.tsx:42-87`, `src/types/package.ts:422-`

---

## Dependencies at Risk

**Reliance on external OS binaries at runtime:**

- Risk: `diskutil`, `df`, `powershell`, and macOS `airport` are invoked via `Command::new`.
  `airport` is already removed on current macOS; PowerShell behavior varies (Windows PowerShell
  vs PowerShell 7).
- Impact: Drive detection / WiFi scan can return empty or error on supported-but-newer OS
  versions.
- Migration plan: Move to native APIs (CoreWLAN, IOKit/DiskArbitration, Win32 volume APIs) or
  machine-readable output formats.
- Files: `src-tauri/src/drives.rs`, `src-tauri/src/wifi.rs`, `src-tauri/src/validate.rs`

---

## Missing Critical Features

**Real download/extract progress reporting:**

- Problem: No streamed progress or cancellation; the install is one opaque blocking call (see
  Tech Debt + Performance).
- Blocks: Trustworthy UX for large downloads; meeting US-007's "download progress reported to
  the frontend" intent.

**Integrity enforcement for store packages without checksums:**

- Problem: Packages lacking a `checksum` install unverified (see Security).
- Blocks: A safe package-store trust model.

---

## Test Coverage Gaps

**Rust CLI-output parsers are untested against real fixtures:**

- What's not tested: The multi-line `diskutil list external`, `df -k`, PowerShell, and
  `diskutil info` parsers; `parse_networksetup_output`.
- Files: `src-tauri/src/drives.rs`, `src-tauri/src/validate.rs:342-`, `src-tauri/src/wifi.rs`
- Risk: A silent format change yields drives with no size/filesystem, or empty drive lists, with
  no failing test.
- Priority: High (drive detection is on the critical write path).

**End-to-end install / package-install flow untested:**

- What's not tested: `install_minui` and `install_package` orchestration (download→extract→copy)
  are not exercised together; only the leaf helpers (`copy_dir_recursive`, `copy_extras_files`,
  `resolve_package_install_path`, `verify_checksum`) have unit tests. The `targetDir` traversal
  case is not tested.
- Files: `src-tauri/src/install.rs:164-245`, `src-tauri/src/package.rs:224-280`
- Risk: Regressions in the orchestration (partial-install handling, path resolution) go
  unnoticed.
- Priority: High.

**Frontend acceptance criteria marked "Verify in browser" were skipped:**

- What's not tested: Many stories' "Verify in browser using dev-browser skill" steps were not
  performed — `scripts/ralph/prd.json` story notes repeatedly say _"Browser testing skipped
  (ChromeDevTools MCP not configured)"_ (e.g. lines 70, 87). Visual/interaction behavior of
  DeviceSelector, ConfirmDialog, etc. was never validated in a real webview.
- Files: `scripts/ralph/prd.json` (notes fields), `src/*.tsx`
- Risk: Layout/interaction regressions and confirm-gating bugs in the write path.
- Priority: Medium (component logic is now unit-tested via Testing Library — see below).

---

## Corrected / Already-Resolved Findings

The following items were flagged in the audit brief but are **not present / already fixed** in
the code as of 2026-06-13. Documented here to prevent re-filing:

1. **`parse_size_str` `rfind` bug — FIXED.** `src-tauri/src/drives.rs:94` now uses
   `s.find(char::is_alphabetic)` (not `rfind`). `drives::tests::test_parse_size_str` **passes**
   (verified: `cargo test --lib drives` → 3 passed). The "1024 bytes" case returns `Some(1024)`.

2. **Dead-code structs `DownloadProgress` / `InstallProgress` / `DeviceInstallPaths` — DO NOT
   EXIST.** No such structs are in `download.rs` or `install.rs` (only `DownloadResult` and
   `InstallResult`, both used). `cargo clippy --lib` is **clean (0 warnings)**. The earlier
   "5 clippy warnings / 3 dead-code structs" no longer applies to the library build. (Current
   clippy noise is test-only — see Known Bugs.)

3. **"Zero React component tests / 79 type-only tests" — OUTDATED.** `npx vitest run` reports
   **100 tests across 12 files passing**, including 4 component suites:
   `src/Home.test.tsx`, `src/PackageStore.test.tsx`, `src/DriveSelector.test.tsx`,
   `src/WifiWizard.test.tsx` (React Testing Library). Type tests still exist under
   `src/types/*.test.ts`.

4. **"Frontend never shows the 'extracting' phase" — INCORRECT.** `src/Home.tsx:149` sets
   `setInstallPhase("extracting")`. The real, accurate concern is that the phase is _coarse and
   simulated_ (backend is one blocking call), not that the phase is missing — captured under
   Tech Debt above.

5. **Registry treated as untrusted — partially TRUE (good) but with gaps.** Schema validation
   exists (`validatePackageRegistry`/`validatePackageEntry`, `src/types/package.ts`), but URL
   scheme/host and `targetDir` traversal are **not** validated, and checksum is optional — see
   Security.

---

_Concerns audit: 2026-06-13_
