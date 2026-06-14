# Codebase Structure

**Analysis Date:** 2026-06-14

## Directory Layout

```
minui-easy-installer/
├── src/                        # React frontend (TypeScript)
│   ├── types/                  # Domain types, API wrappers, data models
│   ├── *.tsx                   # UI components
│   ├── *.test.tsx              # Component tests (vitest)
│   └── styles.css              # Global styles
├── src-tauri/                  # Rust backend (Tauri v2)
│   ├── src/                    # Rust source modules
│   ├── capabilities/           # Tauri v2 capability definitions
│   ├── gen/                    # Tauri generated code
│   ├── icons/                  # Tauri app icons
│   ├── target/                 # Rust build artifacts (gitignored)
│   ├── Cargo.toml              # Rust dependencies
│   ├── Cargo.lock              # Rust dependency lockfile
│   ├── build.rs                # Rust build script
│   └── tauri.conf.json         # Tauri configuration
├── assets/                     # Static assets (logos, banners)
├── icons/                      # App icons (multi-resolution PNGs, ICO, ICNS)
├── scripts/                    # Utility scripts
│   └── ralph/                  # Ralph-related scripts
├── tasks/                      # Product requirement documents
├── .planning/                  # Planning and architecture docs
│   ├── codebase/               # Codebase documentation
│   └── handoffs/               # Session handoff notes
├── index.html                  # Vite HTML entry point
├── package.json                # Node.js dependencies and scripts
├── tsconfig.json               # TypeScript configuration
├── vite.config.ts              # Vite build configuration
├── vitest.config.ts            # Vitest test configuration
├── vitest.setup.ts             # Vitest setup (custom matchers)
├── justfile                    # Just task runner recipes
├── prek.toml                   # Pre-commit hook config
├── AGENTS.md                   # AI agent instructions
├── DESIGN.md                   # Design documentation
├── README.md                   # Project readme
└── LICENSE                     # License file
```

## Directory Purposes

**`src/`:**

- Purpose: React frontend — all UI components, styles, and TypeScript type/API modules
- Contains: `.tsx` components, `.ts` type definitions and API wrappers, `.css` styles, `.test.ts` and `.test.tsx` tests
- Key files: `App.tsx`, `Home.tsx`, `main.tsx`, `types/`

**`src/types/`:**

- Purpose: Domain types, Tauri IPC API wrappers, data models, validation logic, and test files
- Contains: TypeScript interfaces, type aliases, async `invoke()` wrapper functions, static data JSON, and co-located test files
- Key files: `device.ts`, `install.ts`, `release.ts`, `package.ts`, `version.ts`, `validate.ts`, `drive.ts`, `archive.ts`, `store.json`

**`src-tauri/`:**

- Purpose: Rust backend — Tauri app shell, all OS-level operations, IPC command handlers
- Contains: Rust source modules, Cargo project files, Tauri configuration, generated code
- Key files: `src/lib.rs`, `src/main.rs`, `src/install.rs`, `tauri.conf.json`, `Cargo.toml`

**`src-tauri/src/`:**

- Purpose: Rust source code — each module handles a specific domain (drives, download, extract, install, etc.)
- Contains: `.rs` source files with `#[tauri::command]` functions and domain logic
- Key files: `lib.rs` (command registry), `main.rs` (entry point), `install.rs` (core install flow), `download.rs`, `extract.rs`, `drives.rs`, `package.rs`, `validate.rs`, `version.rs`, `wifi.rs`, `fs_utils.rs`

**`assets/`:**

- Purpose: Static brand assets used in the UI
- Contains: SVG files (banner, logo)
- Key files: `banner.svg`, `logo.svg`

**`icons/`:**

- Purpose: Multi-resolution app icons for all platforms (macOS, Windows, Linux)
- Contains: PNG, ICO, ICNS, SVG icon files
- Key files: `icon.icns` (macOS), `icon.ico` (Windows), `icon.png` (Linux), `icon-master.png` (source)

**`scripts/`:**

- Purpose: Utility/build scripts
- Contains: Ralph-related tooling scripts
- Key files: `ralph/` (subdirectory)

**`tasks/`:**

- Purpose: Product requirement documents and feature specifications
- Contains: Markdown PRD files
- Key files: `prd-minui-easy-installer-package-store.md`

**`.planning/`:**

- Purpose: Architecture documentation and session handoff notes
- Contains: Markdown documentation files
- Key files: `codebase/`, `handoffs/`

## Key File Locations

**Entry Points:**

- `src/main.tsx`: React app bootstrap — mounts `<App />` into DOM
- `src/App.tsx`: Top-level component — screen routing (home, store, wifi), shared state
- `src-tauri/src/main.rs`: Rust binary entry point — calls `run()`
- `src-tauri/src/lib.rs`: Tauri app builder — registers all 16 command handlers

**Configuration:**

- `package.json`: Node.js project config — scripts (`dev`, `build`, `test`, `lint`, `typecheck`), dependencies
- `tsconfig.json`: TypeScript compiler configuration
- `vite.config.ts`: Vite build tool configuration
- `vitest.config.ts`: Vitest test runner configuration
- `src-tauri/tauri.conf.json`: Tauri app configuration (app name, window settings, permissions)
- `src-tauri/Cargo.toml`: Rust dependency and project configuration

**Core Logic:**

- `src/Home.tsx`: Main install/update workflow — device selection, drive selection, version check, install orchestration (507 lines)
- `src-tauri/src/install.rs`: Rust install flow — download, extract, copy base+extras, create ROM dirs (457 lines)
- `src/types/device.ts`: Device profile registry — 17 device profiles with platform mappings (156 lines)
- `src/types/package.ts`: Package store logic — registry fetching, package installation, update checking (278 lines)
- `src/types/release.ts`: GitHub release parsing — fetches and parses MinUI release metadata (117 lines)
- `src-tauri/src/download.rs`: Archive download with checksum verification
- `src-tauri/src/extract.rs`: Archive extraction with path traversal protection
- `src-tauri/src/drives.rs`: OS-level removable drive detection and formatting
- `src-tauri/src/validate.rs`: Post-install validation and SD card health checks
- `src-tauri/src/version.rs`: MinUI version detection from SD card filesystem
- `src-tauri/src/wifi.rs`: WiFi network scanning and config writing

**Testing:**

- `src/DriveSelector.test.tsx`: Drive selector component tests
- `src/Home.test.tsx`: Home screen component tests
- `src/PackageStore.test.tsx`: Package store component tests
- `src/WifiWizard.test.tsx`: WiFi wizard component tests
- `src/types/archive.test.ts`: Archive download/extract API tests
- `src/types/device.test.ts`: Device profile tests
- `src/types/device-install-map.test.ts`: Device install map tests
- `src/types/drive.test.ts`: Drive utility tests
- `src/types/install.test.ts`: Install API wrapper tests
- `src/types/package.test.ts`: Package store API tests
- `src/types/release.test.ts`: GitHub release parsing tests
- `src/types/validate.test.ts`: Validation API tests
- `src/types/version.test.ts`: Version check API tests
- `vitest.setup.ts`: Vitest global setup (custom DOM matchers)

## Naming Conventions

**Files:**

- React components: `PascalCase.tsx` — e.g., `Home.tsx`, `PackageStore.tsx`, `ConfirmDialog.tsx`
- TypeScript types/modules: `camelCase.ts` — e.g., `device.ts`, `install.ts`, `release.ts`
- Rust modules: `snake_case.rs` — e.g., `fs_utils.rs`, `download.rs`
- Test files: `*.test.ts` or `*.test.tsx` — co-located with source files
- Static data: `camelCase.json` — e.g., `store.json`, `device-install-map.json`

**Directories:**

- Frontend source: `src/` (lowercase)
- Types subdirectory: `src/types/` (lowercase)
- Backend source: `src-tauri/src/` (kebab-case)
- Planning docs: `.planning/codebase/` (lowercase)

**Exports:**

- Default exports for React components (`export default App`)
- Named exports for types and functions (`export interface DeviceProfile`, `export function getDeviceProfile`)
- Type-only exports preferred (`export type InstallPhase`)

## Where to Add New Code

**New Feature (Full-Stack):**

- Frontend component: `src/NewFeature.tsx`
- Frontend types/API: `src/types/newfeature.ts`
- Rust backend module: `src-tauri/src/newfeature.rs`
- Register command in: `src-tauri/src/lib.rs` (add to `mod` declarations and `generate_handler![]`)
- Tests: `src/NewFeature.test.tsx` and `src/types/newfeature.test.ts`

**New Device Profile:**

- Add entry to `DEVICE_PROFILES` array in `src/types/device.ts`

**New Install Path Rule:**

- Modify `InstallPathRules` interface in `src/types/device.ts` and update `DEFAULT_INSTALL_PATH_RULES`

**New Package Category:**

- Add variant to `PackageCategory` type in `src/types/package.ts`

**New Post-Install Check:**

- Add check logic in `src-tauri/src/validate.rs`
- Add `ValidationCheck` entry in validation result

**New Tauri Command:**

- Implement function in appropriate `src-tauri/src/*.rs` module
- Add `#[tauri::command]` wrapper in `src-tauri/src/lib.rs`
- Register in `generate_handler![]` macro in `src-tauri/src/lib.rs`
- Add TypeScript wrapper in `src/types/*.ts`

## Special Directories

**`src-tauri/target/`:**

- Purpose: Rust compilation artifacts and build cache
- Generated: Yes (by `cargo build`)
- Committed: No (gitignored)

**`src-tauri/gen/`:**

- Purpose: Tauri-generated bindings and TypeScript types
- Generated: Yes (by `tauri build` / `tauri dev`)
- Committed: Partially (depends on team convention)

**`node_modules/`:**

- Purpose: Node.js package dependencies
- Generated: Yes (by `bun install` / `npm install`)
- Committed: No (gitignored)

**`assets/`:**

- Purpose: Brand assets (SVG logos and banners)
- Generated: No (hand-crafted)
- Committed: Yes

**`icons/`:**

- Purpose: Multi-platform app icons derived from master icon
- Generated: Partially (some may be derived from `icon-master.png`)
- Committed: Yes

**`tasks/`:**

- Purpose: Product requirement documents and specifications
- Generated: No (authored documents)
- Committed: Yes

**`.planning/`:**

- Purpose: Architecture documentation, codebase analysis, session handoff notes
- Generated: No (authored documentation)
- Committed: Yes

**`scripts/ralph/`:**

- Purpose: Utility scripts for development/build workflows
- Generated: No
- Committed: Yes
