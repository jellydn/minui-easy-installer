# Technology Stack

## Languages & Runtimes

| Component | Language | Runtime |
|-----------|----------|---------|
| Backend | Rust (edition 2021) | Tauri v2 |
| Frontend | TypeScript 5.x | React 18 (Vite dev server) |
| Package manager | — | Bun |
| OS floor | — | macOS 10.15+, Windows 10+ |

## Core Frameworks

### Backend — `src-tauri/`

| Dependency | Purpose |
|-----------|---------|
| `tauri` v2 | Desktop app framework, IPC bridge, window management |
| `serde` + `serde_json` | Serialization for IPC and config files |
| `reqwest` | HTTP client (GitHub API, package registry) with OnceLock pooling |
| `tokio` + `tokio-util` | Async runtime + CancellationToken for cancellable installs |
| `zip` | Archive extraction |
| `sha2` + `hex` | SHA-256 checksum verification for downloads |
| `tempfile` | Temp directory management (extract before copy pattern) |
| `base64` | BIOS file decoding (frontend sends binary as base64 through JSON IPC) |

### Frontend — `src/`

| Dependency | Purpose |
|-----------|---------|
| React 18 | UI framework |
| `@tauri-apps/api` | Tauri IPC (`invoke`, `listen` for events) |
| `@testing-library/react` | Component tests |
| `vitest` | Test runner (jsdom environment) |

## Build & Development

| Tool | File | Purpose |
|------|------|---------|
| Vite | `vite.config.ts` | Frontend dev server (port 1420), production bundling |
| Cargo | `src-tauri/Cargo.toml` | Rust compilation, dependency management |
| Bun | `package.json` → `bun.lock` | JS dependency management, `bun run` scripts |
| Tauri CLI | `cargo tauri dev/build` | Orchestrates Vite + Cargo, code signing, bundling |

## Linting & Formatting

| Tool | File | Scope |
|------|------|-------|
| `oxlint` | `.oxfmtrc.json` | TypeScript/React linting (95 rules, Rust-based, zero-config) |
| `oxfmt` | `.oxfmtrc.json` | TypeScript/React formatting |
| `cargo fmt` | Rust default | Rust formatting |
| `cargo clippy` | `-D warnings` | Rust linting (strict mode) |
| `prek` | `prek.toml` | Pre-commit hooks (trailing whitespace, EOF fixer, LF normalization) |

## CI/CD — `.github/workflows/react-doctor.yml`

| Job | OS | Checks |
|-----|----|--------|
| `check` | ubuntu-latest | typecheck + lint + vitest |
| `build` (macOS) | macos-latest | `cargo build` + `cargo test` + `cargo clippy` + `cargo fmt --check` |
| `build` (Windows) | windows-latest | `cargo build` |
| `build` (Linux) | ubuntu-latest | `cargo build` + `cargo test` + `cargo clippy` + `cargo fmt --check` |

## Configuration Files

| File | Purpose |
|------|---------|
| `package.json` | npm scripts (`dev`, `build`, `typecheck`, `lint`, `fmt`, `test`) |
| `tsconfig.json` | TypeScript strict mode |
| `vite.config.ts` | Vite config (HMR on port 1420, Tauri integration) |
| `vitest.config.ts` | Vitest config (jsdom env, setup file) |
| `vitest.setup.ts` | Test environment setup |
| `src-tauri/tauri.conf.json` | Tauri app config (CSP, window, bundle settings) |
| `src-tauri/Cargo.toml` | Rust dependencies and metadata |
| `.oxfmtrc.json` | oxlint/oxfmt config |
| `.editorconfig` | Editor settings |
| `prek.toml` | Pre-commit hook config |
| `justfile` | Task runner (`just check`, `just fmt`) |
