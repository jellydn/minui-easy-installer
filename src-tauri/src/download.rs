use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tokio_util::sync::CancellationToken;

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

/// Streaming download primitive used by the install pipeline.
///
/// Streams the HTTP response body directly to disk in 8 KB chunks,
/// invoking `progress(bytes_so_far, total)` after each chunk. This
/// bounds peak memory to a few KB regardless of archive size (was:
/// the full archive held in RAM via `response.bytes().await`).
///
/// `progress` receives the running byte count and the total from
/// Content-Length (or None if the server didn't send one).
///
/// On checksum failure, the partial file is deleted and the temp dir
/// is NOT transferred into the slot — the caller's `slot` stays `None`
/// and the TempDir drops at end of scope.
pub async fn download_archive_streaming(
    slot: &mut Option<TempDir>,
    url: &str,
    expected_checksum: Option<&str>,
    progress: impl Fn(u64, Option<u64>) + Send + 'static,
    cancel: &CancellationToken,
) -> Result<PathBuf, String> {
    let temp_dir = TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let file_name = url.rsplit('/').next().unwrap_or("archive.zip");
    let file_path = temp_dir.path().join(file_name);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download archive: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let total: Option<u64> = response.content_length();
    let mut stream = response.bytes_stream();
    let mut file = tokio::fs::File::create(&file_path)
        .await
        .map_err(|e| format!("Failed to create archive file: {}", e))?;

    let mut bytes_so_far: u64 = 0;
    let mut chunks_since_cancel_check: u32 = 0;
    while let Some(chunk_result) = stream.next().await {
        // Check cancel every 64 chunks (~512 KB at 8 KB each) — cheap
        // enough to be per-chunk, but amortizes the cost.
        chunks_since_cancel_check += 1;
        if chunks_since_cancel_check % 64 == 0 && cancel.is_cancelled() {
            let _ = fs::remove_file(&file_path);
            return Err("cancelled".to_string());
        }
        let chunk = chunk_result.map_err(|e| format!("Failed to read response chunk: {}", e))?;
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
            .await
            .map_err(|e| format!("Failed to write archive chunk: {}", e))?;
        bytes_so_far += chunk.len() as u64;
        progress(bytes_so_far, total);
    }
    tokio::io::AsyncWriteExt::shutdown(&mut file)
        .await
        .map_err(|e| format!("Failed to flush archive file: {}", e))?;

    let file_path_str = file_path.to_str().ok_or("Non-UTF-8 path")?.to_string();

    if let Some(expected) = expected_checksum {
        let verified = verify_checksum(&file_path_str, expected)?;
        if !verified {
            // Best-effort cleanup of the partial file. The TempDir drops
            // at end of scope since we don't transfer it into the slot.
            let _ = fs::remove_file(&file_path);
            return Err("Checksum mismatch".to_string());
        }
    }

    *slot = Some(temp_dir);
    Ok(file_path)
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
