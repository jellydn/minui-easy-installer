# Technical Concerns

## Code Complexity

### Largest Files

| File | Lines | Risk |
|------|-------|------|
| `src-tauri/src/bios.rs` | 667 | BIOS management — moderate complexity with security-sensitive path operations |
| `src/types/package.ts` | 418 | Package registry fetching, parsing, validation — many concerns in one file |
| `src/hooks/useForkInstall.ts` | 399 | Install orchestration hook — complex state machine |
| ~~`src-tauri/src/install.rs`~~ | ~~1,168~~ **381** | ✅ **Fixed** — tests moved to `install_tests.rs` via `#[path]` attribute |
| ~~`src-tauri/src/drives.rs`~~ | ~~743~~ **374** | ✅ **Fixed** — tests moved to `drives_tests.rs` via `#[path]` attribute |

## Platform Risks

### macOS WiFi Deprecation
- WiFi scanning uses the `airport` command-line tool
- macOS 14.4+ may deprecate/remove `airport` access
- **Impact**: WiFi scanning feature breaks on newer macOS
- **Mitigation**: ✅ **Already mitigated** — `system_profiler SPAirPortDataType` fallback in `wifi.rs` handles macOS 14.4+ where `airport` is removed
- **Status**: Monitor macOS releases for further changes

### Windows Drive Detection
- Platform-specific code in `drives.rs` uses Windows API bindings (`windows-sys`)
- Drive letter enumeration and filesystem detection are OS-dependent
- **Impact**: Edge cases on exotic Windows configurations

### No Linux Support
- Phase 1 (MVP) explicitly excludes Linux
- **Impact**: Limited user base; Linux retro handheld users can't use the installer
- **Mitigation**: ✅ Linux drive detection now has `lsblk` fallback in `drives.rs`; WiFi scanning already has `nmcli` support in `wifi.rs`

## Security

### Symlink Race Guards
- Multiple places implement symlink race protection (canonicalize → create → re-validate)
- Currently covered in: `pipeline.rs::create_target_within`, `bios.rs::install_bios_from_bytes`
- **Risk**: New file-writing code paths could miss these guards
- **Mitigation**: Use `create_target_within` and `copy_dir_recursive` helpers consistently

### CSP Restrictions
- Content Security Policy is tightly scoped to specific external domains
- Adding new external services requires CSP update in `tauri.conf.json`
- **Risk**: Forgetting to update CSP when adding new integrations
- **Mitigation**: Document CSP in architecture docs; test with CSP violations

### Registry Trust
- Package registry data (`packages.minui.dev`) is treated as untrusted
- Schema validation exists via `validateStoreEntry()`
- **Risk**: New registry fields or formats could bypass validation
- **Mitigation**: Regular review of validation logic when registry schema changes

## Technical Debt

### No TODO/FIXME
- ~~Zero `TODO`, `FIXME`, `HACK`, or `XXX` comments found in codebase~~ ✅ **Fixed** — TODO comments added for known limitations in `wifi.rs`, `drives.rs`, `lib.rs`, and `package.ts`

### Deprecated Commands
- ✅ **Fixed** — `install_minui` Tauri command removed from `lib.rs`. Underlying function kept with `#[cfg(test)]` for contract tests.

### Fork Support Complexity
- Custom fork support adds configuration surface area
- `ForkContext`, `useForkInstall`, fork-specific version tracking
- **Risk**: Fork-specific edge cases (different archive structures, version formats)

## Testing Gaps

### Platform-Specific Code
- Drive detection (`drives.rs`) has limited test coverage due to platform dependency
- WiFi scanning tests are environment-dependent
- ✅ **Mitigated** — one ignored integration test exists against real SD cards; Linux detection now has `lsblk` fallback

### Frontend Integration
- Heavy unit test coverage but limited end-to-end tests
- No tests for Tauri IPC contract compliance from the frontend side
- **Recommendation**: Consider Tauri end-to-end tests with `tauri-driver`

### Coverage
- ✅ **Fixed** — Vitest coverage thresholds now enforced: 50% statements/lines, 40% branches/functions

## Build & CI

### Pre-commit Complexity
- `prek` hooks can conflict if two hooks touch the same file
- Requires `git add -u` and retry on conflict
- **Risk**: Frustrating developer experience on first commit

### CI Workflow
- ✅ **Fixed** — Rust CI added (`.github/workflows/rust.yml`) with cargo fmt, clippy (`--all-targets`), test (`--all-targets`), and cargo caching

## Future Considerations

### Linux Support
- Drive detection: ✅ `lsblk` fallback added (Phase 2 groundwork)
- WiFi scanning: ✅ `nmcli` support already present in `wifi.rs`
- Filesystem operations: `libc::statvfs` already used (Unix-compatible)
- **Remaining**: Full Linux Tauri build not tested

### Format Support
- `format_drive` command exists but is not implemented in MVP
- Confirmation dialog (`FormatConfirmDialog.tsx`) exists but is unused

### Package Store Scaling
- Current `store.json` bundled fallback is static
- Remote registry (`packages.minui.dev`) has session-scoped cache
- **Risk**: Registry growth could slow initial load
- **Recommendation**: Consider pagination or incremental updates for large registries
