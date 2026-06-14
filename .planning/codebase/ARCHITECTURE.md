# Architecture

**Analysis Date:** 2026-06-14

## Pattern Overview

**Overall:** Tauri v2 Bridge Architecture (IPC-based layered desktop app)

**Key Characteristics:**

- Rust backend handles all OS-level operations (filesystem, network, archive management)
- React frontend provides UI and orchestrates user workflows
- Typed IPC bridge via `@tauri-apps/api/core` `invoke()` calls to Rust `#[tauri::command]` handlers
- Result-Either pattern used throughout for type-safe error propagation (no thrown exceptions at the type layer)
- Event-based progress streaming from backend to frontend via Tauri `emit`/`listen`

## Layers

**Presentation Layer (React Components):**

- Purpose: UI rendering, user interaction, screen navigation
- Location: `src/`
- Contains: React functional components, CSS styles, event handlers
- Depends on: `src/types/` for API calls and type definitions
- Used by: Tauri WebView (user-facing)

**Type/API Layer (TypeScript Types + Invokers):**

- Purpose: Define data models and wrap Tauri IPC `invoke()` calls with typed Result-Either returns
- Location: `src/types/`
- Contains: Interfaces, type aliases, async functions that call `invoke()`, validation logic
- Depends on: `@tauri-apps/api/core` for IPC; Rust backend commands for execution
- Used by: Presentation layer components

**Rust Backend Core (Tauri Commands + Modules):**

- Purpose: All OS-level operations — drive detection, download, extraction, file copy, install orchestration
- Location: `src-tauri/src/`
- Contains: `#[tauri::command]` functions, module implementations, struct definitions
- Depends on: `std::fs`, `tokio` (async), Rust crate ecosystem (reqwest, zip, etc.)
- Used by: Frontend via Tauri IPC bridge

**Rust Backend Entry (Tauri App Bootstrap):**

- Purpose: Bootstrap the Tauri application, register command handlers, open devtools in debug
- Location: `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- Contains: `main()` function, Tauri builder with `generate_handler![]` macro
- Depends on: All Rust modules
- Used by: Tauri runtime

## Data Flow

**MinUI Installation Flow:**

1. User selects device + SD card on Home screen (`src/Home.tsx`)
2. User clicks "Install MinUI" → ConfirmDialog overlay appears (`src/ConfirmDialog.tsx`)
3. On confirm → `fetchMinUIRelease()` fetches latest GitHub release metadata (`src/types/release.ts`)
4. `installMinui()` invokes Rust `install_minui` command (`src/types/install.ts` → `src-tauri/src/lib.rs`)
5. Rust backend orchestrates: download → extract → copy base → download/extract/copy extras → create ROM dirs → write version metadata (`src-tauri/src/install.rs`)
6. Progress events stream to frontend via Tauri `emit("install-progress")` → frontend `listen("install-progress")`
7. On completion → `validateInstallation()` runs post-install checks (`src/types/validate.ts` → `src-tauri/src/validate.rs`)
8. `ValidationReportUI` renders results (`src/ValidationReport.tsx`)

**Version Check Flow:**

1. Drive selected → `useEffect` triggers version check (`src/Home.tsx`)
2. `fetchMinUIRelease()` gets latest version from GitHub (`src/types/release.ts`)
3. `checkMinuiVersion()` invokes Rust `check_minui_version` which reads `minui.txt` or `.minui/version` from SD card (`src/types/version.ts` → `src-tauri/src/version.rs`)
4. `checkPackageUpdates()` compares installed package versions against registry (`src/types/package.ts` → `src-tauri/src/package.rs`)
5. UI displays installed version, update availability, and package updates

**Package Store Flow:**

1. User navigates to "Package Store" tab (`src/App.tsx`)
2. `fetchPackageRegistry()` loads package data from bundled `store.json` (`src/types/package.ts`)
3. `PackageStore` component renders browsable package list (`src/PackageStore.tsx`)
4. User selects package → `installPackage()` invokes Rust `install_package` command
5. Backend downloads, extracts, and copies package to correct SD card location

**WiFi Configuration Flow:**

1. User navigates to "WiFi Setup" tab (`src/App.tsx`)
2. `scan_wifi_networks()` lists available networks (`src-tauri/src/wifi.rs`)
3. User selects network and enters password
4. `write_wifi_config()` writes `wifi.txt` to SD card root (`src-tauri/src/wifi.rs`)

**State Management:**

- Local component state via `useState` hooks (no global state manager)
- `App.tsx` owns top-level navigation state (`screen`) and shared selections (`selectedDevice`, `selectedDrive`)
- `Home.tsx` owns all install-related state (phase, progress, results, validation)
- Tauri events (`listen`/`emit`) bridge long-running backend operations to frontend state
- Cancellation via `cancelled` flag pattern in `useEffect` cleanup

## Key Abstractions

**Device Profile:**

- Purpose: Maps device ID to platform identifiers and install path rules
- Examples: `src/types/device.ts` (DeviceProfile, InstallPathRules)
- Pattern: Static registry of typed objects with lookup functions (`getDeviceProfile`, `getAllDeviceProfiles`)

**Result Either:**

- Purpose: Type-safe error handling without exceptions; every IPC call returns either `{ success: true, data }` or `{ success: false, error }`
- Examples: `src/types/install.ts` (InstallResultEither), `src/types/release.ts` (ReleaseFetchResult), `src/types/validate.ts` (ValidationResultEither)
- Pattern: Discriminated union on `success` field; error objects include typed `code` strings for programmatic handling

**Install Path Rules:**

- Purpose: Define where files should be placed on the SD card for each device type
- Examples: `src/types/device.ts` (InstallPathRules), `src/types/package.ts` (PackageInstallPathRules)
- Pattern: Configuration objects with `baseDir`, `extrasDir`, `toolsDir` paths

**Progress Event:**

- Purpose: Stream install progress from Rust backend to React frontend in real-time
- Examples: `src-tauri/src/install.rs` (InstallProgressEvent), `src/types/install.ts` (InstallProgressEvent)
- Pattern: Backend emits events with `step` + `details`; frontend listens and updates UI state

**Installed Version Detection:**

- Purpose: Determine what version of MinUI is currently on the SD card without relying on installer-written metadata
- Examples: `src/types/version.ts` (VersionCheckResult, InstalledVersion)
- Pattern: Read-only probe of SD card filesystem (checks `minui.txt` or `.minui/version`)

## Entry Points

**Frontend Entry:**

- Location: `src/main.tsx` → `src/App.tsx`
- Triggers: Tauri WebView loads `index.html` which loads the Vite-built bundle
- Responsibilities: Mount React app, render App component with screen routing

**Rust Entry:**

- Location: `src-tauri/src/main.rs` → `src-tauri/src/lib.rs`
- Triggers: Tauri runtime starts the application
- Responsibilities: Register all command handlers, configure app, open devtools in debug mode

**Tauri Command Registry:**

- Location: `src-tauri/src/lib.rs` (lines 166-183)
- Triggers: Frontend `invoke()` calls from TypeScript
- Responsibilities: Maps string command names to Rust async functions

## Error Handling

**Strategy:** Typed error codes with Result-Either pattern throughout both TypeScript and Rust layers.

**Patterns:**

- **Rust → TypeScript:** All Rust commands return `Result<T, String>` or concrete result structs; errors are string messages that get classified by the TypeScript layer
- **TypeScript classification:** `classifyError()` in `src/types/install.ts` infers error codes from message content (e.g., "download" → `DOWNLOAD_ERROR`)
- **Error code enums:** Each domain defines its own error code union type (e.g., `InstallError.code`, `ExtractionError.code`, `PackageRegistryError.code`)
- **Non-fatal errors:** Extras installation failures are captured as warnings (`extras_warning`) rather than hard failures
- **Progress recovery:** Install phase state machine (`InstallPhase`) allows UI to show error state and let user dismiss
- **Validation fallback:** `formatValidationReport()` falls back to client-side formatting if Rust invocation fails

## Cross-Cutting Concerns

**Logging:**

- Rust: Progress events emitted via Tauri `emit()` (structured `InstallProgressEvent` with `step` and `details`)
- TypeScript: Install log accumulated in state array (`installLog: InstallProgressEvent[]`) and displayed in UI
- No external logging framework; errors propagated as Result types

**Validation:**

- Post-install validation via `validateInstallation()` (`src-tauri/src/validate.rs`) checks file presence and SD card health
- SD card health checks (`check_sd_card_health`) verify filesystem, free space, and device compatibility
- Package registry data treated as untrusted — `fetchPackageRegistry()` validates store.json schema before use
- Archive extraction includes path traversal protection (`ExtractionError.code: "PATH_TRAVERSAL"`)

**Authentication:**

- No authentication required; GitHub API accessed unauthenticated (rate-limited)
- WiFi password handling: written to `wifi.txt` on SD card; never logged in plaintext
- No secrets or credentials stored by the application

**Data Preservation:**

- Preserved folders (`ROMS`, `roms`, `Saves`, `saves`, `BIOS`, `bios`, `CHEATS`, `cheats`) are never deleted or overwritten during install
- `is_preserved_path()` check applied during base file copy operations
- ROM directories created only if they don't already exist
