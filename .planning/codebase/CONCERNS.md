# Concerns & Technical Debt

## Resolved Items (v0.2.0)

| Issue | Resolution | Commit |
|-------|-----------|--------|
| macOS 14.4+ airport deprecation | Added `system_profiler SPAirPortDataType` 3-tier fallback in `wifi.rs` | `feat(wifi)` |
| Linux CI gap | Added Linux CI job in `react-doctor.yml` | CI update |
| Shared error handling | `errorMessage()`/`asError()` utilities in `src/types/errors.ts` | `feat(errors)` |
| InstallManager IPC tangle | Extracted `EventDispatcher` trait + `InstallManager` from `lib.rs` | `f4cc561` |
| useForkInstall complexity | Extracted `InstallOrchestrator` vanilla TS class (425→129 lines) | `2f214d6` |
| InstallPhase enum shallowness | Deleted enum; 4 steps inlined in `install_minui_with_cancel()` | `cf77933` |
| Per-request HTTP client | `OnceLock<reqwest::Client>` with connection pooling | `97db2f8` |
| Concurrent package I/O contention | `Promise.all` → sequential `for...of` in `updateAll()` | `68a54e2` |
| useRef render mutation | `useState(() => new InstallOrchestrator())` lazy init | `8ff18a9` |
| Health check: no speed test | 64MB sequential read benchmark with 5 MB/s threshold | `a58e6f1` |
| Health check: macOS-only fs | `fsutil` for Windows filesystem detection | `a58e6f1` |
| Health check: hardcoded PAKs | `scan_pak_dirs()` walks `Tools/` for `*.pak` directories | `a58e6f1` |

## Medium Priority

### rom_dirs_created counter accuracy

- **File**: `src-tauri/src/install.rs` (`create_rom_dirs`)
- **Issue**: After removing the `exists()` guard (perf simplification), `rom_dirs_created` always equals `ROM_DIRS.len()` (16), even when directories already exist
- **Impact**: Misleading counter in `InstallResult` — shows 16 "created" on every install
- **Fix**: Either restore the `exists()` check or drop the field from `InstallResult`

### fetch_url OnceLock panic

- **File**: `src-tauri/src/lib.rs` (`http_client()`)
- **Issue**: `reqwest::Client::builder().build().expect(...)` panics on first use if TLS backend is missing
- **Impact**: App crashes on `fetch_url` instead of returning a graceful error. Extremely unlikely in practice (desktop app with bundled TLS)
- **Fix**: Consider `OnceLock::get_or_try_init` if available, or restore `map_err` at the call site

### WiFi SSID with special characters

- **File**: `src-tauri/src/wifi.rs`
- **Issue**: `wifi.txt` format is `SSID:PASSWORD` per line. SSIDs containing `:` or `#` (comments) could cause parsing issues
- **Impact**: Edge case — rare in practice
- **Fix**: Document limitation or escape special characters

## Low Priority

### async test in install_manager_tests

- **File**: `src-tauri/src/install_manager_tests.rs`
- **Issue**: Test `start_cancels_previous_install` uses `#[tokio::test]` but `start()` spawns a background task that may not complete before the test ends
- **Impact**: Test may be flaky on slow CI
- **Fix**: Add a small delay or use a completion signal

### Large TypeScript files

- `src/lib/InstallOrchestrator.ts`: ~340 lines — state machine with multiple methods
- `src/App.tsx`: ~130 lines — screen router
- `src/PackageStore.tsx`: ~200 lines — package browser
- **Impact**: Readability — all under 500 lines, no immediate concern

### Device platform data

- **File**: `src/types/device-install-map.json`
- **Issue**: JSON loaded at compile time via TypeScript import. Adding a new device requires touching the JSON + `device.ts` + potentially `platform.rs`
- **Impact**: Maintenance overhead for new device support
- **Fix**: Centralize device definitions into a single source of truth

### deprecated install_minui command

- **File**: `src-tauri/src/lib.rs`
- **Issue**: Synchronous `install_minui` command still registered but marked deprecated
- **Impact**: Dead code path — no frontend callers
- **Fix**: Remove when confident no external consumers exist

## No Current Concerns

These areas were reviewed and found clean:

| Area | Status |
|------|--------|
| Symlink escape prevention | Hardened — `create_target_within`, `install_bios_from_bytes`, `copy_dir_recursive` |
| Path traversal in package install | Validated — `create_target_within` canonicalize-before-create |
| CSP bypass | Tightly scoped in `tauri.conf.json` + SSRF `ALLOWED_URLS` |
| WiFi password logging | Never logged — `wifi.txt` written to SD card only |
| Registry data validation | Schema validated before use |
| ROM/save preservation | Case-insensitive preserved folder matching |
| Temp file cleanup | `InstallSession` drop + `benchmark_read_speed` cleanup in all paths |
| Concurrent installs | Prevented — `InstallManager` holds single token, cancels previous on new start |
