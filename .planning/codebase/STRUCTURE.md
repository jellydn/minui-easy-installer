# Codebase Structure

**Analysis Date:** 2026-06-13

## Directory Layout

```
minui-installer/
├── .planning/       # Architecture and project planning documentation
├── assets/          # Static assets (images, icons)
├── scripts/         # Developer tools and automation scripts
├── src/             # Frontend React source code
├── src-tauri/       # Backend Rust source code and Tauri configuration
└── tasks/           # Task tracking and Product Requirements Documents (PRDs)
```

## Directory Purposes

**`src/`:**
- Purpose: Frontend user interface for the installer.
- Contains: React components, CSS styles, and TypeScript domain types/API wrappers.
- Key files: `main.tsx`, `App.tsx`, `Home.tsx`, `styles.css`.

**`src/types/`:**
- Purpose: TypeScript type definitions and Tauri IPC wrapper functions.
- Contains: Domain models (device, drive, release, install) and their corresponding test files.
- Key files: `device.ts`, `install.ts`, `drive.ts`, `release.ts`.

**`src-tauri/src/`:**
- Purpose: Core backend logic for privileged OS operations.
- Contains: Rust source files.
- Key files: `lib.rs` (Tauri commands), `install.rs` (installation flow), `drives.rs` (SD card detection).

**`scripts/`:**
- Purpose: Tooling for project development.
- Contains: Autonomous AI coding loop scripts (`ralph/ralph.sh`).
- Key files: `scripts/ralph/ralph.sh`, `scripts/ralph/progress.txt`.

## Key File Locations

**Entry Points:**
- `src/main.tsx`: Frontend React mount point.
- `src-tauri/src/main.rs`: Backend executable entry point (delegates to `lib.rs`).

**Configuration:**
- `package.json`: Node.js dependencies and script definitions (vite, tsc, vitest, oxlint).
- `src-tauri/tauri.conf.json`: Tauri framework configuration, permissions, and window settings.
- `src-tauri/Cargo.toml`: Rust dependencies and metadata.
- `vite.config.ts`: Vite bundler configuration.
- `tsconfig.json`: TypeScript compiler configuration.

**Core Logic:**
- `src/Home.tsx`: Main UI state machine orchestrating the install flow.
- `src-tauri/src/lib.rs`: Exposes Rust logic to React via Tauri commands.
- `src-tauri/src/install.rs`: Main business logic for extracting and writing to the SD card.

**Testing:**
- `src/types/*.test.ts`: Vitest unit tests for frontend logic and API wrappers.

## Naming Conventions

**Files:**
- React Components: `PascalCase.tsx` (e.g., `ConfirmDialog.tsx`).
- TypeScript utilities/types: `camelCase.ts` or `kebab-case.ts` (e.g., `device.ts`).
- Rust modules: `snake_case.rs` (e.g., `download.rs`).
- Tests: `[name].test.ts`.

**Directories:**
- Frontend/General: `kebab-case` or `lowercase` (e.g., `src-tauri`, `scripts`).

## Where to Add New Code

**New Feature (UI):**
- Primary code: `src/` (create new `.tsx` component)
- Tests: `src/types/` or `src/components/` (if extracted)

**New Feature (System/Backend):**
- Implementation: `src-tauri/src/`
- IPC Binding: `src-tauri/src/lib.rs` and matching frontend wrapper in `src/types/`

**New Component/Module:**
- Implementation: Add to `src/` (or a `src/components` subdirectory if it grows).

**Utilities:**
- Shared helpers: `src/types/` (TypeScript) or new Rust modules in `src-tauri/src/`.

## Special Directories

**`src-tauri/target/`:**
- Purpose: Compiled Rust binaries and build artifacts.
- Generated: Yes
- Committed: No

**`node_modules/`:**
- Purpose: Installed npm dependencies.
- Generated: Yes
- Committed: No

---

*Structure analysis: 2026-06-13*
