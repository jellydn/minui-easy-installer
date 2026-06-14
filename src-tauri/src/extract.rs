use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExtractionResult {
    pub success: bool,
    pub output_path: Option<String>,
    pub files_extracted: Option<u32>,
    pub error: Option<String>,
}

/// Checks if a path is vulnerable to path traversal
fn is_path_traversal(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    normalized.contains("..") || normalized.starts_with('/') || normalized.starts_with('\\')
}

// Determines output directory and returns (ExtractionResult, TempDir if one was created)
fn determine_output(
    destination: Option<&str>,
) -> Result<(PathBuf, Option<tempfile::TempDir>), String> {
    if let Some(dest) = destination {
        Ok((PathBuf::from(dest), None))
    } else {
        let temp_dir =
            tempfile::TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;
        let path = temp_dir.path().to_path_buf();
        Ok((path, Some(temp_dir)))
    }
}

/// Extracts a ZIP archive to a destination directory
///
/// # Arguments
/// * `archive_path` - Path to the ZIP file
/// * `destination` - Directory to extract files into
///
/// # Returns
/// * `ExtractionResult` with success status and output path
/// * `Option<TempDir>` — the temp dir handle, if one was created. Keep it alive until extraction is consumed.
pub fn extract_archive(
    archive_path: &str,
    destination: Option<&str>,
) -> Result<(ExtractionResult, Option<tempfile::TempDir>), String> {
    let archive_file =
        fs::File::open(archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;

    let mut archive =
        ZipArchive::new(archive_file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    // Determine output directory
    let (output_path, _temp_dir) = determine_output(destination)?;

    // Create destination if it doesn't exist
    fs::create_dir_all(&output_path)
        .map_err(|e| format!("Failed to create destination directory: {}", e))?;

    let canonical_output = output_path
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize output path: {}", e))?;

    let mut files_extracted = 0u32;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read archive entry {}: {}", i, e))?;

        let entry_path = entry.name().to_string();

        if is_path_traversal(&entry_path) {
            return Err(format!(
                "Path traversal detected in archive entry: {}",
                entry_path
            ));
        }

        let file_path = output_path.join(&entry_path);

        // Ensure file path is within destination directory
        let canonical_file = file_path
            .parent()
            .unwrap_or(&file_path)
            .canonicalize()
            .or_else(|_| {
                // If parent doesn't exist yet, create it and try again
                fs::create_dir_all(file_path.parent().unwrap_or(&output_path))
                    .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                file_path
                    .parent()
                    .unwrap_or(&file_path)
                    .canonicalize()
                    .map_err(|e| format!("Failed to canonicalize file path: {}", e))
            })
            .map_err(|e| format!("Failed to canonicalize file path: {}", e))?;

        // Security check: ensure extracted path is within destination
        if !canonical_file.starts_with(&canonical_output) {
            return Err(format!(
                "Security violation: entry path escapes destination directory: {}",
                entry_path
            ));
        }

        if entry.is_dir() {
            fs::create_dir_all(&file_path)
                .map_err(|e| format!("Failed to create directory {}: {}", entry_path, e))?;
        } else {
            // Create parent directory if needed
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create parent directory: {}", e))?;
            }

            let mut outfile = fs::File::create(&file_path)
                .map_err(|e| format!("Failed to create file {}: {}", entry_path, e))?;

            let mut buffer = [0u8; 8192];
            loop {
                let bytes_read = entry
                    .read(&mut buffer)
                    .map_err(|e| format!("Failed to read entry {}: {}", entry_path, e))?;

                if bytes_read == 0 {
                    break;
                }

                outfile
                    .write_all(&buffer[..bytes_read])
                    .map_err(|e| format!("Failed to write file {}: {}", entry_path, e))?;
            }

            // Preserve file permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = entry.unix_mode() {
                    if let Err(e) = fs::set_permissions(&file_path, fs::Permissions::from_mode(mode)) {
                        eprintln!("Warning: failed to set permissions on {}: {}", entry_path, e);
                    }
                }
            }

            files_extracted += 1;
        }
    }

    Ok((
        ExtractionResult {
            success: true,
            output_path: Some(
                output_path
                    .to_str()
                    .ok_or("Non-UTF-8 output path")?
                    .to_string(),
            ),
            files_extracted: Some(files_extracted),
            error: None,
        },
        _temp_dir,
    ))
}

/// Extract an archive into a session-owned temp slot, returning just the path.
///
/// The owning InstallSession keeps the TempDir alive for the lifetime of
/// the install pipeline, preventing the extracted files from being deleted.
/// If a destination is provided, files are extracted there directly (no slot needed).
///
/// Returns the output directory path as a PathBuf.
pub fn extract_archive_into(
    slot: &mut Option<tempfile::TempDir>,
    archive_path: &Path,
    destination: Option<&Path>,
) -> Result<PathBuf, String> {
    if let Some(dest) = destination {
        // Extract directly to a caller-specified directory
        let (result, _) = extract_archive(
            archive_path
                .to_str()
                .ok_or("Non-UTF-8 archive path")?,
            Some(dest.to_str().ok_or("Non-UTF-8 dest path")?),
        )?;
        if !result.success {
            return Err(result.error.unwrap_or_else(|| "Extraction failed".to_string()));
        }
        return Ok(dest.to_path_buf());
    }

    // Create a TempDir and extract into it — transfer ownership to slot
    let temp_dir = tempfile::TempDir::new()
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let output_path = temp_dir.path().to_path_buf();

    let (result, _) = extract_archive(
        archive_path.to_str().ok_or("Non-UTF-8 archive path")?,
        Some(output_path.to_str().ok_or("Non-UTF-8 path")?),
    )?;
    if !result.success {
        return Err(result.error.unwrap_or_else(|| "Extraction failed".to_string()));
    }

    *slot = Some(temp_dir);
    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_is_path_traversal() {
        assert!(is_path_traversal("../etc/passwd"));
        assert!(is_path_traversal("..\\windows\\system32"));
        assert!(is_path_traversal("/etc/passwd"));
        assert!(is_path_traversal("\\windows\\system32"));
        assert!(!is_path_traversal("etc/passwd"));
        assert!(!is_path_traversal("file.txt"));
        assert!(!is_path_traversal("folder/subfolder/file.txt"));
    }

    #[test]
    fn test_extract_archive_success() {
        let temp_dir = tempfile::tempdir().unwrap();
        let archive_path = temp_dir.path().join("test.zip");
        let output_dir = temp_dir.path().join("output");

        // Create a test ZIP archive
        {
            let file = fs::File::create(&archive_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            zip.start_file("test.txt", zip::write::FileOptions::default())
                .unwrap();
            zip.write_all(b"Hello, World!").unwrap();
            zip.finish().unwrap();
        }

        let result = extract_archive(
            archive_path.to_str().unwrap(),
            Some(output_dir.to_str().unwrap()),
        );

        assert!(result.is_ok());
        let (result, _temp) = result.unwrap();
        assert!(result.success);
        assert_eq!(result.files_extracted, Some(1));
        assert!(output_dir.join("test.txt").exists());
    }

    #[test]
    fn test_extract_archive_path_traversal() {
        let temp_dir = tempfile::tempdir().unwrap();
        let archive_path = temp_dir.path().join("malicious.zip");

        // Create a ZIP with path traversal
        {
            let file = fs::File::create(&archive_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            zip.start_file("../etc/passwd", zip::write::FileOptions::default())
                .unwrap();
            zip.write_all(b"malicious content").unwrap();
            zip.finish().unwrap();
        }

        let result = extract_archive(archive_path.to_str().unwrap(), None);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Path traversal"));
    }

    #[test]
    fn test_extract_archive_nonexistent_file() {
        let result = extract_archive("/nonexistent/archive.zip", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to open archive"));
    }

    #[test]
    fn test_extract_archive_invalid_zip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let archive_path = temp_dir.path().join("invalid.zip");

        let mut file = fs::File::create(&archive_path).unwrap();
        file.write_all(b"This is not a zip file").unwrap();
        drop(file);

        let result = extract_archive(archive_path.to_str().unwrap(), None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read ZIP archive"));
    }
}
