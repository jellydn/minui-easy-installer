# Testing Patterns

**Analysis Date:** 2026-06-13

> Two independent test stacks: **vitest** for the TS/React frontend (`src/`) and
> **cargo test** with inline `#[cfg(test)]` modules for the Rust backend (`src-tauri/src/`).

## Test Framework

**Runner:**

- Frontend: **vitest** `^4.1.8` (`package.json` devDependencies).
  Config: `vitest.config.ts` (uses `@vitejs/plugin-react`, `environment: "jsdom"`,
  `setupFiles: ["./vitest.setup.ts"]`, `include: ["src/**/*.test.{ts,tsx}"]`).
- Backend: **cargo test** (Rust edition 2021, `src-tauri/Cargo.toml`). Async tests
  run under `tokio` (full features). No separate test crate — tests are inline.

**Assertion Library:**

- Frontend: vitest built-in `expect` + `@testing-library/jest-dom` matchers
  (`toBeInTheDocument`, `toBeDisabled`, `toHaveAttribute`) registered via
  `vitest.setup.ts` → `import "@testing-library/jest-dom/vitest";`.
  Component tests use `@testing-library/react` (`render`, `screen`, `waitFor`,
  `cleanup`) and `@testing-library/user-event`.
- Backend: std `assert!` / `assert_eq!`.

**Run Commands:**

```bash
npm test            # Run all frontend tests once ("test": "vitest run")
npx vitest          # Watch mode (vitest default; no npm script defined)
npx vitest run --coverage   # Coverage via @vitest/coverage-v8 (installed)
cargo test          # Run all Rust backend tests (from src-tauri/)
```

## Test File Organization

**Location:**

- **Co-located** with source in both stacks.
- Frontend: `src/types/<name>.test.ts` sits next to `src/types/<name>.ts`; component
  tests `src/<Component>.test.tsx` sit next to `src/<Component>.tsx`.
- Backend: tests live in the same `.rs` file inside an inline `mod tests`.

**Naming:**

- TS: `*.test.ts` (pure logic/types) and `*.test.tsx` (React components).
- Rust: `#[test]` fns named `test_<behavior>` inside `#[cfg(test)] mod tests`.

**Structure:**

```
src/
  types/
    drive.ts          drive.test.ts        (8 tests)
    version.ts        version.test.ts       (4 tests)
    install.ts        install.test.ts       (7 tests)
    release.ts        release.test.ts      (12 tests)
    package.ts        package.test.ts      (23 tests)
    validate.ts       validate.test.ts      (6 tests)
    device.ts         device.test.ts        (4 tests)
    archive.ts        archive.test.ts      (15 tests)
  DriveSelector.tsx   DriveSelector.test.tsx (5 tests)
  Home.tsx            Home.test.tsx          (4 tests)
  WifiWizard.tsx      WifiWizard.test.tsx    (6 tests)
  PackageStore.tsx    PackageStore.test.tsx  (6 tests)
src-tauri/src/*.rs    (inline #[cfg(test)] mod tests)
```

## Test Structure

**Suite Organization (TS — `src/types/install.test.ts`):**

```typescript
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { InstallResult } from "./install";
import { installMinui } from "./install";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("installMinui", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("returns success with file counts on successful install", async () => {
    const mockResult: InstallResult = {
      success: true,
      error: null,
      base_files_copied: 15,
      extras_files_copied: 3,
    };
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue(mockResult);
    const result = await installMinui({
      baseUrl: "...",
      sdMount: "...",
      platform: "miyoo-mini-plus",
      extrasDir: "/Tools",
    });
    // expect(...)
  });
});
```

**Suite Organization (Rust — `src-tauri/src/version.rs`):**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_minui_version_with_prefix() {
        let content = "MinUI v2024.12.25\nSome other content";
        assert_eq!(parse_minui_version(content), Some("2024.12.25".to_string()));
    }
}
```

**Patterns:**

- Setup: TS `beforeEach(() => vi.clearAllMocks())` in every mocked suite; component
  suites also `afterEach(() => cleanup())` (`src/Home.test.tsx`,
  `src/DriveSelector.test.tsx`, `src/WifiWizard.test.tsx`).
- Teardown: `cleanup()` (Testing Library) in component tests; Rust uses scoped
  `tempfile::tempdir()` that auto-removes when dropped.
- Assertion: discriminated-union results are asserted with a type-narrowing guard,
  e.g. `expect(result.success).toBe(true); if (result.success) { expect(result.data...) }`
  (`src/types/archive.test.ts`).

## Mocking

**Framework:** vitest `vi` (frontend). Rust has no mocking framework — tests use real
filesystem temp dirs.

**Patterns (`src/types/archive.test.ts`, `src/Home.test.tsx`):**

```typescript
// Mock the Tauri IPC bridge so backend invoke() calls are intercepted
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

// Per-test: resolve or reject the mocked invoke
const { invoke } = await import("@tauri-apps/api/core");
vi.mocked(invoke).mockResolvedValue(mockResult);
// vi.mocked(invoke).mockRejectedValue(new Error("scan unavailable"));

// Assert the exact command + args contract
expect(invoke).toHaveBeenCalledWith("write_wifi_config", {
  sdMount: "/Volumes/SD",
  ssid: "MyNetwork",
  password: "secret123",
});

// Partial mock keeping real exports (Home.test.tsx)
vi.mock("./types/package", async (importOriginal) => {
  const actual = await importOriginal<typeof import("./types/package")>();
  return {
    ...actual,
    fetchPackageRegistry: vi.fn(),
    checkPackageUpdates: vi.fn(),
  };
});
```

**What to Mock:**

- The `@tauri-apps/api/core` `invoke` bridge (every test that touches the backend).
- Sibling type-module API fns when testing a component in isolation (`./types/release`,
  `./types/version`, `./types/install`, `./types/validate`, `./types/package` in
  `src/Home.test.tsx`).
- `fetchMinUIRelease` accepts an injectable `fetchFn` param (`src/types/release.ts`),
  enabling fetch to be stubbed without `vi.mock` (dependency injection).

**What NOT to Mock:**

- Pure logic under test (e.g. `formatSize`, `parseGitHubRelease`,
  `formatValidationReport`'s local fallback) is exercised directly.
- Rust: nothing is mocked; real files are written into a `tempfile` dir.

## Fixtures and Factories

**Test Data:**

```typescript
// Inline literal fixtures defined per-file (no shared factory module)
const mockDrive: RemovableDrive = {
  name: "SD_CARD",
  mount_path: "/Volumes/SD_CARD",
  size_bytes: 32_000_000_000,
  filesystem: "FAT32",
  available_bytes: 28_000_000_000,
};
```

```rust
// Rust: build real on-disk fixtures in a temp dir
let temp = tempfile::tempdir().unwrap();
let mut f = fs::File::create(temp.path().join("minui.txt")).unwrap();
f.write_all(b"MinUI v2024.12.25").unwrap();
```

**Location:**

- No shared fixtures/factories directory. Test data is inline at the top of each test
  file (`mockDrive` in `src/Home.test.tsx` & `src/DriveSelector.test.tsx`,
  `mockResult` per `it`). Rust builds throwaway files via `tempfile`.

## Coverage

**Requirements:** None enforced. No coverage threshold config in `vitest.config.ts`;
`@vitest/coverage-v8` is installed but no `npm run coverage` script exists.

**View Coverage:**

```bash
npx vitest run --coverage   # frontend (v8 provider)
```

## Test Types

**Unit Tests:**

- Frontend: 8 `src/types/*.test.ts` files = **79 tests** covering pure logic & the
  thin invoke-wrapper API (drive: 8, version: 4, install: 7, release: 12, package: 23,
  validate: 6, device: 4, archive: 15).
- Backend: inline `#[cfg(test)]` modules = **53 Rust tests** across 8 modules
  (download: 3, drives: 3, extract: 6, install: 5, package: 9, validate: 9,
  version: 11, wifi: 7). `lib.rs`/`main.rs` have none.

**Integration Tests:**

- Component tests (`*.test.tsx`) act as light frontend integration tests: they render
  a component, mock the Tauri bridge + sibling API modules, and assert rendered
  output and `invoke` contracts. 4 files = **21 tests** (DriveSelector: 5, Home: 4,
  WifiWizard: 6, PackageStore: 6).
- **Total frontend: 12 files, 100 tests, all passing** (verified via `npx vitest run`,
  2026-06-13).

**E2E Tests:**

- Not used. No Playwright/WebDriver/`tauri-driver` setup present.

## Coverage Gaps

- **Component test coverage is partial.** Of the React components in `src/`, only
  `DriveSelector`, `Home`, `WifiWizard`, and `PackageStore` have tests. The following
  have **no** `*.test.tsx`: `App.tsx`, `DeviceSelector.tsx`, `ConfirmDialog.tsx`,
  `InstallProgress.tsx`, `HealthCheck.tsx`, `ValidationReport.tsx`.
  > NOTE: an earlier project brief stated "zero React component tests" — that is now
  > **out of date**; 4 component test files (21 tests) exist and pass as of this analysis.
- No end-to-end tests exercising the real Rust↔React IPC path; the bridge is always
  mocked on the frontend side.
- No enforced coverage threshold and no coverage npm script.
- `src-tauri/src/lib.rs` Tauri command wrappers are untested (only the underlying
  module fns they delegate to are tested).

## Common Patterns

**Async Testing:**

```typescript
it("downloads archive successfully", async () => {
  const { invoke } = await import("@tauri-apps/api/core");
  vi.mocked(invoke).mockResolvedValue(mockResult);
  const result = await downloadArchive("https://example.com/archive.zip");
  expect(result.success).toBe(true);
});

// Components: assert post-async UI with waitFor
await waitFor(() => {
  expect(
    screen.getByRole("button", { name: "Install MinUI" }),
  ).toBeInTheDocument();
});
```

```rust
// Rust async backend tests use #[tokio::test] (download/install/package modules)
```

**Error Testing:**

```typescript
// Reject the mock and assert the normalized error union + code
vi.mocked(invoke).mockRejectedValue(new Error("scan unavailable"));
// -> result.success === false, result.error.code === "..._ERROR"

// Fallback-path testing (validate.test.ts): invoke fails in test env, so
// formatValidationReport falls back to local formatting, which is asserted.
```

```rust
#[test]
fn test_verify_checksum_failure() {
    let result = verify_checksum(path, "wrong_checksum");
    assert!(!result.unwrap());
}
```

---

_Testing analysis: 2026-06-13_
