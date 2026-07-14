# Architecture

## Overview

MinUI Easy Installer is a **Tauri v2 desktop application** with a Rust backend and React 18 frontend. It installs/updates MinUI on retro handheld SD cards and provides a package store for community emulators and tools.

```
┌──────────────────────────────────────────────────┐
│                    Tauri v2                       │
│  ┌──────────────────┐   ┌──────────────────────┐ │
│  │  Rust Backend    │◄──│  React Frontend       │ │
│  │  (20 commands)   │IPC│  (AppShell → screens) │ │
│  │                  │──►│                       │ │
│  │  Shell out to:   │   │  Home, PackageStore,  │ │
│  │  df, diskutil,    │   │  WifiWizard,         │ │
│  │  powershell,      │   │  BiosInstaller,      │ │
│  │  airport, nmcli   │   │  Settings             │ │
│  └──────────────────┘   └──────────────────────┘ │
└──────────────────────────────────────────────────┘
```

## IPC Layer

The backend exposes **20 Tauri commands** registered in `src-tauri/src/lib.rs` via `generate_handler!`:

| Category | Commands |
|----------|----------|
| **Drives** | `get_removable_drives`, `format_drive` |
| **Install** | `install_minui`, `start_install`, `cancel_install` |
| **Validation** | `validate_installation`, `format_validation_report` |
| **Package store** | `install_package`, `detect_installed_packages`, `check_package_updates` |
| **WiFi** | `write_wifi_config`, `scan_wifi_networks`, `get_current_wifi_ssid` |
| **BIOS** | `list_bios_catalog`, `get_bios_status`, `install_bios_file` |
| **Health** | `check_sd_card_health` |
| **Version** | `check_minui_version` |
| **Download** | `verify_archive_checksum` |
| **Network** | `fetch_url` |

## Install Pipeline

```
startInstallAndWait() (frontend)
  ├─ listen("install-complete") ───┐
  ├─ listen("install-error") ──────┤─ promise
  └─ startInstall(options) ────────┘

start_install (Rust command)
  ├─ Create CancellationToken
  ├─ Register in InstallRegistry
  └─ tokio::spawn:
       └─ install_minui_with_cancel()
            ├─ install_base()      → Pipeline::run("base")
            ├─ try_install_extras() → Pipeline::run("extras")
            ├─ create_rom_dirs()
            └─ write_version_metadata()
            └─ emit("install-complete") or emit("install-error")
```

### Pipeline phases

Each archive (base/extras/package) goes through:

```
Download → Extract → Copy
   │           │         │
   ▼           ▼         ▼
  TempDir    TempDir    SD card
```

`InstallSession` owns all `TempDir` handles — they're cleaned up atomically when the session drops.

### Cancellation

- `CancellationToken` from `tokio-util` is checked at the start of each pipeline phase
- `cancel_install()` cancels the token
- `InstallRegistry` ensures only one install runs at a time

## Frontend Architecture

```
App (src/App.tsx)
  └─ ForkProvider (src/contexts/ForkContext.tsx)
       └─ AppShell: state-based navigation
            ├─ "home"   → Home (useForkInstall, useVersionCheck)
            ├─ "store"  → PackageStore
            ├─ "wifi"   → WifiWizard
            ├─ "bios"   → BiosInstaller
            └─ "settings" → Settings
```

### State management

| Mechanism | Scope | Owned by |
|-----------|-------|----------|
| `ForkProvider` | Global | `useFork()` hook — active fork config |
| `RegistryCache` | Module | `package.ts` — 5-min TTL cache |
| `InstallRegistry` | Tauri state | `lib.rs` — in-flight install token |
| `useState` | Component-local | Each screen |

### Key hooks

| Hook | Purpose |
|------|---------|
| `useForkInstall` | Orchestrates MinUI install + update-all flow |
| `useVersionCheck` | Compares installed vs latest MinUI version |
| `useFork` | Reads current fork config from context |
| `useMountEffect` | Runs effect only on mount (not re-renders) |
| `useScrollToBottom` | Auto-scrolls install progress log |

## Rust Module Map

```
src-tauri/src/
  main.rs          → Entry point
  lib.rs           → All Tauri commands + generate_handler!
  lib_tests.rs     → Contract tests for Tauri command handlers
  install.rs       → Install flow (InstallPlan orchestrator, phases)
  install_copy_tests.rs → copy_base_files + preserved_path tests
  install_extras_tests.rs → extras + metadata tests
  install_tests.rs → Full pipeline integration test
  pipeline.rs      → Download → extract → copy pipeline
  download.rs      → HTTP download with streaming + checksum
  extract.rs       → ZIP extraction to temp dirs
  drives.rs        → Platform-specific drive detection + DriveDetector trait
  drives/macos.rs  → macOS diskutil parsing helpers
  drives/linux.rs  → Linux lsblk parsing
  drives/windows.rs→ Windows WMI parsing
  package.rs       → Package install logic
  wifi.rs          → WiFi config write + platform dispatchers
  wifi/macos.rs    → macOS airport + system_profiler WiFi scanning
  wifi/linux.rs    → Linux nmcli + iwgetid WiFi scanning
  wifi/windows.rs  → Windows netsh WiFi scanning
  bios.rs          → BIOS catalog + status + file install
  health.rs        → SD card health checks
  validate.rs      → Post-install validation
  version.rs       → MinUI version check (minui.txt)
  platform.rs      → Device ↔ platform mapping
  fs_utils.rs      → Directory copy, disk space, canonicalize
```

## Security Patterns

- **Symlink escape prevention:** `create_target_within` and `install_bios_from_bytes` canonicalize ancestors before AND after directory creation
- **Never write without confirmation:** All SD card writes require explicit user confirmation via `ConfirmDialog`
- **Never format implicitly:** Formatting is opt-in only
- **No secrets in logs:** WiFi passwords are never logged in plaintext
- **Input sanitization:** `extras_platform` restricted to alphanumeric + hyphens; BIOS paths reject path separators and NUL bytes
- **Registry data treated as untrusted:** Validated against schema before use (`src/types/validate.ts`)
