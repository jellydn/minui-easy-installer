# Testing

## Overview

The project uses **Vitest** for frontend testing and Rust's built-in test framework for backend testing. Total test coverage spans ~2,585 lines across 18 test files.

## Frontend Testing (Vitest)

### Configuration

```typescript
// vitest.config.ts
environment: "jsdom"
setupFiles: ["./vitest.setup.ts"]
include: ["src/**/*.test.{ts,tsx}"]
```

- **Test runner**: Vitest
- **Environment**: jsdom (browser-like DOM)
- **Setup**: `vitest.setup.ts` imports `@testing-library/jest-dom/vitest` for DOM matchers

### Test Files (17 files, 2,585 lines)

| Test File | Lines | Focus |
|-----------|-------|-------|
| `src/types/release.test.ts` | 315 | GitHub release parsing |
| `src/hooks/useVersionCheck.test.ts` | 272 | Version checking hook |
| `src/types/install.test.ts` | 260 | Install flow types |
| `src/BiosInstaller.test.tsx` | 250 | BIOS installer component |
| `src/Home.test.tsx` | 237 | Home screen component |
| `src/PackageStore.test.tsx` | 190 | Package store component |
| `src/WifiWizard.test.tsx` | 180 | WiFi wizard component |
| `src/hooks/useForkInstall.test.ts` | 149 | Fork install hook |
| `src/types/validate.test.ts` | 135 | Validation types |
| `src/Settings.test.tsx` | 120 | Settings component |
| `src/types/fork.test.ts` | 112 | Fork types |
| `src/DriveSelector.test.tsx` | 106 | Drive selector component |
| `src/types/drive.test.ts` | 60 | Drive types |
| `src/types/version.test.ts` | 54 | Version parsing |
| `src/types/package.test.ts` | 54 | Package types |
| `src/types/device.test.ts` | 48 | Device profiles |
| `src/types/bios.test.ts` | 43 | BIOS types |

### Testing Libraries
- `@testing-library/react` — Render components, query DOM
- `@testing-library/user-event` — Simulate user interactions
- `@testing-library/jest-dom` — Extended DOM matchers (`toBeInTheDocument()`, etc.)

### Patterns
- **Co-located tests**: Test files next to source files (`src/types/device.test.ts` alongside `src/types/device.ts`)
- **Component tests**: Render with test props, assert DOM content
- **Hook tests**: `renderHook()` from testing library, act() for state changes
- **Type tests**: Pure function tests for validation/parsing logic

### Running Tests

```bash
bun test                    # vitest run (all tests)
bun test -- --reporter=verbose  # verbose output
bun run test:coverage       # vitest run --coverage
```

## Backend Testing (Rust)

### Framework
- Built-in Rust `#[test]` attribute
- `#[tokio::test]` for async test functions
- `#[cfg(test)]` modules within source files

### Test Locations

| Module | Test Location | Focus |
|--------|--------------|-------|
| `lib.rs` | Inline `#[cfg(test)]` | IPC contract tests (error propagation, return shapes) |
| `install.rs` | Inline `#[cfg(test)]` | Copy operations, preserved folders, full pipeline |
| `version/tests.rs` | Separate test file | Version parsing edge cases |
| `fs_utils.rs` | Inline tests | copy_dir_recursive, get_free_space |

### Testing Patterns
- **tempfile**: `tempfile::tempdir()` for filesystem simulation (no real SD card needed)
- **Mock servers**: `TcpListener` one-shot HTTP servers for download tests
- **Full pipeline**: `test_install_minui_with_cancel_full_pipeline` exercises download → extract → copy
- **Cross-platform**: `/nonexistent` paths for health check/file-not-found edge cases
- **Environment-dependent**: WiFi tests have no specific network assertions

### Known Test Quirks
- WiFi tests are environment-dependent (no specific networks asserted)
- Health check tests use `/nonexistent` paths (works cross-platform)

### Running Tests

```bash
cd src-tauri && cargo test           # All Rust tests
cd src-tauri && cargo test --lib     # Library tests only
cd src-tauri && cargo test -- --nocapture  # Show println output
```

## Mocking & Fixtures

### Frontend
- **Mock hooks**: Custom hook mocks via `vi.mock()`
- **Mock Tauri API**: `vi.mock("@tauri-apps/api/core")` for IPC simulation
- **Test data**: Inline test fixtures (no shared fixture files)

### Backend
- **Inline test data**: ZIP archives created programmatically in tests
- **One-shot servers**: `start_one_shot_file_server()` for HTTP mocking
- **Temp directories**: `tempfile::tempdir()` for all filesystem tests

## All Checks

```bash
just check    # lint + typecheck + cargo fmt --check + cargo clippy
just fmt      # oxfmt + cargo fmt
```

Pre-commit via `prek.toml` runs lint and typecheck on staged files.
