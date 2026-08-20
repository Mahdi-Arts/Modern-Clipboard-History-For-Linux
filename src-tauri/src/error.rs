//! Unified error types for the application.
//!
//! All Tauri commands return `Result<T, AppError>` for structured,
//! serialisable error reporting to the frontend.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Clipboard operation failed: {0}")]
    Clipboard(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Persistence error: {0}")]
    Persistence(String),

    #[error("Input simulation failed: {0}")]
    InputSimulation(String),

    #[error("Item '{id}' not found in history")]
    NotFound { id: String },

    #[error("Network error: {0}")]
    Network(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Download too large (max {max} bytes)")]
    DownloadTooLarge { max: u64 },

    #[error("Session error: {0}")]
    Session(String),

    #[error("X11 error: {0}")]
    X11(String),

    #[error("Privacy policy blocked this clipboard item")]
    PrivacyBlocked,

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("{0}")]
    Other(String),
}

pub type AppResult<T> = Result<T, AppError>;

// ---------------------------------------------------------------------------
// From implementations for seamless ? operator usage
// ---------------------------------------------------------------------------

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Other(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::Other(msg.to_string())
    }
}

impl From<arboard::Error> for AppError {
    fn from(err: arboard::Error) -> Self {
        AppError::Clipboard(err.to_string())
    }
}

impl From<crate::clipboard_io::ClipError> for AppError {
    fn from(err: crate::clipboard_io::ClipError) -> Self {
        match err {
            crate::clipboard_io::ClipError::Arboard(e) => AppError::Clipboard(e.to_string()),
            crate::clipboard_io::ClipError::External(msg) => AppError::Clipboard(msg),
            crate::clipboard_io::ClipError::VerificationFailed(msg) => {
                AppError::Clipboard(format!("Verification failed: {msg}"))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tauri serialization (commands return AppError as a string to the frontend)
// ---------------------------------------------------------------------------

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
