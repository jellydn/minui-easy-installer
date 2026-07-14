# Stack

## Runtimes & Frameworks

| Layer | Technology | Version |
|-------|-----------|---------|
| Desktop shell | [Tauri v2](https://v2.tauri.app/) | 2.x |
| Frontend | [React](https://react.dev/) | 18.x |
| Frontend language | [TypeScript](https://www.typescriptlang.org/) | 5.6 |
| Backend | Rust | edition 2021 |
| Async runtime (Rust) | [tokio](https://tokio.rs/) | 1.x |

## Frontend Dependencies

| Package | Purpose |
|---------|---------|
| `react`, `react-dom` | UI framework |
| `@tauri-apps/api` | Tauri IPC bridge (`invoke`, `listen`, `emit`) |
| `@vitejs/plugin-react` | Vite React plugin |
| `@testing-library/react` | Component testing |
| `@testing-library/user-event` | Simulated user interactions |
| `vitest` | Test runner |
| `jsdom` | DOM environment for tests |

## Backend (Rust) Dependencies

| Crate | Purpose |
|-------|---------|
| `tauri` (v2) | Desktop framework |
| `serde`, `serde_json` | Serialization/deserialization |
| `reqwest` | HTTP client (GitHub API, package registry) |
| `tokio` | Async runtime |
| `tokio-util` | CancellationToken |
| `zip` | Archive creation/extraction |
| `tempfile` | Temporary directories |
| `sha2` | SHA-256 checksum verification |
| `base64` | BIOS file encoding |
| `semver` | Version parsing |
| `libc` | Unix system calls (disk space) |
| `windows-sys` | Windows API bindings |

## Tooling

| Tool | Purpose | Config |
|------|---------|--------|
| [Bun](https://bun.sh/) | JS runtime & package manager | `bun.lock` |
| [oxlint](https://oxc-project.github.io/) | TypeScript linter | `.oxlintrc.json` |
| [oxfmt](https://oxc-project.github.io/) | TypeScript formatter | `.oxfmtrc.json` |
| `tsc` | TypeScript type checking | `tsconfig.json` |
| `cargo fmt` | Rust formatter | — |
| `cargo clippy` | Rust linter | `-D warnings` |
| [prek](https://prek.j178.dev/) | Pre-commit hooks | `prek.toml` |
| [Vitest](https://vitest.dev/) | Test runner + coverage | `vitest.config.ts` |
| [Vite](https://vitejs.dev/) | Dev server & bundler | `vite.config.ts` |

## Build Configuration

| File | Purpose |
|------|---------|
| `package.json` | Frontend scripts, dependencies |
| `Cargo.toml` | Rust dependencies, binary target |
| `tauri.conf.json` | App identity, CSP, bundle config |
| `tsconfig.json` | TypeScript (ES2020, react-jsx) |
| `vite.config.ts` | Vite + React plugin |
| `vitest.config.ts` | Test runner + v8 coverage (50/40 thresholds) |
| `.github/workflows/rust.yml` | CI: fmt, clippy, test on ubuntu-latest |

## Platform Support

| OS | Status | Min Version |
|----|--------|-------------|
| macOS | ✅ Supported | 10.15+ |
| Windows | ✅ Supported | 10+ |
| Linux | 🚧 Phase 2 | Not yet supported |

## Security

- **CSP** (in `tauri.conf.json`): `default-src 'self'`; allowlisted `connect-src` for `packages.minui.dev`, `api.github.com`, `github.com`, `*.githubusercontent.com`
- No external scripts, styles, or images beyond data: URIs
- Rust backend validates all registry data before use
