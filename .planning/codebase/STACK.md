# Technology Stack

## Languages

| Layer | Language | Version |
|-------|----------|---------|
| Backend | Rust | 2021 edition |
| Frontend | TypeScript | `tsconfig.json` → `strict: true`, `JSX: react-jsx` |
| Scripts | TypeScript (Bun) | `scripts/*.ts` |

## Runtimes & Bundlers

| Tool | Purpose |
|------|---------|
| [Tauri v2](https://v2.tauri.app/) | Desktop application shell — system tray, shell access, IPC bridge |
| [Vite](https://vitejs.dev/) | Frontend dev server + build (port 1420) |
| [Bun](https://bun.sh/) | JS runtime & package manager (`bun.lock`) |

## Frontend Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| `react` | ^18 | Component framework |
| `react-dom` | ^18 | DOM renderer |
| `@tauri-apps/api` | ^2 | Invoke Rust commands, listen for events |
| `@tauri-apps/plugin-shell` | ^2 | Shell command access (WiFi scanning) |

## Backend Dependencies (Rust — `src-tauri/Cargo.toml`)

| Crate | Purpose |
|-------|---------|
| `tauri` v2 | Desktop framework — commands, events, window management |
| `tauri-plugin-shell` | Shell access for platform-specific commands |
| `tokio` v1 | Async runtime (features: `rt-multi-thread`, `macros`, `sync`) |
| `reqwest` v0.12 | HTTP client with streaming support (features: `stream`, `rustls-tls`) |
| `serde` / `serde_json` | Serialization (derive `Serialize`/`Deserialize`) |
| `zip` | Archive extraction (deflate support) |
| `tempfile` | Temp directories for extraction isolation |
| `sha2` | SHA-256 checksum verification |
| `base64` | BIOS file payload decoding |
| `tokio-util` | `CancellationToken` for install cancellation |

### Platform-specific

| Crate | Platform | Purpose |
|-------|----------|---------|
| `libc` | Unix | `statvfs` for disk space |
| `windows-sys` | Windows | File system APIs |

## Configuration

| File | Purpose |
|------|---------|
| `tsconfig.json` | TypeScript strict mode, `baseUrl: "."`, `resolveJsonModule: true` |
| `vite.config.ts` | Dev server on `localhost:1420`, `clearScreen: false` |
| `tauri.conf.json` | CSP, bundle identifier `com.minui.installer`, app security |
| `Cargo.toml` | Rust edition 2021, dependency versions |
| `prek.toml` | Pre-commit hooks (trailing whitespace, EOF fixer, LF normalize, lint `--fix`) |
| `.editorconfig` | UTF-8, LF line endings, 2-space indent for TS/JSON |
| `justfile` | Build shortcuts: `just check` (lint+typecheck+fmt+clippy), `just fmt` |

## CSP (Content Security Policy)

From `tauri.conf.json`:

```
default-src 'self';
connect-src 'self'
  https://packages.minui.dev
  https://api.github.com
  https://github.com
  https://*.githubusercontent.com;
img-src 'self' data:;
style-src 'self' 'unsafe-inline';
script-src 'self'
```

## OS Support

| OS | Minimum Version | Notes |
|----|-----------------|-------|
| macOS | 10.15+ | 14.4+ has `airport` deprecation risk for WiFi scanning |
| Windows | 10+ | Format not supported |
| Linux | — | Not in MVP Phase 1 |
