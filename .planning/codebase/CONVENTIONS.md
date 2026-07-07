# Conventions

Code style, naming, patterns, and error-handling conventions observed in
`minui-easy-installer`. Every finding has a file path and line numbers.

## 1. Tooling Chain

### 1.1 Formatting & Linting (Three Layers)

The repo runs three formatters/linters, each with a clearly bounded scope.
They are stacked, not alternatives.

| Layer | Tool | Scope | Config |
| --- | --- | --- | --- |
| Editor-level | EditorConfig | All files | `.editorconfig` |
| TS/TSX formatter | `oxfmt` | `src/` | `.oxfmtrc.json` |
| TS/TSX linter | `oxlint` | `src/` (default script) | invoked via `bun run lint` (`package.json:11`) |
| TS/TSX linter (fallback) | ESLint + `@typescript-eslint` | `src/` | `.eslintrc.cjs` |
| Rust formatter | `rustfmt` | `src-tauri/src/` | `just fmt` → `cargo fmt` |
| Rust linter | `clippy` (`-D warnings`) | `src-tauri/src/` | `just check` |

Config files:
- `.oxfmtrc.json` (9 lines) — `useTabs: false`, `tabWidth: 2`, `endOfLine: "lf"`, `singleQuote: false`, `printWidth: 80`, `ignorePatterns: []`
- `.editorconfig` (72 lines) — 2-space indent, LF, UTF-8, final newline, trim trailing whitespace; 4-space for `*.rs`; hard tabs for `Makefile` and `justfile`; unset for lockfiles
- `.eslintrc.cjs` (8 lines) — extends `eslint:recommended` and `plugin:@typescript-eslint/recommended`; plugins `@typescript-eslint`; ignores `dist` and `.eslintrc.cjs`
- `prek.toml` (33 lines) — pre-commit hooks order: built-in checks (`trailing-whitespace`, `end-of-file-fixer`, `check-added-large-files`, `check-yaml`, `check-json` excludes `*.jsonc`, `mixed-line-ending --fix=lf`, `check-merge-conflict`, `check-case-conflict`), then local `bun run lint -- --fix` and `bun run typecheck`

### 1.2 Quotes and Punctuation

TS/TSX source uses **double quotes** throughout (matches `.oxfmtrc.json:5`
`"singleQuote": false`). No single quotes in source. The `eslint`
`@typescript-eslint/recommended` config is consistent with this.

### 1.3 Indent & Newlines

- TS/TSX: 2-space indent
- Rust: 4-space indent (`.editorconfig:42-44`)
- LF line endings enforced by `prek.toml:24` (`mixed-line-ending --fix=lf`)
- Final newline enforced (`.editorconfig:11` and prek `end-of-file-fixer`)
- Trailing whitespace trimmed (`.editorconfig:12` and prek
  `trailing-whitespace`), except `.md`/`.mdx` (`*.md:18-22`)

## 2. TypeScript Conventions

### 2.1 Compiler Settings

`tsconfig.json` (22 lines): `target: "ES2020"`, `module: "ESNext"`,
`moduleResolution: "bundler"`, `strict: true`, plus three
strict-additional flags:

- `noUnusedLocals: true` (line 17)
- `noUnusedParameters: true` (line 18)
- `noFallthroughCasesInSwitch: true` (line 19)
- `forceConsistentCasingInFileNames: true` (line 20)
- `isolatedModules: true` (line 12), `moduleDetection: "force"` (line 13)

JSX: `react-jsx` runtime (`tsconfig.json:14`) — no `import React` required
in component files. The one exception is `src/main.tsx:1` which
`import React from "react"` only because it uses the named export
`React.StrictMode`.

### 2.2 Module System

ESM with `.ts`/`.tsx` extensions enabled (`allowImportingTsExtensions: true`,
`tsconfig.json:10`). JSON imports work as ES modules:

- `import storeData from "./store.json"` in `src/types/package.ts:2`
- `import deviceInstallMap from "./device-install-map.json"` in
  `src/types/device-install-map.ts:1`

JSON module types are declared in `src/vitest.d.ts` (2 lines) which
references `@testing-library/jest-dom`.

### 2.3 File Layout

```
src/
  App.tsx                    screen router
  main.tsx                   React root, StrictMode
  Home.tsx                   main install flow
  DeviceSelector.tsx         device picker
  DriveSelector.tsx          SD card picker + format
  FormatConfirmDialog.tsx    destructive-action modal
  ConfirmDialog.tsx          write-confirmation modal
  HealthCheck.tsx            SD card health panel
  InstallProgress.tsx        install progress UI
  PackageStore.tsx           package registry grid
  PackageCard.tsx            single package card
  ValidationReport.tsx       post-install validation UI
  WifiWizard.tsx             wifi config form
  styles.css                 single global stylesheet
  vitest.d.ts                ambient types
  hooks/
    useMountEffect.ts        mount-only effect wrapper
    useScrollToBottom.ts     auto-scroll ref helper
    useVersionCheck.ts       version check hook with race guard
  types/                     data types, schemas, and IPC wrappers
    archive.ts / archive.test.ts
    device.ts / device.test.ts
    device-install-map.ts / device-install-map.test.ts / .json
    drive.ts / drive.test.ts
    errors.ts
    fork.ts                  fork (MinUI/MinUI-Zero) config
    install.ts / install.test.ts
    package.ts / package.test.ts / store.json
    release.ts / release.test.ts
    validate.ts / validate.test.ts
    version.ts / version.test.ts
```

Tests are **colocated with source**, with the matching name plus
`.test.ts`/`.test.tsx`. Test files live next to the file they cover.

### 2.4 Naming

| Element | Convention | Example |
| --- | --- | --- |
| Types/Interfaces | `PascalCase` | `DeviceProfile`, `MinUIRelease`, `PackageRegistryEntry` (`src/types/device.ts:1-7`, `src/types/release.ts:3-7`, `src/types/package.ts:5-9`) |
| Type unions (string literals) | `PascalCase` | `InstallPhase`, `InstallStatus`, `AppErrorCode` (`src/types/install.ts:30, 35, src/types/errors.ts:2-8`) |
| Functions | `camelCase` | `getDeviceProfile`, `parseGitHubRelease`, `classifyError` (`src/types/device.ts:148`, `src/types/release.ts:38`, `src/types/errors.ts:14`) |
| Constants | `UPPER_SNAKE_CASE` | `DEFAULT_INSTALL_PATH_RULES`, `ROM_DIRS`, `PRESERVED_FOLDERS`, `STEP_ICON`, `ALL_CATEGORIES`, `PHASE_LABELS`, `REGISTRY_URL`, `SHARED_BIOS`, `ESSENTIAL_BASE_PATHS` |
| Enum-ish string maps | `Record<string, T>` | `PHASE_LABELS: Record<InstallPhase, string>` (`src/InstallProgress.tsx:25`), `STEP_ICON: Record<string, string>` (`src/InstallProgress.tsx:117`) |
| React component files | `PascalCase.tsx` | `Home.tsx`, `InstallProgress.tsx`, `PackageCard.tsx` |
| Hooks | `useCamelCase.ts` | `useMountEffect.ts`, `useScrollToBottom.ts`, `useVersionCheck.ts` |
| JSON data files | `kebab-case.json` | `device-install-map.json`, `store.json` |
| Rust types | `PascalCase` | `RemovableDrive`, `InstallResult`, `Pipeline` |
| Rust functions | `snake_case` | `list_removable_drives`, `install_minui`, `is_path_traversal` |
| Rust constants | `UPPER_SNAKE_CASE` | `ROM_DIRS`, `PRESERVED_FOLDERS`, `ESSENTIAL_BASE_PATHS` |
| Tauri command names | `snake_case` matching arg order | `get_removable_drives`, `install_minui`, `write_wifi_config` |

The `useMountEffect` hook comment block (`src/hooks/useMountEffect.ts:3-7`)
explicitly explains the naming:

> Run an effect only on mount. Use for one-time external sync:
> DOM integration, third-party widget init, browser API subscriptions.

### 2.5 React Patterns

- **Function components only**, no class components. Every `*.tsx` file
  in `src/` exports a default function component.
- **Default exports** for components (`export default Home`,
  `export default DriveSelector`, etc.), **named exports** for hooks and
  pure helpers (`export function useMountEffect`,
  `export function getDeviceProfile`).
- **Controlled inputs** for forms; e.g. `WifiWizard.tsx:12-13` uses
  `useState` for `ssid` and `password`, and the inputs read `value` from
  state and call `setSsid`/`setPassword` in `onChange`.
- **Custom hooks for shared state and effects**:
  `useVersionCheck` (race-safe with `requestIdRef` —
  `src/hooks/useVersionCheck.ts:26-72`),
  `useScrollToBottom` (`src/hooks/useScrollToBottom.ts:12-17`),
  `useMountEffect` (`src/hooks/useMountEffect.ts:11-13`).
- **Module-scope constants for stable references**: `INITIAL_INSTALL_STATE`
  is hoisted out of `Home` in `src/Home.tsx:43-54` with a comment
  explaining why ("Hoisted to module scope so the object reference is
  stable across renders").
- **Event-driven data fetching over effects**: `Home.tsx:69-76` calls
  `version.check(drive.mount_path)` from an effect tied to
  `selectedDrive`, but the *actual* network calls are inside the
  imperative `check()` returned by the hook (see
  `src/hooks/useVersionCheck.ts:16-22` — the comment says "encapsulates
  the version-check data fetch that was previously a useEffect in
  Home.tsx…converting the effect into an event-driven pattern").
- **Refs for "latest" reads inside async callbacks**: see
  `installStatesRef` in `src/PackageStore.tsx:40-41` which holds
  `installStates` so `handleInstallAll` can read the current map
  without depending on it.
- **Memoized derivations** with `useMemo`:
  `filteredPackages` (`src/PackageStore.tsx:62-88`) and `installCounts`
  (`src/PackageStore.tsx:141-148`).
- **Stable callbacks with `useCallback`** on every handler that goes to
  a child component or into a `useEffect` dep list, e.g. `Home.tsx:85,
  209, 229`.
- **`useEffect` deps exhaustive**; when a ref is used to silence deps
  the comment explains it, e.g.
  `src/hooks/useMountEffect.ts:11-12` carries an
  `eslint-disable-next-line react-hooks/exhaustive-deps` comment.

### 2.6 Type Modeling

- **Discriminated unions** for fallible operations. Pattern
  `{ success: true; data: T } | { success: false; error: E }` is used
  pervasively:
  - `DownloadResultEither`, `ExtractionResultEither`
    (`src/types/archive.ts:17, 35`)
  - `InstallResultEither` (`src/types/install.ts:21-23`)
  - `ReleaseFetchResult` (`src/types/release.ts:19-22`)
  - `PackageRegistryFetchResult` (`src/types/package.ts:114-116`)
  - `VersionCheckResultEither` (`src/types/version.ts:17-19`)
  - `ValidationResultEither` (`src/types/validate.ts:24-26`)
- **Tagged error codes** for downstream routing:
  `AppErrorCode` (`src/types/errors.ts:2-8`) and
  `DownloadError.code`, `ExtractionError.code`
  (`src/types/archive.ts:8-12, 27-31`).
- **Type aliases over interfaces** for unions:
  `type InstallPhase = "idle" | "downloading" | "..."`
  (`src/types/install.ts:25-32`).
- **Nullable booleans** to distinguish "absent vs false" in IPC
  payloads: `checksum_verified: boolean | null` and
  `success: boolean` paired with `error: string | null`
  (`src/types/archive.ts:3-7, 19-23`).
- **Rust serde structs** mirror TS types 1:1, using
  `#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]`.
  Example: `RemovableDrive` (`src-tauri/src/drives.rs:5-12`),
  `InstallResult` (`src-tauri/src/install.rs:7-15`),
  `HealthCheckResult` (`src-tauri/src/health.rs:6-14`).
- **Field names are snake_case on the Rust side** and match the
  TS camelCase field names because Tauri v2 serializes Rust struct
  fields as-is — the convention is to use snake_case on both sides
  (e.g. `base_files_copied` in
  `src-tauri/src/install.rs:11` ↔ `base_files_copied` in
  `src/types/install.ts:10`).
- **`#[serde(rename_all = "...")]` is not used** — fields are named
  explicitly. Most types that cross the IPC boundary use
  snake_case fields (e.g. `mount_path` in `src-tauri/src/drives.rs:8` ↔
  `mount_path` in `src/types/drive.ts:4`).

### 2.7 Dynamic Tauri Imports

Every IPC wrapper uses a dynamic `import("@tauri-apps/api/core")` rather
than a top-level import. Pattern:

```ts
try {
  const { invoke } = await import("@tauri-apps/api/core");
  const result = await invoke<T>("command_name", { ... });
  // ...
} catch (err) {
  const message = err instanceof Error ? err.message : "Unknown error";
  return { success: false, error: { message, code: "UNKNOWN_ERROR" } };
}
```

Seen in `src/types/archive.ts:36-72` (3 wrappers),
`src/types/install.ts:55-94, 100-118, 124-127`,
`src/types/package.ts:54-94, 96-115, 117-134`,
`src/types/version.ts:23-37`,
`src/types/validate.ts:38-56, 58-99`.

Reason: the dynamic import keeps `@tauri-apps/api/core` out of the
import graph during jsdom tests (see
`src/types/validate.test.ts:1-10` — `formatValidationReport` is mocked
via `vi.mock("@tauri-apps/api/core", ...)` and the production
function falls back to `formatReportLocally` when `invoke` throws).

`src/DriveSelector.tsx:1`, `src/WifiWizard.tsx:1`, and `src/PackageCard.tsx`
instead use a **static** `import { invoke } from "@tauri-apps/api/core"`
because the components are not unit-tested (only the page-level
behavior is exercised via `Home.test.tsx`).

## 3. Rust Conventions

### 3.1 Module Layout (`src-tauri/src/`)

```
lib.rs          #[tauri::command] surface, InstallRegistry, top-level tests
main.rs         3-line entry: calls `minui_easy_installer_lib::run()`
drives.rs       removable drive enumeration + format
download.rs     HTTP download + SHA-256 verification
extract.rs      ZIP extraction with path-traversal guard
fs_utils.rs     copy_dir_recursive, get_disk_space (libc statvfs)
install.rs      install pipeline + file-counting + preserved-folder filter
package.rs      package install + detection + update check
pipeline.rs     Pipeline::run / run_to_extracted; create_target_within (path-traversal-safe mkdir)
validate.rs     post-install checks + report formatter
version.rs      installed-version detect + semver compare
wifi.rs         wifi.txt writer + cross-platform SSID scan
health.rs       SD card health check + support-report builder
```

`mod` declarations: `lib.rs:11-22` (download, drives, extract,
fs_utils, health, install, package, pipeline, validate, version, wifi).

### 3.2 Result Convention

Every fallible function in the Rust backend returns
`Result<T, String>`. The `String` error is formatted with
`format!("…")` and embeds the underlying error:

- `drives::list_removable_drives() -> Result<Vec<RemovableDrive>, String>`
  (`src-tauri/src/drives.rs:17, 314, 377`)
- `drives::format_drive(...) -> Result<(), String>`
  (`src-tauri/src/drives.rs:189, 261, 266`)
- `download::verify_checksum(...) -> Result<bool, String>`
  (`src-tauri/src/download.rs:18`)
- `download::download_archive(...) -> Result<(DownloadResult, TempDir), String>`
  (`src-tauri/src/download.rs:41`)
- `extract::extract_archive(...) -> Result<(ExtractionResult, Option<TempDir>), String>`
  (`src-tauri/src/extract.rs:60`)
- `install::install_minui(...) -> Result<InstallResult, String>`
  (`src-tauri/src/install.rs:207`)
- `wifi::write_wifi_config(...) -> Result<(), String>`
  (`src-tauri/src/wifi.rs:18`)
- `validate::validate_installation(...) -> Result<ValidationResult, String>`
  (`src-tauri/src/validate.rs:147`)
- `pipeline::create_target_within(...) -> Result<PathBuf, String>`
  (`src-tauri/src/pipeline.rs:174`)

`#[tauri::command]` handlers all forward these `Result<_, String>`
through directly, no `?`-to-`Error`-type conversion. The convention is
documented in the contract tests (`src-tauri/src/lib.rs:367-411`) which
assert `result.is_err()` against unreachable inputs and check the
underlying `String` is non-empty.

### 3.3 Tauri Command Patterns

- **Every IPC command is annotated `#[tauri::command]`** and is
  declared at the top of `src-tauri/src/lib.rs` (handlers
  `get_removable_drives`, `format_drive`,
  `download_and_verify_archive`, `verify_archive_checksum`,
  `extract_archive_to_directory`, `install_minui`, `start_install`,
  `cancel_install`, `validate_installation`,
  `format_validation_report`, `check_minui_version`, `install_package`,
  `write_wifi_config`, `scan_wifi_networks`, `get_current_wifi_ssid`,
  `detect_installed_packages`, `check_package_updates`,
  `check_sd_card_health`, `fetch_url`).
- **`#[allow(clippy::too_many_arguments)]`** is applied to
  `install_minui` (`src-tauri/src/lib.rs:62`) — used to suppress
  warnings on commands with many IPC fields rather than refactoring
  the IPC contract.
- **All commands are `async`** except `verify_archive_checksum`
  (`src-tauri/src/lib.rs:46`), `format_validation_report` (line 208),
  and `cancel_install` (line 191) which are sync.
- **Long-running commands use `tokio::spawn` + `AppHandle.emit`**:
  `start_install` (lines 115-186) spawns the install, then emits
  `install-complete` or `install-error` events back to the frontend.
  The frontend listens via
  `listen<InstallProgressEvent>("install-progress", ...)` in
  `src/Home.tsx:101-117`.
- **Cancellation pattern**: `InstallRegistry` holds
  `Mutex<Option<CancellationToken>>` (`src-tauri/src/lib.rs:77-93`).
  `start_install` cancels any prior token before installing a new one
  (lines 122-128, with the comment "Cancel any prior install before
  replacing"). `cancel_install` flips the flag (lines 191-196).
- **Progress events are emitted through `Arc<dyn Fn>` callbacks**:
  `install::ProgressCallback` is `Arc<dyn Fn(InstallProgressEvent) +
  Send + Sync>` (`src-tauri/src/install.rs:23`). The command wires
  this up to `app_handle.emit("install-progress", event)` in
  `src-tauri/src/lib.rs:69-75`.
- **Pipeline structure**: `Pipeline::run` and `Pipeline::run_to_extracted`
  (`src-tauri/src/pipeline.rs:76-138`) are the only entry points for
  download+extract+copy. `InstallSession` (`src-tauri/src/pipeline.rs:25-50`)
  owns `TempDir` slots for `base`, `extras`, and `package` archives and
  extracted dirs, keeping them alive until the session drops.

### 3.4 Path-Safety Conventions

Two layers of path-traversal defense, both with comments explaining
the design:

1. **`extract::is_path_traversal`** (`src-tauri/src/extract.rs:9-12`)
   rejects `..`, leading `/` and leading `\` in ZIP entry names.
2. **`pipeline::create_target_within`** (`src-tauri/src/pipeline.rs:159-227`)
   validates that the target dir resolves inside the SD card
   *before* calling `create_dir_all`, then re-validates the
   canonical path *after* creation to catch symlink-race attacks.
   The function uses `canonicalize_existing_ancestor` (line 234-249)
   to walk up to the first existing ancestor because on a fresh
   install the parent directory tree may not exist yet.

### 3.5 Error-Handling Style

- **`format!` is the default error message style**:
  - `"Failed to read ZIP archive: invalid zip file"`
    (`src-tauri/src/extract.rs:54`)
  - `"Failed to copy {} to {}: {}", src, dst, e`
    (`src-tauri/src/fs_utils.rs:77-83`)
  - `"Security violation: target escapes SD card: {}", target.display()`
    (`src-tauri/src/pipeline.rs:196-199`)
- **`eprintln!` for non-fatal warnings** that should not stop the
  install:
  - Missing Portmaster placeholder
    (`src-tauri/src/install.rs:78-82`)
  - Failed to set Unix file permissions
    (`src-tauri/src/extract.rs:144-150`)
  - Failed to emit install progress event
    (`src-tauri/src/lib.rs:71-74`)
  - Symlink cleanup failure
    (`src-tauri/src/pipeline.rs:215-220`)
- **`unwrap_or` for `eprintln`-and-continue patterns**, e.g.
  `let rom_dirs_created = create_rom_dirs(&options.sd_mount).unwrap_or(0);`
  (`src-tauri/src/install.rs:283`).
- **`if let Ok(...)` to silently ignore errors in best-effort paths**:
  `src/types/package.ts:131-133` returns `[]` from
  `detect_installed_packages` when the Tauri call throws.
- **Explicit `Result::unwrap_err()`** in tests to extract the error
  string and assert on it
  (`src/types/version.test.ts:62`, `src-tauri/src/wifi.rs:300-303`).

## 4. Cross-Cutting Patterns

### 4.1 Stable References & React Performance

Hoisting initial state to module scope is documented as deliberate:

```ts
// src/Home.tsx:43-54
const INITIAL_INSTALL_STATE: InstallState = {
  phase: "idle",
  message: "",
  // ...
};
```

The comment explains the consequence: "Was previously declared inside
`Home` and reallocated on every render, which would defeat
`React.memo` on any child receiving it as a prop."

### 4.2 Imperative Hook API

`useVersionCheck` returns `{ ...state, check, reset }` and the
component calls `version.check(drive.mount_path)` from a
`useEffect` triggered by `selectedDrive` (`src/Home.tsx:69-76`). The
hook itself is the imperative API; the effect just dispatches. Inside
the hook, a `requestIdRef` is used to drop stale results from
superseded `check()` calls (`src/hooks/useVersionCheck.ts:26-72`).

### 4.3 Validation as Schema

`src/types/package.ts:225-260` defines `validateStoreEntry` and
`validateDeviceInstallMap` (`src/types/device-install-map.ts:56-66`).
The test files assert the validation contract:
`device-install-map.test.ts:13-17` checks `result.valid === true` and
`device.test.ts:36-77` cross-checks `device.ts` against
`device-install-map.json` to keep the two sources in sync.

### 4.4 Documentation in Code

The codebase is **comment-rich**. Examples:

- `src/Home.tsx:43-54` — explains why `INITIAL_INSTALL_STATE` is at
  module scope.
- `src/Home.tsx:295-303` — explains why a `Map` is built for O(1)
  lookups and why `Promise.all` is used over sequential awaits
  ("With sequential awaits, total wall time is the sum of all
  installs; with Promise.all, it's the slowest single install.").
- `src/hooks/useVersionCheck.ts:16-22` — explains the
  effect-to-event migration.
- `src-tauri/src/pipeline.rs:159-227` — extensive comments around
  `create_target_within` explain the symlink-race defense in detail
  ("Validation happens on the parent directory BEFORE any
  `create_dir_all` call.").
- `src-tauri/src/lib.rs:28-32` and `33-43` — comments explicitly
  document the `TempDir` lifetime caveat for the deprecated
  `download_and_verify_archive` and `extract_archive_to_directory`
  commands.

### 4.5 Concurrency Patterns

- `Promise.all` for parallel IPC:
  `src/Home.tsx:309-336` (per-package install updates).
- `Promise.allSettled` to ignore per-item errors:
  `src/PackageStore.tsx:135-138`.
- `Mutex` for in-process registry: `InstallRegistry` in
  `src-tauri/src/lib.rs:77-93`.
- `tokio::spawn` for backgrounded long-running commands:
  `src-tauri/src/lib.rs:163-185`.
- `CancellationToken` propagated through download+extract+copy:
  `src-tauri/src/pipeline.rs:31, 86, 121-138`.
- `requestIdRef` in `useVersionCheck` is the
  JS-side equivalent (`src/hooks/useVersionCheck.ts:26`).

### 4.6 Versioning of External Data

`src/types/version.ts:46-67` and `src-tauri/src/version.rs:80-92`
have parallel logic: a strict version shape (`looks_like_version`)
that **rejects** free-form text, plus a semver/date-based comparator
with a fallback to string comparison. The TS side returns
`{ success: false; error: ... }`; the Rust side returns the
`VersionCheckResult` struct directly. The logic is mirrored to keep
the two contracts in agreement.

### 4.7 JSON as the Sole Source of Truth

`src/types/device-install-map.json` is the canonical per-device
config. `device.ts` (`src/types/device.ts:20-156`) re-declares the
device list, but a test in `device.test.ts:36-77` asserts the two
lists stay in sync (different IDs would fail). The TS `device.ts`
version is used at runtime for type-safe lookups; the JSON drives
`getDeviceInstallRules`, `getExtrasPlatform`, `getDevicePaks`, and
`SHARED_BIOS` via `src/types/device-install-map.ts`.

### 4.8 Security-Critical Strings

`src-tauri/src/install.rs:128-135` validates `extras_platform`
must be `[a-zA-Z0-9-]+` and rejects empty input — used as a
filesystem path component, so injection is the concern. Similar
validation in `src-tauri/src/pipeline.rs:174-227` is the more
general path-traversal guard for `target_dir + platform + pak_name`.

### 4.9 Logging

The project does **not** use a logging library. The convention is:

- `eprintln!` for warnings (`src-tauri/src/install.rs:81`,
  `src-tauri/src/lib.rs:71`, `src-tauri/src/extract.rs:148`).
- `console.error`-style behavior on the frontend via
  `setError(message)` (`src/DriveSelector.tsx:46, 73`,
  `src/HealthCheck.tsx:23-29`).
- WiFi passwords are **stored in plaintext** in `wifi.txt` on the
  SD card, which is acknowledged as a constraint in the
  `WifiWizard.tsx:194-198` UI ("Note: Your WiFi password will be
  stored in plain text on the SD card (wifi.txt)…").

### 4.10 Naming Type Aliases vs Interfaces

- **Interfaces** for record/object shapes (e.g. `DeviceProfile`,
  `RemovableDrive`, `MinUIRelease`).
- **Type aliases** for unions and primitives-with-applied-generics
  (e.g. `type InstallPhase = "idle" | "downloading" | ...`,
  `type ProgressCallback = Arc<dyn Fn(...) + Send + Sync>`).
