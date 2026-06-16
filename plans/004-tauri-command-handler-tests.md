# Plan 004 — Add tests for Tauri command handlers in `src-tauri/src/lib.rs`

| Field        | Value                                  |
| ------------ | -------------------------------------- |
| Slug         | `tauri-command-handler-tests`          |
| Status       | pending                                |
| Priority     | High                                   |
| Category     | test coverage                          |
| Impact       | High                                   |
| Effort       | M                                      |
| Risk         | Low (additive)                         |
| Audit commit | `4d6e95a`                              |
| Depends on   | none                                   |
| Blocks       | none                                   |

## Problem

`.planning/codebase/CONCERNS.md` → "Test Coverage Gaps" →
"No Tests for `lib.rs` (Tauri Command Handlers)" — flagged as
**High** priority. The 16 Tauri commands registered in
`src-tauri/src/lib.rs:152-169` are the integration boundary between
the frontend and the backend. Today, the only thing exercising them
end-to-end is the user's machine.

Risks of this gap:
- Serialization mismatches between Rust structs (e.g. `InstallResult`)
  and TypeScript interfaces (e.g. `InstallResult` in
  `src/types/install.ts`) cause silent runtime errors at the IPC layer.
- New fields added to a Rust struct break the frontend until the
  TypeScript types catch up.
- Default-value drift (e.g. an `Option` field's `None` vs the
  frontend's expectation of `null`) goes undetected.

## Goal

A `#[cfg(test)] mod tests` block in `lib.rs` that:
1. Invokes each `#[tauri::command]` function (or its inner
   non-tauri-wrapper) with realistic inputs.
2. Asserts on the returned `Result` shape (success or expected error
   string), the populated fields, and the field types.
3. Acts as a contract test: if a future refactor changes a field name
   or type, this test fails before the change lands.

We do **not** need to spin up a full Tauri app — the command bodies
take their dependencies as plain arguments. The Tauri-specific glue
(`tauri::AppHandle`, `tauri::State`, the `emit` channel) is a thin
wrapper; we test the *body* of each command by calling the underlying
module function directly.

## Files in scope

- `src-tauri/src/lib.rs` — add a `#[cfg(test)] mod tests` block.
  No production code changes.

## Files explicitly out of scope

- `src-tauri/src/install.rs`, `package.rs`, etc. — they have their
  own tests.
- Frontend `*.test.tsx` — already exists for the major components;
  Plan 005 in the next pass would extend component tests for the
  currently untested ones (`ConfirmDialog`, `HealthCheck`, etc.).

## Current state (`src-tauri/src/lib.rs:152-169`)

```rust
.invoke_handler(tauri::generate_handler![
    get_removable_drives,
    format_drive,
    download_and_verify_archive,
    verify_archive_checksum,
    extract_archive_to_directory,
    install_minui,
    validate_installation,
    format_validation_report,
    check_minui_version,
    install_package,
    write_wifi_config,
    scan_wifi_networks,
    get_current_wifi_ssid,
    detect_installed_packages,
    check_package_updates,
    check_sd_card_health,
    fetch_url,
])
```

There are 17 commands (not 16 as CONCERNS.md says — `fetch_url` is
also there). Each is a thin wrapper that delegates to the underlying
module function.

## Approach

For each command, the test calls the underlying module function
directly, with the same arguments the wrapper would pass. We do not
test the Tauri macro attributes (those are a Tauri framework
contract, not our code).

The exceptions are commands that take a `tauri::AppHandle` or
`tauri::State` for the progress event:
- `install_minui` — needs an `AppHandle` to emit progress
- `get_removable_drives` — no Tauri state, easy
- `validate_installation` — no Tauri state, easy
- `check_minui_version` — no Tauri state, easy
- `install_package` — needs an `AppHandle` (or the progress callback
  can be a no-op closure if we can reach the underlying function
  directly)
- `write_wifi_config` — no Tauri state, easy
- `scan_wifi_networks`, `get_current_wifi_ssid` — no Tauri state, easy
- `detect_installed_packages`, `check_package_updates` — no Tauri
  state, easy
- `check_sd_card_health` — no Tauri state, easy
- `fetch_url` — easy
- `download_and_verify_archive`, `verify_archive_checksum`,
  `extract_archive_to_directory` — easy
- `format_drive` — uses `diskutil`, platform-specific

For commands that need an `AppHandle`, the executor should test the
*underlying* module function (e.g. `install::install_minui`, not
`lib::install_minui`) — the wrapper is a one-line passthrough that
adds an `AppHandle` and creates the progress callback. The
underlying functions are pure async functions with no Tauri types in
their signature.

## Step-by-step execution

### Step 1 — `get_removable_drives`

In `lib.rs::tests`:

```rust
#[test]
fn test_get_removable_drives_returns_result_shape() {
    // On a real Mac, this returns at least the boot disk OR an error
    // string. We assert on the Result shape, not the specific drives.
    let result = drives::list_removable_drives();
    // The function either returns Ok(Vec<RemovableDrive>) or Err(String).
    // We don't assert which — the test runs on macOS and Windows CI alike.
    match result {
        Ok(drives) => {
            for d in &drives {
                // Field type contract: mount_path must be non-empty
                assert!(!d.mount_path.is_empty());
                // name must be non-empty
                assert!(!d.name.is_empty());
            }
        }
        Err(msg) => {
            assert!(!msg.is_empty());
        }
    }
}
```

### Step 2 — `format_drive`

```rust
#[test]
#[cfg(target_os = "macos")]
fn test_format_drive_errors_on_nonexistent_mount() {
    // format_drive shells out to diskutil; we don't want to actually
    // format anything in tests. We only assert that nonexistent paths
    // return an error, not a panic.
    let result = format_drive("/nonexistent/this/should/not/exist", "TEST");
    assert!(result.is_err());
}

#[test]
#[cfg(target_os = "windows")]
fn test_format_drive_unsupported_on_windows() {
    let result = format_drive("D:\\", "TEST");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not yet supported"));
}
```

### Step 3 — `install_minui` (the IPC wrapper)

The wrapper:

```rust
async fn install_minui(
    app_handle: AppHandle,
    base_url: String, /* ... */
) -> Result<install::InstallResult, String> {
    let progress = Arc::new(move |event| { /* app_handle.emit(...) */ });
    install::install_minui(&options, progress).await
}
```

Test the underlying function with a no-op progress closure:

```rust
#[tokio::test]
async fn test_install_minui_underlying_errors_on_bad_url() {
    // We don't want to actually download a real archive. Call the
    // underlying function with an unreachable URL and assert the
    // error propagates as a String.
    let options = install::InstallOptions {
        base_url: "http://127.0.0.1:1/never-exists.zip".to_string(),
        extras_url: None,
        base_checksum: None,
        extras_checksum: None,
        sd_mount: "/tmp".to_string(),
        platform: "trimui".to_string(),
        extras_platform: "trimuismart".to_string(),
        version: "test".to_string(),
    };
    let result = install::install_minui(
        &options,
        Arc::new(|_| {}),  // no-op progress
    ).await;
    assert!(result.is_err());
    // Error message must be a String (the IPC contract).
    let err = result.unwrap_err();
    assert!(!err.is_empty());
}
```

### Step 4 — `validate_installation` and `format_validation_report`

```rust
#[test]
fn test_validate_installation_errors_on_nonexistent_mount() {
    let result = validate::validate_installation(
        "/nonexistent/path/that/cannot/exist",
        false,
        "/Tools",
    );
    assert!(result.is_err());
}

#[test]
fn test_validate_installation_on_empty_tempdir() {
    let temp = tempfile::tempdir().unwrap();
    let result = validate::validate_installation(
        temp.path().to_str().unwrap(),
        false,
        "/Tools",
    );
    assert!(result.is_ok());
    let v = result.unwrap();
    // Empty dir = no MinUI files = failures expected
    assert!(!v.success);
    assert!(v.failed_count > 0);
}

#[test]
fn test_format_validation_report_contains_pass_and_fail_lines() {
    let v = validate::ValidationResult {
        success: false,
        checks: vec![
            validate::ValidationCheck { name: "a".into(), passed: true,  message: "ok".into() },
            validate::ValidationCheck { name: "b".into(), passed: false, message: "bad".into() },
        ],
        passed_count: 1,
        failed_count: 1,
        free_space_bytes: Some(1024 * 1024 * 1024),
    };
    let report = validate::format_validation_report(&v);
    assert!(report.contains("ok"));
    assert!(report.contains("bad"));
    assert!(report.contains("1.00 GB"));
}
```

### Step 5 — `check_minui_version` and version helpers

```rust
#[test]
fn test_check_minui_version_on_empty_tempdir() {
    let temp = tempfile::tempdir().unwrap();
    let result = version::check_for_updates(temp.path().to_str().unwrap(), Some("2025.01.01"));
    // No minui.txt on a fresh card → installed = None, update_available = true
    assert!(result.installed.is_none());
    assert!(result.update_available);
}
```

### Step 6 — `install_package` (underlying function, not the wrapper)

```rust
#[tokio::test]
async fn test_install_package_underlying_errors_on_bad_url() {
    let rules = package::PackageInstallPathRules {
        target_dir: "/Tools".to_string(),
        extract_to_root: false,
        pak_name: "test.pak".to_string(),
    };
    let temp = tempfile::tempdir().unwrap();
    let result = package::install_package(
        "http://127.0.0.:1/never.zip",
        None,
        temp.path().to_str().unwrap(),
        &rules,
        "rg35xxplus",
    ).await;
    assert!(result.is_err());
}
```

### Step 7 — `write_wifi_config` (already covered by `wifi.rs` tests, but add a contract test)

The function's tests already live in `wifi.rs` (9 tests). The
contract we want to pin in `lib.rs` is the IPC wrapper exists and
has the right signature. **No new test needed** — skip and document.

### Step 8 — `scan_wifi_networks`, `get_current_wifi_ssid`

```rust
#[test]
fn test_scan_wifi_networks_returns_vec() {
    // Don't assert specific networks (CI-dependent). Just assert it
    // returns a Vec and doesn't panic.
    let _ = wifi::scan_wifi_networks();
}

#[test]
fn test_get_current_wifi_ssid_returns_option_string() {
    // Same — environment-dependent. Just assert the return type.
    let _ = wifi::get_current_wifi_ssid();
}
```

### Step 9 — `detect_installed_packages`, `check_package_updates`

```rust
#[test]
fn test_detect_installed_packages_empty_tempdir() {
    let temp = tempfile::tempdir().unwrap();
    let result = package::detect_installed_packages(temp.path().to_str().unwrap());
    assert!(result.is_empty());
}

#[test]
fn test_check_package_updates_empty_input() {
    let temp = tempfile::tempdir().unwrap();
    let result = package::check_package_updates(
        temp.path().to_str().unwrap(),
        &[],
    );
    assert!(result.is_empty());
}
```

### Step 10 — `check_sd_card_health`

```rust
#[test]
fn test_check_sd_card_health_errors_on_nonexistent() {
    let result = health::check_sd_card_health("/nonexistent/path/here", None);
    assert!(result.is_err());
}

#[test]
fn test_check_sd_card_health_on_empty_tempdir() {
    let temp = tempfile::tempdir().unwrap();
    let result = health::check_sd_card_health(temp.path().to_str().unwrap(), None);
    assert!(result.is_ok());
    let h = result.unwrap();
    // Empty card → no MinUI folders → failed_count > 0
    assert!(h.failed_count > 0);
}
```

### Step 11 — `fetch_url`

```rust
#[tokio::test]
async fn test_fetch_url_errors_on_unreachable() {
    let result = fetch_url("http://127.0.0.1:1/never".to_string()).await;
    assert!(result.is_err());
}
```

### Step 12 — `download_and_verify_archive` and `extract_archive_to_directory`

```rust
#[tokio::test]
async fn test_download_and_verify_archive_errors_on_unreachable() {
    let result = download::download_archive(
        "http://127.0.0.1:1/never.zip",
        None,
    ).await;
    assert!(result.is_err());
}
```

(`extract_archive_to_directory` already has tests in `extract.rs`.)

### Step 13 — Run

```bash
cd src-tauri && cargo test --lib 2>&1 | tail -10
```

Expected: 55 existing + ~12 new = ~67 tests, all pass.

## Done criteria (machine-checkable)

- `cd src-tauri && cargo test --lib` shows ≥ 67 passing tests.
- `git diff --stat src-tauri/src/lib.rs` shows only `+` lines (no
  production code changes).
- The tests reference only the public module functions
  (`drives::list_removable_drives`, `install::install_minui`, etc.)
  — no calls into the `#[tauri::command]` wrappers themselves.

## Test plan

The 12 tests above are listed individually. Each is small and
self-contained. They collectively act as a contract test for the
IPC layer.

## Maintenance note

When a new Tauri command is added, **also** add a test in this
module that calls the underlying function. Reject PRs that add
a new command without a corresponding test. A grep of
`#\[tauri::command\]` in `lib.rs` should equal the number of tests
in this `mod tests` block (give or take commands that take only
`AppHandle` and are tested via the underlying function).

## Escape hatches

- **If a test flakes because of CI-specific env (e.g. `get_free_space`
  returns `None` on a CI runner with no real disk):** add
  `#[cfg(not(target_os = "..."))]` or a `#[ignore = "..."]` attribute
  with a clear comment. Do not delete the test.
- **If `install_minui` underlying test takes too long** (5+ s for
  the unreachable URL timeout): wrap with a `tokio::time::timeout`
  of 10 s and assert the result is `Err`. Document the timeout.
- **If the executor can't reach the `AppHandle` in `install_minui`:**
  the test only needs to call the underlying
  `install::install_minui` function, which takes no `AppHandle`. The
  Tauri wrapper is a one-line passthrough that doesn't need its own
  test.
