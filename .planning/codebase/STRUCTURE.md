# Directory Structure

```
minui-installer/
├── src/                          # Frontend (TypeScript + React)
│   ├── main.tsx                  # Entry point — mounts App
│   ├── App.tsx                   # AppShell with state-based navigation
│   ├── Home.tsx                  # Main install screen + update-all
│   ├── Home.test.tsx             # Home component tests
│   ├── PackageStore.tsx          # Package browser + per-pak install
│   ├── PackageStore.test.tsx     # PackageStore tests
│   ├── PackageCard.tsx           # Individual package card in store
│   ├── InstallProgress.tsx       # Progress bar + event log
│   ├── DriveSelector.tsx         # Removable drive picker
│   ├── DriveSelector.test.tsx    # DriveSelector tests
│   ├── DeviceSelector.tsx        # Handheld device picker
│   ├── WifiWizard.tsx            # WiFi config UI
│   ├── WifiWizard.test.tsx       # WifiWizard tests
│   ├── BiosInstaller.tsx         # BIOS catalog + install UI
│   ├── BiosInstaller.test.tsx    # BiosInstaller tests
│   ├── Settings.tsx              # App settings
│   ├── Settings.test.tsx         # Settings tests
│   ├── ConfirmDialog.tsx         # Overlay confirmation modal
│   ├── FormatConfirmDialog.tsx   # Format confirmation modal
│   ├── ValidationReport.tsx      # Post-install validation UI
│   ├── styles.css                # Global styles (no CSS framework)
│   ├── contexts/
│   │   └── ForkContext.tsx       # ForkProvider + useFork hook
│   ├── hooks/
│   │   ├── useForkInstall.ts     # Install orchestration hook
│   │   ├── useForkInstall.test.ts
│   │   ├── useVersionCheck.ts    # Version comparison hook
│   │   ├── useVersionCheck.test.ts
│   │   ├── useMountEffect.ts     # Mount-only useEffect
│   │   └── useScrollToBottom.ts  # Auto-scroll log
│   └── types/
│       ├── install.ts            # Install IPC + startInstallAndWait
│       ├── install.test.ts       # Install type tests
│       ├── package.ts            # Package registry + install
│       ├── package.test.ts       # Package type tests
│       ├── release.ts            # GitHub release fetching
│       ├── release.test.ts       # Release fetching tests
│       ├── device.ts             # Device profiles
│       ├── device.test.ts        # Device profile tests
│       ├── fork.ts               # Fork configuration types
│       ├── fork.test.ts          # Fork type tests
│       ├── errors.ts             # Error type + classification
│       ├── bios.ts               # BIOS catalog types
│       ├── bios.test.ts          # BIOS type tests
│       ├── validate.ts           # Validation types + IPC
│       ├── validate.test.ts      # Validation type tests
│       ├── version.ts            # Version check types
│       ├── version.test.ts       # Version type tests
│       ├── drive.ts              # RemovableDrive type
│       ├── drive.test.ts         # Drive type tests
│       └── store.json            # Bundled package registry fallback
├── src-tauri/                    # Backend (Rust)
│   ├── Cargo.toml                # Rust dependencies
│   ├── Cargo.lock                # Dependency lockfile
│   ├── tauri.conf.json           # Tauri config (CSP, bundle ID)
│   ├── build.rs                  # Tauri build script
│   ├── capabilities/
│   │   └── default.json          # Tauri v2 permissions
│   ├── icons/                    # App icons
│   └── src/
│       ├── main.rs               # Entry point
│       ├── lib.rs                # Tauri commands + generate_handler!
│       ├── install.rs            # Install flow (base/extras/ROMs)
│       ├── install_tests.rs      # Install unit tests (790 lines)
│       ├── pipeline.rs           # Download → extract → copy
│       ├── download.rs           # HTTP download + streaming + checksum
│       ├── extract.rs            # ZIP extraction
│       ├── drives.rs             # Platform-specific drive detection
│       ├── drives/
│       │   └── macos.rs          # macOS diskutil parsing helpers
│       ├── drives_tests.rs       # Drive unit tests
│       ├── package.rs            # Package install logic
│       ├── wifi.rs               # WiFi config + network scanning
│       ├── bios.rs               # BIOS catalog + install
│       ├── bios_tests.rs         # BIOS unit tests (310 lines)
│       ├── health.rs             # SD card health checks
│       ├── validate.rs           # Post-install validation
│       ├── version/
│       │   ├── mod.rs            # Version detection + comparison
│       │   └── tests.rs          # Version unit tests
│       ├── platform.rs           # Device → platform mapping
│       └── fs_utils.rs           # Dir copy, disk space, canonicalize
├── scripts/                      # Automation scripts (Bun)
│   ├── update-registry.ts        # Auto-update package versions from GitHub
│   ├── discover-packages.ts      # Discover new MinUI paks from contributors
│   └── shared.ts                 # Shared utilities (repoSlug, fetchApi)
├── .github/workflows/            # CI/CD
│   ├── build.yml                 # Tauri compile check (macOS + Windows)
│   ├── release.yml               # DMG/MSI build on v* tags
│   ├── rust.yml                  # Rust fmt/clippy/test
│   ├── react-doctor.yml          # React best-practices linting
│   └── update-registry.yml       # Daily cron: auto-update store.json versions
├── .planning/                    # Project documentation
│   ├── codebase/                 # Architecture docs (these files)
│   └── handoffs/                 # Cross-session handoff notes
├── .changeset/                   # Changeset versioning
├── tasks/                        # PRD documents
├── plans/                        # Implementation plans
├── icons/                        # macOS .icns icon
├── package.json                  # Frontend dependencies + scripts
├── bun.lock                      # Bun lockfile
├── tsconfig.json                 # TypeScript config
├── vite.config.ts                # Vite config
├── vitest.config.ts              # Vitest config
├── vitest.setup.ts               # Vitest setup (jest-dom)
├── .eslintrc.cjs                 # ESLint config
├── .oxfmtrc.json                 # Oxfmt formatter config
├── .editorconfig                 # Editor settings
├── justfile                      # Build shortcuts
├── prek.toml                     # Pre-commit hooks
├── README.md                     # Project overview
├── DESIGN.md                     # Design decisions
├── AGENTS.md                     # Agent instructions
├── LICENSE                       # MIT license
└── install-guide.txt             # User-facing install instructions
```

## File Counts

| Area | Files |
|------|-------|
| Frontend components | 15 `.tsx` |
| Frontend tests | 17 `.test.{ts,tsx}` |
| Frontend types | 10 `.ts` |
| Frontend hooks | 6 `.ts` |
| Rust source | 16 `.rs` (excluding tests) |
| Rust tests | 4 `*_tests.rs` / `tests.rs` |
| CI workflows | 5 `.yml` |
| Scripts | 3 `.ts` |

## Naming Conventions

| Layer | Pattern | Example |
|-------|---------|---------|
| Frontend components | `PascalCase.tsx` | `DriveSelector.tsx` |
| Frontend tests | `PascalCase.test.tsx` | `DriveSelector.test.tsx` |
| Frontend hooks | `useCamelCase.ts` | `useForkInstall.ts` |
| Frontend types | `kebab-case.ts` | `device.ts` |
| Rust modules | `snake_case.rs` | `fs_utils.rs` |
| Rust tests | `snake_case_tests.rs` or inline `#[cfg(test)]` | `drives_tests.rs` |
| Rust submodules | `mod/filename.rs` | `drives/macos.rs` |
