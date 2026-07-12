# Plan 008: Harden BIOS file installation against symlink escape

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat 1f0a569..HEAD -- src-tauri/src/bios.rs`
> If the file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding.

## Status

- **Priority**: P1
- **Effort**: S
- **Risk**: LOW
- **Depends on**: plans/007-consolidate-canonicalize-helper.md
- **Category**: security
- **Planned at**: commit `1f0a569`, 2026-07-12

## Why this matters

The function `install_bios_from_bytes` resolves the target path on the SD card (e.g. `Bios/GB/gb_bios.bin`) and validates that the parent directory stays within the SD card root. However, it does not check if the target *file path itself* is already a symlink. If a symlink already exists at that path pointing outside the SD card, `fs::write` will follow it and overwrite files outside the SD card. Deleting the target file path if it exists before writing breaks the symlink chain, forcing a direct write to the SD card.

## Current state

Relevant file:
- `src-tauri/src/bios.rs` — contains `install_bios_from_bytes` and its unit tests.

Excerpt of write logic in `install_bios_from_bytes` (lines 306-312):
```rust
    // Create the parent dir (e.g. Bios/GB) and write.
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
    }
    fs::write(&target, &bytes)
        .map_err(|e| format!("Failed to write {}: {}", target.display(), e))?;
```

## Commands you will need

| Purpose   | Command | Expected on success |
|-----------|---------|---------------------|
| Run tests | `cargo test --package minui-easy-installer --lib -- bios::tests` | exit 0, all bios tests pass |

## Scope

**In scope**:
- `src-tauri/src/bios.rs`

**Out of scope**:
- Modifying standard file systems permissions.
- Modifying other components of the installer outside of `bios.rs`.

## Git workflow

- Branch: `advisor/008-harden-bios-symlink-escape`
- Commit: `security(bios): delete target file before write to prevent symlink escape`

## Steps

### Step 1: Remove existing file/symlink at target path before writing

Modify `install_bios_from_bytes` in `src-tauri/src/bios.rs` to check for and remove any existing file/symlink at `target` before calling `fs::write`.

Update `src-tauri/src/bios.rs` (just before `fs::write`):
```rust
    // If target exists (or is a symlink), remove it to break any potential symlink escapes.
    if fs::symlink_metadata(&target).is_ok() {
        fs::remove_file(&target)
            .map_err(|e| format!("Failed to remove existing file/symlink at target {}: {}", target.display(), e))?;
    }

    fs::write(&target, &bytes)
        .map_err(|e| format!("Failed to write {}: {}", target.display(), e))?;
```

**Verify**:
Run `cargo test --package minui-easy-installer --lib -- bios::tests` to ensure tests still compile.

### Step 2: Add unit test for leaf-level symlink escape prevention

In `src-tauri/src/bios.rs`, inside `mod tests`, add a new test case checking that writing to a target which is already a symlink resolves to replacing the symlink with a regular file, without following it to the outside path.

Add to `src-tauri/src/bios.rs` (inside `mod tests`):
```rust
    #[test]
    #[cfg(unix)]
    fn test_install_rejects_leaf_symlink_escape() {
        use base64::engine::general_purpose::STANDARD as BASE64;
        use base64::Engine as _;

        let temp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let sd = temp.path();

        let target_dir = sd.join("Bios/GB");
        fs::create_dir_all(&target_dir).unwrap();

        let target_file = target_dir.join("gb_bios.bin");
        let outside_file = outside.path().join("leak.bin");
        fs::write(&outside_file, b"original").unwrap();

        // Create a symlink at target pointing to outside
        symlink(&outside_file, &target_file).unwrap();

        let result = install_bios_from_bytes(
            sd.to_str().unwrap(),
            "gb_bios",
            &BASE64.encode(b"new_data"),
        );

        assert!(result.is_ok());
        // Verify outside file was NOT modified/followed
        assert_eq!(fs::read(&outside_file).unwrap(), b"original");
        // Verify local file was written as a regular file
        assert_eq!(fs::read(&target_file).unwrap(), b"new_data");
        let meta = fs::symlink_metadata(&target_file).unwrap();
        assert!(!meta.file_type().is_symlink(), "Target file must be a regular file, not a symlink");
    }
```

**Verify**:
Run tests and confirm the new test passes:
```bash
cargo test --package minui-easy-installer --lib -- bios::tests::test_install_rejects_leaf_symlink_escape
```

## Test plan

- Run `cargo test --package minui-easy-installer --lib -- bios::tests` to verify that all BIOS installation tests pass.

## Done criteria

- [ ] All 17+ BIOS tests pass.
- [ ] No compilation warnings in `bios.rs`.
- [ ] No files outside `src-tauri/src/bios.rs` are modified.
- [ ] `plans/README.md` status updated.

## STOP conditions

- If `fs::remove_file` fails due to permissions in the test environment, stop and report.
