# Conventions

## Rust

### Module Organization

- **One module per file**. `lib.rs` declares all modules with `mod`.
- **Test splits**: Use `#[cfg(test)] #[path = "module_tests.rs"] mod tests;` for extracted test files. This follows the established `version/tests.rs` pattern. Test files start with `use super::*;` (no `mod tests {}` wrapper).

### Security Patterns

The codebase takes SD card safety seriously. These patterns must be preserved:

1. **Canonicalize before write** (`create_target_within`): Resolve the parent path's canonical form, verify it's within the SD card root, **then** create directories, then re-verify. This catches symlink races.

2. **Extract to temp, then copy**: Archives are never extracted directly to the SD card. All extraction happens in temp directories, then files are selectively copied.

3. **Symlink-dereferencing copy**: `copy_dir_recursive` uses `fs::copy` which dereferences symlinks by default — preventing symlink escape attacks.

4. **Input sanitization**: Platform names are sanitized to alphanumeric + hyphens. BIOS filenames are checked for traversal, NUL bytes, and path separators.

### Platform Gating

- Use `#[cfg(target_os = "macos")]` for macOS-specific code
- Use `#[cfg(not(target_os = "macos"))]` for non-macOS fallbacks
- Use `#[cfg_attr(not(target_os = "macos"), allow(unused_variables))]` for parameters unused on some platforms (prefer over `_param` prefix when the parameter is used on other platforms)
- Gate test functions that call platform-gated production code with the same `#[cfg]` attribute

### Async & Cancellation

- All Tauri commands are `async fn`
- Long-running operations use `tokio::spawn` and check `CancellationToken::is_cancelled()` at phase boundaries
- `InstallRegistry` holds a single `Arc<Mutex<Option<CancellationToken>>>` — new installs cancel old ones
- Progress callbacks use `Arc<dyn Fn(Event) + Send + Sync>` for thread-safe sharing

### Error Handling

- Tauri commands return `Result<T, String>` — errors propagate as user-facing strings
- Internal errors use `Result<T, String>` or `Option<T>` with descriptive messages
- `map_err(|e| format!("context: {}", e))` for adding context to errors
- Contract tests in `lib.rs` verify error propagation shapes (not just success paths)

### Testing

- Unit tests use `tempfile::tempdir()` — no real SD card needed
- `#[tokio::test]` for async test functions
- Contract tests in `lib.rs` test the IPC boundary: error shapes, return types, edge cases
- `#[cfg(test)]` gates on production functions kept only for test use (preferred over `#[allow(dead_code)]`)

## TypeScript

### Component Patterns

- **State-based navigation**: `App.tsx` uses a `Screen` union type (`"home" | "store" | "wifi" | "bios" | "settings"`) — no router
- **Custom hooks** for complex logic: `useForkInstall`, `useVersionCheck`, `useMountEffect`, `useScrollToBottom`
- **Context** for cross-component state: `ForkContext` for fork selection
- **No CSS framework**: All styles in `src/styles.css` (plain CSS)

### Type Organization

- Types live in `src/types/` — one file per domain (`device.ts`, `drive.ts`, `install.ts`, etc.)
- Interfaces for data shapes (`DeviceProfile`, `RemovableDrive`, `InstallProgressEvent`)
- Type-only imports: `import type { ... }` for compile-time safety
- Union types for discriminated states (e.g., `Screen`, `InstallPhase`)

### IPC Conventions

- Frontend calls Rust via `invoke("command_name", { args })` from `@tauri-apps/api/core`
- Events are listened via `listen("event-name", callback)` from `@tauri-apps/api/event`
- IPC wrapper functions in `src/types/` (e.g., `installMinui`, `fetchPackageRegistry`)
- Error types use `AppError` with `classifyError` for safe error handling

### Testing

- Vitest with `jsdom` environment
- `@testing-library/react` for component rendering and queries
- `@testing-library/user-event` for simulated interactions
- Mock Tauri `invoke` with `vi.mock("@tauri-apps/api/core")`
- Test files colocated: `Component.tsx` + `Component.test.tsx`

## Shared Conventions

### Commits

- [Conventional Commits](https://www.conventionalcommits.org/) format: `type(scope): subject`
- Types: `feat`, `fix`, `refactor`, `docs`, `test`, `ci`, `chore`
- Subject in imperative mood, max 72 chars

### Code Quality

- **No dead code**: `#[allow(dead_code)]` only when truly needed; prefer `#[cfg(test)]` for test-only functions
- **No magic values**: Extract unexplained literals into named constants
- **One concern per function**: Extract logical sections into well-named helpers
- **Guard clauses**: Bail out early at the top rather than nesting

### Pre-commit (prek)

- `prek.toml` enforces: trailing whitespace removal, EOF newline, LF normalization, oxlint `--fix`
- If a hook rewrites a file, re-stage with `git add -u` before retrying commit
