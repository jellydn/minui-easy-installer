# AGENTS.md

## Project

MinUI Easy Installer & Package Store — "Balena Etcher for MinUI". Desktop app for installing/updating MinUI on retro handheld SD cards with a built-in package store.

Target devices: TrimUI Brick, TrimUI Smart Pro, Miyoo Mini+, Miyoo A30, Miyoo Flip, RG35XX Plus, RG35XX H, RG35XX SP.

## Tech Stack

- **Tauri v2** — Rust backend, React frontend
- MVP: Windows + macOS only (no Linux Phase 1)
- Package registry: static JSON at `https://packages.minui.dev/registry/index.json`

## Key Constraints

- Never write to SD card without explicit user confirmation
- Never format drives in MVP
- Never log WiFi passwords or secrets in plaintext
- Treat registry data as untrusted — validate schema before use
- Preserve user ROMs/saves/config during MinUI updates
- Extract archives to temp dir before copying to SD card

## Architecture

- **MinUI releases**: GitHub API `api.github.com/repos/shauninman/MinUI/releases/latest` (parsed in `src/types/release.ts`). Checksums not yet parsed from release metadata.
- **Package registry**: Static JSON at `packages.minui.dev/registry/index.json`
- **Device platform mapping**: Device ID = folder name in archive (e.g. `trimui-brick` → `trimui-brick/`). All 8 devices share same `installPathRules`: `baseDir="/"`, `extrasDir="/"`, `toolsDir="/Tools"`.
- **WiFi config**: Always `<sd_root>/wifi.txt` with one `SSID:PASSWORD` per line. Lines starting with `#` are comments. SSIDs can contain spaces. Same for all devices.
- **Version tracking**: MinUI base reads `minui.txt` or `.minui/version` (never written by installer). Packages read `Tools/*/version.txt` (included in archives). Installer does not write version metadata.
- **Extras**: Always installed when release includes extras archive. No user opt-out UI exists.
- **OS floor**: Windows 10+, macOS 10.15+ (Tauri v2 requirement). macOS 14.4+ has `airport` deprecation risk for WiFi scanning.

## Running Dev

```bash
# Frontend dev
bun run dev

# Full Tauri dev (Rust + React)
cargo tauri dev

# Typecheck
bun run typecheck

# Lint
bun run lint

# Test
bun test
```

## Code Organization

- Device profiles and types: `src/types/device.ts`
- Drive types and helpers: `src/types/drive.ts`
- Drive detection backend: `src-tauri/src/drives.rs`
- Archive download and verification: `src-tauri/src/download.rs`
- Archive extraction: `src-tauri/src/extract.rs`
- Archive types and helpers: `src/types/archive.ts`
- Release metadata types: `src/types/release.ts`
- Install flow backend: `src-tauri/src/install.rs`
- Install types and frontend API: `src/types/install.ts`
- Tests: `*.test.ts` files (vitest)
- Rust backend: `src-tauri/src/`
- Confirmation dialogs: `src/ConfirmDialog.tsx` (overlay modal for write operations)
- Install progress UI: `src/InstallProgress.tsx`
