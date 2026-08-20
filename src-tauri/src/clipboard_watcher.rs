//! Clipboard Watcher Module
//!
//! Runs a background thread that polls the system clipboard for changes
//! and emits Tauri events when new content is detected. The watcher
//! reuses a single `arboard::Clipboard` connection to avoid X11/Wayland
//! connection churn.

use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::clipboard_manager::{self, ClipboardManager};

/// Start the clipboard polling watcher in a background thread.
///
/// The watcher:
/// - Reuses one `arboard::Clipboard` instance across all reads.
/// - Reads clipboard *outside* the history mutex (shorter lock window).
/// - Uses adaptive polling: 200ms when active, 800ms when idle.
/// - Emits `clipboard-changed` events for incremental frontend updates.
pub fn start(app: AppHandle, clipboard_manager: Arc<Mutex<ClipboardManager>>) {
    std::thread::Builder::new()
        .name("clipboard-watcher".to_string())
        .spawn(move || {
            let mut clipboard = match arboard::Clipboard::new() {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("[Watcher] Failed to init clipboard: {e}");
                    return;
                }
            };
            let mut last_text_hash: Option<u64> = None;
            let mut last_image_hash: Option<u64> = None;
            let mut cleanup_counter = 0u32;
            let mut idle_ticks = 0u32;

            loop {
                // Adaptive polling: faster when active, slower when idle
                let delay = if idle_ticks == 0 {
                    Duration::from_millis(200)
                } else {
                    Duration::from_millis(800)
                };
                std::thread::sleep(delay);
                cleanup_counter += 1;

                let policy = clipboard_manager.lock().privacy_policy();

                // Privacy: skip sensitive apps (password managers, incognito).
                // On Wayland this is a no-op because compositors do not expose focus identity.
                if policy.exclude_sensitive_apps {
                    if let Some(src) =
                        crate::window_identity::focused_source()
                    {
                        if crate::privacy::is_sensitive_source(
                            &src.class,
                            &src.title,
                            &policy.extra_excluded_apps,
                        ) {
                            idle_ticks = idle_ticks.saturating_add(1);
                            continue;
                        }
                    }
                }

                // Read clipboard outside the mutex
                let (text, html, image) = read_system_clipboard(&mut clipboard);

                let mut manager = clipboard_manager.lock();

                // Periodic cleanup of old items
                if cleanup_counter >= 40 {
                    cleanup_counter = 0;
                    let interval_in_minutes = manager.auto_delete_interval_minutes();
                    if interval_in_minutes > 0
                        && manager.cleanup_old_items(interval_in_minutes)
                    {
                        let _ = app.emit("history-cleared", ());
                    }
                }

                let mut changed = false;

                if let Ok(ref captured) = text {
                    if !captured.is_empty() {
                        let text_hash = clipboard_manager::calculate_hash(captured);
                        if Some(text_hash) != last_text_hash {
                            last_text_hash = Some(text_hash);
                            last_image_hash = None;
                            if let Some(item) = manager.add_text(captured.clone(), html.clone())
                            {
                                let _ = app.emit("clipboard-changed", &item);
                                changed = true;
                            }
                        }
                    }
                }

                if let Ok(Some((image_data, hash))) = image {
                    if Some(hash) != last_image_hash {
                        last_image_hash = Some(hash);
                        last_text_hash = None;
                        if let Some(item) = manager.add_image(image_data, hash) {
                            let _ = app.emit("clipboard-changed", &item.for_ipc());
                            changed = true;
                        }
                    }
                }

                idle_ticks = if changed {
                    0
                } else {
                    idle_ticks.saturating_add(1)
                };
            }
        })
        .expect("Failed to spawn clipboard watcher thread");
}

/// Reads clipboard content without holding the history mutex.
/// All three reads share one `Clipboard` instance.
fn read_system_clipboard(
    clip: &mut arboard::Clipboard,
) -> (
    Result<String, arboard::Error>,
    Option<String>,
    Result<Option<(arboard::ImageData<'static>, u64)>, arboard::Error>,
) {
    let text = clip.get_text();
    let html = clip.get().html().ok();
    let image = match clip.get_image() {
        Ok(img) => {
            let hash = clipboard_manager::calculate_hash(&img.bytes);
            Ok(Some((
                arboard::ImageData {
                    width: img.width,
                    height: img.height,
                    bytes: img.bytes.into_owned().into(),
                },
                hash,
            )))
        }
        Err(arboard::Error::ContentNotAvailable) => Ok(None),
        Err(e) => Err(e),
    };

    (text, html, image)
}
