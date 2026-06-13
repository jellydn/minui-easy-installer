# Codebase Concerns

**Analysis Date:** 2026-06-13

## Tech Debt

**Rust Backend Error Handling:**
- Issue: Heavy reliance on `.unwrap()` and `.unwrap_or("")` in Rust code instead of propagating errors with `?` or returning descriptive error enums.
- Files: `src-tauri/src/drives.rs`, `src-tauri/src/extract.rs`, `src-tauri/src/download.rs`, `src-tauri/src/install.rs`
- Impact: Silent panics or empty strings being used as fallbacks during critical operations like detecting SD cards or extracting archives, causing unhandled crashes.
- Fix approach: Implement proper `Result<T, E>` error bubbling. Replace `.unwrap()` with `?` and map to a custom Error enum for user-facing messaging.

## Known Bugs

**Drive Detection String Parsing:**
- Symptoms: App might fail to detect drives or report wrong sizes/free space.
- Files: `src-tauri/src/drives.rs`
- Trigger: If `diskutil` or `df` on macOS, or `powershell` on Windows outputs data in a slightly different format (e.g. localized strings, different spacing), the string parsing logic fails silently.
- Workaround: Manually verify the selected drive path if it gets detected.

## Security Considerations

**System API Calls (Unsafe FFI):**
- Risk: Potential memory unsafety or crashes when calling C APIs `libc::statvfs` on Unix and `GetDiskFreeSpaceExA` on Windows manually via FFI. String conversions via `CString` without robust boundary checks may fail.
- Files: `src-tauri/src/validate.rs`
- Current mitigation: Basic `CString` conversions.
- Recommendations: Replace manual unsafe FFI with established, safe third-party crates (e.g. `sysinfo` or `fs2`) to handle cross-platform filesystem space checks safely.

**Package Registry Schema Validation:**
- Risk: According to PRD, the package registry is an untrusted static JSON. If the frontend consumes this blindly without validation, malicious payloads could crash the app or trigger unintended behavior.
- Files: N/A (Store not yet implemented but planned).
- Current mitigation: None.
- Recommendations: Implement strict schema validation (using e.g., `zod` in TS or strict `serde` in Rust) before using fetched registry data.

## Performance Bottlenecks

**Drive Detection via Shell Commands:**
- Problem: Drive detection is very slow and blocks the main thread (or wastes CPU).
- Files: `src-tauri/src/drives.rs`
- Cause: Shelling out to system binaries (`diskutil`, `df`, `powershell`) instead of using native OS APIs.
- Improvement path: Use native Rust APIs (like the `sysinfo` crate) or proper Windows API bindings to detect removable drives without subprocess overhead.

## Fragile Areas

**Archive Checksum Verification:**
- Files: `src-tauri/src/download.rs`
- Why fragile: Uses temporary files, hardcoded fallbacks, string conversion for paths, and relies on URL string parsing (`url.rsplit('/').next()`) to get the file name, which fails on URLs with query strings.
- Safe modification: Refactor to stream checksum verification, handle `PathBuf` cleanly without `.to_str().unwrap()`, use `url` crate for robust URL parsing.
- Test coverage: Gaps in real-world archive testing, tests use mock files but fail to test realistic network behavior or URL structures.

## Scaling Limits

**Package Registry Loading:**
- Resource/System: Package Registry & Downloads
- Current capacity: MVP will load a single static JSON index file.
- Limit: Loading all package info upfront in one JSON file will severely slow down network requests and parsing when the registry grows beyond a few hundred packages.
- Scaling path: Implement paginated registry endpoints or categorized indexes. Add robust HTTP download streams with retry mechanisms.

## Dependencies at Risk

**Host OS CLI Tools:**
- Package: `df`, `diskutil`, `powershell`
- Risk: Heavy use of raw shell commands instead of proper crates makes the codebase dependent on the host OS's CLI tool versions and output formats.
- Impact: Drive detection breaks on different localized OS installations or future OS updates.
- Migration plan: Move to cross-platform native Rust crates like `sysinfo`.

## Missing Critical Features

**Package Store and WiFi Wizard:**
- Feature gap: The entire Package Store (US-006, US-012) and Wifi.pak / SSH.pak install flows (US-007, US-008, US-009) are missing from the codebase.
- Blocks: Prevents MVP release as these are critical Phase 1 features defined in the PRD.

## Test Coverage Gaps

**UI Components and Error States:**
- What's not tested: React components like `Home.tsx`, `DeviceSelector.tsx`, `DriveSelector.tsx` have no unit tests. The tests present are only for basic type utility wrappers.
- Files: `src/*.tsx`
- Risk: UI regressions during refactoring, especially since state management and Tauri invoke calls are tightly coupled.
- Priority: High

---

*Concerns audit: 2026-06-13*
