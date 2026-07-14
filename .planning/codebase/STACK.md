# Technology Stack

## Overview

Desktop application for installing/updating MinUI on retro handheld SD cards — "Balena Etcher for MinUI."

## Languages & Runtimes

| Layer | Language | Runtime |
|-------|----------|---------|
| Frontend | TypeScript 5.x (strict mode, ES2020 target) | Bun (package manager), Vite dev server on port 1420 |
| Backend | Rust (edition 2021) | Cargo (via `cargo tauri dev`) |
| Desktop Shell | — | Tauri v2 |

## Core Frameworks

| Framework | Version | Purpose |
|-----------|---------|---------|
| **Tauri** | v2 | Desktop app shell, IPC bridge between Rust backend and React frontend |
| **React** | 18 | Frontend UI library |
| **Vite** | 5.x | Frontend build tool with `@vitejs/plugin-react` |

## Frontend Dependencies

### Production
- `@tauri-apps/api` — Tauri IPC bridge (invoke commands, event listeners)
- `react` ^18, `react-dom` ^18

### Development
- `@testing-library/jest-dom`, `@testing-library/react`, `@testing-library/user-event` — Component testing
- `@types/react`, `@types/react-dom` — TypeScript types
- `@vitejs/plugin-react` — Vite React plugin
- `@vitest/coverage-v8` — Test coverage
- `eslint` + `@eslint/js` + `@typescript-eslint/*` + `eslint-plugin-react-hooks` + `eslint-plugin-react-refresh` — Linting
- `oxlint` — Fast Rust-based linter
- `jsdom` — DOM environment for Vitest
- `typescript` — Type checking
- `vite` — Build tool
- `vitest` — Test runner

## Rust Dependencies

### Core
- `tauri` (v2) — Desktop framework
- `serde` + `serde_json` — Serialization
- `reqwest` (features: `json`, `stream`) — HTTP client with streaming downloads
- `semver` — Version parsing
- `tokio` (features: `full`) — Async runtime
- `tokio-util` (features: `rt`) — CancellationToken support

### Archive & Crypto
- `zip` — ZIP archive extraction
- `sha2` + `hex` — SHA-256 checksum verification
- `base64` — Encoding support

### Platform
- `libc` (unix) — POSIX system calls on macOS
- `windows-sys` — Windows API bindings
- `tempfile` — Secure temporary directories

### Build
- `tauri-build` — Tauri build scripts

## Build & Dev Tools

| Tool | Purpose | Config |
|------|---------|--------|
| **Bun** | Package manager & script runner | `package.json` |
| **Cargo** | Rust build system | `src-tauri/Cargo.toml` |
| **just** | Task runner | `justfile` — `just check`, `just fmt`, `just dev` |
| **prek** | Pre-commit hooks | `prek.toml` — trailing-whitespace, EOF fixer, LF normalization, lint `--fix` |
| **oxlint** | Fast Rust-based linter | `bun run lint` |
| **oxfmt** | Fast Rust-based formatter | `.oxfmtrc.json` — 2-space, LF, 80-char width, double quotes |
| **ESLint** | JavaScript/TypeScript linting | `.eslintrc.cjs` — `recommended` + `@typescript-eslint/recommended` |
| **Vitest** | Test runner | `vitest.config.ts` — jsdom env, `src/**/*.test.{ts,tsx}` |

## Configuration Files

| File | Purpose |
|------|---------|
| `package.json` | npm/bun dependencies and scripts |
| `src-tauri/Cargo.toml` | Rust dependencies and features |
| `src-tauri/tauri.conf.json` | Tauri app config (identifier: `dev.minui.easy-installer`, 800×600 window) |
| `tsconfig.json` | TypeScript config (strict, bundler resolution, react-jsx) |
| `vite.config.ts` | Vite build config |
| `vitest.config.ts` | Vitest runner config |
| `vitest.setup.ts` | Test setup (`@testing-library/jest-dom/vitest`) |
| `.eslintrc.cjs` | ESLint rules |
| `.oxfmtrc.json` | oxfmt formatting rules |
| `prek.toml` | Pre-commit hooks |
| `justfile` | Task runner commands |
| `.editorconfig` | Editor settings |

## Content Security Policy

Configured in `src-tauri/tauri.conf.json`:

```
default-src 'self';
connect-src 'self' https://packages.minui.dev https://api.github.com https://github.com https://*.githubusercontent.com;
img-src 'self' data:;
style-src 'self' 'unsafe-inline';
script-src 'self'
```

## Platform Support

- **Windows**: 10+ (via `windows-sys`)
- **macOS**: 10.15+ (via `libc`)
- **Linux**: Not supported in Phase 1 (MVP)

## No CSS Framework

No Tailwind, shadcn/ui, or other CSS framework — all styling is in plain `src/styles.css`.
