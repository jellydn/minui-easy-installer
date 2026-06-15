use std::fs;
use std::path::Path;

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

/// Copies a directory tree from src to dst, optionally skipping entries via a predicate.
/// The skip function receives both the source and destination paths.
/// The cancel function is checked once per file; if it returns true, the
/// function returns Err("cancelled"). Pass `&|_| false` to disable.
/// Returns the number of files copied.
pub fn copy_dir_recursive<F, C>(
    src: &Path,
    dst: &Path,
    skip: &F,
    cancel: &C,
) -> Result<u32, String>
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
