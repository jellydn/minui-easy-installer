# Architecture

**Analysis Date:** 2026-06-13

## Pattern Overview

**Overall:** Tauri Desktop Application (IPC-based Client-Server pattern)

**Key Characteristics:**
- Separation of concerns between UI (React/TypeScript) and system operations (Rust).
- Secure execution of privileged operations (filesystem access, formatting, SD card writes) via strict Tauri command boundaries.
- Asynchronous message passing (IPC) between frontend and backend.
- Thin frontend relying on a static JSON package registry and robust backend validation.

## Layers

**Frontend (React/TypeScript):**
- Purpose: User Interface, installation flow state management, and device/drive selection.
- Location: `src/`
- Contains: React components, styling (`styles.css`), and TypeScript API wrappers for Tauri commands (`src/types/*.ts`).
- Depends on: Tauri IPC (`@tauri-apps/api`), internal domain types.
- Used by: End users.

**Tauri Command Boundary (Rust):**
- Purpose: Expose secure system-level functions to the frontend.
- Location: `src-tauri/src/lib.rs`
- Contains: `#[tauri::command]` annotated functions.
- Depends on: Core Backend logic.
- Used by: Frontend (via `invoke`).

**Core Backend (Rust):**
- Purpose: Execute heavy and privileged system operations like drive detection, HTTP downloading, archive verification, and extraction.
- Location: `src-tauri/src/` (e.g., `drives.rs`, `install.rs`, `download.rs`, `extract.rs`)
- Contains: Rust business logic.
- Depends on: Local OS filesystem, network.
- Used by: Tauri Command Boundary.

## Data Flow

**Installation Flow:**
1. User selects a Device and an SD Card (Drive) in the React UI (`DeviceSelector`, `DriveSelector`).
2. Frontend fetches release metadata from the package registry (`fetchMinUIRelease`).
3. User confirms installation; Frontend triggers `installMinui` (TypeScript wrapper).
4. Wrapper calls the Rust backend via Tauri IPC `invoke("install_minui", ...)`.
5. Rust backend downloads the archive (`download.rs`), verifies checksums, extracts it to a temp directory (`extract.rs`), and copies necessary files to the SD card (`install.rs`).
6. Rust returns success/failure to the Frontend.
7. Frontend updates UI state and optionally triggers `validate_installation`.

**State Management:**
- Frontend state is localized in React hooks (e.g., `useState` in `Home.tsx` and `App.tsx`).
- No global state management library is used; props are passed down from `Home.tsx` to child components.

## Key Abstractions

**Device Profile:**
- Purpose: Represents the target handheld device and its platform-specific path rules.
- Examples: `src/types/device.ts`
- Pattern: Static configuration mapping device ID to install paths.

**Drive Model:**
- Purpose: Represents a removable SD card detected on the host OS.
- Examples: `src/types/drive.ts`, `src-tauri/src/drives.rs`
- Pattern: Cross-language struct mapping (Rust struct returned as JSON to TypeScript type).

**Archive & Release Metadata:**
- Purpose: Manages MinUI release info, download URLs, and checksums.
- Examples: `src/types/release.ts`, `src/types/archive.ts`

## Entry Points

**Frontend Application:**
- Location: `src/main.tsx`
- Triggers: Tauri webview initialization.
- Responsibilities: Renders the React application root.

**Backend Application:**
- Location: `src-tauri/src/main.rs` & `src-tauri/src/lib.rs`
- Triggers: OS Application launch.
- Responsibilities: Bootstraps the Tauri runtime, registers IPC commands, and opens the initial window.

## Error Handling

**Strategy:** Explicit Result mapping across the IPC boundary.

**Patterns:**
- Rust backend uses `Result<T, String>` to pass errors back to the frontend.
- Frontend wrappers catch IPC errors and convert them into domain-specific Error objects or UI error states.
- `ValidationReport` UI component is used to present detailed system validation errors to the user.

## Cross-Cutting Concerns

**Validation:**
- Strict validation of the SD card installation performed post-install via `validate_installation` command (`src-tauri/src/validate.rs`), ensuring structural integrity of the SD card files.

**Security:**
- User confirmation explicitly required before any SD card writes (`ConfirmDialog.tsx`).
- Checksum validation applied to downloaded archives.

---

*Architecture analysis: 2026-06-13*
