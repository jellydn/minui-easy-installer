# AGENTS.md

## Project

MinUI Easy Installer & Package Store — "Balena Etcher for MinUI". Desktop app for installing/updating MinUI on retro handheld SD cards with a built-in package store.

Target devices: TrimUI Brick, TrimUI Smart Pro, Miyoo Mini+, Miyoo A30, Miyoo Flip, RG35XX Plus, RG35XX H, RG35XX SP.

## Tech Stack

- **Tauri v2** — Rust backend (`src-tauri/`), React 18 frontend (`src/`)
- **Bun** — JS runtime & package manager
- **oxlint** + **oxfmt** — Rust-based linting/formatting (fast, no config)
- **Vitest** — test runner (jsdom env, setup via `vitest.setup.ts`)
- **No CSS framework** — plain `styles.css`, no Tailwind/shadcn
- MVP: Windows + macOS only (no Linux Phase 1)

## Key Constraints

- Never write to SD card without explicit user confirmation
- Never format drives in MVP
- Never log WiFi passwords or secrets in plaintext
- Treat registry data as untrusted — validate schema before use
- Preserve user ROMs/saves/config during MinUI updates (`roms`, `saves`, `save`, `bios`, `cheats` folders are case-insensitively preserved)
- Extract archives to temp dir before copying to SD card

## Architecture

- **MinUI releases**: GitHub API `api.github.com/repos/shauninman/MinUI/releases/latest` (parsed in `src/types/release.ts`)
- **Package registry**: Static JSON fetched from `https://packages.minui.dev/registry/index.json` with session-scoped cache
- **CSP**: Tightly scoped in `tauri.conf.json` — allowlist includes `packages.minui.dev`, `api.github.com`, `github.com`, `*.githubusercontent.com`
- **Install pipeline** (`src-tauri/src/pipeline.rs`): Download → extract → copy, all three phases managed by `InstallSession` which owns temp dirs and drops them atomically on completion
- **Cancellation**: `start_install` spawns in background task with `CancellationToken`, emits `install-progress` / `install-complete` / `install-error` events. Old synchronous `install_minui` command is deprecated.
- **Device platform mapping**: Device ID = folder name in archive (e.g. `trimui-brick`). All 8 devices use `baseDir="/"`, `extrasDir="/"`, `toolsDir="/Tools"`. Platform names come from `device-install-map.json` (`basePlatform` for base archive, `extrasPlatform` for extras archive — can differ per device).
- **WiFi config**: `<sd_root>/wifi.txt`, one `SSID:PASSWORD` per line. `#` comments. SSIDs can contain spaces. Same format for all devices.
- **Version tracking**: Installer now **writes** `minui.txt` (format: `{fork_name} {version}` where `fork_name` defaults to "MinUI"). Packages read `Tools/*/version.txt` (included in archives).
- **Extras**: Always installed when release includes extras archive. No user opt-out UI. Extras failure is non-fatal (logged as warning).
- **OS floor**: Windows 10+, macOS 10.15+. macOS 14.4+ has `airport` deprecation risk for WiFi scanning.

## Running Dev

```bash
# Frontend only (Vite on port 1420)
bun run dev

# Full Tauri dev (Rust + React)
cargo tauri dev

# Build
cargo tauri build
```

## Checks

```bash
# TypeScript
bun run typecheck        # tsc --noEmit
bun run lint             # oxlint src
bun run fmt              # oxfmt src

# Rust
cd src-tauri && cargo fmt --check
cd src-tauri && cargo clippy -- -D warnings

# Test
bun test                 # vitest run (src/**/*.test.{ts,tsx})
cd src-tauri && cargo test

# All checks at once
just check               # lint + typecheck + cargo fmt --check + cargo clippy
just fmt                 # oxfmt + cargo fmt
```

## Pre-commit (prek)

Config in `prek.toml`. Auto-rewrites staged files (trailing whitespace, EOF fixer, LF normalization, lint `--fix`). **If a hook rewrites a file, re-stage with `git add -u` and retry** — two hooks touching the same file in one commit can cause a dirty index that blocks the commit.

## Code Organization

- Frontend entry: `src/main.tsx` → `src/App.tsx` (state-based navigation: "home" | "store" | "wifi" | "bios" | "settings")
- Rust entry: `src-tauri/src/main.rs` → `src-tauri/src/lib.rs` (all Tauri commands registered here)
- Device profiles: `src/types/device.ts` + `src/types/device-install-map.json`
- Drive detection: `src-tauri/src/drives.rs` (platform-specific macOS/Windows)
- Install flow: `src-tauri/src/install.rs` + `src-tauri/src/pipeline.rs`
- Archive download/extraction: `src-tauri/src/download.rs` + `src-tauri/src/extract.rs`
- Package store: `src/PackageStore.tsx` + `src/types/package.ts` + `src/types/store.json`
- SD health check: `src-tauri/src/health.rs`
- Validation: `src-tauri/src/validate.rs`
- WiFi: `src-tauri/src/wifi.rs` (scan via `airport` on macOS, write config)
- BIOS: `src-tauri/src/bios.rs` (catalog + status + install_bios_from_bytes) and `src/BiosInstaller.tsx` (UI). The user supplies copyrighted BIOS files; the installer copies them to the right `Bios/<subdir>/` path.
- Confirmation dialogs: `src/ConfirmDialog.tsx` (overlay modal for write ops)
- Install progress UI: `src/InstallProgress.tsx`

## Security Patterns

- `pipeline.rs::create_target_within`: Canonicalizes path ancestors to verify they stay within SD card root _before_ creating directories, then re-validates _after_ to catch symlink races
- `install.rs::copy_extras_files`: Sanitizes `extras_platform` (alphanumeric + hyphens only)
- `bios.rs::install_bios_from_bytes`: Sanitizes subdir/filename (no traversal, NUL, or path separators), canonicalizes the target parent _before_ write and re-validates the canonical path _after_ write (symlink-race guard)
- `fs_utils.rs::copy_dir_recursive`: `fs::copy` dereferences symlinks (no symlink escape)
- Registry data: validate schema before use (see `src/types/validate.ts`)

## Testing Quirks

- Rust tests use `tempfile` temp dirs extensively — no real SD card needed
- Test functions in `lib.rs` contract-test the IPC boundary: error propagation, return shapes
- WiFi tests are environment-dependent (no specific networks asserted)
- Health check/file-not-found tests use `/nonexistent` paths (works cross-platform)
