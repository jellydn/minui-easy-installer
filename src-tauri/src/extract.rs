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

/// Validates a zip entry path for path traversal attacks
fn validate_entry_path(entry_path: &str) -> Result<(), String> {
    if is_path_traversal(entry_path) {
        return Err(format!(
            "Path traversal detected in archive entry: {}",
            entry_path
        ));
    }

    // Additional check: ensure no absolute paths
    if Path::new(entry_path).is_absolute() {
        return Err(format!(
            "Absolute path detected in archive entry: {}",
            entry_path
        ));
    }

    Ok(())
}

/// Extracts a ZIP archive to a destination directory
///
/// # Arguments
/// * `archive_path` - Path to the ZIP file
/// * `destination` - Directory to extract files into
///
/// # Returns
/// * `ExtractionResult` with success status and output path
pub fn extract_archive(
    archive_path: &str,
    destination: Option<&str>,
) -> Result<ExtractionResult, String> {
    let archive_file =
        fs::File::open(archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;

    let mut archive =
        ZipArchive::new(archive_file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    // Determine output directory
    let output_path = if let Some(dest) = destination {
        PathBuf::from(dest)
    } else {
        let temp_dir =
            tempfile::TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;
        let path = temp_dir.path().to_path_buf();
        // Leak the temp dir to keep it alive
        std::mem::forget(temp_dir);
        path
    };

    // Create destination if it doesn't exist
    fs::create_dir_all(&output_path)
        .map_err(|e| format!("Failed to create destination directory: {}", e))?;

    let mut files_extracted = 0u32;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read archive entry {}: {}", i, e))?;

        let entry_path = entry.name().to_string();

        // Validate for path traversal
        validate_entry_path(&entry_path)?;

        let file_path = output_path.join(&entry_path);

        // Ensure file path is within destination directory
        let canonical_output = output_path
            .canonicalize()
            .map_err(|e| format!("Failed to canonicalize output path: {}", e))?;

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
                    let _ = fs::set_permissions(&file_path, fs::Permissions::from_mode(mode));
                }
            }

            files_extracted += 1;
        }
    }

    Ok(ExtractionResult {
        success: true,
        output_path: Some(output_path.to_str().unwrap_or("").to_string()),
        files_extracted: Some(files_extracted),
        error: None,
    })
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
    fn test_validate_entry_path() {
        assert!(validate_entry_path("etc/passwd").is_ok());
        assert!(validate_entry_path("file.txt").is_ok());
        assert!(validate_entry_path("folder/subfolder/file.txt").is_ok());
        assert!(validate_entry_path("../etc/passwd").is_err());
        assert!(validate_entry_path("/etc/passwd").is_err());
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
        let result = result.unwrap();
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
