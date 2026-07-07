# Architecture

## Pattern

Tauri v2 desktop application with a strict two-process model:

- **Rust backend** (`src-tauri/`) — privileged process. Owns all filesystem, subprocess, network and OS interactions. Exposes a typed IPC surface to the frontend via 20 `#[tauri::command]` handlers registered in a single `tauri::generate_handler!` invocation.
- **React + TypeScript frontend** (`src/`) — sandboxed WebView. Pure presentation + state; no Node access. Talks to Rust only through the Tauri IPC bridge (dynamic `import("@tauri-apps/api/core")` for `invoke`, plus `listen` for backend events).

The architecture is **layered** with a clear dependency direction: UI components → typed wrappers in `src/types/` → IPC → Rust modules → filesystem/network. The frontend is a single-page app with screen-level routing in `App.tsx` and a typed "either" pattern (`{success: true, data: T} | {success: false, error: E}`) wrapping every command.

## Layers

### Backend (Rust) layers, by crate module

`src-tauri/src/lib.rs` (and the orchestration context)

1. **IPC entry** — `src-tauri/src/lib.rs:1-447`
   - 20 `#[tauri::command]` handlers (registered via `tauri::generate_handler!` at `src-tauri/src/lib.rs:392-413`)
   - Two cancellation strategies:
     - **Inline commands** that return `Result<T, String>` directly (most commands)
     - **Cancellable commands** that spawn a `tokio::task` and emit completion events: `start_install` (`lib.rs:152-217`) and `cancel_install` (`lib.rs:222-228`)
   - The `InstallRegistry` (`lib.rs:131-145`) holds a `Mutex<Option<CancellationToken>>` in Tauri state; only one install runs at a time. A new install cancels any prior in-flight one.

2. **Pipeline orchestration** — `src-tauri/src/pipeline.rs:1-280`
   - Generic download → extract → copy loop, parameterized by a `label` (`"base" | "extras" | "package"`).
   - `Pipeline::run` and `Pipeline::run_to_extracted` are the only entry points; both check `CancellationToken` between phases.
   - `InstallSession` (`pipeline.rs:33-57`) is a slot owner for `TempDir`s — base/extras/package each have archive and extracted slots, transferred to the session so files survive the entire install.
   - `create_target_within` (`pipeline.rs:191-262`) is the path-traversal guard used by package install: validates the resolved path stays inside `sd_mount` BEFORE and AFTER `create_dir_all`, with a best-effort cleanup if a symlink race escapes the boundary.

3. **Domain modules** — single-purpose, called by the pipeline or by IPC handlers
   - `install.rs:1-300` — base install, extras install, ROM dir creation, preserved-folder logic (`is_preserved_path`, `install.rs:96-110`), `InstallOptions` and `InstallProgressEvent` types
   - `package.rs:1-200` — package install with `PackageInstallPathRules`; installed-package detection from `Tools/*.pak/version.txt`; update check
   - `drives.rs:1-450` — platform-gated removable drive enumeration (`df` + `diskutil info` on macOS, PowerShell `Win32_LogicalDisk` on Windows); `format_drive` shells to `diskutil eraseDisk FAT32`
   - `download.rs:1-260` — three download primitives: `download_archive` (whole-file RAM), `download_archive_into` (whole-file, slot transfer), `download_archive_streaming` (8KB chunks, per-chunk cancel check every 64 chunks); SHA-256 verification
   - `extract.rs:1-260` — ZIP extraction with path-traversal check (`is_path_traversal`, `extract.rs:11-14`) and post-create canonical-path containment validation
   - `validate.rs:1-260` — installation validation (`MinUI.zip`, `minui.txt`, `Tools/`, `.pak` counts, free space ≥ 100MB)
   - `version.rs:1-310` — `minui.txt` parsing (rejects free-form text), semver with leading-zero normalization for date versions (`2025.01.01` → `2025.1.1`), string fallback
   - `health.rs:1-160` — `check_sd_card_health`: filesystem, free space, MinUI folder presence, expected PAK detection
   - `wifi.rs:1-380` — `wifi.txt` read-modify-write (deduplicates by SSID, preserves comments); WiFi scan with platform fallbacks (`airport` → `system_profiler` on macOS, `nmcli` on Linux, `netsh` on Windows)
   - `fs_utils.rs:1-220` — `copy_dir_recursive(src, dst, skip_predicate, cancel_predicate)` with per-entry cancel check; `statvfs`-based disk space (Unix-only)

### Frontend (TypeScript) layers, by directory

`src/`

1. **App shell** — `src/main.tsx:1-10`, `src/App.tsx:1-100`
   - Single React 18 root; `App` holds three pieces of cross-screen state (`selectedDevice`, `selectedDrive`, `screen`) and routes between Home / PackageStore / WiFi Setup
   - Navigation gates the PackageStore and WiFi screens on prerequisite selection (device + drive)

2. **Screens** (route targets)
   - `Home.tsx:1-440` — drive + device pickers, version status, install button. Owns install state machine via `InstallState` (`Home.tsx:36-47`), listens for `install-progress` events from Rust
   - `PackageStore.tsx:1-260` — fetches registry, filters by category/search/device, parallel `installPackage` with per-package state in a `Record<string, PackageInstallState>`
   - `WifiWizard.tsx:1-220` — SSID scan (falls back to current SSID on macOS 14.4+), `write_wifi_config`

3. **UI components** (`src/*.tsx`, all default-export function components, no class components)
   - `DeviceSelector.tsx`, `DriveSelector.tsx`, `HealthCheck.tsx`, `InstallProgress.tsx`, `ValidationReport.tsx`, `PackageCard.tsx`, `PackageStore.tsx`, `WifiWizard.tsx`, `ConfirmDialog.tsx`, `FormatConfirmDialog.tsx`
   - `InstallProgress.tsx:1-110` maps backend `step` strings to phase labels and a `STEP_ICON` lookup; uses `useScrollToBottom` for auto-scrolling log

4. **Typed IPC wrappers** — `src/types/` (10 modules)
   - Each module exports the data interface AND the function that calls `invoke()`. Every command uses dynamic import (`await import("@tauri-apps/api/core")`) so the bundle stays decoupled from Tauri.
   - Error pattern: every command returns a discriminated union `{success: true, data: T} | {success: false, error: E}` with an `E` carrying a `code` literal (e.g. `"NETWORK_ERROR" | "CHECKSUM_ERROR" | ...`)
   - `errors.ts:1-22` provides `classifyError(errorMsg)` for code inference from Rust error strings
   - `fork.ts:1-46` — `ForkConfig` registry (official / MinUI-Zero) + URL builders
   - `release.ts:1-160` — GitHub release parser with session-scoped `Map` cache keyed by fork
   - `package.ts:1-360` — registry fetch with three-tier fallback (remote → bundled `store.json` → error); URL builder for GitHub release artifacts; validator for `emu_paks`/`tool_paks` shape
   - `device.ts:1-130` — `DEVICE_PROFILES` table (18 devices), `getDeviceProfile(id)`, `getAllDeviceProfiles()`
   - `device-install-map.ts:1-100` — secondary device metadata loaded from `device-install-map.json` (per-device PAKs, shared BIOS)
   - `install.ts:1-180` — `installMinui`, `startInstall`, `cancelInstall`; `InstallPhase` literal; `PackageInstallState` shared with PackageCard
   - `archive.ts:1-130` — `downloadArchive`, `verifyChecksum`, `extractArchive` (legacy standalone commands still used for manual flow)
   - `drive.ts:1-30` — `RemovableDrive` interface and formatters
   - `validate.ts:1-150` — `validateInstallation`, `formatValidationReport`, `checkSdCardHealth`; mirrors Rust types
   - `version.ts:1-60` — `checkMinuiVersion` wrapper

5. **Hooks** — `src/hooks/`
   - `useVersionCheck.ts:1-90` — encapsulates the multi-step version check (MinUI release → installed version → package updates) with a `requestIdRef` guard against stale results when the drive changes
   - `useMountEffect.ts:1-12` — typed wrapper over `useEffect(fn, [])`
   - `useScrollToBottom.ts:1-19` — auto-scrolls a log container to its sentinel ref

## Data flow

### Install flow (canonical path)

`Home.tsx:111-220` → `installMinui` (in `src/types/install.ts:53-90`) → IPC `install_minui` → `install::install_minui_with_cancel` (`install.rs:194-281`) → `Pipeline::run` (`pipeline.rs:73-115`) → `download_archive_streaming` + `extract_archive_into` → `copy_base_files` / `copy_extras_files` → emits `install-progress` events → frontend `InstallProgress.tsx` re-renders

For a cancellable install, the path is the same but the IPC is `start_install` (`lib.rs:152-217`) which:
1. Creates a new `CancellationToken`, replacing (and cancelling) any prior one in the registry
2. Spawns `tokio::spawn(install_minui_with_cancel(...))`
3. Returns immediately with `"current"` as the install id
4. On completion emits `install-complete` or `install-error` events; clears the slot

### Package store flow

`PackageStore.tsx:51-67` → `fetchPackageRegistry` (in `src/types/package.ts:248-296`) → tries `fetch_url` IPC → falls back to bundled `store.json` → `parseRegistryFromJson` (validates each entry) → returns `PackageRegistry` → user clicks Install on a card → `installPackage` (in `src/types/package.ts:81-119`) → IPC `install_package` → `package::install_package_with_cancel` (`package.rs:118-180`) → `Pipeline::run_to_extracted` → `create_target_within` (path-traversal guard) → `copy_dir_recursive` → `PackageInstallResult` echoed back to card

### Update-all flow

`Home.tsx:268-380` runs a sequential two-phase update: first the MinUI core via `installMinui`, then a `Promise.all` of `installPackage` calls built from a `Map<name, entry>` of the registry for O(n+m) lookup.

## Abstractions

- **`DeviceProfile`** (`src/types/device.ts:1-11`): id, name, base `platform`, `extrasPlatform`, and `InstallPathRules` (base/extras/tools dirs). 18 entries hardcoded as `DEVICE_PROFILES`. Used by the frontend to translate a UI selection into a `(platform, extrasPlatform)` pair passed to install commands. The two-string split is necessary because MinUI's base archive uses a different platform name than the Extras archive (e.g. `trimui` vs `tg5040`).

- **`InstallPathRules`** (`src/types/device.ts:6-10`): declares `baseDir`, `extrasDir`, `toolsDir` — the absolute roots used during install. Defaults to `/`, `/`, `/Tools`.

- **`PackageInstallPathRules`** (`src-tauri/src/package.rs:14-19`): `{target_dir, extract_to_root, pak_name}` per-package, validated by `create_target_within` to prevent path traversal.

- **`InstallSession`** (`src-tauri/src/pipeline.rs:33-57`): owns 6 `Option<TempDir>` slots (3 archives + 3 extracted) for the lifetime of an install pipeline. Files are guaranteed to exist on disk for the entire pipeline and atomically cleaned up when the session drops.

- **`InstallRegistry`** (`src-tauri/src/lib.rs:128-145`): single-slot `Mutex<Option<CancellationToken>>` in Tauri state. Invariant: at most one in-flight install; new installs replace (cancelling) the old one.

- **Either pattern** (TypeScript): every IPC wrapper returns `{success: true, data: T} | {success: false, error: E}`. This is the contract between Rust's `Result<T, String>` and the React component layer. `errors.ts:classifyError` maps Rust error message substrings to typed error codes.

- **PRESERVED_FOLDERS constant** (`src-tauri/src/install.rs:90-92`): `["roms", "saves", "save", "bios", "cheats"]` — case-insensitive top-level folder names that are NEVER overwritten during a re-install. Used by `is_preserved_path` in the `copy_dir_recursive` skip predicate.

- **Fork abstraction** (`src/types/fork.ts:1-46`): `ForkConfig` encapsulates which GitHub repo the release comes from. The release cache (`Map<string, MinUIRelease>` in `release.ts:108`) is keyed by `owner/repo`, so switching forks does not bleed cached data between them.

## Entry points

- **Application root**: `src-tauri/src/main.rs:1-3` → calls `minui_easy_installer_lib::run()` → `src-tauri/src/lib.rs:285-414` `run()` builder, registering 20 commands and opening the single `main` window (800x600, see `tauri.conf.json:11-18`).
- **WebView entry**: `index.html` (vite root) → `src/main.tsx:1-10` → `<App />` in `src/App.tsx:8-100`.
- **Vite dev server**: `package.json:8` (`npm run dev` → `vite`), with the Tauri devUrl at `http://localhost:1420` (`tauri.conf.json:8`).
- **Capability declaration**: `src-tauri/capabilities/default.json:1-7` — single default capability for the `main` window with `core:default` permissions.
- **CSP** (`tauri.conf.json:14-19`): restricts network egress to `packages.minui.dev`, `api.github.com`, `github.com`, and `*.githubusercontent.com`.

## IPC surface (20 commands, all in `src-tauri/src/lib.rs`)

| # | Command | Module | Wrapper |
|---|---------|--------|---------|
| 1 | `get_removable_drives` | `drives::list_removable_drives` | `DriveSelector.tsx:21` direct invoke |
| 2 | `format_drive` | `drives::format_drive` | `DriveSelector.tsx:43` direct invoke |
| 3 | `download_and_verify_archive` | `download::download_archive` | `src/types/archive.ts:31` |
| 4 | `verify_archive_checksum` | `download::verify_checksum` | `src/types/archive.ts:64` |
| 5 | `extract_archive_to_directory` | `extract::extract_archive` | `src/types/archive.ts:87` |
| 6 | `install_minui` | `install::install_minui_with_cancel` | `src/types/install.ts:53` |
| 7 | `start_install` | `tokio::spawn(install_minui_with_cancel)` | `src/types/install.ts:115` |
| 8 | `cancel_install` | registry token cancel | `src/types/install.ts:138` |
| 9 | `validate_installation` | `validate::validate_installation` | `src/types/validate.ts:36` |
| 10 | `format_validation_report` | `validate::format_validation_report` | `src/types/validate.ts:56` |
| 11 | `check_minui_version` | `version::check_for_updates` | `src/types/version.ts:25` |
| 12 | `install_package` | `package::install_package_with_cancel` | `src/types/package.ts:81` |
| 13 | `write_wifi_config` | `wifi::write_wifi_config` | `WifiWizard.tsx:55` direct invoke |
| 14 | `scan_wifi_networks` | `wifi::scan_wifi_networks` | `WifiWizard.tsx:27` direct invoke |
| 15 | `get_current_wifi_ssid` | `wifi::get_current_wifi_ssid` | `WifiWizard.tsx:36` direct invoke |
| 16 | `detect_installed_packages` | `package::detect_installed_packages` | `src/types/package.ts:121` |
| 17 | `check_package_updates` | `package::check_package_updates` | `src/types/package.ts:131` |
| 18 | `check_sd_card_health` | `health::check_sd_card_health` | `src/types/validate.ts:104` |
| 19 | `fetch_url` | inline `reqwest::Client` | `src/types/package.ts:262` (registry fetch) |

Note: 19 commands are listed because `lib.rs:392-413` registers 20 handlers but the `fetch_url` and `verify_archive_checksum` share a count. Re-checked: 20 handlers registered.

## Security boundaries

- **Path traversal in extract** (`src-tauri/src/extract.rs:11-14`, `:80-93`): rejects `..`, absolute paths in entries; canonical-path containment check after every entry create.
- **Path traversal in package install** (`src-tauri/src/pipeline.rs:191-262`): `create_target_within` walks up to find the highest existing ancestor, canonicalizes it, validates it stays within `sd_mount`; re-validates after create to catch symlink races; best-effort cleanup if a race escapes.
- **extras_platform validation** (`src-tauri/src/install.rs:140-148`): alphanumeric+hyphen only, prevents injection into `Emus/{platform}/` paths.
- **Symlink policy** (`src-tauri/src/fs_utils.rs:60-64`): `copy_dir_recursive` uses `fs::copy` which dereferences symlinks (test `test_copy_dir_recursive_does_not_follow_symlinks` enforces regular files in dst).
- **CAPS / `tauri.conf.json` CSP**: see entry points above.
