use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use tempfile::TempDir;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DownloadResult {
    pub success: bool,
    pub file_path: Option<String>,
    pub checksum_verified: Option<bool>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
    pub percentage: Option<f64>,
}

pub fn verify_checksum(file_path: &str, expected_checksum: &str) -> Result<bool, String> {
    let mut file = fs::File::open(file_path)
        .map_err(|e| format!("Failed to open file for checksum verification: {}", e))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read file for checksum: {}", e))?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let computed = hex::encode(hasher.finalize());
    Ok(computed.eq_ignore_ascii_case(expected_checksum))
}

pub fn download_archive(
    url: &str,
    expected_checksum: Option<&str>,
) -> Result<DownloadResult, String> {
    let temp_dir =
        TempDir::new().map_err(|e| format!("Failed to create temporary directory: {}", e))?;

    let file_name = url.rsplit('/').next().unwrap_or("archive.zip");

    let file_path = temp_dir.path().join(file_name);

    let response =
        reqwest::blocking::get(url).map_err(|e| format!("Failed to download archive: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .map_err(|e| format!("Failed to read response bytes: {}", e))?;

    fs::write(&file_path, &bytes).map_err(|e| format!("Failed to write archive to disk: {}", e))?;

    let checksum_verified = if let Some(expected) = expected_checksum {
        let verified = verify_checksum(file_path.to_str().unwrap(), expected)
            .map_err(|e| format!("Checksum verification failed: {}", e))?;

        if !verified {
            return Ok(DownloadResult {
                success: false,
                file_path: None,
                checksum_verified: Some(false),
                error: Some("Checksum mismatch".to_string()),
            });
        }

        Some(true)
    } else {
        None
    };

    // Keep the temp directory alive by leaking it
    // In a real implementation, we'd manage this more carefully
    let path = file_path.to_str().unwrap().to_string();
    std::mem::forget(temp_dir);

    Ok(DownloadResult {
        success: true,
        file_path: Some(path),
        checksum_verified,
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_verify_checksum_success() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        write!(temp_file, "test content").unwrap();

        // SHA256 of "test content"
        let expected = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

        let result = verify_checksum(temp_file.path().to_str().unwrap(), expected);
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_checksum_failure() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        write!(temp_file, "test content").unwrap();

        let result = verify_checksum(temp_file.path().to_str().unwrap(), "wrong_checksum");
        assert!(!result.unwrap());
    }

    #[test]
    fn test_verify_checksum_case_insensitive() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        write!(temp_file, "test content").unwrap();

        let expected = "9F86D081884C7D659A2FEAA0C55AD015A3BF4F1B2B0B822CD15D6C15B0F00A08";

        let result = verify_checksum(temp_file.path().to_str().unwrap(), expected);
        assert!(result.unwrap());
    }
}
