# Stack

## Languages & Runtime

- **TypeScript** (`^5.6.3`) — strict, ESNext modules, `react-jsx` (`tsconfig.json:11-22`)
  - Target `ES2020`, libs `["ES2020", "DOM", "DOM.Iterable"]`
  - Strict + `noUnusedLocals` + `noUnusedParameters` + `noFallthroughCasesInSwitch`
- **Rust** — `edition = "2021"` (`src-tauri/Cargo.toml:7`)
  - Crate name `minui_easy_installer_lib`, types `["staticlib", "cdylib", "rlib"]` (`src-tauri/Cargo.toml:9-12`)
- **JavaScript/HTML/CSS** — minimal HTML shell (`index.html`), plain `src/styles.css`, no CSS framework
- **JSX** for React components in `src/`

## Application Framework

- **Tauri v2** — desktop runtime combining Rust backend + web frontend
  - `@tauri-apps/api` `^2.0.0` and `@tauri-apps/cli` `^2.0.0` (`package.json:14, 19`)
  - `tauri` `2` with `tauri-build` `2` (`src-tauri/Cargo.toml:15, 18`)
  - `tauri.conf.json` schema pinned to `tauri-apps/tauri` dev branch

## Frontend Stack

- **React** `^18.3.1` + **react-dom** `^18.3.1` (`package.json:15-16`)
- **React StrictMode** enabled (`src/main.tsx:6-8`)
- Entry: `src/main.tsx` → `src/App.tsx` → screen router (`home` / `store` / `wifi`)
- No router library — local `useState<Screen>` in `App.tsx:6-7`
- No state management library (Redux/Zustand/etc.) — only `useState` / custom hooks
- Custom hooks: `src/hooks/useMountEffect.ts`, `useScrollToBottom.ts`, `useVersionCheck.ts`

## Build Tooling

- **Vite** `^6.0.0` with `@vitejs/plugin-react` `^4.3.4` (`vite.config.ts`)
  - Dev server on port `1420` (strict), HMR on `1421` when `TAURI_DEV_HOST` set
  - `clearScreen: false`, ignores `src-tauri/**` from watcher
- **TypeScript build** — `tsc && vite build` (`package.json:8`)
- **Frontend dist** — `../dist` consumed by Tauri (`src-tauri/tauri.conf.json:11`)

## Backend (Rust) Dependencies

From `src-tauri/Cargo.toml`:
- `tauri` `2` — runtime / IPC / builder
- `serde` `1` (with `derive`), `serde_json` `1` — serialization
- `reqwest` `0.12` (features `json`, `stream`) — HTTP client (downloads, API calls)
- `semver` `1` — version comparison (not yet heavily used)
- `sha2` `0.10`, `hex` `0.4` — SHA-256 checksum verification
- `tempfile` `3` — temp dirs for download/extract staging
- `zip` `0.6` — archive extraction
- `tokio` `1` (features `full`), `tokio-util` `0.7` (feature `rt`) — async runtime, `CancellationToken`
- `futures-util` `0.3` — stream combinators
- **Platform-conditional:**
  - `libc` `0.2` on Unix (`src-tauri/Cargo.toml:25-26`)
  - `windows-sys` `0.59` with `Win32_Storage_FileSystem` on Windows (`src-tauri/Cargo.toml:28-30`)

## Tauri Configuration

- `productName`: "MinUI Easy Installer", `identifier`: `dev.minui.easy-installer` (`src-tauri/tauri.conf.json:4-5`)
- Window: 800×600, resizable, not fullscreen (`src-tauri/tauri.conf.json:14-20`)
- `withGlobalTauri: false` — no global `__TAURI__` injected
- Capability file: `src-tauri/capabilities/default.json` — only `core:default` permissions for the `main` window
- Bundle targets `all`; icons in `icons/` (`tauri.conf.json:30-36`)
- **CSP** (strict, defined inline in `tauri.conf.json:23`):
  - `connect-src 'self' https://packages.minui.dev https://api.github.com https://github.com https://*.githubusercontent.com`
  - `img-src 'self' data:`
  - `style-src 'self' 'unsafe-inline'`
  - `script-src 'self'`

## Package Manager & Runtimes

- **Bun** — used for install + scripts; lockfile `bun.lock` at repo root; `package.json` has `"type": "module"`
  - All `package.json` scripts use `bun run` (also works with `npm run` per `README.md`)
- **Cargo** — Rust build (`cargo tauri dev`, `cargo tauri build`)
- **Tauri CLI** invoked via `cargo tauri` (the `tauri` binary is in `$PATH` via `cargo install tauri-cli` or `bunx`)

## Task Runner

- **just** (`justfile`) — primary task runner wrapping cargo/bun:
  - `just dev` / `just build` / `just tauri-dev` / `just tauri-build`
  - `just typecheck` / `just lint` / `just fmt-ts` / `just fmt`
  - `just check` — runs lint + typecheck + `cargo fmt --check` + `cargo clippy -- -D warnings`
  - `just pre-commit` — runs `prek run --all-files`

## Linting & Formatting

- **oxlint** `^1.69.0` (Rust-based, fast) — `bun run lint` runs `oxlint src` (`package.json:11, 32`)
- **oxfmt** (`oxc-parser` `^0.135.0` dep) — `bun run fmt` runs `oxfmt src` (`package.json:12`)
- **ESLint** `^10.5.0` config also present at `.eslintrc.cjs` (extends `eslint:recommended` + `@typescript-eslint/recommended`, plugins `react-hooks`, `react-refresh`) — used via pre-commit hook
- **rustfmt** + **clippy** — `cargo fmt` / `cargo clippy -- -D warnings` (`justfile:36-37`)
- **EditorConfig** — 2-space, LF, UTF-8, final newline, trim trailing whitespace (`.editorconfig`); 4-space for `.rs`; hard tabs for `Makefile`/`justfile`
- **oxfmtrc** — `tabWidth: 2`, `printWidth: 80`, double quotes, LF (`.oxfmtrc.json`)

## Pre-commit Hooks (prek)

- `prek.toml` configures built-in hooks: `trailing-whitespace`, `end-of-file-fixer`, `check-added-large-files`, `check-yaml`, `check-json` (excludes `*.jsonc`), `mixed-line-ending --fix=lf`, `check-merge-conflict`, `check-case-conflict`
- Local hooks: `bun run lint -- --fix` and `bun run typecheck` for `*.ts`/`*.tsx`

## Testing

- **Frontend: Vitest** `^4.1.8` with **jsdom** `^29.1.1` (`vitest.config.ts:7-11`)
  - Setup file: `vitest.setup.ts` imports `@testing-library/jest-dom/vitest`
  - Coverage via `@vitest/coverage-v8` `^4.1.8`
  - **React Testing Library** `^16.3.2` + `@testing-library/user-event` `^14.6.1` + `@testing-library/jest-dom` `^6.9.1`
  - Test files: `*.test.{ts,tsx}` colocated with source (e.g. `src/Home.test.tsx`, `src/PackageStore.test.tsx`, `src/WifiWizard.test.tsx`, `src/DriveSelector.test.tsx`, `src/types/*.test.ts`, `src/hooks/useVersionCheck.test.ts`)
  - Script: `bun run test` → `vitest run` (`package.json:11`)
- **Backend: `#[cfg(test)]` modules** in each Rust file, plus a large test module in `src-tauri/src/lib.rs`
  - Uses `tempfile` for temp dirs, `tokio::test` for async
  - Run via `cargo test` (no dedicated script in `package.json`)

## Type-checking & Diagnostics

- `bun run typecheck` → `tsc --noEmit` (`package.json:10`)
- `bun run doctor` → `npx react-doctor@latest` (`package.json:13`) — see CI

## CI

- **GitHub Actions: React Doctor** (`.github/workflows/react-doctor.yml`)
  - `millionco/react-doctor@547e1e4ecdb70315d81a91ec3605701d58616ee2 # v2` pinned
  - Triggers: `pull_request` (opened/synchronize/reopened/ready_for_review) and `push` to `main`
  - **Advisory only by default** — posts sticky PR comment + review comments + commit status, does not block
  - Concurrency group `react-doctor-${{ ... }}` with `cancel-in-progress: true`
  - Checkout with `fetch-depth: 0`, `persist-credentials: false`

## Frontend Static / Configuration Files

- `index.html` — Vite shell, references `/src/main.tsx`
- `src/main.tsx` — React root + StrictMode
- `src/styles.css` — single global stylesheet
- `src/vitest.d.ts` — ambient types (e.g. `*.json` module declarations)
- No `.env` file; no bundler env-var config beyond `TAURI_DEV_HOST` in `vite.config.ts:3`

## Data Files (bundled assets)

- `src/types/store.json` — bundled fallback package registry (32 entries: emulators + tool paks)
- `src/types/device-install-map.json` — per-device install rules (17 devices) with `devicePaks` definitions and BIOS list
- These are imported as JSON modules in TS (e.g. `src/types/package.ts:2`)

## Device Targets (Runtime Footprint)

The app itself runs on **Windows 10+ and macOS 10.15+** (Tauri v2 floor, `AGENTS.md:25`). It targets installing MinUI onto SD cards for these handhelds (full list in `src/types/device.ts:31-156` and `src/types/device-install-map.json`):

- **TrimUI**: Brick, Smart Pro
- **Miyoo**: Mini, Mini+, A30, Flip, 355, MY282
- **Anbernic**: RG35XX, RG35XX Plus, RG35XX H, RG35XX SP
- **Other**: M17, GKD Pixel, MagicX, RGB30, Zero 28

## Dev Tooling Snapshots

- `package.json:42` lists `oxc-parser` — present for advanced AST needs (likely used by oxlint/oxfmt internals)
- `oxlint` `^1.69.0` is the primary linter; ESLint config kept for editor interop
- React Doctor pin: comment in `package.json:13` uses `npx react-doctor@latest` (not pinned)
