# Testing Patterns

**Analysis Date:** 2026-06-14

## Test Framework

**Runner:**

- Vitest 4.1.8 (frontend TypeScript tests)
- Config: `vitest.config.ts`
- Environment: `jsdom` (simulates browser DOM for React component testing)
- Setup file: `vitest.setup.ts` — imports `@testing-library/jest-dom/vitest` for DOM matchers

**Assertion Library:**

- Vitest built-in `expect` (compatible with Jest API)
- Extended with `@testing-library/jest-dom` matchers — `toBeInTheDocument()`, `toBeDisabled()`, `toHaveAttribute()`, etc.

**Run Commands:**

```bash
bun test                    # Run all tests once (vitest run)
vitest                      # Watch mode
bun run test                # Same as vitest run
```

## Test File Organization

**Location:**

- Co-located with source files — test files live alongside the modules they test
- Frontend: `src/types/*.test.ts` for type/utility modules, `src/*.test.tsx` for React components
- Backend: `#[cfg(test)] mod tests` blocks at the bottom of each `.rs` source file

**Naming:**

- TypeScript: `<module-name>.test.ts` or `<module-name>.test.tsx`
- Rust: inline `mod tests` with `test_` prefixed functions

**Structure:**

```
src/
├── types/
│   ├── archive.ts              # Source
│   ├── archive.test.ts         # Tests (9 .test.ts files)
│   ├── drive.ts
│   ├── drive.test.ts
│   ├── install.ts
│   ├── install.test.ts
│   ├── package.ts
│   ├── package.test.ts
│   ├── release.ts
│   ├── release.test.ts
│   ├── version.ts
│   ├── version.test.ts
│   ├── validate.ts
│   ├── validate.test.ts
│   ├── device.ts
│   ├── device.test.ts
│   └── device-install-map.test.ts
├── DriveSelector.tsx           # Component source
├── DriveSelector.test.tsx      # Component test (4 .test.tsx files)
├── Home.tsx
├── Home.test.tsx
├── PackageStore.test.tsx
└── WifiWizard.test.tsx
src-tauri/src/
├── download.rs                 # Rust module with inline tests
├── extract.rs                  # Rust module with inline tests
├── install.rs                  # Rust module with inline tests
├── version.rs                  # Rust module with inline tests
├── drives.rs                   # Rust module with inline tests
├── wifi.rs                     # Rust module with inline tests
├── validate.rs                 # Rust module with inline tests
└── package.rs                  # Rust module with inline tests
```

## Test Structure

**Suite Organization:**

```typescript
import { describe, expect, it, vi } from "vitest";

describe("parseGitHubRelease", () => {
	it("parses a valid GitHub release with base and extras", () => {
		// Arrange
		const input = { tag_name: "v25.06.12", assets: [...] };

		// Act
		const result = parseGitHubRelease(input) as MinUIRelease;

		// Assert
		expect(result).toEqual({
			version: "25.06.12",
			baseArchiveUrl: expect.stringContaining("base.zip"),
			extrasArchiveUrl: expect.stringContaining("extras.zip"),
			checksums: null,
		});
	});
});
```

**Patterns:**

- Top-level `describe()` groups tests by function or component name
- Nested `describe()` for sub-groups when testing multiple related functions (e.g., `describe("downloadArchive")`, `describe("verifyChecksum")`, `describe("extractArchive")` in `archive.test.ts`)
- `it()` for individual test cases with descriptive names
- `test()` used interchangeably with `it()` (both appear in the codebase)
- `beforeEach(() => vi.clearAllMocks())` for resetting mock state between tests
- `afterEach(() => cleanup())` for React component tests to prevent DOM leaks
- No global `beforeAll` or `afterAll` usage observed

## Mocking

**Framework:** Vitest built-in `vi.mock()` and `vi.fn()`

**Patterns:**

```typescript
// Mock Tauri invoke (most common — appears in every Tauri-connected test file)
vi.mock("@tauri-apps/api/core", () => ({
	invoke: vi.fn(),
}));

// Mock entire modules (for Home.test.tsx — multiple module mocks)
vi.mock("./types/release", () => ({
	fetchMinUIRelease: vi.fn(),
}));
vi.mock("./types/package", async (importOriginal) => {
	const actual = await importOriginal<typeof import("./types/package")>();
	return {
		...actual,                    // Preserve non-mocked exports
		fetchPackageRegistry: vi.fn(),
		checkPackageUpdates: vi.fn(),
	};
});

// Mock with dynamic import for type modules
const { invoke } = await import("@tauri-apps/api/core");
vi.mocked(invoke).mockResolvedValue(mockResult);

// Mock fetch function parameter (dependency injection pattern)
export async function fetchMinUIRelease(
    fetchFn: typeof globalThis.fetch = globalThis.fetch,
): Promise<ReleaseFetchResult> { ... }
// In tests: fetchMinUIRelease(mockFetch)

// Helper function to mock invoke with command routing (WifiWizard.test.tsx)
function mockInvoke(
    invoke: ReturnType<typeof vi.fn>,
    overrides: Record<string, unknown> = {},
) {
    invoke.mockImplementation((cmd: string) => {
        if (cmd === "get_current_wifi_ssid") return Promise.resolve(null);
        if (cmd in overrides) return Promise.resolve(overrides[cmd]);
        return Promise.resolve([]);
    });
}
```

**What to Mock:**

- `@tauri-apps/api/core` — always mocked (no Tauri runtime in test environment)
- Backend-bound type modules (`./types/release`, `./types/version`, `./types/package`, `./types/install`, `./types/validate`) — mocked in component tests to isolate UI logic
- `globalThis.fetch` — injected as parameter (not mocked globally) in `fetchMinUIRelease()`

**What NOT to Mock:**

- Pure utility functions (`formatSize`, `getDriveDisplayName`, `classifyError`) — tested directly
- Type-only modules (`./types/drive`, `./types/version`) — type shape tests run without mocks
- React hooks from `react` — never mocked

## Fixtures and Factories

**Test Data:**

```typescript
// Inline mock objects (most common pattern)
const mockDrive: RemovableDrive = {
  name: "SD_CARD",
  mount_path: "/Volumes/SD_CARD",
  size_bytes: 32_000_000_000,
  filesystem: "FAT32",
  available_bytes: 28_000_000_000,
};

// Inline mock result objects
const mockResult: InstallResult = {
  success: true,
  error: null,
  base_files_copied: 15,
  extras_files_copied: 3,
  extras_warning: null,
  rom_dirs_created: 0,
};

// Mock registry data (PackageStore.test.tsx)
const mockRegistry: PackageRegistry = {
  version: "1.0",
  packages: [
    {
      name: "Wifi.pak",
      version: "1.0.0",
      category: "Emulators",
      // ...
    },
  ],
};

// GitHub release API mock data (inline in test)
const mockFetch = vi.fn().mockResolvedValue({
  ok: true,
  json: () =>
    Promise.resolve({
      tag_name: "v25.06.12",
      assets: [{ browser_download_url: "..." }],
    }),
});
```

**Location:**

- No separate fixture files — all test data is defined inline within test functions or at `describe` scope
- Mock objects are typed with their corresponding interface (`RemovableDrive`, `InstallResult`, `PackageRegistry`, etc.)

## Coverage

**Requirements:** None enforced (no coverage threshold in `vitest.config.ts`)

**Dependencies:** `@vitest/coverage-v8` is installed as a devDependency

**View Coverage:**

```bash
npx vitest --coverage        # Generate coverage report
```

## Test Types

**Unit Tests:**

- TypeScript: 9 type/utility test files covering pure functions and type shapes (`drive.test.ts`, `install.test.ts`, `release.test.ts`, `version.test.ts`, `package.test.ts`, `validate.test.ts`, `archive.test.ts`, `device.test.ts`, `device-install-map.test.ts`)
- Focus on: parsing logic, error classification, type validation, function return shapes, edge cases (null inputs, empty arrays, missing data)
- No network calls — fetch functions use dependency injection or mocked modules

**Integration Tests:**

- React component tests: 4 files (`DriveSelector.test.tsx`, `Home.test.tsx`, `PackageStore.test.tsx`, `WifiWizard.test.tsx`)
- Test component rendering, user interactions (`userEvent.click`, `userEvent.type`), async state updates (`waitFor`), and Tauri IPC flow (mocked)
- Verify UI states: loading, error, empty, populated, confirmation dialogs

**E2E Tests:** Not used. No Playwright, Cypress, or other E2E framework observed.

**Rust Tests:**

- 8 Rust modules with `#[cfg(test)] mod tests` blocks
- Total Rust test functions: ~40 tests across `download.rs` (3), `extract.rs` (5), `install.rs` (4), `version.rs` (9), `drives.rs` (3), `wifi.rs` (9), `validate.rs` (7), `package.rs` (5)
- Use `tempfile::tempdir()` for filesystem isolation
- Test real file I/O: create archives, write files, verify checksums, detect versions
- Platform-specific tests: `#[cfg(target_os = "macos")]` and `#[cfg(target_os = "windows")]` for OS-dependent parsing

## Common Patterns

**Async Testing:**

```typescript
// waitFor for async state updates in React components
it("displays package cards after loading", async () => {
	const { fetchPackageRegistry } = await import("./types/package");
	vi.mocked(fetchPackageRegistry).mockResolvedValue({
		success: true,
		data: mockRegistry,
	});

	render(<PackageStore selectedDevice="miyoo-mini-plus" selectedDrive="/Volumes/SD" />);

	await waitFor(() => {
		expect(screen.getByText("Wifi.pak")).toBeInTheDocument();
	});
});

// Async test functions for mocked modules
it("returns success with file counts on successful install", async () => {
	const { invoke } = await import("@tauri-apps/api/core");
	vi.mocked(invoke).mockResolvedValue(mockResult);
	const result = await installMinui({ ... });
});
```

**Error Testing:**

```typescript
// Testing error responses from mocked IPC
it("returns error with copy code on failed install", async () => {
	const mockResult: InstallResult = {
		success: false,
		error: "Failed to copy file to SD card",
		// ...
	};
	const { invoke } = await import("@tauri-apps/api/core");
	vi.mocked(invoke).mockResolvedValue(mockResult);

	const result = await installMinui({ ... });

	expect(result.success).toBe(false);
	if (!result.success) {
		expect(result.error.code).toBe("COPY_ERROR");
		expect(result.error.message).toContain("copy");
	}
});

// Testing component error states
it("shows error state with retry on fetch failure", async () => {
	vi.mocked(fetchPackageRegistry).mockResolvedValue({
		success: false,
		error: { message: "Network error", code: "NETWORK_ERROR" },
	});
	render(<PackageStore ... />);
	await waitFor(() => {
		expect(screen.getByText(/Failed to load packages/)).toBeInTheDocument();
	});
	expect(screen.getByRole("button", { name: "Retry" })).toBeInTheDocument();
});
```

**Rust Test Pattern:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_verify_checksum_success() {
        // Arrange: create temp file with known content
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        write!(temp_file, "test content").unwrap();

        // Act + Assert
        let expected = "6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72";
        let result = verify_checksum(temp_file.path().to_str().unwrap(), expected);
        assert!(result.unwrap());
    }
}
```
