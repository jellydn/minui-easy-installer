# Architecture

## Pattern: Tauri IPC with Command Handlers

The application follows a client-server pattern via Tauri's IPC bridge:

```
┌─────────────────────┐     IPC (invoke)     ┌──────────────────────┐
│   React Frontend    │ ◄──────────────────► │    Rust Backend      │
│   (src/)            │     events           │    (src-tauri/src/)  │
└─────────────────────┘                      └──────────────────────┘
```

All commands are registered in `src-tauri/src/lib.rs` (~335 lines). Each accepts a single `#[derive(Deserialize)]` struct parameter for clean IPC boundaries.

## Core Domains

### Install Pipeline

```
Download ──► Extract ──► Copy (base) ──► Copy (extras) ──► ROM dirs ──► Version metadata
```

Files: `install.rs` (512 lines), `download.rs`, `extract.rs`, `pipeline.rs`

- **Cancellation**: `start_install` spawns in a background `tokio` task with `CancellationToken`. Emits `install-progress` / `install-complete` / `install-error` Tauri events.
- **Base archive filtering**: `copy_base_files` copies only the selected device folder + shared items (`Bios`, `Roms`, `Saves`, `MinUI.zip`), leaving other platforms behind.
- **Extras**: Always installed when available. Failure is non-fatal (logged as warning).
- **User data preservation**: `roms`, `saves`, `save`, `bios`, `cheats` folders are case-insensitively preserved during updates.

### Package Store

File: `src/types/package.ts` (236 lines), `src/PackageStore.tsx` (266 lines)

- Registry fetched from `packages.minui.dev/registry/index.json`
- `RegistryCache` class with injectable TTL
- Per-device platform paths from `store.json` (e.g., `/Emus/tg5040/DC.pak/`)
- Schema validation via `validate.ts` before trust

### Drive Detection

Platform-specific via `#[cfg]`-gated modules:
- `drives/macos.rs` (265 lines) — `df` + `diskutil` parsing
- `drives/windows.rs` — PowerShell `Get-Volume`
- `drives/linux.rs` — `lsblk` JSON parsing

Shared `DriveDetector` trait with `list()` method for testability.

### WiFi

Platform-specific modules:
- `wifi/macos.rs` (345 lines) — 3-tier fallback: `airport` → `system_profiler` → `current_ssid()`
- `wifi/windows.rs` — `netsh wlan show networks` parsing
- `wifi/linux.rs` — `nmcli` parsing

Configuration written to `<sd_root>/wifi.txt` (one `SSID:PASSWORD` per line, `#` comments).

### BIOS Installation

File: `bios.rs` (369 lines), `BiosInstaller.tsx`

- Catalog of known BIOS files with target paths
- Status check (present/missing) per BIOS entry
- `install_bios_from_bytes`: sanitizes filenames, canonicalizes target paths (symlink-race guard)

### Version Tracking

- **Write side**: Installer writes `minui.txt` to SD root: `{fork_name} {version}`
- **Read side**: `version/mod.rs` reads `minui.txt` back, parses with `semver`
- **Packages**: Read `Tools/*/version.txt` (included in archives)

### Health Check

File: `health.rs` — filesystem integrity, free space, presence of MinUI directories.

## Frontend Navigation

`App.tsx` uses state-based navigation (no router):

```
"home" ──► Home.tsx (drive selection, install)
"store" ──► PackageStore.tsx
"wifi" ──► WifiWizard.tsx
"bios" ──► BiosInstaller.tsx
"settings" ──► Settings.tsx (fork selection)
```

## Key Design Decisions

1. **Single-struct IPC**: All Tauri commands accept one `#[derive(Deserialize)]` struct, avoiding 4-9 individual params
2. **Platform modules**: `drives/{macos,linux,windows}.rs` and `wifi/{macos,linux,windows}.rs` use `#[cfg]`-gated modules, not traits (except `DriveDetector` for testability)
3. **No router**: State-based navigation (`"home" | "store" | "wifi" | "bios" | "settings"`), no React Router
4. **Session cache**: Release data and registry both use module-level caches with TTL
5. **CancellationToken**: Background installs use Tokio cancellation for clean abort
6. **Fork system**: `ForkConfig` interface with presets + custom `owner/repo` input, persisted to `localStorage`
