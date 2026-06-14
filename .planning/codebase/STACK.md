# Technology Stack

**Analysis Date:** 2026-06-14

## Languages

**Primary:**
- TypeScript 5.6.3 — React frontend (`src/`)
- Rust 2021 edition — Tauri backend (`src-tauri/src/`)

**Secondary:**
- HTML/CSS — UI layout and styling (implied by Tauri + React)

## Runtime

**Environment:**
- Tauri v2 — Desktop runtime (Rust + WebView)

**Package Manager:**
- npm / Bun — JavaScript (`package.json` scripts use `bun run` in `justfile`)
- Cargo — Rust (`src-tauri/Cargo.toml`)

**Lockfile:**
- `package-lock.json` — npm lockfile (present)
- `Cargo.lock` — Rust lockfile (present, committed with crate)

## Frameworks

**Core:**
- Tauri v2 — Desktop app framework (Rust backend, WebView frontend)
- React 18.3.1 — UI component library

**Testing:**
- Vitest 4.1.8 — JavaScript test runner
- @testing-library/react 16.3.2 — React component testing utilities
- @testing-library/jest-dom 6.9.1 — Custom DOM matchers
- @testing-library/user-event 14.6.1 — User interaction simulation
- jsdom 29.1.1 — DOM environment for tests
- @vitest/coverage-v8 4.1.8 — Code coverage via V8
- Rust built-in `#[cfg(test)]` — Rust unit tests (per-module)

**Build/Dev:**
- Vite 6.0.0 — Frontend dev server and bundler
- @vitejs/plugin-react 4.3.4 — React Fast Refresh for Vite
- tauri-build 2.x — Tauri build helper (Rust build dependency)
- just 0.x — Command runner (`justfile`)
- oxlint 1.69.0 — Rust-powered TypeScript linter
- oxc-parser 0.135.0 — Rust-powered TypeScript parser
- ESLint 10.5.0 — JavaScript linter (legacy config in `.eslintrc.cjs`)

## Key Dependencies

**Critical:**
- `@tauri-apps/api` ^2.0.0 — Tauri IPC bridge between frontend and Rust backend
- `@tauri-apps/cli` ^2.0.0 — Tauri CLI for dev/build
- `reqwest` 0.12 (with `json`, `blocking` features) — HTTP client for downloading archives
- `sha2` 0.10 — SHA-256 checksum verification for downloaded archives
- `zip` 0.6 — ZIP archive extraction
- `tempfile` 3 — Temporary file/directory management for downloads and extractions

**Infrastructure:**
- `serde` 1 (with `derive`) — Rust serialization/deserialization
- `serde_json` 1 — JSON parsing in Rust
- `tokio` 1 (full features) — Async runtime for Rust HTTP operations
- `time` 0.3.36 — Date/time handling in Rust
- `hex` 0.4 — Hex encoding for checksums

**Platform-Specific:**
- `libc` 0.2 — Unix-only: `statvfs` for disk space queries
- `windows-sys` 0.59 (`Win32_Storage_FileSystem`) — Windows-only: filesystem operations

## Configuration

**Environment:**
- `TAURI_DEV_HOST` — Optional env var for Vite dev server HMR host (used in `vite.config.ts`)
- No `.env` file or dotenv dependency detected

**Build Config Files:**
- `tsconfig.json` — TypeScript config: ES2020 target, strict mode, React JSX, bundler module resolution
- `vite.config.ts` — Vite config: React plugin, port 1420, strict port, HMR on port 1421
- `vitest.config.ts` — Vitest config: jsdom environment, setup file, test glob `src/**/*.test.{ts,tsx}`
- `src-tauri/tauri.conf.json` — Tauri app config: window 800x600, CSP policy, bundle targets
- `src-tauri/Cargo.toml` — Rust crate config: edition 2021, staticlib+cdylib+rlib output
- `.eslintrc.cjs` — ESLint config: browser env, TypeScript plugin
- `justfile` — Task runner: dev, build, lint, fmt, check, pre-commit recipes

**Security/CSP:**
- CSP in `tauri.conf.json`: `default-src 'self'; connect-src 'self' https://packages.minui.dev https://api.github.com https://github.com https://*.githubusercontent.com`
- Capabilities in `src-tauri/capabilities/default.json`: `core:default` permissions only

## Platform Requirements

**Development:**
- macOS 10.15+ or Windows 10+ (Tauri v2 requirement)
- Node.js / Bun for frontend dev
- Rust toolchain (stable, 2021 edition)
- `just` command runner (optional, for `justfile` recipes)
- `cargo tauri dev` for full-stack development

**Production:**
- macOS `.dmg` or Windows `.msi`/`.exe` installer
- Targets: `"all"` (all Tauri-supported bundle formats)
- Icons: 32x32, 128x128, 128x128@2x, `.icns` (macOS), `.ico` (Windows)

**Frontend TypeScript (strict):**
- Target: ES2020
- Strict mode enabled (`noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`)
- Module: ESNext with bundler resolution
- JSX: react-jsx (automatic runtime)
