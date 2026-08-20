//! GIF Manager
//! Handles downloading GIFs and preparing them for clipboard paste.
//!
//! IMPORTANT: This module handles specific OS-level clipboard commands (wl-copy/xclip)
//! to ensure GIFs are pasted as files (text/uri-list) rather than raw bytes or text.
//! This is required for rich media pasting in apps like Discord/Chrome on Linux.

use crate::clipboard_io::{self, ClipError, Payload};
use crate::session;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use tracing::{debug, error, warn};

// --- Constants ---

const APP_CACHE_DIR: &str = "win11-clipboard-history/gifs";
const MIME_URI_LIST: &str = "text/uri-list";
const DOWNLOAD_TIMEOUT: u64 = 10;
const WL_COPY_SETTLE_TIME: u64 = 150;

// --- Cache Management ---

struct GifCache;

impl GifCache {
    /// Get (and create if missing) the cache directory.
    fn get_dir() -> Result<PathBuf, String> {
        let cache_dir = dirs::cache_dir()
            .ok_or("Failed to resolve system cache directory")?
            .join(APP_CACHE_DIR);

        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)
                .map_err(|e| format!("Failed to create cache dir: {}", e))?;
        }

        Ok(cache_dir)
    }

    /// Generate a file path based on the URL hash (stable FNV for cache persistence).
    fn get_path_for_url(url: &str) -> Result<PathBuf, String> {
        // Use the stable FNV-1a hash (same as clipboard_manager) for cross-restart cache
        let hash = crate::clipboard_manager::calculate_hash(&url);
        Ok(Self::get_dir()?.join(format!("{:016x}.gif", hash)))
    }
}

// --- Downloader ---

struct Downloader;

impl Downloader {
    const MAX_GIF_BYTES: u64 = 10 * 1024 * 1024; // 10 MB

    /// Downloads a URL to a local file with SSRF protection and size limit.
    pub fn download(url: &str, destination: &Path) -> Result<(), String> {
        // SSRF Protection: validate URL
        let parsed = url::Url::parse(url).map_err(|e| format!("Invalid URL: {e}"))?;
        if parsed.scheme() != "https" {
            return Err("Only HTTPS URLs are allowed for GIF downloads".into());
        }

        // Block private IPs
        if let Some(host) = parsed.host_str() {
            if let Ok(ip) = host.parse::<std::net::IpAddr>() {
                if ip.is_loopback() || ip.is_private() || ip.is_unspecified() {
                    return Err(format!("Blocked download from private IP: {ip}"));
                }
            }
        }

        // Check cache TTL (re-download if older than 24h)
        const CACHE_TTL: Duration = Duration::from_secs(24 * 3600);
        if let Ok(meta) = fs::metadata(destination) {
            if meta.len() > 0 {
                if let Ok(modified) = meta.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed < CACHE_TTL {
                            debug!("[GifManager] Cache hit for {url}");
                            return Ok(());
                        }
                    }
                }
            }
        }

        debug!("[GifManager] Downloading: {url}");
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(DOWNLOAD_TIMEOUT))
            .build()
            .map_err(|e| format!("Client build error: {e}"))?;

        let response = client
            .get(url)
            .send()
            .map_err(|e| format!("Network request failed: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("HTTP Error: {}", response.status()));
        }

        // Stream with size limit
        let mut file = fs::File::create(destination)
            .map_err(|e| format!("File creation failed: {e}"))?;

        let mut downloaded: u64 = 0;
        let mut reader = response.take(Self::MAX_GIF_BYTES + 1); // +1 to detect truncation
        let mut buffer = [0u8; 4096];

        loop {
            let n = reader.read(&mut buffer)
                .map_err(|e| format!("Read error: {e}"))?;
            if n == 0 {
                break;
            }
            downloaded += n as u64;
            if downloaded > Self::MAX_GIF_BYTES {
                let _ = fs::remove_file(destination);
                return Err(format!(
                    "GIF download exceeds {} MB limit",
                    Self::MAX_GIF_BYTES / (1024 * 1024)
                ));
            }
            file.write_all(&buffer[..n])
                .map_err(|e| format!("File write failed: {e}"))?;
        }

        info!("[GifManager] Saved {downloaded} bytes to {:?}", destination);
        Ok(())
    }
}

// --- Clipboard Logic (The Critical Part) ---

struct ClipboardHandler;

impl ClipboardHandler {
    fn make_file_uri(path: &Path) -> String {
        format!("file://{}\n", path.to_string_lossy())
    }

    fn copy_uri(path: &Path) -> Result<(), String> {
        let uri = Self::make_file_uri(path);
        clipboard_io::write(&Payload::FileUri(&uri))
            .map_err(|e| format!("GIF clipboard copy failed: {e}"))
    }

    fn copy_url_fallback(url: &str) -> Result<(), String> {
        clipboard_io::write(&Payload::Text(url))
            .map_err(|e| e.to_string())
    }
}

// --- Public API ---

/// Downloads a GIF from the URL and returns the local file path.
/// Uses the stable FNV hash as filename (cross-restart cache persistence)
/// and enforces SSRF protection + size limits.
pub fn download_gif_to_file(url: &str) -> Result<PathBuf, String> {
    let target_path = GifCache::get_path_for_url(url)?;
    Downloader::download(url, &target_path)?;
    Ok(target_path)
}

/// Downloads GIF and sets clipboard.
/// Returns Ok(Some(uri)) if successful (for history marking),
/// Ok(Some(url)) if fallback used,
/// Err if everything failed.
pub fn paste_gif_to_clipboard_with_uri(url: &str) -> Result<Option<String>, String> {
    debug!("[GifManager] paste_gif_to_clipboard_with_uri: {url}");

    // 1. Attempt Download with SSRF protection and size limit
    let gif_path = match download_gif_to_file(url) {
        Ok(path) => path,
        Err(e) => {
            warn!("[GifManager] Download failed ({e}), using URL fallback.");
            ClipboardHandler::copy_url_fallback(url)?;
            return Ok(Some(url.to_string()));
        }
    };

    // 2. Copy file URI to clipboard using unified module
    match ClipboardHandler::copy_uri(&gif_path) {
        Ok(_) => {
            let uri = format!("file://{}", gif_path.to_string_lossy());
            info!("[GifManager] GIF ready: {uri}");
            Ok(Some(uri))
        }
        Err(e) => {
            warn!("[GifManager] File copy failed ({e}), using URL fallback.");
            ClipboardHandler::copy_url_fallback(url)?;
            Ok(Some(url.to_string()))
        }
    }
}

/// Convenience wrapper for cases where the URI return isn't needed.
pub fn paste_gif_to_clipboard(url: &str) -> Result<(), String> {
    paste_gif_to_clipboard_with_uri(url).map(|_| ())
}

/// Helper for external use if needed (legacy support)
pub fn copy_url_to_clipboard(url: &str) -> Result<(), String> {
    ClipboardHandler::copy_url_fallback(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_resolution() {
        let dir = GifCache::get_dir();
        assert!(dir.is_ok());
        assert!(dir.unwrap().ends_with("win11-clipboard-history/gifs"));
    }

    #[test]
    fn test_path_generation() {
        let path = GifCache::get_path_for_url("http://example.com/cat.gif");
        assert!(path.is_ok());
        assert!(path.unwrap().extension().unwrap() == "gif");
    }
}
