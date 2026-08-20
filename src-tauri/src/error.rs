//! Unified error types for the application.
//! Replaces stringly-typed `Result<(), String>` throughout the codebase.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Clipboard operation failed: {0}")]
    Clipboard(#[from] arboard::Error),

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
    Network(#[from] reqwest::Error),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Download too large (max {max} bytes)")]
    DownloadTooLarge { max: u64 },

    #[error("Session error: {0}")]
    Session(String),

    #[error("X11 error: {0}")]
    X11(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Persistence(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::Persistence(msg.to_string())
    }
}