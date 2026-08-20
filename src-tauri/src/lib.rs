//! Windows 11 Clipboard History For Linux Library
//! This module re-exports the core functionality for use as a library

use std::sync::OnceLock;
use tracing_appender::non_blocking::WorkerGuard;

static TRACING_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

/// Initialize tracing/logging. Called once at app startup.
/// The worker guard is kept alive for the process lifetime so logs flush.
pub fn init_tracing() {
    use tracing_subscriber::prelude::*;

    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("win11-clipboard-history/logs");
    let _ = std::fs::create_dir_all(&log_dir);
    crate::fs_atomic::restrict_permissions(&log_dir);

    let file_appender = tracing_appender::rolling::daily(&log_dir, "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let _ = TRACING_GUARD.set(guard);

    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::filter::EnvFilter::new("info,win11_clipboard_history=debug")
            }),
        )
        .with_writer(non_blocking)
        .with_ansi(false)
        .try_init();
}

pub mod autostart_manager;
pub mod clipboard_io;
pub mod clipboard_manager;
pub mod config_manager;
pub mod emoji_manager;
pub mod error;
pub mod focus_manager;
pub mod fs_atomic;
pub mod gif_manager;
pub mod image_store;
pub mod input_simulator;
pub mod paste_sync;
pub mod permission_checker;
pub mod privacy;
pub mod rendering_env;
pub mod session;
pub mod shortcut_conflict_detector;
pub mod shortcut_setup;
pub mod ssrf;
pub mod theme_manager;
pub mod user_settings;

pub mod linux_shortcut_manager;
pub mod tenor_api;
pub mod window_identity;
