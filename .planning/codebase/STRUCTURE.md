# Directory Structure

```
.
├── src/                          # React 18 frontend (TypeScript)
│   ├── main.tsx                  # React entry point
│   ├── App.tsx                   # State-based navigation (5 screens)
│   ├── styles.css                # All styles (no CSS framework)
│   ├── Home.tsx                  # Main install flow (device + drive + install button)
│   ├── PackageStore.tsx          # Package browser with install/update
│   ├── PackageCard.tsx           # Single package card in store
│   ├── BiosInstaller.tsx         # BIOS catalog + install UI
│   ├── WifiWizard.tsx            # WiFi network scanner + config
│   ├── HealthCheck.tsx           # SD card health diagnostics (auto-run)
│   ├── Settings.tsx              # Fork selection + settings
│   ├── InstallProgress.tsx       # Real-time install progress display
│   ├── ValidationReport.tsx      # Post-install validation results
│   ├── DeviceSelector.tsx        # Device dropdown (8 handhelds)
│   ├── DriveSelector.tsx         # Removable drive list
│   ├── FormatConfirmDialog.tsx   # Format confirmation modal
│   ├── ConfirmDialog.tsx         # Generic confirmation modal (write ops)
│   ├── types/                    # TypeScript types + IPC wrappers
│   │   ├── device.ts             # Device profiles, getDeviceProfile()
│   │   ├── device-install-map.json  # Platform mapping per device
│   │   ├── drive.ts              # RemovableDrive, formatSize()
│   │   ├── install.ts            # InstallOptions, startInstallAndWait(), cancelInstall()
│   │   ├── release.ts            # MinUIRelease, fetchMinUIRelease()
│   │   ├── package.ts            # PackageRegistry, installPackage(), fetchPackageRegistry()
│   │   ├── fork.ts               # ForkConfig, FORK_PRESETS (official, minuitsp)
│   │   ├── validate.ts           # ValidationResult, validateInstallation()
│   │   ├── version.ts            # VersionCheckResult, version parsing
│   │   ├── bios.ts               # BiosEntry, bios status types
│   │   ├── errors.ts             # errorMessage(), asError(), classifyError()
│   │   └── store.json            # Package store test data
│   ├── lib/
│   │   └── InstallOrchestrator.ts    # Vanilla TS state machine (no React)
│   ├── hooks/
│   │   ├── useForkInstall.ts         # Thin React adapter (~129 lines)
│   │   ├── useVersionCheck.ts        # Version check hook
│   │   ├── useMountEffect.ts         # Mount/unmount lifecycle
│   │   └── useScrollToBottom.ts      # Auto-scroll utility
│   └── contexts/
│       └── ForkContext.tsx        # Fork selection React context
│
├── src-tauri/                    # Rust backend (Tauri v2)
│   ├── Cargo.toml                # Rust dependencies
│   ├── tauri.conf.json           # Tauri config (CSP, window, bundle)
│   ├── build.rs                  # Tauri build script
│   ├── capabilities/default.json # Tauri v2 capability permissions
│   └── src/
│       ├── main.rs               # Entry point → lib::run()
│       ├── lib.rs                # 20 Tauri commands + app setup
│       ├── lib_tests.rs          # IPC contract tests (17 commands)
│       ├── install.rs            # install_minui_with_cancel(), copy_base_files()
│       ├── install_tests.rs      # Install function tests
│       ├── install_copy_tests.rs # Copy function tests
│       ├── install_extras_tests.rs # Extras copy tests
│       ├── install_manager.rs    # EventDispatcher trait + InstallManager
│       ├── install_manager_tests.rs  # Manager tests (poison, cancel, smoke)
│       ├── pipeline.rs           # Pipeline::run(), InstallSession, create_target_within()
│       ├── download.rs           # Streaming archive downloads + checksum
│       ├── extract.rs            # Archive extraction
│       ├── package.rs            # Package install, detect, update check
│       ├── health.rs             # SD card health check (speed, fs, PAKs)
│       ├── wifi.rs               # WiFi scanning + config (airport + system_profiler)
│       ├── bios.rs               # BIOS catalog + install from bytes
│       ├── bios_tests.rs         # BIOS tests
│       ├── drives.rs             # Platform-specific drive detection
│       ├── validate.rs           # Post-install validation
│       ├── version/              # Version parsing + update checking
│       │   ├── mod.rs
│       │   └── tests.rs
│       ├── platform.rs           # Device platform mapping
│       └── fs_utils.rs           # copy_dir_recursive(), get_free_space(), canonicalize_existing_ancestor()
│
├── .planning/codebase/           # This codebase map (7 docs)
├── .planning/handoffs/           # Session handoff docs
├── plans/                        # Implementation plans (9 files)
├── scripts/ralph/                # Build system scripts
├── .github/workflows/            # CI (react-doctor.yml)
├── icons/                        # App icons
├── justfile                      # Task runner (check, fmt)
├── package.json                  # npm scripts + dependencies
├── bun.lock                      # Bun lockfile
├── tsconfig.json                 # TypeScript config
├── vite.config.ts                # Vite config
├── vitest.config.ts              # Vitest config
├── vitest.setup.ts               # Test env setup
├── prek.toml                     # Pre-commit hooks
├── .oxfmtrc.json                 # oxlint/oxfmt config
├── .editorconfig                 # Editor settings
├── DESIGN.md                     # UI design notes
├── LICENSE                       # MIT
└── README.md                     # Project readme with download links
```

## Naming Conventions

| Convention | Example |
|-----------|---------|
| Rust modules | `snake_case` files: `install_manager.rs`, `fs_utils.rs` |
| Rust test modules | `#[path = "foo_tests.rs"]` next to source |
| TypeScript components | `PascalCase`: `HealthCheck.tsx`, `PackageStore.tsx` |
| TypeScript tests | `ComponentName.test.tsx` co-located |
| TypeScript types | `camelCase` files: `device.ts`, `release.ts` |
| Tauri commands | `snake_case`: `check_sd_card_health` |
| IPC options structs | `camelCase` JSON: `#[serde(rename_all = "camelCase")]` |
| React hooks | `use` prefix: `useForkInstall`, `useVersionCheck` |
| React contexts | `PascalCase` + Provider: `ForkProvider` |
