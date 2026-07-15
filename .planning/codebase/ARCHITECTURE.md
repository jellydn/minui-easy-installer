# Architecture

## System Layers

```
┌─────────────────────────────────────────────────────┐
│  React UI (src/)                                     │
│  State-based navigation: home|store|wifi|bios|settings│
│  InstallOrchestrator: vanilla TS state machine        │
│  useForkInstall: thin React adapter (~129 lines)      │
│  Tauri IPC: invoke() + listen()                       │
├─────────────────────────────────────────────────────┤
│  Tauri IPC Layer (src-tauri/src/lib.rs)               │
│  TauriAppDispatcher: AppHandle → EventDispatcher      │
│  20 registered #[tauri::command] handlers              │
│  InstallManager: tauri::State (singleton)              │
├─────────────────────────────────────────────────────┤
│  Rust Domain (src-tauri/src/)                         │
│  install.rs: install_minui_with_cancel()              │
│  pipeline.rs: download → extract → copy               │
│  package.rs, health.rs, wifi.rs, bios.rs, drives.rs   │
│  version/, validate.rs, fs_utils.rs, platform.rs      │
└─────────────────────────────────────────────────────┘
```

## Key Abstractions

### InstallManager (`src-tauri/src/install_manager.rs`)

Owns the install lifecycle. Manages a `CancellationToken` behind a `Mutex`. Exposes:
- `start(&Arc<Self>, dispatcher, options)` — spawns `tokio::spawn` background task
- `cancel(&self)` — cancels in-flight install

Uses the `EventDispatcher` trait to emit progress/complete/error events without knowing about Tauri. `TauriAppDispatcher` bridges `AppHandle` to the trait. `MockDispatcher` records events for tests.

### InstallOrchestrator (`src/lib/InstallOrchestrator.ts`)

Vanilla TypeScript state machine (no React). Owns the install + update-all workflow:
- `start(fork, device, sdMount)` — full install flow with Tauri event listener management
- `updateAll(fork, device, sdMount, ...)` — MinUI update + sequential package installs
- `cancel()` — cancels in-flight install
- `subscribe(listener)` — observer pattern for React sync

### Pipeline (`src-tauri/src/pipeline.rs`)

Orchestrates download → extract → copy for any archive type:
- `Pipeline::run()` — full pipeline returning files copied count
- `Pipeline::run_to_extracted()` — download + extract only (used by packages)
- `InstallSession` — owns temp dirs; atomic cleanup on drop
- `create_target_within()` — symlink-race-safe directory creation within SD root

### Event System

```
Frontend                    Backend
─────────                   ───────
listen("install-progress")  →  InstallManager::start()
                               → tokio::spawn
                                 → install_minui_with_cancel()
                                   → ProgressCallback
                                     → EventDispatcher::emit_progress()
                                       → AppHandle::emit("install-progress")
listen("install-complete")  ←  EventDispatcher::emit_complete()
listen("install-error")     ←  EventDispatcher::emit_error()
```

## Data Flow — Install

1. User selects device + SD card in `Home.tsx`
2. `useForkInstall.installMinUI()` → `InstallOrchestrator.start()`
3. Orchestrator: `listen("install-progress")` for real-time updates
4. Orchestrator: `fetchMinUIRelease(fork)` → GitHub API → version + archive URLs
5. Orchestrator: `startInstallAndWait({...})` → Tauri IPC → `start_install` command
6. `start_install`: wraps `AppHandle` in `TauriAppDispatcher`, calls `manager.start()`
7. `InstallManager.start()`: creates `CancellationToken`, spawns `tokio::spawn`
8. Background task: `install_minui_with_cancel()` → 4 sequential steps:
   - Step 1: Base archive — Pipeline::run("base", ...) → download → extract → copy to SD
   - Step 2: Extras archive — Pipeline::run("extras", ...) → download → extract → copy (non-fatal)
   - Step 3: ROM directories — `create_rom_dirs()` → standard folders
   - Step 4: Version metadata — `write_version_metadata()` → minui.txt
9. Task completes → `emit_complete(InstallResult)` → frontend receives "install-complete"
10. Orchestrator runs `validateInstallation()` → shows result in UI

## Data Flow — Package Install

1. `PackageStore.tsx` fetches registry → `fetchPackageRegistry()` → `fetch_url()` → packages.minui.dev
2. User clicks "Install" → `installPackage({...})` → Tauri IPC
3. Backend: `Pipeline::run_to_extracted("package", ...)` → download → extract
4. Backend: `create_target_within(sd_mount, targetDir, platform, pakName)` → validated dir
5. Files copied from extracted temp to validated SD target
6. UI updates via `detect_installed_packages()` to reflect installed state

## Data Flow — Health Check

1. `HealthCheck.tsx` auto-runs on `sdMount` change (useEffect)
2. `checkSdCardHealth({sdMount, devicePlatform})` → Tauri IPC
3. Backend (`health.rs`):
   - `detect_filesystem()` — diskutil (macOS) or fsutil (Windows)
   - `get_free_space()` — fs_utils
   - `benchmark_read_speed()` — writes 64MB test file, reads back, measures MB/s
   - `scan_pak_dirs()` — walks Tools/ for *.pak directories
   - MinUI folder check (Tools, Emus)
4. Returns `HealthCheckResult` with checks, speed, support report

## Preserved Folders

During install, these folders are **never** overwritten or deleted (case-insensitive matching):

```rust
const PRESERVED_FOLDERS: &[&str] = &["roms", "saves", "save", "bios", "cheats"];
```

The `is_preserved_path()` function in `install.rs` checks whether a destination path is under one of these folders and skips it during `copy_dir_recursive`.
