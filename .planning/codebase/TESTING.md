# Testing

## Frontend (TypeScript)

### Framework

- **Runner**: [Vitest](https://vitest.dev/)
- **Environment**: `jsdom` (configured in `vitest.config.ts`)
- **Setup**: `vitest.setup.ts`
- **UI Testing**: `@testing-library/react` + `@testing-library/user-event`

### Structure

Test files are colocated with source files:

```
src/
├── Home.tsx
├── Home.test.tsx
├── PackageStore.tsx
├── PackageStore.test.tsx
├── types/
│   ├── package.ts
│   ├── package.test.ts
│   ├── release.ts
│   └── release.test.ts
```

### Mocking Tauri

Tauri IPC calls are mocked at the module level:

```typescript
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
```

Events are mocked via:

```typescript
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));
```

### Coverage

- **Provider**: v8 (built into Vitest)
- **Thresholds**: 50% statements, 50% lines, 40% branches, 40% functions
- **Exclusions**: Test files (`*.test.*`), `main.tsx` (entry point)
- **Config**: `vitest.config.ts`

### Running

```bash
bun test                    # Run all tests
bun run test:coverage       # Run with coverage report
```

## Backend (Rust)

### Framework

- **Runner**: `cargo test`
- **Async tests**: `#[tokio::test]` attribute
- **Temp files**: `tempfile::tempdir()` for isolated test directories
- **No mocking library**: Tests use real implementations with temp dirs

### Structure

Three layers of tests:

| Layer | Location | Purpose |
|-------|----------|---------|
| Unit tests | Inline in source files or `*_tests.rs` | Test individual functions |
| Contract tests | `lib.rs` (`#[cfg(test)] mod tests`) | Verify IPC boundary: error shapes, return types, edge cases |
| Integration tests | `drives_tests.rs` (ignored) | Test against real SD cards |

### Test File Pattern

Test modules use the `#[path]` attribute:

```rust
// In drives.rs:
#[cfg(test)]
#[path = "drives_tests.rs"]
mod tests;
```

```rust
// In drives_tests.rs:
use super::*;
// ... test functions without `mod tests {}` wrapper
```

### Contract Tests (lib.rs)

Each Tauri command handler has at least one contract test:
- `get_removable_drives` → tests return shape (Ok or Err, non-empty strings)
- `start_install` → tests unreachable URL returns error
- `validate_installation` → tests nonexistent path, empty tempdir
- `install_package` → tests bad URL error propagation
- `write_wifi_config` → (covered by wifi.rs tests)
- `check_minui_version` → tests empty tempdir returns None installed
- `install_bios_file` → tests round-trip write + read
- `check_sd_card_health` → tests nonexistent path, empty tempdir

### Running

```bash
cargo test                  # All tests
cargo test --all-targets    # Including doctests
cargo test -p <crate>       # Specific crate
```

## CI

### Rust CI (`.github/workflows/rust.yml`)

Runs on PR open/sync/reopen and push to `main`:

| Step | Command |
|------|---------|
| Format | `cargo fmt --check` |
| Clippy | `cargo clippy --all-targets -- -D warnings` |
| Test | `cargo test --all-targets` |

Includes cargo cache (`actions/cache@v4`) for `~/.cargo` and `target/`.

### Pre-commit

`prek.toml` runs on staged files:
- Trailing whitespace removal
- EOF newline enforcement  
- LF normalization
- oxlint `--fix`

## Test Philosophy

1. **No real SD cards in tests** — `tempfile` everywhere
2. **Contract tests verify IPC shapes** — error propagation, return types, not just success
3. **Security tests for path safety** — symlink escape, traversal, canonicalize guards
4. **Platform-gated tests** — Mac-only code gets `#[cfg(target_os = "macos")]` tests
5. **No flaky tests** — avoid network calls in tests, use local TCP listeners for mock servers
