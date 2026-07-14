# Directory Structure

## Top-Level Layout

```
minui-easy-installer/
├── src/                    # React frontend (TypeScript)
├── src-tauri/              # Rust backend (Tauri v2)
├── icons/                  # App icon resources
├── scripts/                # Build/automation scripts
├── .planning/              # Codebase documentation & handoffs
├── .changeset/             # Changeset tracking
├── .github/workflows/      # CI workflows
├── package.json            # npm/bun dependencies
├── bun.lock                # Bun lockfile
├── tsconfig.json           # TypeScript config
├── vite.config.ts          # Vite build config
├── vitest.config.ts        # Vitest test config
├── vitest.setup.ts         # Test environment setup
├── .eslintrc.cjs           # ESLint config
├── .oxfmtrc.json           # oxfmt config
├── prek.toml               # Pre-commit hooks config
├── justfile                # Task runner
├── .editorconfig           # Editor settings
├── index.html              # HTML entry point
├── AGENTS.md               # Agent/developer guide
├── DESIGN.md               # Design documentation
├── LICENSE                 # License file
├── README.md               # Project readme
└── install-guide.txt       # User-facing install guide
```

## Frontend (`src/`)

```
src/
├── main.tsx                # Entry point — renders App into #root
├── App.tsx                 # Root component, state-based navigation
├── Home.tsx                # Home screen (device/drive selection, install flow)
├── PackageStore.tsx        # Package store screen
├── PackageCard.tsx         # Individual package display card
├── DeviceSelector.tsx      # Device picker UI
├── DriveSelector.tsx       # SD card picker UI
├── InstallProgress.tsx     # Install progress display (phases, log, results)
├── ValidationReport.tsx    # Post-install validation report
├── ConfirmDialog.tsx       # Overlay modal for write confirmation
├── FormatConfirmDialog.tsx # Format confirmation dialog (MVP: unused)
├── WifiWizard.tsx          # WiFi network scanner and config UI
├── BiosInstaller.tsx       # BIOS file selection and install UI
├── HealthCheck.tsx         # SD card health display
├── Settings.tsx            # Settings screen (fork selection)
├── styles.css              # Global styles (all components, no framework)
│
├── contexts/
│   └── ForkContext.tsx     # Fork selection context provider
│
├── hooks/
│   ├── useForkInstall.ts   # Install orchestration hook (fork-aware)
│   ├── useVersionCheck.ts  # Version checking hook
│   ├── useMountEffect.ts   # Strict-mode-safe mount effect
│   └── useScrollToBottom.ts # Scroll-tracking hook
│
├── types/
│   ├── device.ts           # Device profiles (18+ devices)
│   ├── drive.ts            # RemovableDrive type, formatSize()
│   ├── install.ts          # Install state types
│   ├── package.ts          # Package registry types and fetch logic
│   ├── release.ts          # GitHub release parsing
│   ├── bios.ts             # BIOS catalog types
│   ├── version.ts          # Version parsing types
│   ├── validate.ts         # Validation result types
│   ├── errors.ts           # Error classification
│   ├── fork.ts             # Fork type definitions
│   └── store.json          # Bundled package store fallback data
│
└── *.test.ts(x)            # Co-located test files (18 files, ~2585 lines)
```

## Backend (`src-tauri/`)

```
src-tauri/
├── Cargo.toml              # Rust dependencies
├── Cargo.lock              # Locked dependency versions
├── build.rs                # Tauri build script
├── tauri.conf.json         # Tauri app configuration
│
├── capabilities/
│   └── default.json        # Tauri v2 capability permissions
│
├── gen/schemas/            # Generated JSON schemas
│   ├── capabilities.json
│   ├── acl-manifests.json
│   ├── macOS-schema.json
│   └── desktop-schema.json
│
├── icons/
│   └── icon.icns           # macOS app icon
│
└── src/                    # Rust source code
    ├── main.rs             # Entry point
    ├── lib.rs              # Tauri command registration
    ├── install.rs          # Install flow (1168 lines — largest file)
    ├── pipeline.rs         # Pipeline abstraction + path validation
    ├── download.rs         # Streaming HTTP downloads
    ├── extract.rs          # ZIP archive extraction
    ├── drives.rs           # Platform-specific drive detection (743 lines)
    ├── health.rs           # SD card health checks
    ├── validate.rs         # Post-install validation
    ├── package.rs          # Community package management
    ├── wifi.rs             # WiFi scanning & config
    ├── bios.rs             # BIOS file management (667 lines)
    ├── fs_utils.rs         # Filesystem utilities
    ├── platform.rs         # Device platform mappings
    └── version/
        ├── mod.rs          # Version parsing
        └── tests.rs        # Version parsing tests
```

## Documentation (`.planning/`)

```
.planning/
├── codebase/               # Codebase map (this directory)
│   ├── STACK.md
│   ├── ARCHITECTURE.md
│   ├── STRUCTURE.md
│   ├── CONVENTIONS.md
│   ├── TESTING.md
│   ├── INTEGRATIONS.md
│   └── CONCERNS.md
├── fork-support/
│   └── plan.md             # Fork support implementation plan
└── handoffs/
    ├── 2026-06-13-fix-store-install-platform.md
    └── 2026-06-13-per-device-extras-install.md
```

## Key File Sizes (Complexity Indicators)

| File | Lines | Area |
|------|-------|------|
| `src-tauri/src/install.rs` | 1,168 | Install flow + tests |
| `src-tauri/src/drives.rs` | 743 | Drive detection |
| `src-tauri/src/bios.rs` | 667 | BIOS management |
| `src/types/package.ts` | 418 | Package registry |
| `src/hooks/useForkInstall.ts` | 399 | Install orchestration |
| `src/PackageStore.tsx` | 266 | Store UI |
