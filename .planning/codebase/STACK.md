# Technology Stack

**Analysis Date:** 2026-06-13

## Languages

**Primary:**

- TypeScript ~5.6.3 - Frontend UI and shared types (`package.json`, `tsconfig.json`, all of `src/`)
- Rust (edition 2021) - Tauri backend, native OS integration (`src-tauri/Cargo.toml`, all of `src-tauri/src/`)

**Secondary:**

- TSX/JSX (react-jsx) - React components (`src/App.tsx`, `src/Home.tsx`, `tsconfig.json` `"jsx": "react-jsx"`)
- CSS - App styling (`src/styles.css`)
- Shell snippets embedded in Rust - PowerShell command for Windows drive detection (`src-tauri/src/drives.rs`)

## Runtime

**Environment:**

- Node.js (ESM, `"type": "module"`) for the frontend toolchain (`package.json`)
- Tauri v2 native runtime (WebView + Rust host process) for the desktop app (`src-tauri/tauri.conf.json`)
- Tokio 1 async runtime (`features = ["full"]`) for Rust backend (`src-tauri/Cargo.toml`)

**Package Manager:**

- npm (frontend) - Lockfile: present (`package-lock.json`)
- Cargo (Rust) - Lockfile: present (`src-tauri/Cargo.lock`)

## Frameworks

**Core:**

- React 18.3.1 + react-dom 18.3.1 - Frontend UI (`package.json`, `src/main.tsx`)
- Tauri 2 - Desktop shell, IPC command bridge, bundling (`src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`)
- @tauri-apps/api ^2.0.0 - Frontend → Rust `invoke` bridge (`package.json`, `src/DriveSelector.tsx`)

**Testing:**

- Vitest 4.1.8 - Unit/component test runner, jsdom environment (`vitest.config.ts`, `package.json`)
- @testing-library/react 16.3.2 + jest-dom 6.9.1 + user-event 14.6.1 - React component testing (`package.json`, `vitest.setup.ts`)
- jsdom 29.1.1 - DOM environment for tests (`vitest.config.ts`)
- @vitest/coverage-v8 4.1.8 - Coverage reporting (`package.json`)
- Rust built-in `#[cfg(test)]` unit tests - Backend tests (e.g. `src-tauri/src/drives.rs`, `src-tauri/src/wifi.rs`, `src-tauri/src/download.rs`)

**Build/Dev:**

- Vite 6.0.0 + @vitejs/plugin-react 4.3.4 - Dev server (port 1420) and build (`vite.config.ts`, `package.json`)
- TypeScript compiler (`tsc`) - Typecheck + pre-build (`package.json` `build`/`typecheck`)
- @tauri-apps/cli ^2.0.0 - Tauri dev/build orchestration (`package.json`)
- tauri-build 2 - Rust build script dependency (`src-tauri/Cargo.toml`)

## Key Dependencies

**Critical:**

- reqwest 0.12 (`features = ["json", "blocking"]`) - HTTP download of MinUI archives (`src-tauri/src/download.rs`)
- zip 0.6 - Archive extraction with path-traversal guards (`src-tauri/src/extract.rs`)
- sha2 0.10 + hex 0.4 - SHA-256 checksum verification of downloads (`src-tauri/src/download.rs`)
- tempfile 3 - Temp dir staging before SD-card copy (`src-tauri/src/download.rs`, `src-tauri/src/extract.rs`)
- serde 1 (derive) + serde_json 1 - (De)serialization of IPC payloads and PowerShell JSON (`src-tauri/src/drives.rs`, `src-tauri/src/package.rs`)
- tokio 1 - Async runtime backing async Tauri commands (`src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`)

**Infrastructure:**

- time 0.3.36 - Timestamps (`src-tauri/Cargo.toml`)
- libc 0.2 (unix target only) - Native unix calls (`src-tauri/Cargo.toml`)
- windows-sys 0.59 (`Win32_Storage_FileSystem`, windows target only) - Native Windows storage APIs (`src-tauri/Cargo.toml`)

## Configuration

**Environment:**

- `TAURI_DEV_HOST` - Optional dev host for HMR over LAN (`vite.config.ts`)
- No `.env` files or secret-based config; registry/GitHub URLs are hard-coded constants (`src/types/package.ts` `REGISTRY_URL`, `src/types/release.ts` `GITHUB_API_URL`)

**Build:**

- `tsconfig.json` - Strict TS (`strict`, `noUnusedLocals`, `noUnusedParameters`), bundler module resolution, `noEmit`
- `vite.config.ts` - Vite + React, fixed port 1420, ignores `src-tauri/**`
- `vitest.config.ts` + `vitest.setup.ts` - jsdom env, `src/**/*.test.{ts,tsx}` include
- `.eslintrc.cjs` - ESLint with `@typescript-eslint` (note: `lint` script actually runs `oxlint src`; `fmt` runs `oxfmt src`)
- `src-tauri/tauri.conf.json` - App identifier `dev.minui.easy-installer`, 800×600 window, bundle targets `all`, CSP null
- `src-tauri/capabilities/default.json` - Capability set granting `core:default` to the main window
- `src-tauri/Cargo.toml` - Crate `minui_easy_installer_lib` (`staticlib`, `cdylib`, `rlib`)

## Platform Requirements

**Development:**

- Node.js + npm and Rust/Cargo toolchain (Rust edition 2021)
- Tauri v2 prerequisites (platform WebView, build tooling)
- Dev tooling: Vite, Vitest, oxlint/oxfmt, @tauri-apps/cli

**Production:**

- Desktop app, MVP targets Windows + macOS only (per `AGENTS.md`; no Linux in Phase 1, though some Rust paths are `cfg(target_os = "linux")`)
- Bundled installers via Tauri (`bundle.targets = "all"`, icons in `src-tauri/icons/`, `src-tauri/tauri.conf.json`)

---

_Stack analysis: 2026-06-13_
