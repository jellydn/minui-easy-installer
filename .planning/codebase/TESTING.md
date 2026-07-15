# Testing

## Test Runners

| Layer | Runner | Config |
|-------|--------|--------|
| Rust | `cargo test` | `Cargo.toml` |
| TypeScript | `vitest` | `vitest.config.ts` |
| Full suite | `just check` + `cargo test` + `vitest run` | `justfile` |

## Test Counts

| Layer | Tests | Files |
|-------|-------|-------|
| Rust | 170 passed, 1 ignored | 7 test files + inline `#[cfg(test)]` modules |
| TypeScript | 127 passed | 17 test files |
| **Total** | **297** | **24** |

## Rust Testing

### Test File Organization

| File | Lines | Scope |
|------|-------|-------|
| `lib_tests.rs` | 393 | Tauri command contract tests (IPC boundary) |
| `install_tests.rs` | — | Full install pipeline tests |
| `install_copy_tests.rs` | 478 | `copy_base_files` and copy-specific tests |
| `install_extras_tests.rs` | — | Extras installation edge cases |
| `bios_tests.rs` | 298 | BIOS catalog, status, file installation |
| `drives_tests.rs` | 417 | Drive detection mocking |
| `wifi_tests.rs` | — | WiFi config write + platform-specific scan tests |
| `version/tests.rs` | 369 | Version parsing and comparison |

### Patterns
- `tempfile` crate for temporary directories (no real SD card needed)
- Inline `#[cfg(test)] mod tests` for parser/utility functions
- External `*_tests.rs` for integration and IPC contract tests
- `#[cfg(target_os = "macos")]` gating for platform-specific test modules

### Mocking
- `DriveDetector` trait with `&dyn DriveDetector` for mock drive detection
- Test-specific functions use controlled inputs (no external shell/network calls)
- Health check tests use `/nonexistent` paths (cross-platform)

## TypeScript Testing

### Framework
- **Vitest** with `jsdom` environment
- **@testing-library/react** for component rendering
- **@testing-library/user-event** for simulated user interaction

### Test File Organization

| File | Type | Scope |
|------|------|-------|
| `types/fork.test.ts` | Unit | Fork presets, URL building, rehydration (20 tests) |
| `types/install.test.ts` | Unit | Install IPC functions, error handling (260 lines) |
| `types/release.test.ts` | Unit | GitHub release parsing, caching (315 lines) |
| `types/package.test.ts` | Unit | Registry fetch, RegistryCache TTL (13 cache tests) |
| `types/validate.test.ts` | Unit | Schema validation |
| `types/version.test.ts` | Unit | Version parsing |
| `types/device.test.ts` | Unit | Device profile lookup |
| `types/drive.test.ts` | Unit | Drive size formatting |
| `types/bios.test.ts` | Unit | BIOS catalog |
| `hooks/useForkInstall.test.ts` | Hook | Install hook with ForkProvider (272 lines) |
| `hooks/useVersionCheck.test.ts` | Hook | Version check hook |
| `Home.test.tsx` | Component | Home screen rendering |
| `PackageStore.test.tsx` | Component | Package store rendering |
| `DriveSelector.test.tsx` | Component | Drive picker rendering |
| `BiosInstaller.test.tsx` | Component | BIOS installer rendering |
| `Settings.test.tsx` | Component | Settings/fork selection |
| `WifiWizard.test.tsx` | Component | WiFi wizard rendering |

### Patterns
- `vi.mock("@tauri-apps/api/core")` for Tauri IPC mocking
- `vi.useFakeTimers()` for `RegistryCache` TTL tests
- `ForkProvider` wrapper for components that use `useFork()`
- `makeRegistry()` factory function for test data fixtures

## CI Testing

| Workflow | Runner | What |
|----------|--------|------|
| `rust.yml` | `ubuntu-latest` | `cargo fmt --check`, `cargo clippy`, `cargo test` |
| `build.yml` | `macos-latest`, `windows-latest`, `ubuntu-latest` | `cargo build --release` |
| `react-doctor.yml` | — | React code health check |
