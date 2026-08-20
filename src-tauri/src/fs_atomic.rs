//! Atomic file I/O utilities.
//! Provides crash-safe writes that never corrupt the target file.

use std::fs;
use std::path::Path;
use tracing::error;

/// Writes `contents` to `path` atomically:
/// 1. Write to a temporary sibling file (`.tmp`)
/// 2. Rename (atomic on POSIX) to replace the original
pub fn write_atomic(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, contents)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Writes pretty-printed JSON atomically.
pub fn write_json_atomic<T: serde::Serialize>(path: &Path, value: &T) -> std::io::Result<()> {
    let content = serde_json::to_string_pretty(value)?;
    write_atomic(path, content.as_bytes())
}

/// Reads and deserializes JSON from path. Returns Ok(None) if file does not exist.
pub fn read_json_optional<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Option<T>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    let value = serde_json::from_str(&content).map_err(|e| format!("Failed to parse {}: {e}", path.display()))?;
    Ok(Some(value))
}

/// Ensures the parent directory of `path` exists.
pub fn ensure_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
    } else {
        Ok(())
    }
}

/// Migration helper: copies a legacy JSON file to a new path before overwriting.
/// Returns the old content if the legacy file existed.
pub fn migrate_legacy_file(old_path: &Path, new_path: &Path) -> Result<Option<String>, String> {
    if !old_path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(old_path)
        .map_err(|e| format!("Failed to read legacy file {}: {e}", old_path.display()))?;
    ensure_parent(new_path).map_err(|e| e.to_string())?;
    fs::copy(old_path, new_path)
        .map_err(|e| format!("Failed to copy {} to {}: {e}", old_path.display(), new_path.display()))?;
    Ok(Some(content))
}