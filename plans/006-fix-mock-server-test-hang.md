# Plan 006: Fix Mock Server Connection Count in Cancel Install Test

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat 1f0a569..HEAD -- src-tauri/src/install.rs`
> If the file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding.

## Status

- **Priority**: P1
- **Effort**: S
- **Risk**: LOW
- **Depends on**: none
- **Category**: tests
- **Planned at**: commit `1f0a569`, 2026-07-12

## Why this matters

The test `test_install_minui_with_cancel_full_pipeline` starts a mock TCP server that accepts `max_connections` (configured as `3`). Since the test options do not specify an extras archive URL, the installer only makes `1` download connection. The mock server thread blocks on `listener.accept()` waiting for the second and third connections. The test thread then blocks on `server_handle.join()`, causing the test runner to hang indefinitely. Changing the mock server to expect exactly `1` connection fixes the hang.

## Current state

Relevant file:
- `src-tauri/src/install.rs` — contains the installer tests and `start_file_server` helper.

Excerpts:
`src-tauri/src/install.rs` (lines 457-460):
```rust
        let zip_path = archive_dir.join("MinUI.zip");
        create_minui_base_zip(&zip_path, "miyoo354");

        let (url, server_handle) = start_file_server(zip_path, 3);
```

`src-tauri/src/install.rs` (lines 417-434):
```rust
    fn start_file_server(
        file_path: std::path::PathBuf,
        max_connections: usize,
    ) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let handle = thread::spawn(move || {
            for _ in 0..max_connections {
                let mut connection = match listener.accept() {
                    Ok((c, _)) => c,
                    Err(_) => break,
                };
```

## Commands you will need

| Purpose   | Command | Expected on success |
|-----------|---------|---------------------|
| Run test  | `cargo test --package minui-easy-installer --lib -- install::tests::test_install_minui_with_cancel_full_pipeline --exact` | exit 0, test passes |
| Run all tests | `cargo test` | exit 0, all 156+ tests pass without hanging |

## Scope

**In scope** (the only files you should modify):
- `src-tauri/src/install.rs`

**Out of scope**:
- Modifying `start_file_server` to dynamically close or time out. A hardcoded connection count of `1` is correct for the current test setup.

## Git workflow

- Branch: `advisor/006-fix-mock-server-test-hang`
- Commit: `test(install): fix mock server connection count in cancel test`

## Steps

### Step 1: Change connection count from 3 to 1 in full pipeline cancel test

In `src-tauri/src/install.rs`, locate `test_install_minui_with_cancel_full_pipeline` and update the `start_file_server` connection limit.

Change:
```rust
        let (url, server_handle) = start_file_server(zip_path, 3);
```
To:
```rust
        let (url, server_handle) = start_file_server(zip_path, 1);
```

**Verify**:
Run the specific test to make sure it compiles and passes:
```bash
cargo test --package minui-easy-installer --lib -- install::tests::test_install_minui_with_cancel_full_pipeline --exact
```

## Test plan

- Run the entire Rust backend test suite to make sure it finishes successfully in a few seconds without hanging:
```bash
cargo test
```

## Done criteria

- [ ] `cargo test` runs all backend tests and exits 0 in under 15 seconds.
- [ ] No files outside `src-tauri/src/install.rs` are modified.
- [ ] `plans/README.md` status updated.

## STOP conditions

- If the test still hangs after changing the connection count to 1, stop and report.
- If `install.rs` does not contain `start_file_server(zip_path, 3)`, stop and report.

## Maintenance notes

- If this test is expanded in the future to download an extras zip as well, `max_connections` in the server must be incremented to match the number of mock HTTP requests the installer will issue.
