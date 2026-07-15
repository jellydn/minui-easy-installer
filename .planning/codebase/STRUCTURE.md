# Directory Structure

```
.
├── README.md                   # Project overview, download links
├── AGENTS.md                   # AI coding agent instructions
├── DESIGN.md                   # Design decisions and rationale
├── LICENSE                     # MIT
├── package.json                # Frontend dependencies & scripts
├── bun.lock                    # Bun lockfile
├── vite.config.ts              # Vite bundler config
├── vitest.config.ts            # Vitest test runner config
├── vitest.setup.ts             # Test setup (jsdom, testing-library)
├── tsconfig.json               # TypeScript config
├── .eslintrc.cjs               # ESLint rules
├── .oxfmtrc.json               # oxfmt formatter config
├── prek.toml                   # Pre-commit hook config
├── justfile                    # Task runner commands
├── skills-lock.json            # Skills version lock
│
├── .github/
│   └── workflows/
│       ├── rust.yml            # Rust CI: fmt, clippy, test (ubuntu)
│       ├── build.yml           # Build CI: macOS, Windows, Linux compile
│       ├── release.yml         # Release: tag-triggered DMG/MSI/EXE build
│       ├── react-doctor.yml    # React Doctor health check
│       └── update-registry.yml # Package registry auto-update cron
│
├── .planning/
│   └── codebase/               # Codebase map (this directory)
│       ├── STACK.md
│       ├── INTEGRATIONS.md
│       ├── ARCHITECTURE.md
│       ├── STRUCTURE.md
│       ├── CONVENTIONS.md
│       ├── TESTING.md
│       └── CONCERNS.md
│
├── src-tauri/                  # ── Rust Backend ──
│   ├── Cargo.toml              # Rust dependencies
│   ├── Cargo.lock
│   ├── tauri.conf.json         # Tauri app config, CSP, window settings
│   ├── build.rs                # Tauri build script
│   ├── entitlements.plist      # macOS entitlements (JIT, USB, network, file access)
│   ├── icons/                  # App icons (icns, ico, iconset)
│   ├── capabilities/
│   │   └── default.json        # Tauri v2 capability permissions
│   └── src/
│       ├── main.rs             # Binary entry point → calls lib::run()
│       ├── lib.rs              # Library root: module declarations, all Tauri commands
│       ├── install.rs          # Install orchestration (512 lines)
│       ├── pipeline.rs         # InstallSession: temp dirs, file copy pipeline
│       ├── download.rs         # Streaming HTTP downloads with checksums
│       ├── extract.rs          # ZIP extraction with path traversal guards
│       ├── package.rs          # Package detection, update check
│       ├── bios.rs             # BIOS catalog, status, install (369 lines)
│       ├── health.rs           # SD card health checks
│       ├── validate.rs         # Post-install validation (420 lines)
│       ├── fs_utils.rs         # Filesystem utilities (copy_dir_recursive, free space)
│       ├── platform.rs         # Platform detection helpers
│       ├── drives.rs           # Drive detection dispatcher
│       ├── drives/
│       │   ├── macos.rs        # macOS: df + diskutil (265 lines)
│       │   ├── windows.rs      # Windows: PowerShell Get-Volume
│       │   └── linux.rs        # Linux: lsblk JSON
│       ├── wifi.rs             # WiFi config write + dispatcher
│       ├── wifi/
│       │   ├── macos.rs        # macOS: airport → system_profiler → current_ssid (345 lines)
│       │   ├── windows.rs      # Windows: netsh wlan
│       │   └── linux.rs        # Linux: nmcli
│       ├── version/
│       │   ├── mod.rs          # Version checking & comparison
│       │   └── tests.rs        # Version unit tests (369 lines)
│       ├── *_tests.rs          # Rust test files (7 files)
│       └── lib_tests.rs        # Tauri command contract tests (393 lines)
│
├── src/                        # ── React Frontend ──
│   ├── main.tsx                # React entry point
│   ├── App.tsx                 # Root component, state-based navigation
│   ├── Home.tsx                # Home screen: drive/device selection, install
│   ├── PackageStore.tsx        # Package store: browse, search, install (266 lines)
│   ├── BiosInstaller.tsx       # BIOS file upload and status
│   ├── WifiWizard.tsx          # WiFi configuration wizard
│   ├── DriveSelector.tsx       # SD card drive picker
│   ├── DeviceSelector.tsx      # Retro handheld device picker
│   ├── InstallProgress.tsx     # Real-time install progress log
│   ├── Settings.tsx            # Fork selection (presets + custom)
│   ├── ConfirmDialog.tsx       # Overlay modal for destructive operations
│   ├── FormatConfirmDialog.tsx # Format confirmation dialog
│   ├── ValidationReport.tsx    # Post-install validation report
│   ├── HealthCheck.tsx         # SD card health status
│   ├── PackageCard.tsx         # Individual package card in store
│   ├── styles.css              # All CSS (no framework)
│   ├── contexts/
│   │   └── ForkContext.tsx     # Fork selection state (localStorage persisted)
│   ├── hooks/
│   │   ├── useForkInstall.ts   # Install orchestration hook (425 lines)
│   │   ├── useVersionCheck.ts  # Release/update version checking
│   │   ├── useMountEffect.ts   # useEffect on mount helper
│   │   └── useScrollToBottom.ts # Auto-scroll for progress log
│   └── types/
│       ├── device.ts           # Device profiles & platform mapping
│       ├── fork.ts             # ForkConfig, presets, URL building
│       ├── release.ts          # GitHub release fetching & parsing
│       ├── package.ts          # Package registry fetch, RegistryCache
│       ├── install.ts          # Install/cancel IPC functions
│       ├── drive.ts            # RemovableDrive types & formatting
│       ├── bios.ts             # BIOS catalog types
│       ├── validate.ts         # Registry schema validation
│       ├── version.ts          # Version parsing
│       ├── errors.ts           # Error classification
│       ├── device-install-map.json  # Device-to-platform mapping
│       ├── store.json          # Package store schema
│       └── *.test.ts           # TypeScript tests (10 files)
│
├── assets/                     # Brand assets
│   ├── banner.svg              # README banner (1200×300)
│   └── logo.svg                # Circular logo mark
│
└── icons/
    └── icon.svg                # App icon source (512px)
```
