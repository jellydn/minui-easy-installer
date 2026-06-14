# Coding Conventions

**Analysis Date:** 2026-06-14

## Naming Patterns

**Files:**

- React components: PascalCase — `DriveSelector.tsx`, `Home.tsx`, `InstallProgress.tsx`, `ConfirmDialog.tsx`
- Type/utility modules: camelCase — `drive.ts`, `install.ts`, `release.ts`, `version.ts`, `package.ts`, `archive.ts`
- Rust modules: snake_case — `download.rs`, `install.rs`, `version.rs`, `extract.rs`, `fs_utils.rs`
- Test files: co-located with source, suffixed `.test.ts` or `.test.tsx` — `release.test.ts`, `Home.test.tsx`
- Static data: lowercase JSON — `store.json` (in `src/types/`)

**Functions:**

- Frontend: camelCase — `formatSize()`, `getDriveDisplayName()`, `fetchMinUIRelease()`, `installMinui()`, `checkMinuiVersion()`, `classifyError()`
- Backend (Tauri commands): snake_case matching Rust convention — `get_removable_drives`, `install_minui`, `check_minui_version`
- Internal Rust helpers: snake_case — `detect_installed_version()`, `parse_minui_version()`, `is_update_available()`, `is_preserved_path()`
- React event handlers: `handle` prefix — `handleInstallClick()`, `handleCancelInstall()`, `handleConfirmInstall()`, `handleDismissInstall()`
- Async fetch wrappers: `fetch` prefix — `fetchDrives()`, `fetchMinUIRelease()`, `fetchPackageRegistry()`

**Variables:**

- Frontend state: camelCase with descriptive names — `installPhase`, `installMessage`, `showConfirmDialog`, `isCheckingVersion`
- Rust constants: SCREAMING_SNAKE_CASE — `ROM_DIRS`, `PRESERVED_FOLDERS`, `GITHUB_API_URL`
- Local variables: camelCase in TS, snake_case in Rust

**Types:**

- Interfaces: PascalCase — `RemovableDrive`, `InstallResult`, `MinUIRelease`, `VersionCheckResult`, `PackageRegistryEntry`
- Type aliases: PascalCase — `InstallPhase`, `PackageCategory`, `InstallErrorCode`, `ReleaseFetchResult`
- Discriminated unions: `...Either` suffix — `InstallResultEither`, `VersionCheckResultEither`, `DownloadResultEither`, `ExtractionResultEither`
- Error types: `...Error` suffix with `message: string` and `code: string` — `InstallError`, `DownloadError`, `ReleaseFetchError`
- Result types: `...FetchResult` or `...Result` suffix — `PackageRegistryFetchResult`, `ReleaseFetchResult`
- Props interfaces: `...Props` suffix — `DriveSelectorProps`, `HomeProps`

## Code Style

**Formatting:**

- Tool: `oxfmt` (Oxidation Compiler formatter) — run via `bun run fmt`
- Indentation: Tabs (observed consistently across all `.ts`, `.tsx`, `.cjs`, and `.json` files)
- Semicolons: Used (TypeScript files use semicolons)
- Quotes: Double quotes for strings in TS/TSX
- Trailing commas: Used in multi-line structures (object literals, function parameters, arrays)
- JSX: Indented with consistent 4-space JSX content within components

**Linting:**

- Primary linter: `oxlint` (Oxidation Compiler linter) — run via `bun run lint`
- Secondary: ESLint configured in `.eslintrc.cjs` with `@typescript-eslint/recommended` rules
- TypeScript: `tsconfig.json` enforces `strict: true`, `noUnusedLocals: true`, `noUnusedParameters: true`, `noFallthroughCasesInSwitch: true`, `forceConsistentCasingInFileNames: true`
- Type checking: `bun run typecheck` (runs `tsc --noEmit`)

## Import Organization

**Order:**

1. Tauri API imports — `import { invoke } from "@tauri-apps/api/core"`, `import { listen } from "@tauri-apps/api/event"`
2. React/React-DOM imports — `import { useCallback, useEffect, useState } from "react"`
3. Local type imports (type-only) — `import type { RemovableDrive } from "./types/drive"`
4. Local value imports — `import { formatSize, getDriveDisplayName } from "./types/drive"`
5. Local component imports — `import ConfirmDialog from "./ConfirmDialog"`

**Pattern for type imports:** Always use `import type` for pure type imports. Value imports are separated from type imports even when from the same module (e.g., `import type { RemovableDrive }` on one line, `import { formatSize }` on the next).

**Path Aliases:**

- No path aliases configured — all imports use relative paths (`./types/drive`, `./Home`, `@tauri-apps/api/core`)

**Dynamic imports:** Used in type modules to lazily load Tauri API — `const { invoke } = await import("@tauri-apps/api/core")` (seen in `install.ts`, `version.ts`, `package.ts`, `archive.ts`). This avoids circular dependency issues and keeps type modules testable without Tauri runtime.

## Error Handling

**Patterns:**

- **Either/Result discriminated unions** — The primary error handling pattern across the entire codebase. Every async API function returns `Promise<XxxResultEither>` where `success: true` carries `data` and `success: false` carries `error` with a typed `code` string.
  ```typescript
  type InstallResultEither =
    | { success: true; data: InstallResult }
    | { success: false; error: InstallError };
  ```
- **Error classification** — Errors from Rust IPC are classified by string matching on the error message. The `classifyError()` function in `install.ts` maps error messages to error codes (`DOWNLOAD_ERROR`, `EXTRACTION_ERROR`, `COPY_ERROR`, `CHECKSUM_ERROR`).
- **Catch-all with string fallback** — Every `catch` block converts unknown errors: `const message = err instanceof Error ? err.message : "Unknown error"`
- **Non-fatal errors** — Version check failures, package update failures, and extras installation failures are treated as non-fatal. They log warnings but do not block the main flow (e.g., extras failure becomes `extras_warning` on `InstallResult`).
- **Tauri commands** — Rust backend returns `Result<T, String>` where the error variant is always a `String`. Error strings are formatted with descriptive prefixes (e.g., `"Base download failed: {e}"`, `"Failed to create Roms/{dir}: {e}"`).

## Logging

**Framework:** None (no structured logging on frontend or backend)

**Patterns:**

- Frontend: State-driven UI messages via `installMessage` and `installLog` arrays — progress events are accumulated in `InstallProgressEvent[]` and displayed in the `InstallProgressUI` component
- Frontend: Non-fatal errors silently caught — e.g., version check failures have an empty `catch {}` block
- Backend (Rust): No `log` or `tracing` crate usage observed — errors are returned as `String` values in `Result` types and propagated via Tauri IPC
- Progress reporting: Rust backend emits `InstallProgressEvent` via Tauri's `app_handle.emit("install-progress", event)` — received on frontend via `listen<InstallProgressEvent>("install-progress", callback)`

## Comments

**When to Comment:**

- Doc comments on public Rust functions explaining purpose, parameters, and expected behavior — `/// Detect installed MinUI version from SD card metadata.`
- Inline comments explaining non-obvious logic — `// MinUI format: SSID:PASSWORD on one line`
- Comments explaining data formats — `/// Expected format:\n/// MinUI v2024.12.25`
- Comments on constants explaining their purpose — `/// Folders that must never be deleted or overwritten during install`
- No unnecessary comments on obvious code

**JSDoc/TSDoc:**

- Single-line `/** ... */` comments on functions with non-obvious behavior — `/** Infers error code from a Rust error message string */`
- No full JSDoc parameter documentation observed — types serve as documentation
- TSDoc not used for component props — TypeScript interfaces are self-documenting

## Function Design

**Size:** Functions are kept small and focused. Complex flows are decomposed into helper functions (e.g., `install_minui` calls `download_archive`, `extract_archive`, `copy_base_files`, `copy_extras_files`, `create_rom_dirs`). UI components are kept under ~200 lines.

**Parameters:**

- Options objects for functions with 3+ parameters — `installMinui(options: { baseUrl, extrasUrl, baseChecksum, ... })`, `checkMinuiVersion(options: { sdMount, latestVersion })`
- Direct parameters for 1-2 arg functions — `formatSize(bytes)`, `getDriveDisplayName(drive)`, `verifyChecksum(filePath, expectedChecksum)`
- Tauri command parameters match frontend invocation names exactly (camelCase on both sides)

**Return Values:**

- All async API functions return discriminated union `Either` types — `{ success: true, data } | { success: false, error }`
- Simple utility functions return primitive values or `null`
- Rust functions return `Result<T, String>` for fallible operations
- State-changing functions return void (React setState is side-effect based)

## Module Design

**Exports:**

- One primary type/function per module — each file in `src/types/` defines a focused domain (drive, install, release, version, package, archive, validate)
- Helper functions are co-located with their types — `formatSize()` lives in `drive.ts` alongside `RemovableDrive`
- No re-exports or barrel files observed

**Barrel Files:** Not used. Each module is imported directly by path.

**Rust modules:** Organized by domain (`download.rs`, `install.rs`, `version.rs`, `extract.rs`, `drives.rs`, `fs_utils.rs`, `wifi.rs`, `validate.rs`, `package.rs`) with `lib.rs` as the central registration point for Tauri commands.
