# Codebase Structure

**Analysis Date:** 2026-06-13

## Directory Layout

```
2026-06-13-minui-installer/
├── index.html              # Vite HTML entry (mounts #root)
├── package.json            # Frontend deps + scripts (dev/build/test/lint/typecheck)
├── vite.config.ts          # Vite config (React plugin, dev port 1420)
├── vitest.config.ts        # Vitest config (jsdom)
├── vitest.setup.ts         # Test setup (jest-dom)
├── tsconfig.json           # TypeScript config
├── .eslintrc.cjs           # ESLint config (lint via oxlint per package.json)
├── justfile                # Task runner shortcuts
├── prek.toml               # Pre-commit hook config
├── README.md / AGENTS.md / plan.md
├── assets/                 # banner.svg, logo.svg
├── src/                    # React + TypeScript frontend
│   ├── main.tsx            # React entry — renders <App/>
│   ├── App.tsx             # Root component, top nav, cross-screen state
│   ├── Home.tsx            # Install/update orchestrator (largest component)
│   ├── DeviceSelector.tsx  # Pick handheld device profile
│   ├── DriveSelector.tsx   # Detect + pick removable drive
│   ├── ConfirmDialog.tsx   # Write-confirmation overlay modal
│   ├── InstallProgress.tsx # Install phase progress UI
│   ├── ValidationReport.tsx# Post-install validation report UI
│   ├── HealthCheck.tsx     # SD card health check UI
│   ├── PackageStore.tsx    # Browse/install registry packages
│   ├── WifiWizard.tsx      # WiFi scan + write config
│   ├── styles.css          # Global styles
│   ├── vitest.d.ts         # Test type declarations
│   ├── *.test.tsx          # Component tests (co-located)
│   └── types/              # IPC wrappers + shared types + pure logic
│       ├── drive.ts        # RemovableDrive type + formatSize
│       ├── device.ts       # DeviceProfile table + getDeviceProfile
│       ├── install.ts      # installMinui() invoke wrapper + InstallPhase
│       ├── archive.ts      # download/verify/extract invoke wrappers
│       ├── validate.ts     # validateInstallation/health invoke wrappers
│       ├── version.ts      # checkMinuiVersion invoke wrapper
│       ├── package.ts      # registry fetch/validate + package invoke wrappers
│       ├── release.ts      # GitHub release fetch + parse
│       └── *.test.ts       # Co-located unit tests
├── src-tauri/              # Rust backend (Tauri core)
│   ├── Cargo.toml          # Rust deps (tauri, reqwest, sha2, zip, tokio…)
│   ├── Cargo.lock
│   ├── build.rs            # Tauri build script
│   ├── tauri.conf.json     # Tauri app config (window, bundle, dev URL)
│   ├── capabilities/
│   │   └── default.json    # IPC permission set (core:default)
│   ├── icons/              # App icons (png/icns/ico)
│   ├── gen/schemas/        # Generated capability/ACL schemas
│   └── src/
│       ├── main.rs         # Native entry → calls lib::run()
│       ├── lib.rs          # Command definitions + generate_handler! registry
│       ├── drives.rs       # Removable drive enumeration (OS-specific)
│       ├── download.rs     # HTTP download + SHA-256 checksum verify
│       ├── extract.rs      # ZIP extraction with path-traversal guards
│       ├── install.rs      # MinUI install: download→extract→copy (preserves ROMs)
│       ├── validate.rs     # Install validation + SD health check
│       ├── version.rs      # Installed MinUI version detection
│       ├── package.rs      # Package install + installed/update detection
│       └── wifi.rs         # WiFi scan + wifi.txt write
├── scripts/ralph/          # Autonomous "Ralph" agent loop (prompts, ralph.sh)
└── tasks/                  # PRD: prd-minui-easy-installer-package-store.md
```

## Directory Purposes

**`src/`:**

- Purpose: React/TypeScript frontend (webview process).
- Contains: `.tsx` components at the top level, co-located `.test.tsx` tests, global `styles.css`.
- Key files: `main.tsx` (entry), `App.tsx` (root/nav), `Home.tsx` (install orchestrator).

**`src/types/`:**

- Purpose: IPC wrapper layer — typed `invoke()` calls mirroring Rust structs — plus shared types and pure browser logic (release/registry parsing, formatting).
- Contains: `.ts` modules and co-located `.test.ts` unit tests.
- Key files: `install.ts`, `package.ts`, `release.ts`, `device.ts`.

**`src-tauri/`:**

- Purpose: Rust/Tauri native backend.
- Contains: crate config, build script, app config, capabilities, icons, generated schemas, and source in `src/`.
- Key files: `tauri.conf.json`, `Cargo.toml`, `capabilities/default.json`.

**`src-tauri/src/`:**

- Purpose: Rust command surface + privileged domain logic.
- Contains: one module per domain, each with `Serialize` result structs and `#[cfg(test)]` unit tests.
- Key files: `lib.rs` (command registry), `install.rs`, `drives.rs`.

**`scripts/ralph/`:**

- Purpose: Autonomous coding-loop tooling (`ralph.sh`, per-agent prompt files, `progress.txt`).

**`tasks/`:**

- Purpose: Product requirements doc driving the build.

## Key File Locations

**Entry Points:**

- `src/main.tsx`: React bootstrap, renders `<App/>`.
- `src-tauri/src/main.rs`: native `main()` → `minui_easy_installer_lib::run()`.
- `src-tauri/src/lib.rs`: `run()` builds the Tauri app and registers all 14 IPC commands.

**Configuration:**

- `src-tauri/tauri.conf.json`: window size, bundle targets, dev URL (`localhost:1420`).
- `vite.config.ts` / `tsconfig.json` / `vitest.config.ts`: frontend build/type/test.
- `src-tauri/capabilities/default.json`: IPC permission allowlist.
- `package.json` / `Cargo.toml`: dependency manifests.

**Core Logic:**

- `src-tauri/src/install.rs`: download→extract→copy with preserved-folder protection.
- `src-tauri/src/drives.rs`, `download.rs`, `extract.rs`, `validate.rs`, `version.rs`, `package.rs`, `wifi.rs`: per-domain native logic.
- `src/Home.tsx`: frontend install/update orchestration and state machine.
- `src/types/*.ts`: IPC bridge wrappers.

**Testing:**

- `src/**/*.test.tsx` and `src/types/*.test.ts`: Vitest/jsdom tests, co-located with sources.
- `#[cfg(test)] mod tests` blocks inside each `src-tauri/src/*.rs`: Rust unit tests.

## Naming Conventions

**Files:**

- React components: `PascalCase.tsx` (e.g. `DriveSelector.tsx`).
- TS type/IPC modules: `lowercase.ts` named after the Rust module (`install.ts` ↔ `install.rs`).
- Tests: co-located, same base name + `.test.tsx`/`.test.ts`.
- Rust modules: `snake_case.rs`.

**Identifiers:**

- TS interfaces mirror Rust struct field casing (snake_case fields like `mount_path`, `base_files_copied` kept as-is across IPC).
- Tauri commands: `snake_case` (`get_removable_drives`, `install_minui`); TS wrapper functions: `camelCase` (`installMinui`, `checkMinuiVersion`).

**Directories:**

- `kebab-case` project root; flat `src/` (no nested feature folders) with a single `types/` subdir.

## Where to Add New Code

**New Feature (new native capability):**

- Backend: add a domain module `src-tauri/src/<feature>.rs`, declare `mod <feature>;` + a `#[tauri::command]` in `src-tauri/src/lib.rs`, and add it to `generate_handler![...]`.
- Frontend bridge: add `src/types/<feature>.ts` with typed `invoke()` wrappers + mirror interfaces.
- UI: add a `PascalCase.tsx` component in `src/`, wire into `App.tsx` nav or `Home.tsx`.
- Tests: co-locate `*.test.ts(x)`; add `#[cfg(test)]` tests in the Rust module.

**New Component/Module:**

- Implementation: `src/<Name>.tsx` (component) or `src/types/<name>.ts` (logic/IPC).

**Utilities:**

- Shared frontend helpers: alongside their type in `src/types/*.ts` (e.g. `formatSize` in `drive.ts`); no separate utils dir exists.

## Special Directories

**`src-tauri/gen/schemas/`:**

- Purpose: Capability/ACL JSON schemas.
- Generated: Yes (by Tauri build). Committed: Yes.

**`src-tauri/target/` (in `.gitignore`):**

- Purpose: Rust build output. Generated: Yes. Committed: No.

**`node_modules/` / `dist/`:**

- Purpose: npm deps / Vite build output. Generated: Yes. Committed: No.

**`scripts/ralph/`:**

- Purpose: Autonomous agent loop assets. Generated: No. Committed: Yes.

---

_Structure analysis: 2026-06-13_
