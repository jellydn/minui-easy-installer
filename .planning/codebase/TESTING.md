# Testing

## Test Frameworks

| Layer | Framework | Runner | Environment |
|-------|-----------|--------|-------------|
| Frontend | Vitest | `bun test` / `npx vitest` | `jsdom` |
| Backend | Rust built-in | `cargo test` | Native |

## Configuration

### Frontend

| File | Purpose |
|------|---------|
| `vitest.config.ts` | Includes `src/**/*.test.{ts,tsx}`, jsdom env, coverage thresholds |
| `vitest.setup.ts` | Imports `@testing-library/jest-dom/vitest` for DOM matchers |

### Backend

Rust tests use:
- `#[test]` or `#[tokio::test]` annotations
- `tempfile` crate for isolated temp directories
- Tests are located either inline (`#[cfg(test)] mod tests`) or external (`#[path = "drives_tests.rs"]`)

## Test File Inventory

### Frontend (17 test files)

| File | Tests |
|------|-------|
| `src/Home.test.tsx` | Home screen rendering + state transitions |
| `src/PackageStore.test.tsx` | Package store browse + install |
| `src/DriveSelector.test.tsx` | Drive picker interaction |
| `src/WifiWizard.test.tsx` | WiFi config form |
| `src/BiosInstaller.test.tsx` | BIOS catalog + install flow |
| `src/Settings.test.tsx` | Settings screen |
| `src/hooks/useForkInstall.test.ts` | Install orchestration hook (mocked IPC) |
| `src/hooks/useVersionCheck.test.ts` | Version comparison hook |
| `src/types/install.test.ts` | Install types + IPC contract |
| `src/types/package.test.ts` | Package registry validation |
| `src/types/release.test.ts` | GitHub release parsing |
| `src/types/device.test.ts` | Device profile lookups |
| `src/types/fork.test.ts` | Fork configuration |
| `src/types/bios.test.ts` | BIOS catalog types |
| `src/types/validate.test.ts` | Validation report types |
| `src/types/version.test.ts` | Version parsing |
| `src/types/drive.test.ts` | RemovableDrive type |

### Backend (4 test files)

| File | Lines | Tests |
|------|-------|-------|
| `src-tauri/src/install_tests.rs` | 789 | Install flow unit tests (temp dirs) |
| `src-tauri/src/drives_tests.rs` | ~230 | Drive detection + serialization (macOS-focused) |
| `src-tauri/src/bios_tests.rs` | 310 | BIOS catalog + install round-trip |
| `src-tauri/src/version/tests.rs` | — | Version parsing + comparison |

### Inline tests

`lib.rs` contains extensive inline `#[cfg(test)] mod tests` with **contract tests** for every Tauri command handler:
- `test_get_removable_drives_returns_result_shape`
- `test_install_minui_command_errors_on_bad_url`
- `test_validate_installation_on_empty_tempdir`
- `test_check_minui_version_on_empty_tempdir`
- `test_install_package_underlying_errors_on_bad_url`
- `test_scan_wifi_networks_returns_vec`
- `test_list_bios_catalog_returns_all_entries`
- `test_install_bios_file_underlying_round_trip`
- `test_detect_installed_packages_empty_tempdir`
- `test_check_package_updates_empty_input`
- `test_check_sd_card_health_errors_on_nonexistent`
- `test_fetch_url_errors_on_unreachable`
- `test_verify_archive_checksum_matches_correct_hash`

These contract tests verify that commands return proper error shapes, not Tauri transport errors — catching the `#[cfg(test)]` regression where a command was removed from the production handler but still called by the frontend.

## Mocking Patterns

### Frontend (Vitest)

```typescript
// Mock Tauri invoke
import { invoke } from "@tauri-apps/api/core";
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
const mockInvoke = invoke as Mock;
mockInvoke.mockResolvedValue({ success: true });

// Mock Tauri event listener
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));
```

### Backend (Rust)

Rust tests use real implementations — no mocking framework:
- `tempfile::tempdir()` for isolated filesystem state
- Unreachable URLs (`http://127.0.0.1:1/never.zip`) to test error paths
- Known-good binary data for round-trip tests (BIOS, checksums)

## Test Isolation

| Layer | Strategy |
|-------|----------|
| Rust | `tempfile::tempdir()` — each test gets a fresh temp directory, dropped on scope exit |
| TS | `vi.mock()` resets with `vi.resetAllMocks()` (auto via vitest config) |

## Running Tests

```bash
# Frontend
bun test                    # All TS tests
bun test -- --reporter=verbose

# Backend
cargo test                  # All Rust tests
cargo test --lib install    # Specific module tests
cargo test -- --ignored     # Include ignored tests (e.g., real SD card)

# Full check
just check                  # lint + typecheck + fmt + clippy + test
```

## CI Test Coverage

| Workflow | Tests Run |
|----------|-----------|
| `rust.yml` | `cargo fmt --check` + `cargo clippy -- -D warnings` + `cargo test` |
| `build.yml` | `cargo build` on macOS + Windows (catches platform-specific compile errors) |
| `react-doctor.yml` | React best-practices linting |
