# Coding Conventions

## TypeScript

### Configuration
- **Target**: ES2020
- **Module**: ESNext with bundler resolution
- **Strict mode**: Enabled
- **JSX**: `react-jsx`
- **No emit**: TypeScript used for type-checking only (Vite handles bundling)

### Naming
- **Files**: PascalCase for components (`App.tsx`, `PackageCard.tsx`), camelCase for utilities/hooks/types (`device.ts`, `useForkInstall.ts`)
- **Types/Interfaces**: PascalCase (`DeviceProfile`, `InstallResult`, `PackageRegistryEntry`)
- **Functions**: camelCase (`getDeviceProfile`, `copyBaseFiles`, `fetchPackageRegistry`)
- **Test files**: Co-located `.test.ts` / `.test.tsx` files

### React Patterns
- **Functional components** only (no class components)
- **Hooks** for state and side effects
- **No `useEffect` for data fetching** — use `useMountEffect` escape hatch or direct event handlers
- **State-based navigation** — `useState<Screen>()` with conditional rendering
- **Props drilling** — passed through component tree (no global state manager)
- **Context** for cross-cutting concerns (`ForkContext`)

### Error Handling
- **Either types**: `PackageInstallResultEither`, `PackageRegistryFetchResult`
- **Error codes**: `"INVALID_ENTRY" | "VALIDATION_ERROR" | "PARSE_ERROR" | "NETWORK_ERROR" | "NOT_FOUND" | "UNKNOWN_ERROR"`
- **`classifyError()`** in `src/types/errors.ts` for categorizing backend errors
- **Try/catch** with typed fallback values for Tauri `invoke()` calls

### Data Validation
- Registry data from external sources is validated before use (`isStoreRegistry`, `validateStoreEntry`)
- `store.json` is bundled as a fallback when the remote registry is unavailable
- Schema validation uses type guards and shape checks

## Rust

### Style
- **Edition**: 2021
- **Formatting**: `cargo fmt` (standard Rust style)
- **Linting**: `cargo clippy -- -D warnings`
- **Line width**: 80 characters (matching oxfmt)

### Module Organization
- **One module per file** (e.g., `install.rs`, `wifi.rs`, `bios.rs`)
- **Sub-modules for complex domains**: `version/mod.rs` + `version/tests.rs`
- **Command registration**: All Tauri commands registered in `lib.rs`
- **Public API**: Functions exposed to Tauri commands are `pub`

### Error Handling
- **Result<T, String>** — primary error type (simple, serializable)
- **Error propagation**: `?` operator with `.map_err()` for context
- **String errors**: `format!("Failed to do X: {}", e)`

### Async
- **Tokio** runtime with `#[tokio::test]` for async tests
- **`tokio_util::sync::CancellationToken`** for cancellation
- **`reqwest`** with `stream` feature for streaming downloads

### Testing Patterns
- `#[test]` for unit tests, `#[tokio::test]` for async
- `#[cfg(test)]` modules within source files
- `tempfile::tempdir()` for filesystem simulation
- IPC contract tests in `lib.rs` test error propagation and return shapes
- `/nonexistent` paths for file-not-found tests

## Security Conventions

### Path Validation
- **Canonicalize before create**: Validate parent directory is within expected root
- **Re-validate after create**: Catch symlink race conditions
- **`create_target_within()`** — canonical ancestor check → create → canonicalize → re-check
- **`copy_dir_recursive()`** — `fs::copy` dereferences symlinks (no traversal)

### Input Sanitization
- **Platform names**: Alphanumeric + hyphens only (`copy_extras_files`)
- **BIOS paths**: No traversal (`..`, `/`), NUL bytes, or path separators
- **Registry data**: Full schema validation before use

### Atomic Operations
- **Temp dirs**: `tempfile::TempDir` for all archive extraction
- **InstallSession**: Owns temp dirs, drops atomically on completion
- **No partial writes**: All files extracted to temp before copying to SD

### General Security Rules
- Never write to SD card without explicit user confirmation (`ConfirmDialog`)
- Never format drives in MVP
- Never log WiFi passwords or secrets in plaintext
- Treat registry data as untrusted

## Formatting & Linting

| Tool | Scope | Config |
|------|-------|--------|
| **oxfmt** | TypeScript/TSX | 2-space, LF, 80 width, double quotes |
| **ESLint** | TypeScript/TSX | `recommended` + `@typescript-eslint/recommended` |
| **oxlint** | TypeScript/TSX | Fast Rust-based linting |
| **cargo fmt** | Rust | Standard Rust style |
| **cargo clippy** | Rust | `-D warnings` (deny all) |

### Pre-commit (`prek.toml`)
- `trailing-whitespace` — Remove trailing whitespace
- `end-of-file-fixer` — Ensure files end with newline
- `check-added-large-files` — Prevent large file commits
- `mixed-line-ending --fix=lf` — Enforce LF
- `check-merge-conflict` — Detect unresolved conflicts
- `check-case-conflict` — Detect case-sensitive filename conflicts
- `bun-lint` — Run `eslint --fix`
- `bun-typecheck` — Run `tsc`

## Commit Convention

Uses changesets (`.changeset/`) with conventional commit messages:
- `feat:` — New features
- `fix:` — Bug fixes
- `refactor:` — Code restructuring
- `test:` — Test additions/changes
- `docs:` — Documentation
- `chore:` — Maintenance

Recent examples:
- `refactor: delete dead parallel device system and deprecated archive commands`
- `fix(installer): copy only selected device files, improve validation`
- `feat: custom fork support for community MinUI builds`
- `feat(install): stream archive downloads and add cancel mechanism`
