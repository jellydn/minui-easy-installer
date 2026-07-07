# Testing

Test framework, structure, mocking, and coverage in `minui-easy-installer`.
Every finding has a file path and line numbers.

## 1. Test Frameworks

| Layer | Framework | Config | Runner |
| --- | --- | --- | --- |
| TypeScript / TSX | **Vitest** `^4.1.8` | `vitest.config.ts` (12 lines) | `bun run test` → `vitest run` (`package.json:11`) |
| TS DOM env | **jsdom** `^29.1.1` | `vitest.config.ts:7` (`environment: "jsdom"`) | per-file `// @vitest-environment jsdom` comment |
| React rendering | **@testing-library/react** `^16.3.2` | imports in `*.test.tsx` | per-test |
| User events | **@testing-library/user-event** `^14.6.1` | `userEvent.click/type` in `Home.test.tsx`, `DriveSelector.test.tsx`, `PackageStore.test.tsx`, `WifiWizard.test.tsx` | per-test |
| Matchers | **@testing-library/jest-dom** `^6.9.1` | `vitest.setup.ts:1` (`import "@testing-library/jest-dom/vitest"`) | global |
| Coverage | **@vitest/coverage-v8** `^4.1.8` | listed in devDeps (`package.json:24`) | `npx vitest run --coverage` (not wired to a script) |
| Rust | **`#[cfg(test)]` + `cargo test`** | in-file `mod tests` blocks in every Rust module | `cargo test` from `src-tauri/` |
| Rust async | **`#[tokio::test]`** | `tokio = { version = "1", features = ["full"] }` (`src-tauri/Cargo.toml:24`) | per-test |

The `AGENTS.md` project doc still says `bun test`
(`AGENTS.md:40`), but `package.json:11` runs `vitest run`. The two
are compatible: `bun test` resolves to the `test` script and runs
Vitest.

## 2. Test File Layout

Test files are **colocated with source**, with the same base name
plus `.test.ts` or `.test.tsx`:

```
src/types/
  archive.ts               archive.test.ts
  device.ts                device.test.ts
  device-install-map.ts    device-install-map.test.ts
  drive.ts                 drive.test.ts
  install.ts               install.test.ts
  package.ts               package.test.ts
  release.ts               release.test.ts
  validate.ts              validate.test.ts
  version.ts               version.test.ts
src/hooks/
  useVersionCheck.ts       useVersionCheck.test.ts
src/
  DriveSelector.tsx        DriveSelector.test.tsx
  Home.tsx                 Home.test.tsx
  PackageStore.tsx         PackageStore.test.tsx
  WifiWizard.tsx           WifiWizard.test.tsx
```

`vitest.config.ts:9` glob: `include: ["src/**/*.test.{ts,tsx}"]`.

**Not tested:** `App.tsx`, `DeviceSelector.tsx`,
`FormatConfirmDialog.tsx`, `ConfirmDialog.tsx`, `HealthCheck.tsx`,
`InstallProgress.tsx`, `PackageCard.tsx`, `ValidationReport.tsx`,
`main.tsx`, `styles.css`, `App.test.tsx`-like top-level integration
tests, `fork.ts`, `errors.ts`, `device-install-map.json` (only
indirectly via the device-install-map test).

**Rust tests live in-file** at the bottom of every `*.rs` module:

```
src-tauri/src/
  lib.rs         #[cfg(test)] mod tests (~250 lines, 19 tests)
  drives.rs      #[cfg(test)] mod tests (~360 lines, 22+ tests, mostly macOS-conditional)
  download.rs    #[cfg(test)] mod tests (3 tests)
  extract.rs     #[cfg(test)] mod tests (4 tests)
  fs_utils.rs    #[cfg(test)] mod tests (8 tests)
  install.rs     #[cfg(test)] mod tests (7 tests)
  package.rs     #[cfg(test)] mod tests (3 tests)
  pipeline.rs    (no test module — tested transitively via install/package tests)
  validate.rs    #[cfg(test)] mod tests (6 tests)
  version.rs     #[cfg(test)] mod tests (~20 tests, 14 visible in grep)
  wifi.rs        #[cfg(test)] mod tests (~12 tests, platform-conditional)
  health.rs      #[cfg(test)] mod tests (3 tests)
```

## 3. Test Counts (per file)

Counts are derived from `grep` of `it\(|test\(`. Frontend tests
sum to **~73** `it`/`test` blocks. Rust tests sum to roughly
**~80** `#[test]` / `#[tokio::test]` blocks (some are
`#[cfg(target_os = "macos")]` or `#[cfg(unix)]`-gated).

| Test file | Count | Notable coverage |
| --- | --- | --- |
| `src/types/archive.test.ts` | 13 | success/checksum-mismatch/network/path-traversal/security/file-error/archive-error/unknown |
| `src/types/install.test.ts` | 7 | success/copy-error/download-error/extraction-error/IPC-exception/zero-extras/extras-with-checksum |
| `src/types/release.test.ts` | 14 | parse-valid/only-base/strip-v/null/missing-tag/no-base/empty-assets + fetch-success/custom-fork/cache-per-fork/clear-cache/404/HTTP-error/network-error/parse-error |
| `src/types/drive.test.ts` | 8 | formatSize variants + getDriveDisplayName with/without size |
| `src/types/package.test.ts` | 1 | loads+converts `store.json` |
| `src/types/version.test.ts` | 4 | shape tests only (no IPC call) |
| `src/types/device.test.ts` | 5 | lookup/unknown/17 devices/required fields/in-sync with JSON map |
| `src/types/device-install-map.test.ts` | 12 | valid structure/all IDs/lookup rules/extras platform mapping for 5 devices/base platform/device paks/empty paks/shared BIOS |
| `src/types/validate.test.ts` | 6 | shape tests + formatReportLocally pass/fail/GB |
| `src/hooks/useVersionCheck.test.ts` | 2 | race-condition guard for `check()` + `reset()` — uses deferred promises to interleave |
| `src/DriveSelector.test.tsx` | 5 | empty state/listing/selected details/error/missing filesystem |
| `src/Home.test.tsx` | 4 | title/install-button/status-summary/confirmation-dialog |
| `src/PackageStore.test.tsx` | 6 | loading/loaded/error/search/no-results/category-filter |
| `src/WifiWizard.test.tsx` | 6 | fields/fallback-manual/scan-success/disabled-save/cancel/writes-config |

The two most thorough files are `archive.test.ts` (13 tests, every
error path) and `release.test.ts` (14 tests, every fetch path and
the per-fork cache).

## 4. Test Structure (Arrange / Act / Assert)

Vitest tests follow a clear **Arrange / Act / Assert** shape, often
with three blank lines between phases:

```ts
// src/types/archive.test.ts:14-37 (paraphrased)
it("downloads archive successfully", async () => {
  // Arrange
  const mockResult: DownloadResult = { success: true, ... };
  const { invoke } = await import("@tauri-apps/api/core");
  (invoke as Mock).mockResolvedValue(mockResult);

  // Act
  const result = await downloadArchive("https://example.com/archive.zip");

  // Assert
  expect(result.success).toBe(true);
  if (result.success) {
    expect(result.data.file_path).toBe("/tmp/test-archive.zip");
  }
  expect(invoke).toHaveBeenCalledWith("download_and_verify_archive", {
    url: "https://example.com/archive.zip",
    checksum: null,
  });
});
```

Conventions observed across the test files:

- **`describe(...)` groups by function or feature**:
  - `describe("downloadArchive", ...)`, `describe("verifyChecksum", ...)`,
    `describe("extractArchive", ...)` in `archive.test.ts:10, 122, 163`.
  - `describe("parseGitHubRelease", ...)` and
    `describe("fetchMinUIRelease", ...)` in `release.test.ts:13, 105`.
  - `describe("useVersionCheck race-condition guard", ...)` in
    `useVersionCheck.test.ts:42`.
  - `describe("getDeviceProfile", ...)` and
    `describe("getAllDeviceProfiles", ...)` in `device.test.ts:5`.
  - `describe("formatReportLocally", ...)` in `validate.test.ts:59`.
- **`it(...)` is the dominant keyword**. `test(...)` is used in
  `package.test.ts`, `version.test.ts`, and `validate.test.ts`
  for shape-only tests.
- **Test names read as sentences**: "downloads archive successfully",
  "handles checksum mismatch", "handles path traversal detection",
  "drops the stale result when a newer check() supersedes it".
- **`beforeEach` for cleanup**:
  ```ts
  beforeEach(async () => {
    vi.clearAllMocks();
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as Mock).mockResolvedValue([]);
  });
  ```
  (`Home.test.tsx:62-66`, `PackageStore.test.tsx:54-56`,
  `WifiWizard.test.tsx:26-28`, `DriveSelector.test.tsx:25-27`.)
- **`afterEach(() => cleanup())`** is used in every component test
  to unmount the rendered tree
  (`Home.test.tsx:58-60`, `DriveSelector.test.tsx:21-23`,
  `PackageStore.test.tsx:50-52`, `WifiWizard.test.tsx:22-24`).
- **Narrowed type assertions after discriminated-union narrowing**:
  ```ts
  expect(result.success).toBe(false);
  if (!result.success) {
    expect(result.error.code).toBe("CHECKSUM_ERROR");
  }
  ```
  Pattern appears in every error-path test in
  `archive.test.ts`, `install.test.ts`, `release.test.ts`.
- **`Mock` typing**: tests use `import { type Mock, vi } from "vitest"`
  and cast mocks as `(invoke as Mock).mockResolvedValue(...)`.

## 5. Mocking Strategy

### 5.1 Frontend: Mocking the Tauri API

The Tauri core is mocked at the module boundary with
`vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }))`:

- `archive.test.ts:6-8`
- `install.test.ts:5-7`
- `validate.test.ts:9-11`
- `Home.test.tsx:8-10`
- `DriveSelector.test.tsx:8-10`
- `WifiWizard.test.tsx:7-9`

Mocks are imported dynamically inside each test:

```ts
const { invoke } = await import("@tauri-apps/api/core");
(invoke as Mock).mockResolvedValue(mockResult);
```

This works because the production code uses
`await import("@tauri-apps/api/core")` (see CONVENTIONS.md §2.7) so
Vitest's module-mock hoisting catches every import site.

`validate.test.ts:9-12` is the canonical example of the "let
`invoke` throw and the production code falls back to a local
implementation" pattern. The test mocks `invoke` to *always reject*
and asserts the local fallback runs:

```ts
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockRejectedValue(new Error("no tauri in test env")),
}));
```

The comment on `validate.test.ts:1-7` documents why:

> `formatValidationReport` dynamically imports `@tauri-apps/api/core`
> and calls `invoke`. In the jsdom test env there is no Tauri runtime,
> so the `invoke` call throws and the function falls back to
> `formatReportLocally`. Mock the module to make that fallback
> deterministic.

### 5.2 Frontend: Mocking Domain Modules

Tests that depend on `types/*` functions mock those modules
**partially** (preserve the real exports, override the ones they
need):

```ts
// Home.test.tsx:12-15
vi.mock("./types/release", () => ({
  fetchMinUIRelease: vi.fn(),
}));
```

```ts
// Home.test.tsx:20-27 — preserves real exports
vi.mock("./types/package", async () => {
  const actual = await import("./types/package");
  return {
    ...actual,
    fetchPackageRegistry: vi.fn(),
    checkPackageUpdates: vi.fn(),
  };
});
```

Same pattern in `PackageStore.test.tsx:8-14`,
`useVersionCheck.test.ts:8-15` (3 different modules), and
`Home.test.tsx:29-35` for `./types/install`.

### 5.3 Frontend: Mocking `fetch`

`fetchMinUIRelease` accepts a `fetchFn` parameter
(`src/types/release.ts:117`) and tests pass a `vi.fn()`:

```ts
// release.test.ts:111-121
const mockFetch = vi.fn().mockResolvedValue({
  ok: true,
  json: () => Promise.resolve({ ... }),
});
const result = await fetchMinUIRelease(OFFICIAL_FORK, mockFetch);
```

This avoids the global `fetch` mock entirely and lets the cache
behavior be tested deterministically (see
`release.test.ts:170-195` "caches per fork and invalidates when
fork changes").

### 5.4 Frontend: Deferred Promises for Race Tests

`useVersionCheck.test.ts:31-38` defines a deferred-promise helper:

```ts
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((r) => { resolve = r; });
  return { promise, resolve };
}
```

Two deferred promises are used to interleave two `check()` calls so
the test can prove that the second call's result commits and the
first call's result is dropped
(`useVersionCheck.test.ts:42-152`). This is the most sophisticated
test in the repo; the comments explain exactly what is being proven
at each `act` boundary.

### 5.5 Frontend: `@testing-library/user-event`

User interactions use `userEvent` (not `fireEvent`):

- `await userEvent.click(screen.getByText("SD_CARD"))`
  (`DriveSelector.test.tsx:53`)
- `await userEvent.type(searchInput, "SSH")`
  (`PackageStore.test.tsx:121`)
- `await userEvent.type(
     screen.getByPlaceholderText("Enter WiFi network name"),
     "MyNetwork",
   )` (`WifiWizard.test.tsx:108-110`)

Queries are mostly `screen.getByRole(...)`,
`screen.getByLabelText(...)`, `screen.getByText(...)`,
`screen.getByPlaceholderText(...)`, and `screen.queryByText(...)` for
absence assertions.

### 5.6 Rust: No Mocking Framework

Rust tests don't use a mocking crate. The pattern is **integration
testing against `tempfile::tempdir()`**:

```rust
// src-tauri/src/install.rs:344-372
#[test]
fn test_copy_dir_recursive_skips_preserved_folders() {
    let temp = tempfile::tempdir().unwrap();
    let src = temp.path().join("src");
    let sd_root = temp.path().join("sdcard");

    fs::create_dir_all(src.join("ROMS")).unwrap();
    fs::create_dir_all(src.join("Saves")).unwrap();
    // ...

    let copied = fs_utils::copy_dir_recursive(
        &src, &sd_root,
        &|_src, dst| is_preserved_path(dst, &sd_root),
        &|| false,
    ).unwrap();
    assert_eq!(copied, 1);
}
```

The convention is to use `tempfile::tempdir()` (which auto-cleans on
drop), seed a realistic layout, call the production function, and
assert on returned values and on-disk state.

For functions that hit the network, the convention is **use an
unreachable URL and assert the error path**:

```rust
// src-tauri/src/lib.rs:379-403
#[tokio::test]
async fn test_install_minui_underlying_errors_on_bad_url() {
    let options = install::InstallOptions {
        base_url: "http://127.0.0.1:1/never-exists.zip".to_string(),
        // ...
    };
    let result = install::install_minui(&options, Arc::new(|_| {})).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(!err.is_empty());
}
```

This is documented in the comment block above the test ("We don't
want to actually download a real archive. Call the underlying
function with an unreachable URL…").

For functions that shell out to `df`/`diskutil`/`airport`/etc., tests
are **platform-gated** with `#[cfg(target_os = "macos")]` (e.g.
`drives.rs:417, 457, 466, …`, `wifi.rs:386, 398, 411, 423`).
CI-independent tests live in the non-gated block; macOS-only
parsing/logic tests are gated.

### 5.7 Rust: Real-Disk Integration Test

`drives.rs:616-696` has one `#[ignore]`-tagged integration test
that asserts `list_removable_drives` finds a real SD card at
`/Volumes/MinUI`. The comment block explicitly documents the
requirement:

> Marked `#[ignore]` because it requires physical hardware (an SD
> card inserted into the Mac) and a specific mount path. To run:
> `cargo test --lib drives -- --ignored list_removable_drives_finds_real_sd_card`

The test gracefully bails if the volume is not currently mounted
(via `eprintln!` + early `return`), making it safe to run on a
developer machine without an SD card inserted.

## 6. Contract Tests for Tauri Command Handlers

A deliberate **contract-test layer** was added to `src-tauri/src/lib.rs`
as part of a recent initiative. The pattern is: for every
`#[tauri::command]` handler in `lib.rs`, write a
`#[cfg(test)] mod tests` block that exercises the **underlying
function** (not the `#[tauri::command]` wrapper itself) to verify
the contract that the IPC layer depends on.

This is documented in the test comments themselves
(`lib.rs:367-411`):

> We don't want to actually download a real archive. Call the
> underlying function with an unreachable URL and assert the error
> propagates as a String (the IPC contract).

Coverage of every command:

| `#[tauri::command]` | Contract test | Notes |
| --- | --- | --- |
| `get_removable_drives` | `test_get_removable_drives_returns_result_shape` (line 347) | Accepts either `Ok` or `Err`; only asserts field-type contract |
| `format_drive` | `test_format_drive_errors_on_nonexistent_mount` (line 367) | `#[cfg(target_os = "macos")]` — would actually format otherwise |
| `install_minui` | `test_install_minui_underlying_errors_on_bad_url` (line 379) | Uses `127.0.0.1:1` |
| `validate_installation` | `test_validate_installation_errors_on_nonexistent_mount` (line 406), `test_validate_installation_on_empty_tempdir` (line 416) | Two tests: error path + happy-but-empty path |
| `format_validation_report` | `test_format_validation_report_contains_pass_and_fail_lines` (line 433) | Asserts both pass and fail entries appear in output |
| `check_minui_version` | `test_check_minui_version_on_empty_tempdir` (line 461) | Asserts `installed.is_none()` and `update_available == true` |
| `install_package` | `test_install_package_underlying_errors_on_bad_url` (line 473) | Uses `127.0.0.1:1` |
| `scan_wifi_networks` | `test_scan_wifi_networks_returns_vec` (line 494) | "Don't assert specific networks (CI-dependent). Just assert it returns a Vec and doesn't panic." |
| `get_current_wifi_ssid` | `test_get_current_wifi_ssid_returns_option_string` (line 501) | "Environment-dependent. Just assert the return type." |
| `detect_installed_packages` | `test_detect_installed_packages_empty_tempdir` (line 513) | |
| `check_package_updates` | `test_check_package_updates_empty_input` (line 522) | |
| `check_sd_card_health` | `test_check_sd_card_health_errors_on_nonexistent` (line 531), `test_check_sd_card_health_on_empty_tempdir` (line 537) | Two tests: error path + happy-but-empty path |
| `fetch_url` | `test_fetch_url_errors_on_unreachable` (line 549) | Replicates the command body inline — comment explains: "The actual Tauri wrapper just plumbs the AppHandle, so the body is the contract." |
| `download_and_verify_archive` | `test_download_and_verify_archive_errors_on_unreachable` (line 565) | |
| `verify_archive_checksum` | `test_verify_archive_checksum_errors_on_missing_file` (line 573), `test_verify_archive_checksum_matches_correct_hash` (line 579) | Uses real `sha256("hello world")` |
| `extract_archive_to_directory` | (no contract test) | Comment in `lib.rs:600-602`: "Already covered by `extract.rs` tests. Contract test in lib.rs would duplicate that work; skip and document." |
| `write_wifi_config` | (no contract test) | Comment in `lib.rs:488-491`: "Already covered by `wifi.rs` tests. Contract test in lib.rs would duplicate that work; skip and document." |
| `start_install` / `cancel_install` | (no contract test) | Not exercised; the cancellation flow is covered by `useVersionCheck.test.ts` on the TS side indirectly. |

The pattern is: every command has **at least one contract assertion
on the underlying function**. The exception is when the underlying
function is already covered in its own module; in that case a
comment explains why the contract test is omitted.

## 7. Coverage Gaps (what is **not** tested)

Things intentionally or unintentionally untested:

- **No tests for** `App.tsx`, `DeviceSelector.tsx`,
  `FormatConfirmDialog.tsx`, `ConfirmDialog.tsx`, `HealthCheck.tsx`,
  `InstallProgress.tsx`, `PackageCard.tsx`, `ValidationReport.tsx`,
  `main.tsx`. (Component-level coverage is partial — the most
  behavior-heavy components are tested, the rest are visual
  primitives.)
- **No E2E tests.** No Playwright, no Spectron, no Tauri WebDriver.
- **No test for `start_install` / `cancel_install` cancellation
  flow** in Rust. The TS-side race-condition test in
  `useVersionCheck.test.ts` covers *one* class of race; the
  `CancellationToken` propagation in
  `src-tauri/src/pipeline.rs:86-138` is only exercised in passing
  via `download_archive_streaming` (which has a cancel check inside
  its chunk loop at `src-tauri/src/download.rs:228-232`).
- **No coverage for `pipeline.rs`** in a dedicated test module.
  `create_target_within` (`pipeline.rs:159-227`) and
  `canonicalize_existing_ancestor` (line 234-249) are tested only
  transitively through `package::install_package`.
- **No snapshot tests.** No `toMatchSnapshot` usage anywhere.
- **No integration test for the `validate_installation` Rust
  function across a populated SD card** — only error and empty
  tempdir cases
  (`src-tauri/src/validate.rs:206, 212, 225, 242, 262, 278`).
- **No test for `format_validation_report`'s Rust implementation
  with free-space formatting** — only one test
  (`src-tauri/src/validate.rs:262`).
- **No test for `check_minui_version` with a populated minui.txt**
  — the inverse (`test_check_for_updates_with_install` at
  `src-tauri/src/version.rs:313`) covers it transitively.
- **`App.test.tsx` does not exist** — there is no top-level
  integration test that mounts `App` and walks the three screens
  (`home` / `store` / `wifi`).
- **`fork.ts` is not directly tested** — only covered indirectly
  through `release.test.ts:144-168` ("fetches from custom fork
  (MinUI-Zero)").

## 8. Running the Tests

- **Frontend:** `bun run test` (alias for `vitest run`).
  Single-file: `bunx vitest run src/types/archive.test.ts`.
  Watch mode: `bunx vitest`.
- **Rust:** `cd src-tauri && cargo test`.
  Ignored-only (real-SD-card test):
  `cargo test --lib drives -- --ignored list_removable_drives_finds_real_sd_card`.
- **All checks together:** `just check` runs
  `bun run lint && bun run typecheck && cd src-tauri && cargo fmt --check && cd src-tauri && cargo clippy -- -D warnings`
  (`justfile:34-37`).
- **CI:** `.github/workflows/react-doctor.yml` is **advisory only**
  — it posts sticky PR comments and review comments; it does not
  block. There is no CI workflow that runs `vitest` or `cargo test`.

## 9. Conventions Summary

| Decision | Convention |
| --- | --- |
| Test framework | Vitest + jsdom (`vitest.config.ts:7-11`) |
| Test file naming | `<source>.test.{ts,tsx}` colocated with source |
| Test discovery glob | `src/**/*.test.{ts,tsx}` (`vitest.config.ts:9`) |
| TS assertion style | `expect(...).toBe/toEqual/toHaveBeenCalledWith/toMatch` |
| TS error assertion | Narrow discriminated union then assert on `error.code` |
| Mocking the Tauri API | `vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }))` |
| Mocking domain modules | Partial mocks that spread `...actual` and override selected exports |
| User events | `@testing-library/user-event` v14 |
| React cleanup | `afterEach(() => cleanup())` |
| Deferred promises for races | Local `deferred<T>()` helper (see `useVersionCheck.test.ts:31-38`) |
| Rust test framework | `#[cfg(test)] mod tests` block at the bottom of every module |
| Rust temp dirs | `tempfile::tempdir()` — auto-cleaned on drop |
| Rust network test | Use `http://127.0.0.1:1/never-exists` to force error path |
| Rust platform-conditional tests | `#[cfg(target_os = "macos")]` (and `#[cfg(unix)]`) gates |
| Rust real-hardware tests | `#[ignore = "requires a real SD card mounted at the configured path"]` |
| Test names | Full English sentences describing behavior |
