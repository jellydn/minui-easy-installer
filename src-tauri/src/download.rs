use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::time::Duration;
use tempfile::TempDir;

#[derive(Debug, serde::Serialize)]
pub struct DownloadResult {
    pub success: bool,
    pub file_path: Option<String>,
    pub checksum_verified: Option<bool>,
    pub error: Option<String>,
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

pub async fn download_archive(
    url: &str,
    expected_checksum: Option<&str>,
) -> Result<(DownloadResult, TempDir), String> {
    let temp_dir =
        TempDir::new().map_err(|e| format!("Failed to create temporary directory: {}", e))?;

    let file_name = url.rsplit('/').next().unwrap_or("archive.zip");

    let file_path = temp_dir.path().join(file_name);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client.get(url).send().await
        .map_err(|e| format!("Failed to download archive: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response bytes: {}", e))?;

    fs::write(&file_path, &bytes).map_err(|e| format!("Failed to write archive to disk: {}", e))?;

    let file_path_str = file_path.to_str().ok_or("Non-UTF-8 path")?.to_string();

    let checksum_verified = if let Some(expected) = expected_checksum {
        let verified = verify_checksum(&file_path_str, expected)?;

        if !verified {
            return Ok((
                DownloadResult {
                    success: false,
                    file_path: None,
                    checksum_verified: Some(false),
                    error: Some("Checksum mismatch".to_string()),
                },
                temp_dir,
            ));
        }

        Some(true)
    } else {
        None
    };

    Ok((
        DownloadResult {
            success: true,
            file_path: Some(file_path_str),
            checksum_verified,
            error: None,
        },
        temp_dir,
    ))
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
        let expected = "6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72";

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

        let expected = "6AE8A75555209FD6C44157C0AED8016E763FF435A19CF186F76863140143FF72";

        let result = verify_checksum(temp_file.path().to_str().unwrap(), expected);
        assert!(result.unwrap());
    }
}
