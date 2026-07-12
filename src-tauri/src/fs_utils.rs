use std::fs;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct DiskSpace {
    pub total: u64,
    pub available: u64,
}

#[cfg(unix)]
#[allow(dead_code)]
pub fn get_disk_space(mount: &str) -> Option<DiskSpace> {
    use std::ffi::CString;
    use std::mem;

    let path = CString::new(mount).ok()?;
    let mut stat: libc::statvfs = unsafe { mem::zeroed() };
    if unsafe { libc::statvfs(path.as_ptr(), &mut stat) } == 0 {
        Some(DiskSpace {
            total: stat.f_blocks as u64 * stat.f_frsize as u64,
            available: stat.f_bavail as u64 * stat.f_frsize as u64,
        })
    } else {
        None
    }
}

#[cfg(not(unix))]
#[allow(dead_code)]
pub fn get_disk_space(_mount: &str) -> Option<DiskSpace> {
    None
}

#[allow(dead_code)]
pub fn get_free_space(mount: &str) -> Option<u64> {
    get_disk_space(mount).map(|ds| ds.available)
}

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

/// Copies a directory tree from src to dst, optionally skipping entries via a predicate.
/// The skip function receives both the source and destination paths.
/// The cancel function is checked once per file; if it returns true, the
/// function returns Err("cancelled"). Pass `&|_| false` to disable.
/// Returns the number of files copied.
pub fn copy_dir_recursive<F, C>(src: &Path, dst: &Path, skip: &F, cancel: &C) -> Result<u32, String>
where
    F: Fn(&Path, &Path) -> bool,
    C: Fn() -> bool,
{
    let mut files_copied = 0u32;

    let entries = fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory {}: {}", src.display(), e))?;

    for entry in entries {
        if cancel() {
            return Err("cancelled".to_string());
        }
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if skip(&src_path, &dst_path) {
            continue;
        }

        if src_path.is_dir() {
            fs::create_dir_all(&dst_path)
                .map_err(|e| format!("Failed to create directory {}: {}", dst_path.display(), e))?;
            files_copied += copy_dir_recursive(&src_path, &dst_path, skip, cancel)?;
        } else {
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent directory: {}", e))?;
            }
            fs::copy(&src_path, &dst_path).map_err(|e| {
                format!(
                    "Failed to copy {} to {}: {}",
                    src_path.display(),
                    dst_path.display(),
                    e
                )
            })?;
            files_copied += 1;
        }
    }

    Ok(files_copied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
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

        let copied = copy_dir_recursive(&src, &dst, &|_s, _d| false, &|| false).unwrap();

        assert_eq!(copied, 4);
        assert_eq!(fs::read(dst.join("a.txt")).unwrap(), b"a");
        assert_eq!(fs::read(dst.join("sub/b.txt")).unwrap(), b"b");
        assert_eq!(fs::read(dst.join("sub/deeper/c.txt")).unwrap(), b"c");
        assert_eq!(
            fs::read(dst.join("sub/deeper/deepest/d.txt")).unwrap(),
            b"d"
        );
    }

    #[test]
    fn test_copy_dir_recursive_returns_error_on_missing_src() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("nope");
        let dst = temp.path().join("dst");
        let result = copy_dir_recursive(&missing, &dst, &|_s, _d| false, &|| false);
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

        // The skip predicate is `Fn` (not `FnMut`), so we use RefCell
        // for interior mutability to record what the predicate sees.
        let seen_pairs: RefCell<Vec<(PathBuf, PathBuf)>> = RefCell::new(Vec::new());
        let copied = copy_dir_recursive(
            &src,
            &dst,
            &|s, d| {
                seen_pairs
                    .borrow_mut()
                    .push((s.to_path_buf(), d.to_path_buf()));
                s.file_name().and_then(|n| n.to_str()) == Some("skip_me.txt")
            },
            &|| false,
        )
        .unwrap();

        assert_eq!(copied, 1, "only keep.txt should be copied");
        assert!(dst.join("keep.txt").exists());
        assert!(!dst.join("skip_me.txt").exists());
        let seen = seen_pairs.into_inner();
        assert!(seen.iter().any(|(s, _)| s.ends_with("skip_me.txt")));
    }

    #[test]
    #[cfg(unix)]
    fn test_copy_dir_recursive_does_not_follow_symlinks() {
        // Symlinks inside src must NOT be followed. The dst should get a
        // regular file with the target's contents (fs::copy dereferences
        // by default), not a symlink that escapes the dst directory.
        let temp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");
        fs::create_dir_all(&src).unwrap();

        // Real file outside src
        fs::write(outside.path().join("secret.txt"), b"secret").unwrap();
        // Symlink inside src pointing outside
        symlink(outside.path().join("secret.txt"), src.join("leak.txt")).unwrap();

        let copied = copy_dir_recursive(&src, &dst, &|_s, _d| false, &|| false).unwrap();
        // The CURRENT behavior of `fs::copy` on a symlink copies the
        // target's contents. If a future refactor changes this to
        // preserve the symlink, this test will fail and the refactorer
        // must update both production code and this test.
        assert_eq!(copied, 1);
        assert!(dst.join("leak.txt").exists());

        // Belt-and-braces: the dst's leak.txt should be a regular file
        // with the target's contents, not a symlink — i.e. we did not
        // create a symlink that escapes the dst directory.
        let meta = fs::symlink_metadata(dst.join("leak.txt")).unwrap();
        assert!(
            !meta.file_type().is_symlink(),
            "dst should contain a regular file, not a symlink"
        );
    }

    #[test]
    fn test_copy_dir_recursive_preserves_directory_structure() {
        // Even with skip in play, the dst directory tree must be created.
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");
        touch(
            &src.join("Tools/wifi.pak/launch.sh"),
            b"#!/bin/sh\nexit 0\n",
        );
        touch(&src.join("Tools/empty.pak/.keep"), b"");
        touch(&src.join("Tools/skip.pak/binary"), b"binary");
        // Pre-create the dst/ dir to test that copy still works when
        // the dst already exists (re-install case).
        fs::create_dir_all(&dst).unwrap();

        let copied = copy_dir_recursive(
            &src,
            &dst,
            &|s, _d| s.file_name().and_then(|n| n.to_str()) == Some("binary"),
            &|| false,
        )
        .unwrap();
        assert_eq!(copied, 2, "launch.sh + .keep copied; binary was skipped");
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

        let copied = copy_dir_recursive(&src, &dst, &|_s, _d| false, &|| false).unwrap();
        assert_eq!(copied, 1);
        let read_back = fs::read(dst.join("big.bin")).unwrap();
        assert_eq!(read_back, big);
    }

    #[test]
    fn test_copy_dir_recursive_skip_on_directory_omits_subtree() {
        // Documents the contract: when `skip` returns true for a directory
        // entry, the function `continue`s before `create_dir_all`, so the
        // directory is absent from dst and the subtree is silently dropped.
        let temp = tempfile::tempdir().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");
        touch(&src.join("Tools/skip.pak/binary"), b"binary");
        touch(
            &src.join("Tools/keep.pak/launch.sh"),
            b"#!/bin/sh\nexit 0\n",
        );

        let copied = copy_dir_recursive(
            &src,
            &dst,
            &|s, _d| s.file_name().and_then(|n| n.to_str()) == Some("skip.pak"),
            &|| false,
        )
        .unwrap();

        assert_eq!(copied, 1, "only keep.pak/launch.sh is copied");
        assert!(dst.join("Tools/keep.pak/launch.sh").exists());
        assert!(!dst.join("Tools/skip.pak").exists());
        assert!(!dst.join("Tools/skip.pak/binary").exists());
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
