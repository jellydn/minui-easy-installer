# Architecture

## System Pattern

**Desktop Application with IPC Bridge**

The application follows a two-process architecture:
- **Rust backend** (Tauri v2): System operations, HTTP downloads, filesystem access, archive extraction
- **React frontend** (HTML/CSS/TS): User interface, state management, user input

Communication flows through Tauri's command-based IPC system: the frontend calls `invoke("command_name", { args })` and the backend responds with serialized JSON.

```
┌─────────────────────────────────────────┐
│              React Frontend              │
│  src/App.tsx (state-based navigation)   │
│  ┌──────┐ ┌───────┐ ┌──────┐ ┌───────┐ │
│  │ Home │ │ Store │ │ WiFi │ │ BIOS  │ │
│  └──┬───┘ └───┬───┘ └──┬───┘ └───┬───┘ │
│     │         │         │         │      │
│     └─────────┴────┬────┴─────────┘      │
│                    │ invoke()             │
├────────────────────┼─────────────────────┤
│              Tauri IPC Bridge             │
├────────────────────┼─────────────────────┤
│             Rust Backend                  │
│  src-tauri/src/lib.rs (commands)         │
│  ┌──────────┬──────────┬──────────┐      │
│  │ install  │ download │ extract  │      │
│  ├──────────┼──────────┼──────────┤      │
│  │ drives   │ health   │ validate │      │
│  ├──────────┼──────────┼──────────┤      │
│  │ wifi     │ bios     │ package  │      │
│  └──────────┴──────────┴──────────┘      │
│                                          │
│  Filesystem / HTTP / System APIs         │
└──────────────────────────────────────────┘
```

## Frontend Architecture

### Navigation (`src/App.tsx`)

State-based screen navigation using a `Screen` union type:

```typescript
type Screen = "home" | "store" | "wifi" | "bios" | "settings";
```

State is lifted to `AppShell` and passed as props. The `ForkContext` provider wraps the entire app.

### Component Tree

```
App
└── ForkProvider
    └── AppShell
        ├── Home
        │   ├── DeviceSelector
        │   ├── DriveSelector
        │   ├── StatusSummary (inline)
        │   ├── InstallProgress / ValidationReport
        │   ├── HealthCheck
        │   └── ConfirmDialog
        ├── PackageStore
        │   └── PackageCard[]
        ├── WifiWizard
        ├── BiosInstaller
        └── Settings
```

### State Management

- **Local state**: React `useState` for component-level state (screen, selectedDevice, selectedDrive)
- **Context**: `ForkContext` for fork selection (shared across Home, Settings, package store)
- **Custom hooks**: `useForkInstall`, `useVersionCheck`, `useMountEffect`, `useScrollToBottom`
- **No external state library** (no Redux, Zustand, etc.)

## Backend Architecture

### Tauri Command Registry (`src-tauri/src/lib.rs`)

All frontend-accessible commands are registered in `lib.rs`:

| Command | Module | Purpose |
|---------|--------|---------|
| `get_removable_drives` | `drives.rs` | Detect SD cards and removable drives |
| `format_drive` | `drives.rs` | Format a drive (MVP: not implemented) |
| `verify_archive_checksum` | `download.rs` | Verify SHA-256 checksum |
| `install_minui` | `install.rs` | Orchestrate full install flow (deprecated sync) |
| `start_install` | `lib.rs` | Start async install with cancellation |
| `cancel_install` | `lib.rs` | Cancel running install |
| `fetch_url` | `download.rs` | Simple HTTP GET (for package registry) |
| `scan_wifi` | `wifi.rs` | Scan for WiFi networks (macOS `airport`) |
| `write_wifi_config` | `wifi.rs` | Write `wifi.txt` to SD card |
| `check_sd_health` | `health.rs` | Check SD card health |
| `install_bios` / `install_bios_from_bytes` | `bios.rs` | Copy BIOS files to SD |
| `install_package` | `package.rs` | Install a community package |
| `detect_installed_packages` | `package.rs` | Scan SD for installed packages |
| `check_package_updates` | `package.rs` | Check for package updates |
| `validate_install` | `validate.rs` | Post-install validation |

### Module Organization

```
src-tauri/src/
├── main.rs          # Entry point — calls lib::run()
├── lib.rs           # Tauri command registration
├── install.rs       # Install flow (copy_base_files, copy_extras_files, rom dirs)
├── pipeline.rs      # Pipeline abstraction (download → extract → copy)
│                    # InstallSession (temp dir lifecycle)
│                    # create_target_within (path traversal guard)
├── download.rs      # Streaming HTTP downloads, checksum verification
├── extract.rs       # ZIP archive extraction
├── drives.rs        # Platform-specific drive detection (macOS/Windows)
├── health.rs        # SD card health checks
├── validate.rs      # Post-install validation
├── package.rs       # Community package install/detect/update
├── wifi.rs          # WiFi scanning and config writing
├── bios.rs          # BIOS file catalog and installation
├── fs_utils.rs      # Filesystem helpers (copy_dir_recursive, canonicalize, free space)
├── platform.rs      # Device platform mappings
└── version/
    ├── mod.rs       # Version parsing (minui.txt, version.txt)
    └── tests.rs     # Version parsing tests
```

## Install Pipeline

The core installation follows a three-phase pipeline:

```
Download → Extract → Copy
```

### Pipeline (`pipeline.rs`)

- `InstallSession` owns temporary directories (created by download/extract)
- `Pipeline::run()` orchestrates download → extract → copy for any archive type
- `Pipeline::run_to_extracted()` runs download → extract, returning path for custom copy logic
- Temp dirs drop atomically when `InstallSession` drops → cleanup after all operations

### Install Flow (`install.rs`)

1. **Download base archive** → streaming with progress, SHA-256 verification
2. **Extract base archive** → to temp directory
3. **Copy base files** → only shared items (`Bios`, `Roms`, `Saves`, `MinUI.zip`) + selected device folder
4. **Download extras archive** → (if available, non-fatal on failure)
5. **Extract extras** → to temp directory
6. **Copy extras** → filtered by `extras_platform`
7. **Create ROM directories** → standard folder structure
8. **Write `minui.txt`** → `{fork_name} {version}` metadata

### Cancellation

- `start_install` spawns a background task with `CancellationToken`
- Cancellation checked at phase boundaries in `Pipeline::run()`
- Emits `install-progress`, `install-complete`, `install-error` events

## Install Path Rules

Three directories per device profile:

| Rule | Value | Purpose |
|------|-------|---------|
| `baseDir` | `"/"` | Base archive target (SD root) |
| `extrasDir` | `"/"` | Extras archive target |
| `toolsDir` | `"/Tools"` | Tool packages target |

All 8 primary devices use the same defaults. Defined in `src/types/device.ts`.

## Security Patterns

### Path Traversal Prevention

- `create_target_within()` — Validates parent canonical path before directory creation, re-validates after
- `copy_dir_recursive()` — `fs::copy` dereferences symlinks (no symlink escape)
- `install_bios_from_bytes` — Sanitizes subdir/filename, canonicalizes target parent before and after write

### Input Sanitization

- `extras_platform`: Alphanumeric + hyphens only
- BIOS paths: No traversal characters, NUL bytes, or path separators
- Registry data: Full schema validation before use

### Atomic Operations

- Temporary directories via `tempfile::TempDir`
- Temp dirs live in `InstallSession` — cleanup on drop after all operations complete

## Version Tracking

- **Installer writes**: `minui.txt` — `{fork_name} {version}` (e.g., `MinUI 2025.01.01`)
- **Packages read**: `Tools/*/version.txt` (included in archives)
- **Fork name**: Configurable via `Settings`, defaults to `"MinUI"`
