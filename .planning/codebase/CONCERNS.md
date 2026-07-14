# Concerns

## TODO Items (Actionable)

| # | Priority | File | Item | Detail |
|---|----------|------|------|--------|
| ~~1~~ | ~~Medium~~ | ~~`src/types/package.ts:45`~~ | ~~Package cache TTL~~ | ✅ **Resolved.** Registry cache now expires after 5 minutes (`CACHE_TTL_MS`). `cachedAt` timestamp tracked alongside `cachedRegistry`. |
| ~~2~~ | ~~Medium~~ | ~~`src-tauri/src/lib.rs:89`~~ | ~~Byte-level download progress~~ | ✅ **Resolved.** `InstallProgressEvent` now carries `currentBytes`/`totalBytes` fields. `download_progress` callback in `start_install` emits `install-progress` events with byte-level data. Frontend `InstallProgress.tsx` already had the `<progress>` bar waiting for these fields. |
| ~~3~~ | ~~Low~~ | ~~`src-tauri/src/wifi.rs:74`~~ | ~~Linux WiFi~~ | ✅ **Resolved.** `get_current_wifi_ssid_linux()` added — tries `nmcli` first, falls back to `iwgetid -r`. `get_current_wifi_ssid()` dispatches to it on `cfg(target_os = "linux")`. |
| ~~4~~ | ~~Low~~ | ~~`src-tauri/src/drives.rs:373`~~ | ~~Linux udev detection~~ | ✅ **Resolved.** `lsblk` now includes the `RM` (removable) column. Only devices with `RM=1` are included. Internal SSDs/HDDs (RM=0) are filtered out. Also handles `children` in lsblk JSON output. |

## Platform Limitations

### No Linux Support (Phase 2)

- `AGENTS.md` specifies MVP is Windows + macOS only
- `drives.rs` now has `lsblk` with removable filtering (`RM` column)
- `wifi.rs` now has `get_current_wifi_ssid_linux()` via `nmcli` / `iwgetid`
- Phase 2 remaining: `mkfs` formatting, full CI + testing

### macOS WiFi Deprecation (14.4+)

- **Risk**: Apple removed `airport` from macOS 14.4+. `networksetup -getairportnetwork` is also broken.
- **Mitigation**: Tiered fallback in `wifi.rs` — tries `airport -s` first, falls back to `system_profiler SPAirPortDataType`
- **Status**: ✅ Mitigated for macOS 14.x. Monitor WWDC/release notes for `system_profiler` deprecation in macOS 15+.

### No Formatting in MVP

- `format_drive` exists as a Tauri command but is gated behind confirmation dialogs
- Never format drives without explicit user confirmation (per `AGENTS.md` constraint)

## Large Files

| File | Lines | Concern |
|------|-------|---------|
| ~~`bios.rs`~~ | ~~668~~ | ~~Mixed production + inline tests~~ → ✅ **Resolved.** Tests split to `bios_tests.rs` (234 lines). `bios.rs` now 311 lines. |
| `install_tests.rs` | 789 | Test file — acceptable but approaching 1k line soft limit |
| `lib.rs` | 619 | 17 command handlers + contract tests — natural for a Tauri command registry |

## Performance Considerations

- **Package registry**: Fetched once per session with 5-minute TTL. Long-running sessions will re-fetch when cache expires.
- **Archive streaming**: Downloads stream to temp files with progress callbacks. Byte-level progress now surfaces `currentBytes`/`totalBytes` to the frontend.
- **Clone usage**: Some `Arc::clone()` calls in progress callbacks and `CancellationToken` propagation — idiomatic for Rust async, not a concern.
- **`unwrap()` in tests**: 209 occurrences across test files — standard Rust test pattern, not a concern in production code.

## Security

All findings from previous audits are resolved or mitigated:
- ✅ CI workflow added (Rust fmt + clippy + test)
- ✅ Test files split from production modules (`install_tests.rs`, `drives_tests.rs`, `bios_tests.rs`)
- ✅ Deprecated `install_minui` command removed
- ✅ `#[allow(dead_code)]` replaced with `#[cfg(test)]`
- ✅ `#[allow(unused_variables)]` added for platform-gated parameters
- ✅ Linux `lsblk` fallback replacing hard error + removable filtering
- ✅ TODO comments addressed or resolved
- ✅ WiFi deprecation mitigated (system_profiler fallback) + Linux WiFi implemented
- ✅ Package cache TTL added
- ✅ Byte-level download progress wired up

## Pending (No Action Needed)

- **E2E tests**: No infrastructure for end-to-end testing (requires real SD cards + devices). Not planned for MVP.
- **React Doctor**: CI check exits with scan status on PR events — pre-existing project-wide issue, not branch-related.
