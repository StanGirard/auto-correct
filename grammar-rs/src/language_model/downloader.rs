//! Auto-download N-gram data from R2 storage
//!
//! This module provides functionality to automatically download N-gram data
//! from Cloudflare R2 when it's not present locally.
//!
//! Enable with feature flag `ngram-download` and environment variable:
//! ```bash
//! GRAMMAR_RS_AUTO_DOWNLOAD=1 cargo run --features ngram-download
//! ```

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

/// Default R2 bucket URL for N-gram data
/// Override with GRAMMAR_RS_R2_URL environment variable
const DEFAULT_R2_URL: &str = "https://pub-8068a615549c43e1893eb3f9a35a0e17.r2.dev/ngrams";

/// Get the R2 base URL from environment or default
fn get_r2_url() -> String {
    std::env::var("GRAMMAR_RS_R2_URL").unwrap_or_else(|_| DEFAULT_R2_URL.to_string())
}

/// Check if auto-download is enabled via environment variable
pub fn is_auto_download_enabled() -> bool {
    std::env::var("GRAMMAR_RS_AUTO_DOWNLOAD")
        .map(|v| v == "1" || v.to_lowercase() == "true")
        .unwrap_or(false)
}

/// Ensure N-gram data is available, downloading if necessary
///
/// Returns:
/// - `Ok(true)` if data exists or was successfully downloaded
/// - `Ok(false)` if data is missing and auto-download is disabled
/// - `Err(...)` if download failed
///
/// # Arguments
/// * `lang` - Language code ("en" or "fr")
/// * `data_dir` - Directory where N-gram data should be stored
#[cfg(feature = "ngram-download")]
pub fn ensure_ngram_data(lang: &str, data_dir: &Path) -> io::Result<bool> {
    let filename = format!("{}_ngrams.bin", lang);
    let target = data_dir.join(&filename);

    // Already exists - nothing to do
    if target.exists() {
        return Ok(true);
    }

    // Check if auto-download is enabled
    if !is_auto_download_enabled() {
        tracing::debug!(
            "N-gram data not found at {:?}, auto-download disabled (set GRAMMAR_RS_AUTO_DOWNLOAD=1 to enable)",
            target
        );
        return Ok(false);
    }

    let base_url = get_r2_url();
    tracing::info!(
        "Downloading {} N-gram data from {}...",
        lang.to_uppercase(),
        base_url
    );

    // Create directory if needed
    fs::create_dir_all(data_dir)?;

    // Download the file
    let url = format!("{}/{}", base_url, filename);
    let temp_path = data_dir.join(format!("{}.download", filename));

    match download_with_progress(&url, &temp_path) {
        Ok(()) => {}
        Err(e) => {
            // Clean up partial download
            let _ = fs::remove_file(&temp_path);
            return Err(e);
        }
    }

    // Verify checksum
    let checksum_url = format!("{}/{}.sha256", base_url, filename);
    match verify_checksum(&temp_path, &checksum_url) {
        Ok(()) => {}
        Err(e) => {
            let _ = fs::remove_file(&temp_path);
            return Err(e);
        }
    }

    // Move temp file to final location
    fs::rename(&temp_path, &target)?;

    tracing::info!(
        "Successfully downloaded {} N-gram data to {:?}",
        lang.to_uppercase(),
        target
    );

    Ok(true)
}

/// Fallback when ngram-download feature is not enabled
#[cfg(not(feature = "ngram-download"))]
pub fn ensure_ngram_data(lang: &str, data_dir: &Path) -> io::Result<bool> {
    let filename = format!("{}_ngrams.bin", lang);
    let target = data_dir.join(&filename);

    if target.exists() {
        Ok(true)
    } else {
        tracing::debug!(
            "N-gram data not found at {:?}, ngram-download feature not enabled",
            target
        );
        Ok(false)
    }
}

/// Download a file from URL with progress logging
#[cfg(feature = "ngram-download")]
fn download_with_progress(url: &str, target: &Path) -> io::Result<()> {
    use sha2::{Sha256, Digest};

    let response = ureq::get(url)
        .call()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("HTTP request failed: {}", e)))?;

    let status = response.status();
    if status != 200 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("HTTP {} from {}", status, url),
        ));
    }

    let total_size = response
        .header("content-length")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let size_mb = total_size / (1024 * 1024);
    tracing::info!("  File size: {} MB", size_mb);

    let mut file = File::create(target)?;
    let mut downloaded = 0u64;
    let mut buffer = [0u8; 65536]; // 64KB buffer
    let mut reader = response.into_reader();
    let mut last_progress = 0u64;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;

        // Log progress every ~5%
        let progress = if total_size > 0 {
            downloaded * 100 / total_size
        } else {
            0
        };

        if progress >= last_progress + 5 {
            last_progress = progress;
            tracing::info!(
                "  Progress: {}% ({} / {} MB)",
                progress,
                downloaded / (1024 * 1024),
                size_mb
            );
        }
    }

    file.flush()?;
    tracing::info!("  Download complete: {} MB", downloaded / (1024 * 1024));

    Ok(())
}

/// Verify file checksum against remote .sha256 file
#[cfg(feature = "ngram-download")]
fn verify_checksum(file: &Path, checksum_url: &str) -> io::Result<()> {
    use sha2::{Sha256, Digest};

    tracing::info!("  Verifying checksum...");

    // Fetch expected checksum (optional - don't fail if not found)
    let response = match ureq::get(checksum_url).call() {
        Ok(r) if r.status() == 200 => r,
        Ok(r) => {
            tracing::warn!(
                "  Checksum file not found (HTTP {}), skipping verification",
                r.status()
            );
            return Ok(());
        }
        Err(e) => {
            tracing::warn!("  Could not fetch checksum: {}, skipping verification", e);
            return Ok(());
        }
    };

    let checksum_text = response
        .into_string()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let expected = checksum_text
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_lowercase();

    if expected.is_empty() {
        tracing::warn!("  Empty checksum file, skipping verification");
        return Ok(());
    }

    // Calculate actual checksum
    let mut hasher = Sha256::new();
    let mut f = File::open(file)?;
    io::copy(&mut f, &mut hasher)?;
    let actual = format!("{:x}", hasher.finalize());

    if expected != actual {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Checksum mismatch!\n  Expected: {}\n  Got:      {}",
                expected, actual
            ),
        ));
    }

    tracing::info!("  Checksum verified OK");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_auto_download_enabled() {
        // Default: disabled
        std::env::remove_var("GRAMMAR_RS_AUTO_DOWNLOAD");
        assert!(!is_auto_download_enabled());

        // Enabled with "1"
        std::env::set_var("GRAMMAR_RS_AUTO_DOWNLOAD", "1");
        assert!(is_auto_download_enabled());

        // Enabled with "true"
        std::env::set_var("GRAMMAR_RS_AUTO_DOWNLOAD", "true");
        assert!(is_auto_download_enabled());

        // Disabled with other values
        std::env::set_var("GRAMMAR_RS_AUTO_DOWNLOAD", "0");
        assert!(!is_auto_download_enabled());

        std::env::set_var("GRAMMAR_RS_AUTO_DOWNLOAD", "false");
        assert!(!is_auto_download_enabled());

        // Clean up
        std::env::remove_var("GRAMMAR_RS_AUTO_DOWNLOAD");
    }

    #[test]
    fn test_get_r2_url() {
        // Default URL
        std::env::remove_var("GRAMMAR_RS_R2_URL");
        assert!(get_r2_url().contains("r2.dev"));

        // Custom URL
        std::env::set_var("GRAMMAR_RS_R2_URL", "https://custom.example.com/ngrams");
        assert_eq!(get_r2_url(), "https://custom.example.com/ngrams");

        // Clean up
        std::env::remove_var("GRAMMAR_RS_R2_URL");
    }
}
