use std::fs;
use std::path::Path;

/// Copies a directory tree from src to dst, optionally skipping entries via a predicate.
/// Returns the number of files copied.
pub fn copy_dir_recursive<F>(src: &Path, dst: &Path, skip: &F) -> Result<u32, String>
where
    F: Fn(&Path) -> bool,
{
    let mut files_copied = 0u32;

    let entries = fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory {}: {}", src.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if skip(&src_path) {
            continue;
        }

        if src_path.is_dir() {
            fs::create_dir_all(&dst_path)
                .map_err(|e| format!("Failed to create directory {}: {}", dst_path.display(), e))?;
            files_copied += copy_dir_recursive(&src_path, &dst_path, skip)?;
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

/// Copies contents of src directory to dst directory.
pub fn copy_dir_contents(src: &Path, dst: &Path) -> Result<u32, String> {
    let mut files_copied = 0u32;

    let entries = fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory {}: {}", src.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            fs::create_dir_all(&dst_path)
                .map_err(|e| format!("Failed to create directory {}: {}", dst_path.display(), e))?;
            files_copied += copy_dir_recursive(&src_path, &dst_path, &|_| false)?;
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
