# Technology Stack

## Languages & Runtimes

| Layer | Language | Runtime |
|-------|----------|---------|
| Backend | Rust (edition 2021) | Tauri v2 |
| Frontend | TypeScript 5.6+ | React 18 + Vite 6 |
| Package manager | — | Bun |

## Backend Dependencies (`src-tauri/Cargo.toml`)

| Crate | Version | Purpose |
|-------|---------|---------|
| `tauri` | 2 | Desktop app framework |
| `serde` + `serde_json` | 1 | IPC serialization |
| `reqwest` | 0.12 | HTTP client (streaming downloads) |
| `semver` | 1 | Version parsing/comparison |
| `sha2` + `hex` + `base64` | 0.10 / 0.4 / 0.22 | Checksum verification |
| `tempfile` | 3 | Temporary directories for extraction |
| `zip` | 0.6 | Archive extraction |
| `tokio` + `tokio-util` + `futures-util` | 1 / 0.7 / 0.3 | Async runtime |
| `libc` | 0.2 | Unix system calls (disk space, stat) |
| `windows-sys` | 0.59 | Windows filesystem API |

## Frontend Dependencies (`package.json`)

### Runtime
| Package | Version | Purpose |
|---------|---------|---------|
| `react` + `react-dom` | ^18.3 | UI framework |
| `@tauri-apps/api` | ^2.0 | Tauri IPC bridge |

### Dev
| Package | Version | Purpose |
|---------|---------|---------|
| `@tauri-apps/cli` | ^2.0 | Tauri build tooling |
| `vite` | ^6.0 | Bundler & dev server |
| `typescript` | ^5.6 | Static typing |
| `oxlint` | ^1.69 | Fast Rust-based linter |
| `vitest` | ^4.1 | Test runner |
| `jsdom` | ^29.1 | DOM environment for tests |
| `@testing-library/react` | ^16.3 | Component testing |
| `@testing-library/user-event` | ^14.6 | User interaction simulation |
| `@vitest/coverage-v8` | ^4.1 | Code coverage |

## Toolchain

| Tool | Purpose |
|------|---------|
| `just` | Task runner (`justfile`) |
| `prek` | Pre-commit hooks (trailing whitespace, EOF, LF, lint) |
| `oxfmt` | TypeScript formatter (oxc-based, zero-config) |
| `cargo fmt` | Rust formatter |
| `cargo clippy` | Rust linter (deny warnings in CI) |
| `eslint` | JavaScript rules (no-async-promise-executor, etc.) |

## No-CSS-Framework Policy

Plain `src/styles.css` — no Tailwind, no shadcn, no CSS-in-JS. All styling is hand-written CSS.

## Platform Targets

| OS | Minimum Version | Notes |
|----|-----------------|-------|
| macOS | 10.15+ | Apple Silicon (aarch64) DMG |
| Windows | 10+ | x64 MSI + EXE installer |
| Linux | Ubuntu (CI only) | Not in MVP; `ubuntu-latest` in build matrix for regression catches |
