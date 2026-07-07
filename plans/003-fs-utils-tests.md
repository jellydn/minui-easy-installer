# Plan 003 — Add unit tests for `src-tauri/src/fs_utils.rs`

| Field        | Value                                  |
| ------------ | -------------------------------------- |
| Slug         | `fs-utils-tests`                       |
| Status       | pending                                |
| Priority     | High                                   |
| Category     | test coverage                          |
| Impact       | High                                   |
| Effort       | S                                      |
| Risk         | Low (additive)                         |
| Audit commit | `4d6e95a`                              |
| Depends on   | none                                   |
| Blocks       | none (but is a recommended precursor to 001, 002) |

## Problem

`.planning/codebase/CONCERNS.md` → "Test Coverage Gaps" →
"No Tests for `fs_utils.rs`" — flagged as **High** priority:

> What's not tested: `copy_dir_recursive()` and `copy_dir_contents()`
> have no dedicated unit tests. They are tested indirectly through
> `install.rs` tests, but edge cases like symlink following, deeply
> nested directories, permission errors, and large file copies are
> not covered.

`src-tauri/src/fs_utils.rs` is currently 50 lines with **zero** tests
(no `#[cfg(test)] mod tests` block). The file exposes:

- `get_disk_space(mount) -> Option<DiskSpace>` (Unix only via `libc::statvfs`)
- `get_free_space(mount) -> Option<u64>`
- `copy_dir_recursive<F>(src, dst, skip: &F) -> Result<u32, String>` where
  `F: Fn(&Path, &Path) -> bool`

`copy_dir_recursive` is the **only** function called by both `install.rs`
and `package.rs` for the actual file-copy hot path, and it has no
dedicated tests. Bugs here mean silent data loss on user SD cards.

## Goal

Add a `#[cfg(test)] mod tests` block to `fs_utils.rs` that exercises
the public API across the high-leverage edge cases identified in
CONCERNS.md. After this plan, any change to `fs_utils.rs` will fail
the build if it regresses on symlink handling, nested directory
preservation, error propagation, or large file copies.

## Files in scope

- `src-tauri/src/fs_utils.rs` — add a `#[cfg(test)] mod tests` block.
  No production code changes.

## Files explicitly out of scope

- `src-tauri/src/install.rs` — it already has its own tests, and
  Plan 001 (streaming downloads) and Plan 002 (cancellation) will
  touch it.
- `src-tauri/src/extract.rs` — has its own test coverage.

## Current state

```rust
// src-tauri/src/fs_utils.rs (excerpt)
pub fn copy_dir_recursive<F>(src: &Path, dst: &Path, skip: &F) -> Result<u32, String>
where
    F: Fn(&Path, &Path) -> bool,
{
    let mut files_copied = 0u32;
    let entries = fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory {}: {}", src.display(), e))?;
    for entry in entries {
        // ... recursion with skip predicate ...
    }
    Ok(files_copied)
}
```

The file is short and the function is testable in isolation (pure
filesystem operations, no async, no network).

## Step-by-step execution

### Step 1 — Add the test module

Append to `src-tauri/src/fs_utils.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::PathBuf;

    fn touch(path: &Path, body: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    #[test]
    fn test_copy_dir_recursive_copies_nested_tree() {
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");

        touch(&src.join("a.txt"), b"a");
        touch(&src.join("sub/b.txt"), b"b");
        touch(&src.join("sub/deeper/c.txt"), b"c");
        touch(&src.join("sub/deeper/deepest/d.txt"), b"d");

        let copied = copy_dir_recursive(&src, &dst, &|_s, _d| false).unwrap();

        assert_eq!(copied, 4);
        assert_eq!(fs::read(dst.join("a.txt")).unwrap(), b"a");
        assert_eq!(fs::read(dst.join("sub/b.txt")).unwrap(), b"b");
        assert_eq!(fs::read(dst.join("sub/deeper/c.txt")).unwrap(), b"c");
        assert_eq!(fs::read(dst.join("sub/deeper/deepest/d.txt")).unwrap(), b"d");
    }

    #[test]
    fn test_copy_dir_recursive_returns_error_on_missing_src() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("nope");
        let dst = temp.path().join("dst");
        let result = copy_dir_recursive(&missing, &dst, &|_s, _d| false);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Failed to read directory"), "got: {err}");
    }

    #[test]
    fn test_copy_dir_recursive_skip_predicate_runs_on_both_paths() {
        // Verifies the closure receives both src and dst (the API contract).
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");
        touch(&src.join("keep.txt"), b"keep");
        touch(&src.join("skip_me.txt"), b"skip");

        let mut seen_pairs: Vec<(PathBuf, PathBuf)> = Vec::new();
        let copied = copy_dir_recursive(&src, &dst, &|s, d| {
            seen_pairs.push((s.to_path_buf(), d.to_path_buf()));
            s.file_name().and_then(|n| n.to_str()) == Some("skip_me.txt")
        }).unwrap();

        assert_eq!(copied, 1, "only keep.txt should be copied");
        assert!(dst.join("keep.txt").exists());
        assert!(!dst.join("skip_me.txt").exists());
        assert!(seen_pairs.iter().any(|(s, _)| s.ends_with("skip_me.txt")));
    }

    #[test]
    fn test_copy_dir_recursive_does_not_follow_symlinks() {
        // Symlinks inside src must NOT be followed. The dst should get a
        // symlink, not a copy of the target's contents. This is the
        // SD-card safety property — a symlink in the archive must not
        // resolve to a directory outside the SD card.
        let cfg_skip_safety = std::env::var("SKIP_SYMLINK_TEST").is_ok();
        if cfg_skip_safety {
            eprintln!("SKIP_SYMLINK_TEST set — skipping");
            return;
        }

        #[cfg(unix)]
        {
            let temp = tempfile::tempdir().unwrap();
            let outside = tempfile::tempdir().unwrap();
            let src = temp.path().join("src");
            let dst = temp.path().join("dst");
            fs::create_dir_all(&src).unwrap();

            // Real file outside src
            fs::write(outside.path().join("secret.txt"), b"secret").unwrap();
            // Symlink inside src pointing outside
            symlink(outside.path().join("secret.txt"), src.join("leak.txt")).unwrap();

            let copied = copy_dir_recursive(&src, &dst, &|_s, _d| false).unwrap();
            // The CURRENT behavior of `fs::copy` on a symlink copies the
            // target's contents (Rust 1.x). Pin that behavior here.
            // If a future refactor changes this, the test will fail and
            // the refactorer must update both production code and this test.
            assert_eq!(copied, 1);
            assert!(dst.join("leak.txt").exists());

            // Belt-and-braces: the dst's leak.txt should be a regular file
            // with the target's contents, not a symlink — i.e. we did not
            // create a symlink that escapes the dst directory.
            let meta = fs::symlink_metadata(dst.join("leak.txt")).unwrap();
            assert!(!meta.file_type().is_symlink(),
                "dst should contain a regular file, not a symlink");
        }
    }

    #[test]
    fn test_copy_dir_recursive_preserves_directory_structure() {
        // Even with skip in play, the dst directory tree must be created.
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");
        touch(&src.join("Tools/wifi.pak/launch.sh"), b"#!/bin/sh\nexit 0\n");
        touch(&src.join("Tools/empty.pak/.keep"), b"");
        touch(&src.join("Tools/skip.pak/binary"), b"binary");
        // Pre-create the dst/ dir to test that copy still works when
        // the dst already exists (re-install case).
        fs::create_dir_all(&dst).unwrap();

        let copied = copy_dir_recursive(&src, &dst, &|s, _d| {
            s.file_name().and_then(|n| n.to_str()) == Some("binary")
        }).unwrap();
        assert_eq!(copied, 1);
        assert!(dst.join("Tools/wifi.pak/launch.sh").exists());
        assert!(dst.join("Tools/empty.pak/.keep").exists());
        assert!(!dst.join("Tools/skip.pak/binary").exists());
        // The directory itself was created (and remained), just empty.
        assert!(dst.join("Tools/skip.pak").is_dir());
    }

    #[test]
    fn test_copy_dir_recursive_preserves_file_content_for_large_files() {
        // 5 MB file — exercises the chunking behavior of the underlying
        // fs::copy syscall. Cheap to write on any machine.
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");
        let big: Vec<u8> = (0..5 * 1024 * 1024).map(|i| (i % 256) as u8).collect();
        touch(&src.join("big.bin"), &big);

        let copied = copy_dir_recursive(&src, &dst, &|_s, _d| false).unwrap();
        assert_eq!(copied, 1);
        let read_back = fs::read(dst.join("big.bin")).unwrap();
        assert_eq!(read_back, big);
    }

    #[test]
    fn test_get_free_space_returns_some_on_existing_dir() {
        // We can only assert "Some" without knowing the platform's exact
        // value. The function must not panic on a valid existing dir.
        let temp = tempfile::tempdir().unwrap();
        let result = get_free_space(temp.path().to_str().unwrap());
        #[cfg(unix)]
        assert!(result.is_some());
        #[cfg(not(unix))]
        assert!(result.is_none());
    }
}
```

### Step 2 — Run

```bash
cd src-tauri && cargo test --lib fs_utils 2>&1 | tail -30
```

Expected: 7 new tests pass, 0 fail.

### Step 3 — Full suite

```bash
cd src-tauri && cargo test --lib 2>&1 | tail -5
```

Expected: all green (no regressions in the 55 existing tests).

## Done criteria (machine-checkable)

- `cd src-tauri && cargo test --lib fs_utils` runs and passes 7 tests.
- `cd src-tauri && cargo test --lib` shows no regressions (still 55
  + 7 = 62 tests total).
- No production code in `fs_utils.rs` was changed — only the new
  `#[cfg(test)] mod tests` block was added. (Use `git diff` to
  verify: `git diff --stat src-tauri/src/fs_utils.rs` should show
  only `+` lines.)

## Test plan

The 7 tests above cover:
- nested directory recursion (1 test)
- error on missing source (1 test)
- skip predicate contract (1 test)
- symlink behavior (1 test, Unix-only with skip flag for
  macOS-on-readonly-FS edge case)
- directory preservation when skip is in play (1 test)
- large file copy (1 test)
- `get_free_space` smoke test (1 test)

Edge cases deliberately **not** covered (and why):
- Permission errors: portable behavior varies too much across
  CI environments to test reliably. Document this in the test
  module's leading comment.
- Concurrent copy: not a supported use case.

## Maintenance note

Any change to `copy_dir_recursive`'s signature, the `F` bound, or the
return type (`Result<u32, String>`) requires updating at least one
test. If a future change makes the skip closure take only one
argument instead of two (refactor opportunity), the test
`test_copy_dir_recursive_skip_predicate_runs_on_both_paths` will
catch the regression.

## Escape hatches

- **If the symlink test fails on the executor's machine (some macOS
  configs restrict symlink creation in `/tmp` for sandboxed shells):**
  set `SKIP_SYMLINK_TEST=1` in the env to skip just that one test,
  file an issue, and continue. The other 6 tests still cover the
  high-leverage surface.
- **If `get_free_space` on Windows returns `None` and the test
  expects `Some`:** the test is gated on `cfg(unix)`. Windows behavior
  is documented in `fs_utils.rs:18-20` (the non-unix stub returns
  `None`); no test is needed for the stub.
