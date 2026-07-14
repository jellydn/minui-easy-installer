# Structure

## Directory Layout

```
.
├── src/                          # React frontend (TypeScript)
│   ├── main.tsx                  # Entry point
│   ├── App.tsx                   # Shell with state-based navigation
│   ├── styles.css                # All styles (no CSS framework)
│   ├── Home.tsx                  # Main screen: device/drive select, install
│   ├── PackageStore.tsx          # Browse and install community packages
│   ├── PackageCard.tsx           # Individual package display card
│   ├── DeviceSelector.tsx        # Device dropdown
│   ├── DriveSelector.tsx         # SD card dropdown with refresh
│   ├── WifiWizard.tsx            # WiFi scan + config
│   ├── BiosInstaller.tsx         # BIOS file upload + install
│   ├── Settings.tsx              # Fork selection (presets + custom)
│   ├── HealthCheck.tsx           # SD card health report
│   ├── ValidationReport.tsx      # Post-install validation results
│   ├── InstallProgress.tsx       # Real-time install progress log
│   ├── ConfirmDialog.tsx         # Overlay modal for write confirmation
│   ├── FormatConfirmDialog.tsx   # Format confirmation modal
│   ├── contexts/
│   │   └── ForkContext.tsx       # Fork selection state + persistence
│   ├── hooks/
│   │   ├── useForkInstall.ts     # Install orchestration hook (largest: 399 lines)
│   │   ├── useVersionCheck.ts    # Version polling hook
│   │   ├── useMountEffect.ts     # useEffect on mount
│   │   └── useScrollToBottom.ts  # Auto-scroll progress log
│   └── types/
│       ├── device.ts             # Device profiles (16 devices)
│       ├── drive.ts              # RemovableDrive type, formatting
│       ├── install.ts            # Install types, IPC wrappers
│       ├── package.ts            # Package registry, install, cache (420 lines)
│       ├── release.ts            # GitHub release fetching
│       ├── version.ts            # Version parsing and checking
│       ├── bios.ts               # BIOS catalog types, IPC wrappers
│       ├── validate.ts           # Post-install validation
│       ├── fork.ts               # Fork config, presets, URL building
│       ├── errors.ts             # AppError type, error classification
│       └── store.json            # Local dev registry fallback
│
├── src-tauri/                    # Rust backend
│   ├── Cargo.toml                # Rust dependencies
│   ├── tauri.conf.json           # Tauri config, CSP, bundle
│   ├── build.rs                  # Tauri build script
│   ├── capabilities/default.json # Tauri capabilities
│   ├── icons/                    # App icons
│   └── src/
│       ├── main.rs               # Entry point (calls lib::run)
│       ├── lib.rs                # 17 Tauri commands + contract tests (619 lines)
│       ├── install.rs            # Install orchestration (~290 lines production)
│       ├── install_tests.rs      # Install tests (789 lines)
│       ├── pipeline.rs           # Download→Extract→Copy pipeline + path safety
│       ├── download.rs           # Streaming archive download + checksum
│       ├── extract.rs            # Archive extraction
│       ├── drives.rs             # Removable drive detection (418 lines)
│       ├── drives_tests.rs       # Drive detection tests (366 lines)
│       ├── wifi.rs               # WiFi scan + config (480 lines)
│       ├── bios.rs               # BIOS catalog, status, install (668 lines)
│       ├── package.rs            # Package install, detect, update
│       ├── health.rs             # SD card health checks
│       ├── validate.rs           # Post-install validation
│       ├── fs_utils.rs           # copy_dir_recursive, disk space, canonicalize
│       ├── platform.rs           # Device base item mappings
│       └── version/
│           ├── mod.rs            # Version parsing + detection
│           └── tests.rs          # Version tests
│
├── .github/workflows/
│   ├── rust.yml                  # Rust CI: fmt, clippy, test
│   └── react-doctor.yml          # React Doctor scan
│
├── .planning/
│   ├── codebase/                 # This codebase map
│   └── handoffs/                 # Session handoff documents
│
├── scripts/ralph/                # Autonomous agent loop
├── tasks/                        # PRDs and task definitions
├── .changeset/                   # Changeset for version tracking
├── icons/                        # macOS .icns
├── AGENTS.md                     # Agent instructions
├── DESIGN.md                     # Design guidelines
├── package.json                  # Frontend deps + scripts
├── bun.lock                      # Bun lockfile
├── vitest.config.ts              # Test runner config
├── vite.config.ts                # Vite config
├── tsconfig.json                 # TypeScript config
├── prek.toml                     # Pre-commit hooks
├── justfile                      # Task runner (just)
└── install-guide.txt             # Upstream MinUI install guide (reference)
```

## File Size Summary

| Largest Rust Files | Lines |
|--------------------|-------|
| `install_tests.rs` | 789 |
| `bios.rs`          | 668 |
| `lib.rs`           | 619 |
| `wifi.rs`          | 480 |
| `drives.rs`        | 418 |

| Largest TS/TSX Files | Lines |
|----------------------|-------|
| `types/package.ts`   | 420 |
| `hooks/useForkInstall.ts` | 399 |
| `types/release.test.ts` | 315 |
| `hooks/useVersionCheck.test.ts` | 272 |
| `PackageStore.tsx`   | 266 |

## Naming Conventions

### Rust

- **Test files**: `#[path = "module_tests.rs"] mod tests;` (following `version/tests.rs` pattern)
- **Module files**: One module per file, `mod` declarations in `lib.rs`
- **Types**: PascalCase (`InstallSession`, `CancellationToken`)
- **Functions**: snake_case (`copy_dir_recursive`, `create_target_within`)
- **Platform-gated**: `#[cfg(target_os = "macos")]`, `#[cfg(not(target_os = "macos"))]`

### TypeScript

- **Components**: PascalCase (`PackageStore`, `DeviceSelector`)
- **Hooks**: camelCase with `use` prefix (`useForkInstall`, `useVersionCheck`)
- **Types/Interfaces**: PascalCase (`DeviceProfile`, `RemovableDrive`)
- **Test files**: `*.test.ts` or `*.test.tsx` colocated with source
