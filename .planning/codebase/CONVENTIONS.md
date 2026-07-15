# Coding Conventions

## Rust (`src-tauri/`)

### Formatting & Linting
- `cargo fmt` — enforced in CI (`cargo fmt --check`)
- `cargo clippy -- -D warnings` — zero warnings allowed

### Naming
- `snake_case` for functions, variables, modules
- `CamelCase` for types, structs, enums
- `SCREAMING_SNAKE_CASE` for constants

### Documentation
- `///` doc comments on all `pub` and `pub(crate)` functions
- `//` inline comments for non-obvious logic (explain *why*, not *what*)

### Module Organization
- Platform-specific code in `drives/{macos,linux,windows}.rs` and `wifi/{macos,linux,windows}.rs`
- `#[cfg(target_os = "...")]` module declarations in the dispatcher file
- Tests in `#[cfg(test)] mod tests` within platform files, or in `*_tests.rs` for cross-platform tests

### Error Handling
- IPC commands return `Result<T, String>` — errors are serialized as strings
- `eprintln!` used for non-fatal warnings (event emit failure, temp cleanup failure)
- Panic allowed only for unrecoverable state (e.g., mutex poison on main thread)

### Patterns
- Tauri commands accept single `#[derive(Deserialize)]` structs, not individual params
- `if let Ok(...)` for graceful failure handling in spawned tasks
- `.unwrap()` only in main-thread code where failure means app crash

### Security
- Path canonicalization + re-validation for symlink race guards (`create_target_within`, `install_bios_from_bytes`)
- Platform name sanitization (alphanumeric + hyphens only)
- `fs::copy` dereferences symlinks (no symlink escape via `copy_dir_recursive`)

## TypeScript (`src/`)

### Formatting & Linting
- `oxfmt` — zero-config formatter
- `oxlint` — Rust-based linter (0 warnings, 0 errors in CI)
- ESLint for additional rules (`no-async-promise-executor`, React hooks)

### Naming
- `camelCase` for variables, functions
- `PascalCase` for components, types, interfaces
- `kebab-case` for file names

### Patterns
- No `any` or `@ts-ignore` in main `src/` codebase
- `console.error` only for genuine errors; no `console.log` in production
- React components use plain CSS classes from `styles.css` (no CSS-in-JS)
- State management via React context (`ForkContext`) + `useState`/`useCallback`
- No router — state-based navigation in `App.tsx`

### Component Structure
- Each component in its own file
- Custom hooks in `hooks/` directory
- Type definitions in `types/` directory
- Tests co-located with source files as `*.test.ts` / `*.test.tsx`

### Fork System
- `FORK_PRESETS` is the single source of truth for known forks
- `buildCustomFork()` creates `ForkConfig` from `owner/repo` string
- `rehydrateFork()` matches stored state to a preset or creates custom
- Fork selection persisted to `localStorage` via `ForkProvider`

## General

### File Size
- Source files target <500 lines
- Test files may exceed (e.g., `install_copy_tests.rs` at 478 lines)
- Largest source files: `install.rs` (512), `useForkInstall.ts` (425), `validate.rs` (420)

### Git
- Conventional commits: `feat:`, `fix:`, `docs:`, `style:`, `refactor:`, `test:`, `ci:`, `chore:`
- Pre-commit hooks via `prek` (trailing whitespace, EOF, LF, lint --fix)
- Squash-merge to main

### No TODOs
- Zero `TODO`, `FIXME`, `HACK`, `XXX`, `WORKAROUND`, or `BUG` annotations
- Tracked work in GitHub Issues rather than inline comments
