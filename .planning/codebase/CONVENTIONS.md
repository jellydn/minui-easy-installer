# Coding Conventions

## TypeScript

### Linting & Formatting

| Tool | Config | Command |
|------|--------|---------|
| ESLint | `.eslintrc.cjs` | `oxlint src` (Rust-based, fast) |
| Oxfmt | `.oxfmtrc.json` | `oxfmt src` |
| TypeScript | `tsconfig.json` (strict) | `tsc --noEmit` |

### Error Handling

Two patterns are used:

**Either pattern** (preferred for UI-bound operations):
```typescript
type Result = { success: true; data: T } | { success: false; error: AppError };
```

**Try/catch** (for unexpected failures):
```typescript
try {
  const result = await invoke("command", args);
} catch (err) {
  const message = err instanceof Error ? err.message : "Unknown error";
}
```

Error classification uses `classifyError()` from `src/types/errors.ts`.

### Imports

- `@tauri-apps/api/core` → `invoke` for IPC calls
- `@tauri-apps/api/event` → `listen` for event subscriptions
- Dynamic imports for Tauri APIs (avoid bundling issues in non-Tauri contexts)

### Component Patterns

- **No CSS framework** — plain `styles.css`
- State-based navigation in `App.tsx`: `"home" | "store" | "wifi" | "bios" | "settings"`
- Confirmation dialogs as overlay modals (`ConfirmDialog`, `FormatConfirmDialog`)
- Props are typed inline or with explicit interfaces

### Testing

- See `TESTING.md` for full testing conventions
- `vi.mock()` for module mocking
- `@testing-library/jest-dom` matchers via `vitest.setup.ts`

## Rust

### Linting & Formatting

| Tool | Command |
|------|---------|
| `cargo fmt` | Format Rust code |
| `cargo clippy` | Lint with `-- -D warnings` (deny all) |
| `cargo check` | Fast compile check (no codegen) |

### Error Handling

**Return type:** `Result<T, String>` — errors are human-readable strings:

```rust
fn do_thing() -> Result<u32, String> {
    let output = Command::new("df").output()
        .map_err(|e| format!("Failed to run df: {}", e))?;
    // ...
}
```

**Expect:** Used sparingly in:
- `generate_context!()` — build-time failure
- `lock().unwrap()` — mutex poisoning is unrecoverable

### Module Organization

```rust
// In lib.rs:
mod bios;
mod download;
mod drives;
// ...
#[cfg(target_os = "macos")]
mod macos;  // conditional compilation
```

Tests are either:
- Inline: `#[cfg(test)] mod tests { ... }` inside the source file (e.g., `lib.rs`)
- External: `#[cfg(test)] #[path = "drives_tests.rs"] mod tests;`

### Naming

| Item | Convention | Example |
|------|-----------|---------|
| Functions | `snake_case` | `copy_base_files` |
| Structs | `PascalCase` | `InstallOptions` |
| Enums | `PascalCase` | `VolumeKind` |
| Constants | `SCREAMING_SNAKE_CASE` | `PRESERVED_FOLDERS` |
| Modules | `snake_case` | `fs_utils` |

### Platform Gating

```rust
#[cfg(target_os = "macos")]
pub fn list_removable_drives() -> Result<Vec<RemovableDrive>, String> { ... }

#[cfg(target_os = "windows")]
pub fn list_removable_drives() -> Result<Vec<RemovableDrive>, String> { ... }
```

### Security Patterns

- **Path sanitization:** All paths that touch the SD card are canonicalized and validated to stay within the mount root
- **Input validation:** Platform names restricted to alphanumeric + hyphens
- **No secret logging:** WiFi passwords and sensitive data are not logged
- **Atomic temp cleanup:** `InstallSession` owns `TempDir` handles — dropped atomically

### Clippy Suppressions

Only used where justified:
```rust
#[allow(clippy::too_many_arguments)]
```
or on the `pub(crate) use` re-exports for tests.

## Git

### Commits

Follow [conventional commits](https://www.conventionalcommits.org/):

```
type(scope): description

Types: feat, fix, docs, refactor, test, chore, ci, build, perf
```

### Pre-commit

`prek.toml` runs on staged files:
- Trailing whitespace removal
- EOF newline fixer
- LF normalization
- Lint `--fix`

If a hook rewrites a file, re-stage with `git add -u`.

## Build Commands

```bash
bun run dev          # Frontend only (Vite on port 1420)
cargo tauri dev      # Full Tauri dev (Rust + React)
cargo tauri build    # Production build
just check           # All checks: lint + typecheck + fmt + clippy
bun test             # Frontend tests
cargo test           # Rust tests
```
