# Testing

## Frameworks

| Scope | Framework | Runner | Environment |
|-------|-----------|--------|-------------|
| TypeScript unit/component | Vitest | `bun test` / `npx vitest run` | jsdom |
| TypeScript hook tests | `@testing-library/react` | Vitest | jsdom |
| TypeScript vanilla tests | Vitest | Vitest | Node (no jsdom needed) |
| Rust unit tests | `#[test]` + `#[tokio::test]` | `cargo test` | Native |
| Rust IPC contract tests | `#[test]` in `lib_tests.rs` | `cargo test` | Native |

## Running Tests

```bash
# TypeScript (Vitest)
bun test                      # Run all
npx vitest run                # Run all (no watch)

# Rust
cd src-tauri && cargo test    # Run all (175+ tests)

# All checks
just check                    # lint + typecheck + cargo fmt + cargo clippy
```

## Test Organization

### TypeScript — `src/`

```
Component.tsx           →  Component.test.tsx          (co-located)
types/foo.ts            →  types/foo.test.ts           (co-located)
hooks/useFoo.ts         →  hooks/useFoo.test.ts        (co-located)
lib/InstallOrchestrator.ts → lib/InstallOrchestrator.test.ts  (co-located)
```

19 test files, 147 tests total.

### Rust — `src-tauri/src/`

```
foo.rs                  →  #[cfg(test)] #[path = "foo_tests.rs"] mod tests;
lib.rs                  →  #[cfg(test)] #[path = "lib_tests.rs"] mod tests;
```

Named test modules: `install_tests.rs`, `install_copy_tests.rs`, `install_extras_tests.rs`, `install_manager_tests.rs`, `bios_tests.rs`, `version/tests.rs`, `lib_tests.rs`.

## Mocking Strategy

### TypeScript

```typescript
// Tauri event listener
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

// IPC functions (one per domain)
vi.mock("../types/release", () => ({ fetchMinUIRelease: vi.fn() }));
vi.mock("../types/install", () => ({
  startInstallAndWait: vi.fn(),
  cancelInstall: vi.fn(),
}));
vi.mock("../types/validate", () => ({ validateInstallation: vi.fn() }));
vi.mock("../types/package", () => ({
  fetchPackageRegistry: vi.fn(),
  installPackage: vi.fn(),
}));
```

Mock functions return `Result<T, AppError>` objects or reject with `Error`/`string` (Tauri v2 invoke).

### Rust

- `MockDispatcher` in `install_manager.rs` records events into `Vec`s for assertions
- `tempfile::tempdir()` for isolated filesystem tests
- `/nonexistent/path/here` for file-not-found scenarios
- Real `reqwest::Client` with unreachable URLs (`127.0.0.1:1`) for network error tests

## Test Categories

### InstallOrchestrator tests (9 tests, no React)

- Initial state (idle, isInstalling=false)
- Subscribe emits initial state synchronously
- dismissInstall resets to idle
- dismissValidation clears validationResult
- cancel sets error phase
- start errors when device unknown
- start errors when install fails
- start completes successfully with validation
- isInstalling getter (downloading=true, idle/complete/error=false)

### useForkInstall tests (4 tests, jsdom + ForkProvider)

- installMinUI surfaces error when startInstallAndWait throws
- installMinUI handles Tauri v2 plain-string rejections
- installMinUI surfaces version-metadata warning
- dismissInstall resets to initial state

### InstallManager tests (4 tests)

- Poisoned mutex returns error
- Cancel on idle is no-op
- Start cancels previous install
- Smoke: start doesn't panic

### Health check tests (5 tests)

- Nonexistent mount errors
- Empty card reports missing folders
- Card with folders reports them present
- Read speed benchmark creates + cleans temp file
- scan_pak_dirs discovers .pak directories recursively

### IPC contract tests (lib_tests.rs, 17+ tests)

Each `#[tauri::command]` has a contract test that calls the underlying function directly:
- Shape tests: verify return types and error propagation
- Error tests: verify errors on nonexistent paths / bad URLs
- Round-trip tests: verify real operations on temp dirs

## Coverage Expectations

- All IPC commands have at least one contract test
- All new modules include co-located tests
- Security-critical paths (symlink guards, path sanitization) have targeted tests
- High coverage is not enforced; correctness of critical paths is prioritized
