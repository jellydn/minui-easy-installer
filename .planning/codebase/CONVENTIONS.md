# Code Conventions

## Formatting & Linting

| Scope | Tool | Command |
|-------|------|---------|
| TypeScript | oxfmt | `bun run fmt` |
| TypeScript | oxlint | `bun run lint` |
| Rust | cargo fmt | `cd src-tauri && cargo fmt` |
| Rust | cargo clippy | `cd src-tauri && cargo clippy -- -D warnings` |
| All | just | `just check` (runs all four) |

Pre-commit hooks (`prek.toml`): trailing whitespace removal, EOF newline, LF normalization, lint `--fix`.

## TypeScript Style

- **Strict mode** enabled in `tsconfig.json`
- No CSS framework — all styles in `src/styles.css`
- IPC wrappers in `src/types/` — one file per domain (`install.ts`, `release.ts`, etc.)
- Components co-located with tests: `HealthCheck.tsx` ↔ `HealthCheck.test.tsx`
- Mocking: `vi.mock()` for Tauri IPC modules (`@tauri-apps/api/event`, `../types/release`, etc.)
- Error handling: `errorMessage(err)` normalizes `Error | string | {message}` values
- IPC results: `Result<T, AppError>` pattern from `src/types/errors.ts`

## Rust Style

- `Result<T, String>` for all Tauri commands (Tauri v2 rejects with plain strings)
- `#[serde(rename_all = "camelCase")]` on all IPC types
- `#[cfg(test)]` modules with `#[path = "..."]` for test file location
- Comments: `///` doc comments on public items, `//` for inline explanation
- No `unsafe` code
- `tempfile` for all temp directory needs (auto-cleanup)
- `cargo fmt --check` + `cargo clippy -- -D warnings` in CI

## Error Handling

### TypeScript (`src/types/errors.ts`)

```typescript
// Normalize any error value to a string
errorMessage(err: unknown): string

// Wrap any value in an Error with stack trace
asError(err: unknown): Error

// Classify Rust error strings into error codes
classifyError(errorMsg: string, defaultCode?: AppErrorCode): AppErrorCode
```

### Rust

- IPCs return `Result<T, String>` (Tauri v2 convention)
- Extras archive failure is non-fatal (logged as warning in `InstallResult.extras_warning`)
- Install cancellation returns `Err("Install cancelled")`
- Poisoned mutex returns `Err("Internal error: state lock is poisoned")`

## Security Patterns

### Symlink Escape Prevention

| Function | File | Pattern |
|----------|------|---------|
| `create_target_within` | `pipeline.rs` | Canonicalize ancestor → validate → create → re-validate canonical |
| `install_bios_from_bytes` | `bios.rs` | Sanitize subdir/filename → canonicalize parent → write → re-validate canonical |
| `copy_dir_recursive` | `fs_utils.rs` | `fs::copy` dereferences symlinks (no symlink escape) |
| `copy_extras_files` | `install.rs` | Sanitize `extras_platform` (alphanumeric + hyphens only) |

### Other Security Rules

- Never write to SD card without explicit user confirmation (`ConfirmDialog.tsx`)
- Never format drives in MVP (command exists but never called from UI)
- Never log WiFi passwords or secrets
- Registry data validated before use (`validate.ts`)
- `fetch_url` restricted to hardcoded `ALLOWED_URLS` (SSRF prevention)
- CSP tightly scoped in `tauri.conf.json`

## Commit Convention

Conventional commits: `<type>(<scope>): <subject>`

| Type | Usage |
|------|-------|
| `feat` | New feature |
| `fix` | Bug fix |
| `refactor` | Code restructuring without behavior change |
| `perf` | Performance improvement |
| `test` | Test addition or fix |
| `docs` | Documentation only |
| `style` | Formatting (no logic change) |
| `chore` | Build/maintenance |

## IPC Conventions

- Options structs with `#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]` and `#[serde(rename_all = "camelCase")]`
- Commands prefixed by domain: `check_sd_card_health`, `install_package`, `scan_wifi_networks`
- Async commands for I/O-bound operations, sync for in-memory operations
- `tauri::State<'_, Arc<InstallManager>>` for shared state
- `AppHandle` for event emission in commands
