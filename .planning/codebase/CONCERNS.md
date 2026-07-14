# Technical Concerns

## Code Complexity

### Largest Files

| File | Lines | Risk |
|------|-------|------|
| `src-tauri/src/install.rs` | 1,168 | Install flow + comprehensive test suite — high complexity, but tests are thorough |
| `src-tauri/src/drives.rs` | 743 | Platform-specific drive detection (macOS/Windows) — two code paths, hard to test |
| `src-tauri/src/bios.rs` | 667 | BIOS management — moderate complexity with security-sensitive path operations |
| `src/types/package.ts` | 418 | Package registry fetching, parsing, validation — many concerns in one file |
| `src/hooks/useForkInstall.ts` | 399 | Install orchestration hook — complex state machine |

**Recommendation**: `install.rs` and `drives.rs` are good candidates for splitting when they grow further.

## Platform Risks

### macOS WiFi Deprecation
- WiFi scanning relies on the `airport` command-line tool
- macOS 14.4+ may deprecate/remove `airport` access
- **Impact**: WiFi scanning feature breaks on newer macOS
- **Mitigation**: Monitor macOS releases; consider CoreWLAN framework integration

### Windows Drive Detection
- Platform-specific code in `drives.rs` uses Windows API bindings (`windows-sys`)
- Drive letter enumeration and filesystem detection are OS-dependent
- **Impact**: Edge cases on exotic Windows configurations

### No Linux Support
- Phase 1 (MVP) explicitly excludes Linux
- **Impact**: Limited user base; Linux retro handheld users can't use the installer
- **Mitigation**: Planned for future phase

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
- Zero `TODO`, `FIXME`, `HACK`, or `XXX` comments found in codebase
- This suggests either very clean code or lack of documentation of known issues
- **Recommendation**: Consider adding TODO comments for known limitations

### Deprecated Commands
- `install_minui` (synchronous) is deprecated in favor of `start_install` (async with cancellation)
- Old command may still have callers or references
- **Status**: Recent refactor removed dead code (`refactor: delete dead parallel device system and deprecated archive commands`)

### Fork Support Complexity
- Custom fork support adds configuration surface area
- `ForkContext`, `useForkInstall`, fork-specific version tracking
- **Risk**: Fork-specific edge cases (different archive structures, version formats)

## Testing Gaps

### Platform-Specific Code
- Drive detection (`drives.rs`) has limited test coverage due to platform dependency
- WiFi scanning tests are environment-dependent
- **Recommendation**: Add integration tests against real SD cards (one exists but is `#[ignore]`d)

### Frontend Integration
- Heavy unit test coverage but limited end-to-end tests
- No tests for Tauri IPC contract compliance from the frontend side
- **Recommendation**: Consider Tauri end-to-end tests with `tauri-driver`

### Coverage
- No coverage thresholds enforced
- Vitest coverage configured but not run in CI by default (`@vitest/coverage-v8`)

## Build & CI

### Pre-commit Complexity
- `prek` hooks can conflict if two hooks touch the same file
- Requires `git add -u` and retry on conflict
- **Risk**: Frustrating developer experience on first commit

### CI Workflow
- Single workflow: `react-doctor.yml` in `.github/workflows/`
- No Rust CI checks (cargo clippy, cargo test) in CI
- **Recommendation**: Add Rust CI pipeline

## Future Considerations

### Linux Support
- Phase 1 intentionally excludes Linux
- Drive detection, WiFi scanning, and filesystem operations would need Linux implementations

### Format Support
- `format_drive` command exists but is not implemented in MVP
- Confirmation dialog (`FormatConfirmDialog.tsx`) exists but is unused

### Package Store Scaling
- Current `store.json` bundled fallback is static
- Remote registry (`packages.minui.dev`) has session-scoped cache
- **Risk**: Registry growth could slow initial load
- **Recommendation**: Consider pagination or incremental updates for large registries
