# Architecture

**Analysis Date:** 2026-06-13

## Pattern Overview

**Overall:** Tauri v2 desktop app — thin React/TypeScript UI over a Rust command backend, communicating via Tauri IPC (`invoke`).

**Key Characteristics:**

- Two-process model: a webview frontend (React in `src/`) and a native Rust core (`src-tauri/src/`). The frontend never touches the filesystem, network drives, or shell directly — it calls Rust `#[tauri::command]` functions through `invoke()`.
- Command boundary is explicitly registered in one place: `tauri::generate_handler![...]` inside `run()` in `src-tauri/src/lib.rs`. 14 commands are exposed.
- Each Rust domain module (`drives`, `download`, `extract`, `install`, `validate`, `version`, `package`, `wifi`) has a mirrored TypeScript module under `src/types/` that wraps `invoke()` and re-declares the result shape, giving end-to-end typing across the IPC line.
- Result-type discipline: most TS wrappers return a discriminated `…Either` union (`{ success: true; data } | { success: false; error }`) rather than throwing; Rust commands return `Result<T, String>`.

## Layers

**Presentation (React components):**

- Purpose: Render screens, collect user intent, drive the install/update flow, show progress and reports.
- Location: `src/*.tsx` — `App.tsx` (root + nav), `Home.tsx` (install orchestrator), `DriveSelector.tsx`, `DeviceSelector.tsx`, `ConfirmDialog.tsx`, `InstallProgress.tsx`, `ValidationReport.tsx`, `PackageStore.tsx`, `WifiWizard.tsx`, `HealthCheck.tsx`.
- Contains: JSX, React hooks (`useState`/`useEffect`/`useCallback`), local component state.
- Depends on: the IPC wrapper layer in `src/types/`.
- Used by: bootstrapped by `src/main.tsx`.

**IPC / Domain wrappers (TypeScript):**

- Purpose: Type-safe bridge to Rust commands; also holds pure browser-side logic (release parsing, registry validation, byte formatting) that needs no native access.
- Location: `src/types/*.ts` — `drive.ts`, `device.ts`, `install.ts`, `archive.ts`, `validate.ts`, `version.ts`, `package.ts`, `release.ts`.
- Contains: `invoke<T>("command_name", args)` calls, interface declarations matching Rust structs, `…Either` result unions, and pure helpers.
- Depends on: `@tauri-apps/api/core` (`invoke`), `globalThis.fetch` for HTTP (GitHub API, registry).
- Used by: presentation components.
- Note: `release.ts` and `package.ts` fetch over HTTP **from the frontend** (GitHub releases API, `packages.minui.dev` registry) rather than via Rust.

**Command surface (Rust):**

- Purpose: Declares the IPC API and adapts frontend args to domain functions.
- Location: `src-tauri/src/lib.rs` — each `#[tauri::command]` thin-wraps a domain module call; `run()` registers them and opens devtools in debug.
- Depends on: the domain modules below.
- Used by: the frontend via `invoke`.

**Domain core (Rust):**

- Purpose: All privileged work — drive enumeration, downloads, checksum verification, archive extraction, file copy to SD card, validation, version/package detection, WiFi config.
- Location: `src-tauri/src/` — `drives.rs`, `download.rs`, `extract.rs`, `install.rs`, `validate.rs`, `version.rs`, `package.rs`, `wifi.rs`.
- Contains: `serde::Serialize` result structs, OS-specific code via `#[cfg(target_os = ...)]`, shell-outs (`diskutil`, `airport`, `nmcli`, `netsh`), `reqwest`/`sha2`/`zip` usage.
- Depends on: external crates (`reqwest`, `sha2`, `hex`, `tempfile`, `zip`, `tokio`, `libc`/`windows-sys`).
- Used by: the command surface in `lib.rs`.

## Data Flow

**Install MinUI (primary flow, orchestrated in `src/Home.tsx`):**

1. Drive detect — `DriveSelector.tsx` calls `invoke("get_removable_drives")` → `drives::list_removable_drives()` (`src-tauri/src/drives.rs`, parses `diskutil list external` on macOS).
2. Device select — `DeviceSelector.tsx` picks a `DeviceProfile` from the static table in `src/types/device.ts` (platform + install path rules).
3. Version pre-check — on drive select, `Home.tsx` fetches latest release (`fetchMinUIRelease` in `release.ts`, GitHub API) then `check_minui_version` → `version::check_for_updates` (`src-tauri/src/version.rs`, reads `minui.txt`/`.minui/version`), and `check_package_updates`.
4. Confirm — `ConfirmDialog.tsx` gates the write (per project rule: never write without explicit confirmation).
5. Download + extract + copy — `installMinui()` (`src/types/install.ts`) calls one blocking command `install_minui` → `install::install_minui` (`src-tauri/src/install.rs`): downloads base archive (`download.rs`, SHA-256 verify via `sha2`/`hex`), extracts to temp (`extract.rs`, with path-traversal guards), copies into the SD root **skipping `PRESERVED_FOLDERS`** (ROMS/Saves/BIOS/CHEATS), then repeats for the optional extras archive (extras failure is non-fatal).
6. Validate — on success, `validateInstallation` → `validate::validate_installation` checks essential paths (`minui.pak`, `boot.sh`, `DMG.png`); rendered by `ValidationReport.tsx`. `HealthCheck.tsx` separately calls `check_sd_card_health`.

**Other flows:** Package Store (`PackageStore.tsx` → `fetchPackageRegistry` HTTP + `install_package` → `package.rs`); WiFi setup (`WifiWizard.tsx` → `scan_wifi_networks` + `write_wifi_config` → `wifi.rs`, writes `wifi.txt`).

**State Management:**

- No external store. State is React component-local `useState`. `App.tsx` holds the top-level `screen` (`home`/`store`/`wifi`) plus the cross-screen selections (`selectedDevice`, `selectedDrive`) and passes them down as props. `Home.tsx` owns the full install/version/update state machine.

## Key Abstractions

**Command pair (Rust struct ↔ TS interface):**

- Purpose: One serialized result shape shared across the IPC boundary.
- Examples: `RemovableDrive` in `src-tauri/src/drives.rs` ↔ `src/types/drive.ts`; `InstallResult` in `src-tauri/src/install.rs` ↔ `src/types/install.ts`; `ValidationResult`/`HealthCheckResult` in `src-tauri/src/validate.rs` ↔ `src/types/validate.ts`.
- Pattern: Rust `#[derive(Serialize)]` struct returned via `Result<T, String>`; TS `invoke<T>(...)` typed to the mirror interface.

**`…Either` result union:**

- Purpose: Explicit success/error handling without exceptions at call sites.
- Examples: `InstallResultEither` (`src/types/install.ts`), `ValidationResultEither` (`src/types/validate.ts`), `VersionCheckResultEither` (`src/types/version.ts`), `ReleaseFetchResult` (`src/types/release.ts`).
- Pattern: discriminated union on `success`; wrappers also map raw error strings to coded enums (e.g. `DOWNLOAD_ERROR`, `CHECKSUM_ERROR`).

**Device profile table:**

- Purpose: Per-device platform name + install path rules (`baseDir`/`extrasDir`/`toolsDir`).
- Examples: `DEVICE_PROFILES` + `getDeviceProfile()` in `src/types/device.ts`.
- Pattern: static in-memory lookup table (8 supported handhelds).

**`InstallPhase` state machine:**

- Purpose: UI progress states for the coarse-grained backend install.
- Examples: `idle | downloading | extracting | copying | complete | error` in `src/types/install.ts`, driven in `Home.tsx`, rendered by `InstallProgress.tsx`.

## Entry Points

**Frontend bootstrap:**

- Location: `src/main.tsx`.
- Triggers: webview loads `index.html` (Vite dev server on port 1420, see `vite.config.ts`/`tauri.conf.json`).
- Responsibilities: `ReactDOM.createRoot(...).render(<App/>)` under `React.StrictMode`.

**Native entry:**

- Location: `src-tauri/src/main.rs` → calls `minui_easy_installer_lib::run()` in `src-tauri/src/lib.rs`.
- Triggers: process launch.
- Responsibilities: `run()` builds the Tauri app, registers the 14 commands via `generate_handler!`, opens devtools in debug, and runs the event loop.

## Error Handling

**Strategy:** Rust commands return `Result<T, String>` (errors surface as rejected `invoke` promises); TS wrappers catch and convert into typed `…Either` errors so UI code branches on `result.success` instead of `try/catch`.

**Patterns:**

- Error-string classification: `install.ts` inspects the message ("download"/"extract"/"checksum") to assign an `InstallError.code`.
- Non-fatal degradation: extras install failure and the drive version pre-check in `Home.tsx` are swallowed so the primary flow continues.
- Direct `try/catch` in components that call `invoke` straight (`DriveSelector.tsx`, `WifiWizard.tsx`) storing `error` in local state.

## Cross-Cutting Concerns

**Logging:** No structured logging framework; Rust returns descriptive error strings, debug builds open webview devtools (`lib.rs`). Project rule: never log WiFi passwords/secrets in plaintext (`wifi.rs` writes them only to `wifi.txt` on the card).

**Validation:** Untrusted external data validated before use — `validatePackageEntry`/registry validation in `src/types/package.ts`, GitHub release parsing in `src/types/release.ts`; archive path-traversal guards (`is_path_traversal`/`validate_entry_path`) in `src-tauri/src/extract.rs`; post-install checks in `src-tauri/src/validate.rs`.

**Authentication:** None — no accounts; only outbound public HTTP to GitHub API and the static registry. Native capability is restricted by `src-tauri/capabilities/default.json` (`core:default` only).

---

_Architecture analysis: 2026-06-13_
