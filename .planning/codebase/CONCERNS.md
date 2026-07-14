# Concerns

## TODO Items (Actionable)

| # | Priority | File | Item | Detail |
|---|----------|------|------|--------|
| 1 | Medium | `src/types/package.ts:45` | Package cache TTL | Registry is fetched once per session. Long-running sessions won't pick up new package releases. Add a TTL to the cache. |
| 2 | Medium | `src-tauri/src/lib.rs:89` | Byte-level download progress | `download_progress` callback receives `(bytes, total)` but discards it. `InstallProgressEvent` needs `currentBytes`/`totalBytes` fields and the frontend needs a progress bar. |
| 3 | Low | `src-tauri/src/wifi.rs:74` | Linux WiFi | No `get_current_wifi_ssid` implementation on Linux. Needs `nmcli` or `iwconfig` fallback. |
| 4 | Low | `src-tauri/src/drives.rs:373` | Linux udev detection | `lsblk` fallback doesn't filter by removable flag (`RM` column). Add udev-based detection via `/sys/block/*/removable`. |

## Platform Limitations

### No Linux Support (Phase 2)

- `AGENTS.md` specifies MVP is Windows + macOS only
- `drives.rs` now has a basic `lsblk` fallback, but no WiFi, no formatting, and no removable-filtering
- Phase 2 would need: udev detection, `nmcli`/`iwconfig` WiFi, `mkfs` formatting, full testing

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
| `install_tests.rs` | 789 | Test file — acceptable but approaching 1k line soft limit |
| `bios.rs` | 668 | Mixed production + inline tests — could split tests to `bios_tests.rs` |
| `lib.rs` | 619 | 17 command handlers + contract tests — natural for a Tauri command registry |

## Performance Considerations

- **Package registry**: Fetched once per session. No incremental updates. Acceptable for MVP (session-scoped).
- **Archive streaming**: Downloads stream to temp files with progress callbacks. Cancellation checks at phase boundaries (not mid-chunk).
- **Clone usage**: Some `Arc::clone()` calls in progress callbacks and `CancellationToken` propagation — idiomatic for Rust async, not a concern.
- **`unwrap()` in tests**: 209 occurrences across test files — standard Rust test pattern, not a concern in production code.

## Security

All `CONCERNS.md` findings from the previous audit have been addressed or were already mitigated:
- ✅ CI workflow added (Rust fmt + clippy + test)
- ✅ Test files split from production modules
- ✅ Deprecated `install_minui` command removed
- ✅ `#[allow(dead_code)]` replaced with `#[cfg(test)]`
- ✅ `#[allow(unused_variables)]` added for platform-gated parameters
- ✅ Linux `lsblk` fallback replacing hard error
- ✅ TODO comments added for known gaps
- ✅ WiFi deprecation already mitigated (system_profiler fallback)

## Pending (No Action Needed)

- **E2E tests**: No infrastructure for end-to-end testing (requires real SD cards + devices). Not planned for MVP.
- **React Doctor**: CI check exits with scan status on PR events — pre-existing project-wide issue, not branch-related.
