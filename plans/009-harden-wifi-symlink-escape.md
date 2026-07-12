# Plan 009: Harden WiFi config writing against symlink escape

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat 1f0a569..HEAD -- src-tauri/src/wifi.rs`
> If the file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding.

## Status

- **Priority**: P1
- **Effort**: S
- **Risk**: LOW
- **Depends on**: none
- **Category**: security
- **Planned at**: commit `1f0a569`, 2026-07-12

## Why this matters

The function `write_wifi_config` writes the WiFi configuration directly to `wifi.txt` at the SD card root. However, it does not check if `wifi.txt` is a pre-existing symlink. If a symlink exists at that path pointing outside the SD card, `fs::write` will follow it and overwrite the destination file. Deleting `wifi.txt` if it exists (or is a symlink) before writing ensures that the file is created directly on the SD card.

## Current state

Relevant file:
- `src-tauri/src/wifi.rs` — contains `write_wifi_config` and its unit tests.

Excerpt of write logic in `write_wifi_config` (lines 54-58):
```rust
    let content = format!("{}\n", entries.join("\n"));

    fs::write(&wifi_path, content).map_err(|e| format!("Failed to write wifi.txt: {}", e))?;

    Ok(())
```

## Commands you will need

| Purpose   | Command | Expected on success |
|-----------|---------|---------------------|
| Run tests | `cargo test --package minui-easy-installer --lib -- wifi::tests` | exit 0, all wifi tests pass |

## Scope

**In scope**:
- `src-tauri/src/wifi.rs`

**Out of scope**:
- Modifying other components of the installer outside of `wifi.rs`.

## Git workflow

- Branch: `advisor/009-harden-wifi-symlink-escape`
- Commit: `security(wifi): delete existing wifi.txt before write to prevent symlink escape`

## Steps

### Step 1: Remove existing file/symlink at wifi.txt before writing

Modify `write_wifi_config` in `src-tauri/src/wifi.rs` to check for and remove any existing file/symlink at `wifi_path` before calling `fs::write`.

Update `src-tauri/src/wifi.rs` (just before `fs::write`):
```rust
    let content = format!("{}\n", entries.join("\n"));

    // If wifi.txt exists (or is a symlink), remove it to break any potential symlink escapes.
    if fs::symlink_metadata(&wifi_path).is_ok() {
        fs::remove_file(&wifi_path)
            .map_err(|e| format!("Failed to remove existing wifi.txt file/symlink: {}", e))?;
    }

    fs::write(&wifi_path, content).map_err(|e| format!("Failed to write wifi.txt: {}", e))?;
```

**Verify**:
Run `cargo test --package minui-easy-installer --lib -- wifi::tests` to ensure tests compile.

### Step 2: Add unit test for wifi.txt symlink escape prevention

In `src-tauri/src/wifi.rs`, inside `mod tests`, add a new test case checking that writing WiFi config to a target that is already a symlink resolves to replacing the symlink with a regular file, without following it to the outside path.

Add to `src-tauri/src/wifi.rs` (inside `mod tests`):
```rust
    #[test]
    #[cfg(unix)]
    fn test_write_wifi_config_rejects_symlink_escape() {
        let temp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let sd = temp.path();

        let wifi_path = sd.join("wifi.txt");
        let outside_file = outside.path().join("leak.txt");
        fs::write(&outside_file, b"original").unwrap();

        // Create a symlink at target pointing to outside
        std::os::unix::fs::symlink(&outside_file, &wifi_path).unwrap();

        write_wifi_config(sd.to_str().unwrap(), "SSID", "password").unwrap();

        // Verify outside file was NOT modified/followed
        assert_eq!(fs::read(&outside_file).unwrap(), b"original");
        // Verify local file was written as a regular file containing the SSID config
        let content = fs::read_to_string(&wifi_path).unwrap();
        assert!(content.contains("SSID:password"));
        let meta = fs::symlink_metadata(&wifi_path).unwrap();
        assert!(!meta.file_type().is_symlink(), "wifi.txt must be a regular file, not a symlink");
    }
```

**Verify**:
Run tests and confirm the new test passes:
```bash
cargo test --package minui-easy-installer --lib -- wifi::tests::test_write_wifi_config_rejects_symlink_escape
```

## Test plan

- Run `cargo test --package minui-easy-installer --lib -- wifi::tests` to verify that all WiFi tests pass.

## Done criteria

- [ ] All 10+ WiFi tests pass.
- [ ] No compilation warnings in `wifi.rs`.
- [ ] No files outside `src-tauri/src/wifi.rs` are modified.
- [ ] `plans/README.md` status updated.

## STOP conditions

- If `fs::remove_file` fails due to permissions in the test environment, stop and report.
