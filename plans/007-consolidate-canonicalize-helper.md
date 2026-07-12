# Plan 007: Consolidate duplicate canonicalize_existing_ancestor helper

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat 1f0a569..HEAD -- src-tauri/src/bios.rs src-tauri/src/pipeline.rs src-tauri/src/fs_utils.rs`
> If any of these files changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding.

## Status

- **Priority**: P2
- **Effort**: S
- **Risk**: LOW
- **Depends on**: none
- **Category**: tech-debt
- **Planned at**: commit `1f0a569`, 2026-07-12

## Why this matters

The function `canonicalize_existing_ancestor` is duplicated verbatim in `src-tauri/src/bios.rs` and `src-tauri/src/pipeline.rs`. Code duplication poses maintenance and security risks: any changes to how ancestors are canonicalized or errors handled must be maintained in sync. Moving this function into `src-tauri/src/fs_utils.rs` eliminates the duplication.

## Current state

The relevant files:
- `src-tauri/src/fs_utils.rs` — contains filesystem utility functions.
- `src-tauri/src/bios.rs` — contains duplicate definition (lines 331–354).
- `src-tauri/src/pipeline.rs` — contains duplicate definition (lines 227–252).

Duplicate implementation excerpt (from `src-tauri/src/bios.rs`):
```rust
fn canonicalize_existing_ancestor(path: &Path) -> std::io::Result<PathBuf> {
    let mut current: &Path = path;
    loop {
        match current.canonicalize() {
            Ok(canonical) => return Ok(canonical),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => match current.parent() {
                Some(parent) => current = parent,
                None => return Err(e),
            },
            Err(e) => return Err(e),
        }
    }
}
```

## Commands you will need

| Purpose   | Command | Expected on success |
|-----------|---------|---------------------|
| Check build | `cargo check --tests` | exit 0, compiles with no errors |
| Check format | `cargo fmt --check` | exit 0, code formatted correctly |
| Check lint | `cargo clippy --all-targets -- -D warnings` | exit 0, no clippy warnings |
| Run tests | `cargo test` | exit 0, all tests pass |

## Scope

**In scope** (the only files you should modify):
- `src-tauri/src/fs_utils.rs`
- `src-tauri/src/bios.rs`
- `src-tauri/src/pipeline.rs`

**Out of scope**:
- Changing the logic of parent walk-up and canonicalization. Keep the implementation identical to preserve existing platform/directory resolution semantics.

## Git workflow

- Branch: `advisor/007-consolidate-canonicalize-helper`
- Commit: `refactor(fs_utils): consolidate canonicalize_existing_ancestor helper`

## Steps

### Step 1: Add canonicalize_existing_ancestor to fs_utils.rs

In `src-tauri/src/fs_utils.rs`, append `canonicalize_existing_ancestor` as a public helper.

Add to `src-tauri/src/fs_utils.rs`:
```rust
use std::path::PathBuf;

/// Walk up `path` until we find an existing ancestor, then canonicalize it.
///
/// `Path::canonicalize` requires every component to exist. On a fresh
/// install, the target parent directory tree may not exist yet. This
/// helper finds the highest existing ancestor and canonicalizes that,
/// so the caller can still reason about the path's location relative
/// to the SD card root.
///
/// Symlink-safety: if any existing ancestor is a symlink pointing outside
/// the SD card, `canonicalize` resolves through the symlink and the caller's
/// checks will reject it.
pub fn canonicalize_existing_ancestor(path: &Path) -> std::io::Result<PathBuf> {
    let mut current: &Path = path;
    loop {
        match current.canonicalize() {
            Ok(canonical) => return Ok(canonical),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => match current.parent() {
                Some(parent) => current = parent,
                None => return Err(e),
            },
            Err(e) => return Err(e),
        }
    }
}
```

**Verify**:
Run `cargo check` to verify the module compiles.

### Step 2: Use fs_utils helper in bios.rs

In `src-tauri/src/bios.rs`:
1. Remove the private `canonicalize_existing_ancestor` definition (lines 331–354).
2. Change the call in `install_bios_from_bytes` to use `fs_utils::canonicalize_existing_ancestor`.

Update:
```rust
    let canonical_parent = fs_utils::canonicalize_existing_ancestor(parent)
        .map_err(|e| format!("Failed to resolve target parent: {}", e))?;
```

**Verify**:
Run `cargo check` to ensure `bios.rs` compiles.

### Step 3: Use fs_utils helper in pipeline.rs

In `src-tauri/src/pipeline.rs`:
1. Remove the private `canonicalize_existing_ancestor` definition (lines 227–252).
2. Use the imported or fully qualified `crate::fs_utils::canonicalize_existing_ancestor` (or import `fs_utils` if not imported).

Update `pipeline.rs`:
```rust
    let canonical_parent = crate::fs_utils::canonicalize_existing_ancestor(parent)
        .map_err(|e| format!("Failed to resolve target parent: {}", e))?;
```

**Verify**:
Run `cargo check` to ensure `pipeline.rs` compiles.

## Test plan

- Run `cargo test` to verify that all existing installer tests (which exercise parent directory creation and checks) compile and pass.

## Done criteria

- [ ] `cargo check --tests` exits 0.
- [ ] `cargo clippy --all-targets -- -D warnings` exits 0.
- [ ] Duplicate `canonicalize_existing_ancestor` functions are removed from `bios.rs` and `pipeline.rs`.
- [ ] `plans/README.md` status updated.

## STOP conditions

- If `fs_utils::canonicalize_existing_ancestor` fails to compile because of path importing, resolve imports correctly rather than restoring the duplicate definition.
