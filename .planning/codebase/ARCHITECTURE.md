# Architecture

## Pattern: Tauri v2 Desktop App

```
┌─────────────────────────────────────┐
│            React Frontend            │
│  (src/) TypeScript + CSS            │
│                                      │
│  App.tsx → state-based navigation   │
│  ├── Home (install/update)          │
│  ├── PackageStore (browse/install)  │
│  ├── WifiWizard (scan/config)       │
│  ├── BiosInstaller (upload/copy)    │
│  └── Settings (fork selection)      │
│                                      │
│  invoke("command", args) ─────────┐ │
│  listen("event", callback) ←────┐ │ │
└──────────────────────────────────┼─┼─┘
                                   │ │
                    Tauri IPC Bridge │
                                   │ │
┌──────────────────────────────────┼─┼─┐
│            Rust Backend           │ │ │
│  (src-tauri/src/)                │ │ │
│                                   │ │ │
│  lib.rs — 17 Tauri commands ─────┘ │ │
│  ├── get_removable_drives          │ │
│  ├── start_install                 │ │
│  ├── cancel_install                │ │
│  ├── validate_installation         │ │
│  ├── install_package               │ │
│  ├── write_wifi_config             │ │
│  ├── scan_wifi_networks            │ │
│  ├── install_bios_file             │ │
│  ├── check_sd_card_health          │ │
│  └── ...                           │ │
│                                     │ │
│  Emit events ───────────────────────┘ │
│  ├── install-progress                │
│  ├── install-complete                │
│  └── install-error                   │
└───────────────────────────────────────┘
```

## Install Pipeline

The core architecture follows a **Pipeline** pattern: Download → Extract → Copy.

```
Pipeline::run(label, url, checksum, copy_fn, progress, cancel, session)
    │
    ├── 1. DOWNLOAD (streaming)
    │   download_archive_streaming() → TempDir
    │   - Checks CancellationToken before starting
    │   - Verifies SHA-256 checksum if provided
    │   - Emits byte-level progress (frontend not yet wired)
    │
    ├── 2. EXTRACT
    │   extract_archive_into() → TempDir
    │   - Checks CancellationToken before starting
    │   - Extracts to temp dir (never directly to SD card)
    │
    └── 3. COPY
        copy_fn(extracted_path) → u32 (files_copied)
        - copy_base_files: shared items + device folder
        - copy_extras_files: extras platform folder (non-fatal on failure)
        - copy_dir_recursive: walks tree, skips preserved folders
```

### InstallSession

`InstallSession` owns all `TempDir` handles for the lifetime of an install:

```rust
pub struct InstallSession {
    _base_archive: Option<TempDir>,
    _base_extracted: Option<TempDir>,
    _extras_archive: Option<TempDir>,
    _extras_extracted: Option<TempDir>,
    _package_archive: Option<TempDir>,
    _package_extracted: Option<TempDir>,
}
```

When `InstallSession` drops, all temp directories are cleaned up atomically — even on cancellation or error.

### Cancellation

- Single `InstallRegistry` (global `Arc<Mutex<Option<CancellationToken>>>`)
- New install cancels any prior install (replaces token)
- `start_install` spawns background task, returns immediately with `"current"`
- `cancel_install` triggers the token; pipeline checks at phase boundaries
- Frontend receives `install-error` with "Install cancelled" on cancellation

## Frontend Architecture

### Navigation (State-based)

`App.tsx` uses a `Screen` type with 5 states:

| Screen | Component | Requires |
|--------|-----------|----------|
| `home` | `Home` | — |
| `store` | `PackageStore` | Device + Drive selected |
| `wifi` | `WifiWizard` | Drive selected |
| `bios` | `BiosInstaller` | Drive selected |
| `settings` | `Settings` | — |

### State Management

- **ForkContext** (`src/contexts/ForkContext.tsx`): Selected MinUI fork (default or custom), provides `fork` + `setFork`
- **Local state**: Each screen manages its own UI state (device selection, drive selection, install progress)
- **Custom hooks**: `useForkInstall` (install orchestration), `useVersionCheck` (version polling), `useMountEffect` (mount-side-effect), `useScrollToBottom` (progress log auto-scroll)

### Data Flow

```
User action → invoke Tauri command → Rust backend → emit event → frontend listener → state update → re-render
```

## Security Patterns

### Path Containment (create_target_within)

`pipeline.rs::create_target_within` validates that package install paths stay within the SD card:
1. Canonicalize SD card root
2. Walk up target parent to first existing ancestor, canonicalize
3. Verify ancestor is within SD card root **before** creating directories
4. Create directories
5. Re-canonicalize and re-verify **after** creation (symlink race guard)
6. On violation: best-effort cleanup of newly-created directories only

### Symlink Safety

- `fs_utils::copy_dir_recursive` uses `fs::copy` which dereferences symlinks (copies target contents, not symlinks)
- `canonicalize_existing_ancestor` resolves symlinks, so path-escaping symlinks are detected
- BIOS install sanitizes subdir/filename (no traversal, NUL, or path separators)

### Input Validation

- Registry data validated via `src/types/validate.ts` before use
- Extras platform name sanitized (alphanumeric + hyphens only)
- Package repository URLs must start with `https://github.com/`

## Device Platform Mapping

16 device profiles in `src/types/device.ts`:
- Each device has `id`, `platform` (base archive folder), `extrasPlatform` (extras archive folder)
- Platform names can differ: e.g. TrimUI uses `trimui` for base, `tg5040` for extras
- `src-tauri/src/platform.rs` maps platforms to base archive items (mostly folders, `em_ui.sh` for M17)
