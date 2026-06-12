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

## Running Dev

```bash
# Frontend dev
npm run dev

# Full Tauri dev (Rust + React)
cargo tauri dev

# Typecheck
npm run typecheck

# Lint
npm run lint

# Test
npm test
```

## Code Organization

- Device profiles and types: `src/types/device.ts`
- Drive types and helpers: `src/types/drive.ts`
- Drive detection backend: `src-tauri/src/drives.rs`
- Tests: `*.test.ts` files (vitest)
- Rust backend: `src-tauri/src/`
- Confirmation dialogs: `src/ConfirmDialog.tsx` (overlay modal for write operations)

## Ralph Agent Loop

The `scripts/ralph/ralph.sh` script runs autonomous coding iterations:

```bash
# Run with defaults (amp, 10 iterations)
./scripts/ralph/ralph.sh

# Run with opencode
./scripts/ralph/ralph.sh 10 opencode

# Run with mino (opencode fork)
./scripts/ralph/ralph.sh 10 mino
```

PRD lives at `tasks/prd-minui-easy-installer-package-store.md`. Progress logged to `scripts/ralph/progress.txt`.

## Open Questions (from PRD)

1. Authoritative source for MinUI release metadata/checksums?
2. Exact platform folder mapping per device?
3. Canonical `wifi.txt` location/format per device?
4. MVP: choose Extras install or always default?
5. How to record installed package versions?
6. Registry: mirror under packages.minui.dev or point to GitHub releases?
7. Minimum Windows/macOS versions?

Resolve these before implementing affected stories.
