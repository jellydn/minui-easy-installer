# Coding Conventions

**Analysis Date:** 2026-06-13

> Scope: Tauri v2 desktop app — Rust backend (`src-tauri/src/`) + React/TypeScript
> frontend (`src/`). Conventions below are derived by reading the actual source.

## Naming Patterns

**Files:**

- TS pure-logic / types modules: lowercase single word — `src/types/drive.ts`,
  `src/types/version.ts`, `src/types/install.ts`, `src/types/release.ts`,
  `src/types/package.ts`, `src/types/validate.ts`, `src/types/device.ts`,
  `src/types/archive.ts`.
- React components: PascalCase `.tsx` — `src/DriveSelector.tsx`, `src/Home.tsx`,
  `src/WifiWizard.tsx`, `src/PackageStore.tsx`, `src/ConfirmDialog.tsx`,
  `src/InstallProgress.tsx`, `src/App.tsx`, `src/main.tsx`.
- Rust modules: lowercase single word — `src-tauri/src/drives.rs`,
  `src-tauri/src/download.rs`, `src-tauri/src/extract.rs`, `src-tauri/src/install.rs`,
  `src-tauri/src/version.rs`, `src-tauri/src/package.rs`, `src-tauri/src/validate.rs`,
  `src-tauri/src/wifi.rs`. Registered in `src-tauri/src/lib.rs`.
- Test files co-located with `.test.ts` / `.test.tsx` suffix (see `TESTING.md`).

**Functions:**

- TS: `camelCase` — `formatSize`, `getDriveDisplayName` (`src/types/drive.ts`),
  `checkMinuiVersion` (`src/types/version.ts`), `installMinui` (`src/types/install.ts`),
  `parseGitHubRelease`, `fetchMinUIRelease` (`src/types/release.ts`).
- React components: `PascalCase` function declarations, e.g.
  `function DriveSelector({ ... }: DriveSelectorProps)` (`src/DriveSelector.tsx`).
- Rust: `snake_case` — `verify_checksum`, `download_archive` (`src-tauri/src/download.rs`),
  `detect_installed_version`, `parse_minui_version`, `is_update_available`,
  `check_for_updates` (`src-tauri/src/version.rs`).
- Tauri command handlers in `src-tauri/src/lib.rs` are thin `snake_case` wrappers
  annotated `#[tauri::command]` that delegate to a module fn (e.g.
  `get_removable_drives` → `drives::list_removable_drives()`).

**Variables:**

- TS: `camelCase` locals (`mockDrive`, `baseArchiveUrl`, `extrasArchiveUrl`).
- Rust: `snake_case` locals (`temp_dir`, `file_path`, `checksum_verified`).
- TS module constants: `SCREAMING_SNAKE_CASE` — `GITHUB_API_URL` (`src/types/release.ts`);
  module-private data tables use `SCREAMING_SNAKE_CASE` too — `DEVICE_PROFILES`
  (`src/types/device.ts`).

**Types:**

- TS: `PascalCase` interfaces and type aliases — `RemovableDrive` (`src/types/drive.ts`),
  `InstallResult`, `InstallError`, `InstallPhase` (`src/types/install.ts`),
  `MinUIRelease`, `ReleaseChecksums`, `ReleaseFetchResult` (`src/types/release.ts`).
- Rust: `PascalCase` structs — `DownloadResult` (`src-tauri/src/download.rs`),
  `VersionCheckResult`, `InstalledVersion` (`src-tauri/src/version.rs`),
  `PackageInstallResult`, `PackageInstallPathRules`, `InstalledPackage`,
  `PackageUpdateInfo` (`src-tauri/src/package.rs`).
- **Cross-boundary field casing:** structs that cross the Tauri IPC boundary keep
  `snake_case` fields on _both_ sides — Rust serde struct fields are `snake_case`
  with no `rename_all`, and the mirrored TS interface also uses `snake_case`
  (e.g. `mount_path`, `size_bytes`, `available_bytes` in both
  `src-tauri/src/drives.rs`'s `RemovableDrive` and `src/types/drive.ts`'s
  `RemovableDrive`; `base_files_copied` in both `install.rs` and `src/types/install.ts`).
  Pure-frontend types that never round-trip through serde use `camelCase`
  (e.g. `baseArchiveUrl`, `extrasArchiveUrl` in `src/types/release.ts`;
  `installPathRules`, `baseDir` in `src/types/device.ts`).
- **invoke argument casing:** TS `invoke()` call sites pass `camelCase` keys
  (`sdMount`, `latestVersion`, `baseUrl`) which Tauri auto-maps to the Rust
  handler's `snake_case` params (`sd_mount`, `latest_version`, `base_url`) — see
  `src/types/version.ts` calling `check_minui_version` → `src-tauri/src/lib.rs`.

## Code Style

**Formatting:**

- TS/TSX: **tabs** for indentation (verified: leading `\t` in `src/types/drive.ts`).
  Formatter is `oxfmt` via `npm run fmt` → `"fmt": "oxfmt src"` (`package.json`).
- Rust: **4 spaces** for indentation (verified: leading spaces in
  `src-tauri/src/version.rs`). Standard `cargo fmt` defaults; no `rustfmt.toml` present.
- Double-quoted strings in TS; `format!`/`"..."` in Rust.

**Linting:**

- TS lint tool is **oxlint**, NOT eslint — `"lint": "oxlint src"` (`package.json`).
  `oxlint` is the installed devDependency that scripts invoke.
- A legacy `.eslintrc.cjs` exists (extends `eslint:recommended`,
  `plugin:@typescript-eslint/recommended`) and `eslint` is listed in devDependencies,
  but **no npm script runs eslint** — the `.eslintrc.cjs` is effectively unused/vestigial.
  No `.oxlintrc.json` config file is present, so oxlint runs with defaults.
- Typecheck is separate from lint: `"typecheck": "tsc --noEmit"` (`package.json`).
- TS strictness via `tsconfig.json`: `"strict": true`, `"noUnusedLocals": true`,
  `"noUnusedParameters": true`, `"noFallthroughCasesInSwitch": true`,
  `"forceConsistentCasingInFileNames": true`.

## Import Organization

**Order (TS, observed in `src/DriveSelector.tsx` / `src/Home.test.tsx`):**

1. External packages — `@tauri-apps/api/core`, `react`, `@testing-library/*`, `vitest`.
2. Local value/type imports from sibling/`./types/*` modules.
3. `import type { ... }` used for type-only imports (e.g.
   `import type { RemovableDrive } from "./types/drive";`), kept separate from value
   imports of the same module.

**Order (Rust, observed in `src-tauri/src/download.rs` / `package.rs`):**

1. External crates (`use sha2::...`, `use tempfile::TempDir;`).
2. `std` imports (`use std::fs;`, `use std::path::Path;`).
3. Crate-local modules (`use crate::download;`, `use crate::extract;`).
4. In `#[cfg(test)]` modules: `use super::*;` first.

**Path Aliases:**

- None configured. TS uses relative paths (`./types/drive`).
  `tsconfig.json` sets `"moduleResolution": "bundler"` and
  `"allowImportingTsExtensions": true` but defines no `paths` aliases.

## Error Handling

**Patterns:**

- **Rust → Result<T, String>:** all fallible backend fns and every
  `#[tauri::command]` return `Result<T, String>` with human-readable error strings
  built via `.map_err(|e| format!("...: {}", e))` (e.g. `verify_checksum`,
  `download_archive` in `src-tauri/src/download.rs`). Some commands that cannot fail
  return the value directly (e.g. `check_minui_version` returns
  `version::VersionCheckResult`; `scan_wifi_networks` returns `Vec<String>`).
- **Result struct pattern:** operations also carry a `success: bool` + `error:
Option<String>` payload struct (e.g. `DownloadResult`, `InstallResult`,
  `PackageInstallResult`) so partial/expected failures (checksum mismatch) return
  `Ok(DownloadResult{ success:false, ... })` rather than `Err`.
- **TS discriminated-union "Either":** frontend API wrappers return a tagged union
  `{ success: true; data: T } | { success: false; error: E }` —
  `InstallResultEither`, `VersionCheckResultEither`, `ValidationResultEither`,
  `ReleaseFetchResult`. Errors carry a `code` string-literal union (e.g.
  `InstallError.code: "DOWNLOAD_ERROR" | "EXTRACTION_ERROR" | ...` in
  `src/types/install.ts`). `installMinui` maps backend error substrings to a `code`.
- **try/catch normalization:** TS wrappers catch and normalize unknown errors:
  `const message = err instanceof Error ? err.message : "Unknown error";`
  (`src/types/version.ts`, `src/types/install.ts`).
- **Untrusted-data parsing:** `parseGitHubRelease(data: unknown)`
  (`src/types/release.ts`) narrows `unknown` defensively and returns an error object
  instead of throwing — matches the AGENTS.md "treat registry data as untrusted" rule.
- **React components:** local `error` state set in `catch` and rendered
  (`setError(String(err))` in `src/DriveSelector.tsx`); `try/catch/finally` with a
  `loading` flag.

## Logging

**Framework:** No logging framework. No structured logger; `console.*` is avoided in
the type modules. Errors are surfaced via return values / component state rather than
logged. (Aligns with AGENTS.md: never log WiFi passwords or secrets.)

**Patterns:**

- Backend errors propagate as `Result`/`error` strings to the frontend, which renders
  them; no plaintext logging of secrets.

## Comments

**When to Comment:**

- Sparse, intent-focused. Inline comments explain non-obvious decisions, e.g.
  `// Keep the temp directory alive by leaking it` (`src-tauri/src/download.rs`) and
  `// No installed version means update available` (`src-tauri/src/version.rs`).

**JSDoc/TSDoc:**

- TS modules use essentially no JSDoc — types are self-documenting via interfaces.
- Rust uses `///` doc comments on public fns, sometimes with fenced examples of file
  formats (e.g. `detect_installed_version`, `parse_minui_version`,
  `is_update_available` in `src-tauri/src/version.rs`;
  `resolve_package_install_path` in `src-tauri/src/package.rs`).

## Function Design

**Size:** Small, single-purpose functions. Logic is decomposed into helpers
(`parse_minui_version`, `is_update_available`, `detect_installed_version` compose into
`check_for_updates` in `src-tauri/src/version.rs`).

**Parameters:**

- TS public API fns take a single **options object** with named fields, e.g.
  `installMinui(options: { baseUrl; extrasUrl?; sdMount; platform; extrasDir })`
  (`src/types/install.ts`), `checkMinuiVersion(options: { sdMount; latestVersion? })`.
- Optional params expressed as `?:` in TS and `Option<&str>` in Rust
  (`download_archive(url: &str, expected_checksum: Option<&str>)`).
- Rust fns take borrowed refs (`&str`, `&PackageInstallPathRules`); `lib.rs` handlers
  own `String`/`Option<String>` and pass `.as_deref()` down.
- React components destructure a typed `Props` interface in the signature
  (`{ selectedDrive, onSelectDrive }: DriveSelectorProps`).

**Return Values:**

- See Error Handling: Rust `Result<T, String>` or plain value; TS discriminated-union
  Either types or plain values.

## Module Design

**Exports:**

- TS: named exports only (`export function`, `export interface`, `export type`,
  `export const`). React components use `export default function` (default export per
  component file).
- Rust: `pub fn` / `pub struct` for module API; module-private helpers are unexported
  (`fn parse_minui_version`, `fn copy_package_files`).
- Rust structs that cross IPC derive `#[derive(Debug, Clone, serde::Serialize,
serde::Deserialize)]` (Serialize-only for outbound-only, e.g. `DownloadResult` is
  `serde::Serialize` only).

**Barrel Files:**

- None. There is no `index.ts` re-export barrel; consumers import directly from each
  `src/types/<name>.ts`. Rust modules are declared and individually wired in
  `src-tauri/src/lib.rs`.

---

_Convention analysis: 2026-06-13_
