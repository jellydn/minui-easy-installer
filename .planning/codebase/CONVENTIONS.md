# Coding Conventions

**Analysis Date:** 2026-06-13

## Naming Patterns

**Files:**
- React components: PascalCase (e.g., `ConfirmDialog.tsx`, `DeviceSelector.tsx`, `Home.tsx`)
- TypeScript utilities/types: kebab-case or snake_case (e.g., `device.ts`, `archive.test.ts`)
- Rust modules: snake_case (e.g., `download.rs`, `extract.rs`, `install.rs`)

**Functions:**
- TypeScript: camelCase (e.g., `getDeviceProfile`, `getAllDeviceProfiles`)
- Rust: snake_case (e.g., `verify_checksum`, `download_archive`)

**Variables:**
- TypeScript: camelCase for standard variables, UPPER_SNAKE_CASE for constants (e.g., `DEVICE_PROFILES`)
- Rust: snake_case for local variables and parameters

**Types:**
- TypeScript: PascalCase for `interface` and `type` definitions (e.g., `DeviceProfile`, `InstallPathRules`)
- Rust: PascalCase for `struct` and `enum` definitions (e.g., `DownloadResult`, `DownloadProgress`)

## Code Style

**Formatting:**
- **Tool used:** `oxfmt` for TypeScript/React (frontend), `cargo fmt` implicitly for Rust.
- **Key settings:** Stated in `package.json` (`npm run fmt` uses `oxfmt src`).

**Linting:**
- **Tool used:** `oxlint` (configured via `package.json` script `"lint": "oxlint src"`) and `eslint` with `@typescript-eslint` for deeper static analysis. Rust utilizes `cargo clippy`.
- **Key rules:**
  - TypeScript: Strict mode enabled (`"strict": true` in `tsconfig.json`). Unused locals/parameters checked.

## Import Organization

**Order:**
- Standard ES module imports are used. External modules (e.g., React, Tauri APIs) typically precede internal imports.
- In Rust, `use` statements generally follow `std::...` and external crates, grouped at the top.

**Path Aliases:**
- No custom path aliases observed in `tsconfig.json` or `vite.config.ts`. Relative imports are standard (`../types/device`, `./device`).

## Error Handling

**Patterns:**
- **Rust (Tauri Backend):** Errors are explicitly handled using the `Result<T, String>` pattern to easily pass stringified error messages back to the Tauri frontend. Extensively utilizes `.map_err(|e| format!("...", e))?` for context-rich error propagation.
- **TypeScript:** Tauri command invocations return Promises that resolve or reject, usually handled via standard async/await try-catch blocks.

## Logging

**Framework:** `console` for frontend. Standard `println!` or logging frameworks for Rust backend if required.

**Patterns:**
- Simple console logging for frontend debugging.

## Function Design

**Size:**
- Small to medium. Tauri command wrappers in `lib.rs` are extremely thin, delegating directly to implementation files (e.g., `extract::extract_archive`).

**Parameters:**
- Typed arguments. Rust commands heavily use `Option<String>` for optional frontend inputs, converted using `.as_deref()` before passing to underlying logic.

**Return Values:**
- Structured typed objects (e.g., `Result<ValidationResult, String>` in Rust, Promise-wrapped objects in TS).

## Module Design

**Exports:**
- TypeScript: Named exports are standard (`export function...`, `export interface...`). Default exports are mostly reserved for configuration files (like `vite.config.ts`) and main React components (e.g., `App`).
- Rust: Functions intended for Tauri frontend are annotated with `#[tauri::command]` in `lib.rs` or explicitly exported as `pub fn` from child modules (`pub fn download_archive`).

---

*Convention analysis: 2026-06-13*
