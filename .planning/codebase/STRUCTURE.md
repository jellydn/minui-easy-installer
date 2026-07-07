# Structure

## Repository layout (top level)

```
2026-06-13-minui-installer/
├── src/                       # React + TypeScript frontend
├── src-tauri/                 # Rust backend (Tauri v2)
├── assets/, icons/            # App icons + static assets
├── scripts/                   # Helper shell scripts
├── plans/                     # Long-form planning docs
├── tasks/                     # Per-task working notes
├── .changeset/                # Versioned changelog fragments
├── .commandcode/              # Tooling internal config
├── .mimocode/                 # Tooling internal config
├── .github/                   # CI / issue templates
├── .planning/                 # Generated codebase docs (this dir)
├── index.html                 # Vite entry
├── package.json               # Frontend deps + npm scripts
├── tsconfig.json              # TS strict, noEmit
├── vite.config.ts             # Vite + React plugin
├── vitest.config.ts           # Vitest + jsdom
├── vitest.setup.ts            # @testing-library/jest-dom matchers
├── justfile                   # Task runner recipes
├── prek.toml                  # Pre-commit hook config
├── .oxfmtrc.json              # oxfmt config
├── .eslintrc.cjs              # ESLint config
├── AGENTS.md                  # AI-agent operational notes
├── DESIGN.md                  # Design rationale
├── README.md
├── plan.md                    # Top-level project plan
├── tweak.md                   # Patch notes
└── install-guide.txt          # End-user install instructions
```

## Frontend directory layout — `src/`

```
src/
├── main.tsx                   # React root, renders <App /> with StrictMode
├── App.tsx                    # Screen router (home | store | wifi)
├── styles.css                 # Global stylesheet
├── vitest.d.ts                # Type augmentations for vitest
│
├── Home.tsx                   # Main install + status screen
├── Home.test.tsx
├── DeviceSelector.tsx         # Device picker (uses DEVICE_PROFILES)
├── DriveSelector.tsx          # SD card picker, Format flow
├── DriveSelector.test.tsx
├── InstallProgress.tsx        # Live install log + phase indicator
├── ConfirmDialog.tsx          # Pre-install confirmation modal
├── FormatConfirmDialog.tsx    # FAT32 format confirmation modal
├── HealthCheck.tsx            # SD card health check + support report
├── ValidationReport.tsx       # Post-install validation summary
├── PackageStore.tsx           # Add-on package browser
├── PackageStore.test.tsx
├── PackageCard.tsx            # Single package card
├── WifiWizard.tsx             # SSID scan + wifi.txt write
├── WifiWizard.test.tsx
│
├── hooks/                     # Custom React hooks
│   ├── useMountEffect.ts
│   ├── useScrollToBottom.ts
│   ├── useVersionCheck.ts     # Drive-change → version check (with stale-request guard)
│   └── useVersionCheck.test.ts
│
└── types/                     # Typed IPC wrappers + domain types
    ├── archive.ts             # DownloadResult, ExtractionResult + invoke()
    ├── archive.test.ts
    ├── device.ts              # DEVICE_PROFILES table (18 devices), getDeviceProfile
    ├── device.test.ts
    ├── device-install-map.ts  # Secondary device metadata (per-device paks)
    ├── device-install-map.test.ts
    ├── device-install-map.json# Source data for the above
    ├── drive.ts               # RemovableDrive + formatSize helper
    ├── drive.test.ts
    ├── errors.ts              # AppErrorCode + classifyError()
    ├── fork.ts                # ForkConfig (official / MinUI-Zero / custom)
    ├── install.ts             # InstallPhase, InstallResult, installMinui/startInstall/cancelInstall
    ├── install.test.ts
    ├── package.ts             # PackageRegistry, installPackage, fetchPackageRegistry
    ├── package.test.ts
    ├── release.ts             # MinUIRelease + fetchMinUIRelease (with fork cache)
    ├── release.test.ts
    ├── store.json             # Bundled package registry (offline fallback)
    ├── validate.ts            # ValidationResult + validateInstallation + checkSdCardHealth
    ├── validate.test.ts
    ├── version.ts             # VersionCheckResult + checkMinuiVersion
    └── version.test.ts
```

### Frontend naming conventions

- Components: PascalCase `.tsx` files, default-exported function components. Props interface suffixed with `Props` (e.g. `HomeProps`, `DeviceSelectorProps`).
- State interfaces named `*State` when local; shared types live in `src/types/`.
- Test files co-located: `Foo.tsx` → `Foo.test.tsx`. Mirrors `vitest.config.ts` setup.
- `INITIAL_*_STATE` constants hoisted to module scope (see `Home.tsx:36-47`) for stable references across renders.
- All TypeScript is `strict` with `noUnusedLocals`, `noUnusedParameters`, and `noFallthroughCasesInSwitch` (see `tsconfig.json:13-17`).
- All `invoke()` calls use dynamic `import("@tauri-apps/api/core")` rather than a top-level import — keeps the wrapper decoupled from bundle mode.
- Every IPC wrapper returns the discriminated union `{success: true, data: T} | {success: false, error: E}` with a typed `code` field. The `classifyError` helper in `src/types/errors.ts` infers codes from Rust error message substrings.

## Backend directory layout — `src-tauri/`

```
src-tauri/
├── Cargo.toml                 # Crate: minui_easy_installer_lib, deps (tokio, reqwest, zip, sha2, semver, tempfile, futures-util, libc, windows-sys)
├── Cargo.lock
├── build.rs                   # tauri-build
├── tauri.conf.json            # App config: 800x600 window, CSP, icons
├── icons/                     # Bundle icons
├── gen/                       # Tauri-generated bindings (do not edit)
├── capabilities/
│   └── default.json           # Default capability for main window, core:default perms
└── src/
    ├── main.rs                # Calls minui_easy_installer_lib::run()
    ├── lib.rs                 # 20 #[tauri::command] handlers + InstallRegistry + run()
    ├── pipeline.rs            # Pipeline::run / run_to_extracted; InstallSession slots; create_target_within (path-traversal guard)
    ├── install.rs             # install_minui, install_minui_with_cancel, copy_base_files, copy_extras_files, create_rom_dirs, is_preserved_path
    ├── package.rs             # install_package, detect_installed_packages, check_package_updates, PackageInstallPathRules
    ├── drives.rs              # list_removable_drives (macOS/Windows), format_drive, classify_volume, is_removable_volume
    ├── download.rs            # download_archive, download_archive_into, download_archive_streaming, verify_checksum (SHA-256)
    ├── extract.rs             # extract_archive, extract_archive_into, is_path_traversal
    ├── validate.rs            # validate_installation, format_validation_report, format_bytes
    ├── version.rs             # check_for_updates, detect_installed_version, compare_versions (semver + date fallback)
    ├── health.rs              # check_sd_card_health, generate_support_report, detect_filesystem
    ├── wifi.rs                # write_wifi_config, scan_wifi_networks, get_current_wifi_ssid
    └── fs_utils.rs            # copy_dir_recursive (skip+cancel predicates), get_disk_space (libc::statvfs)
```

### Backend naming conventions

- All modules declared with `mod foo;` in `lib.rs:14-25`.
- All public IPC functions are annotated `#[tauri::command]`. Argument names are camelCase in Rust but `serde` snake_case fields are renamed on the JS side via Tauri's automatic conversion (e.g. `mount_path` ↔ `mountPath`).
- Domain types are `pub struct` with `#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]`. All `Result` returns use `Result<T, String>` to match the IPC error contract.
- Cancellation is centralized: any function that may take a long time accepts a `CancellationToken` from `tokio_util::sync::CancellationToken`.
- Test modules are `#[cfg(test)] mod tests` at the bottom of each file. Tests are co-located (not in a separate `tests/` dir).
- Platform-gated code uses `#[cfg(target_os = "macos")]` / `#[cfg(target_os = "windows")]` / `#[cfg(unix)]` / `#[cfg(not(...))]` for fallbacks.
- Temp directory ownership is the `InstallSession` (see `pipeline.rs:33-57`): `Option<TempDir>` slots are filled by download/extract and live for the entire install.

## Key file cross-references

- `src-tauri/src/lib.rs:1-447` — single source of truth for the IPC surface. Adding a new command requires (1) a `#[tauri::command] fn` here, (2) registration in the `tauri::generate_handler!` macro at `lib.rs:392-413`, and (3) a corresponding typed wrapper in `src/types/`.
- `src-tauri/src/pipeline.rs:1-280` — the install/pipeline contract. Any new install flavor (e.g. a "package update" pipeline) reuses `Pipeline::run` or `Pipeline::run_to_extracted`.
- `src-tauri/src/install.rs:90-110` — the `PRESERVED_FOLDERS` list and `is_preserved_path` are the single source of truth for what survives a re-install. To add a preserved folder, edit this constant and the corresponding test.
- `src/types/device.ts:24-128` — the `DEVICE_PROFILES` array. Adding a new supported device means adding an entry here and (optionally) a corresponding entry in `device-install-map.json` for per-device PAK support.
- `src/types/package.ts:248-296` — the registry fetch function. The three-tier fallback (remote → bundled → error) is the contract the PackageStore relies on.
- `src/hooks/useVersionCheck.ts:1-90` — the version-check state machine. Uses a `requestIdRef` to discard stale results when the user changes the selected drive mid-fetch.
- `src-tauri/tauri.conf.json:14-19` — CSP. Any new network destination for downloads / registry fetches must be added here.

## Configuration / data files

- `package.json` — npm scripts (`dev`, `build`, `tauri`, `typecheck`, `lint`, `test`, `fmt`, `doctor`); React 18.3, Tauri 2.0, Vite 6, Vitest 4, TypeScript 5.6.
- `tsconfig.json` — strict TS, ES2020 target, `react-jsx` JSX, `noEmit` (Vite handles emit).
- `vite.config.ts` — Vite + React plugin.
- `vitest.config.ts` — Vitest with jsdom environment, aliasing for `@/` style imports if any.
- `Cargo.toml` — Tauri 2, tokio (full), reqwest (json + stream), zip 0.6, sha2, semver, tempfile, futures-util, libc (unix), windows-sys (windows).
- `tauri.conf.json` — `productName: "MinUI Easy Installer"`, identifier `dev.minui.easy-installer`, single 800x600 window, CSP restricted to `packages.minui.dev`, `api.github.com`, `github.com`, `*.githubusercontent.com`.
- `src/types/store.json` — bundled offline copy of the package registry (emu_paks + tool_paks shape, validated by `parseRegistryFromJson` in `src/types/package.ts:185-246`).
- `src/types/device-install-map.json` — per-device PAK overrides, shared BIOS list (`gba_bios.bin`, `syscard3.pce`, `bios.min`, `sgb.bios`).
