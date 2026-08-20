// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! # Windows 11 Clipboard History — Application Entry Point
//!
//! This file is intentionally thin. All domain logic lives in the library
//! crates under `src-tauri/src/`. The `main()` function:
//!
//! 1. Initialises tracing and the rendering environment.
//! 2. Builds the Tauri application with plugins and shared state.
//! 3. Registers all Tauri commands (from `commands.rs`).
//! 4. Starts the clipboard watcher and theme listener.

use parking_lot::Mutex;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

use win11_clipboard_history_lib::autostart_manager;
use win11_clipboard_history_lib::clipboard_manager::ClipboardManager;
use win11_clipboard_history_lib::commands;
use win11_clipboard_history_lib::config_manager::ConfigManager;
use win11_clipboard_history_lib::emoji_manager::EmojiManager;
use win11_clipboard_history_lib::permission_checker;
use win11_clipboard_history_lib::rendering_env;
use win11_clipboard_history_lib::session;
use win11_clipboard_history_lib::shortcut_setup;
use win11_clipboard_history_lib::tenor_api;
use win11_clipboard_history_lib::user_settings::UserSettingsManager;
use win11_clipboard_history_lib::window_controller::{
    SettingsController, WindowController, STARTED_IN_BACKGROUND,
};
use win11_clipboard_history_lib::AppState;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    win11_clipboard_history_lib::init_tracing();

    let args: Vec<String> = std::env::args().collect();

    // Handle --version / -v
    if args.iter().any(|arg| arg == "--version" || arg == "-v") {
        println!("win11-clipboard-history {VERSION}");
        return;
    }

    // Handle --help / -h
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return;
    }

    // MUST run before Tauri / WebKit init
    rendering_env::init();

    let start_in_background = args.iter().any(|arg| arg == "--background");
    if start_in_background {
        tracing::info!("[Startup] Starting in background mode (system tray only)");
        STARTED_IN_BACKGROUND.store(true, Ordering::SeqCst);
    }

    let open_settings_on_start = args.iter().any(|arg| arg == "--settings");
    let open_emoji_on_start = args.iter().any(|arg| arg == "--emoji");

    win11_clipboard_history_lib::session::init();

    let is_mouse_inside = Arc::new(AtomicBool::new(false));
    let base_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("win11-clipboard-history");

    if let Err(e) = std::fs::create_dir_all(&base_dir) {
        tracing::error!("Failed to create base directory: {e}");
    }

    let history_path = base_dir.join("history.json");
    let user_settings = UserSettingsManager::new().load();

    let clipboard_manager = Arc::new(Mutex::new(ClipboardManager::new(
        history_path,
        user_settings.max_history_size,
    )));
    {
        let mut manager = clipboard_manager.lock();
        manager.set_privacy_policy(user_settings.privacy_policy());
        manager.set_auto_delete_interval_minutes(user_settings.auto_delete_interval_in_minutes());
    }
    win11_clipboard_history_lib::linux_shortcut_manager::set_allow_wm_config_rewrite(
        user_settings.allow_wm_config_rewrite,
    );

    let emoji_manager = Arc::new(Mutex::new(EmojiManager::new(base_dir.clone())));
    let config_manager = Arc::new(Mutex::new(ConfigManager::new(base_dir)));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if argv.iter().any(|arg| arg == "--settings") {
                tracing::info!("[SingleInstance] Opening settings...");
                SettingsController::show(app);
            } else if argv.iter().any(|arg| arg == "--emoji") {
                tracing::info!("[SingleInstance] Opening emoji picker...");
                WindowController::toggle_with_tab(app, Some("emoji"));
            } else {
                tracing::info!("[SingleInstance] Toggling window...");
                WindowController::toggle(app);
            }
        }))
        .manage(AppState {
            clipboard_manager: clipboard_manager.clone(),
            emoji_manager: emoji_manager.clone(),
            config_manager: config_manager.clone(),
            is_mouse_inside: is_mouse_inside.clone(),
            paste_gate: tokio::sync::Mutex::new(()),
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                if window.label() == "setup" {
                    if win11_clipboard_history_lib::permission_checker::is_first_run() {
                        tracing::info!("[Setup] Setup window closed without completion. Exiting.");
                        window.app_handle().exit(0);
                    }
                }
            }
        })
        .setup(move |app| {
            let app_handle = app.handle().clone();

            win11_clipboard_history_lib::input_simulator::init();
            win11_clipboard_history_lib::paste_sync::init();

            // Background mode: immediately hide the main window
            if start_in_background {
                if let Some(main_window) = app.get_webview_window("main") {
                    let _ = main_window.hide();
                    tracing::debug!("[Setup] Immediately hiding main window for background mode");
                }
            }

            // Auto-migrate old autostart entries
            match autostart_manager::autostart_migrate() {
                Ok(true) => tracing::info!("[Setup] Migrated autostart entry to use wrapper script"),
                Ok(false) => {}
                Err(e) => tracing::warn!("[Setup] Failed to migrate autostart: {e}"),
            }

            // Build system tray
            build_tray(app, &app_handle)?;

            // Verify settings window
            if app.get_webview_window("settings").is_none() {
                tracing::error!("[Setup] FATAL: Settings window missing from config");
            } else {
                tracing::info!("[Setup] Settings window created successfully from config");
            }

            // Register window event handlers
            if let Some(main_window) = app.get_webview_window("main") {
                win11_clipboard_history_lib::window_controller::register_window_events(
                    &main_window,
                    &app_handle,
                );

                // Start clipboard watcher
                win11_clipboard_history_lib::clipboard_watcher::start(
                    app_handle.clone(),
                    clipboard_manager.clone(),
                );

                // Start theme change listener
                let app_handle_for_theme = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) =
                        win11_clipboard_history_lib::theme_manager::start_theme_listener(
                            app_handle_for_theme,
                        )
                        .await
                    {
                        tracing::error!("[ThemeManager] Failed to start theme listener: {e}");
                    }
                });

                // Handle --settings flag
                if open_settings_on_start {
                    SettingsController::show(&app_handle);
                }

                // Handle --emoji flag
                if open_emoji_on_start {
                    let app_handle_for_emoji = app_handle.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        let _ = app_handle_for_emoji.emit("switch-tab", "emoji");
                    });
                }

                // Background mode enforcer
                if start_in_background {
                    win11_clipboard_history_lib::window_controller::spawn_background_enforcer(
                        &main_window,
                    );
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // History
            commands::get_history,
            commands::clear_history,
            commands::delete_item,
            commands::toggle_pin,
            // Emoji
            commands::get_recent_emojis,
            // Mouse
            commands::set_mouse_state,
            // Settings
            commands::get_user_settings,
            commands::set_user_settings,
            commands::is_settings_window_visible,
            commands::get_default_settings,
            commands::set_app_language,
            // Theme
            commands::get_system_theme,
            commands::refresh_system_theme,
            commands::is_theme_listener_active,
            // Paste
            commands::paste_item,
            commands::paste_text,
            commands::paste_gif_from_url,
            commands::finish_paste,
            commands::copy_text_to_clipboard,
            // Setup
            commands::finish_setup,
            // Tenor
            tenor_api::search_tenor,
            // Permissions
            permission_checker::check_permissions,
            permission_checker::fix_permissions_now,
            permission_checker::is_first_run,
            permission_checker::mark_first_run_complete,
            permission_checker::reset_first_run,
            // Shortcuts
            shortcut_setup::get_desktop_environment,
            shortcut_setup::register_de_shortcut,
            shortcut_setup::check_shortcut_tools,
            shortcut_setup::detect_conflicts,
            shortcut_setup::resolve_conflicts,
            // Autostart
            autostart_manager::autostart_enable,
            autostart_manager::autostart_disable,
            autostart_manager::autostart_is_enabled,
            autostart_manager::autostart_migrate,
            // Rendering
            rendering_env::get_rendering_environment,
            session::get_session_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Build the system tray icon and menu
fn build_tray(app: &tauri::App, app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let settings_manager = UserSettingsManager::new();
    let settings = settings_manager.load();
    let (show_label, settings_label, quit_label) = if settings.language == "fa" {
        ("نمایش کلیپ‌بورد", "تنظیمات", "خروج")
    } else {
        ("Show Clipboard", "Settings", "Quit")
    };
    let show = MenuItem::with_id(app, "show", show_label, true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", settings_label, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &settings_item, &quit])?;

    let temp_dir = std::env::temp_dir().join("win11-clipboard-history");
    std::fs::create_dir_all(&temp_dir).ok();

    win11_clipboard_history_lib::theme_manager::update_dynamic_tray_flag(
        settings.enable_dynamic_tray_icon,
    );

    let (icon, use_template_icon) =
        win11_clipboard_history_lib::theme_manager::initial_tray_icon(&settings);

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .icon_as_template(use_template_icon)
        .tooltip("Clipboard History")
        .temp_dir_path(temp_dir)
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            "show" => WindowController::toggle(app),
            "settings" => SettingsController::show(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            } = event
            {
                WindowController::toggle(tray.app_handle());
            }
        })
        .build(app)?;

    // Update icon asynchronously if dynamic is enabled
    if settings.enable_dynamic_tray_icon {
        let app_handle_bg = app_handle.clone();
        let settings_bg = settings.clone();
        tauri::async_runtime::spawn(async move {
            win11_clipboard_history_lib::theme_manager::refresh_tray_icon(
                &app_handle_bg,
                &settings_bg,
            )
            .await;
        });
    }

    Ok(())
}

/// Print help message
fn print_help() {
    println!("win11-clipboard-history {VERSION}");
    println!();
    println!("USAGE:");
    println!("    win11-clipboard-history [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Show this help message");
    println!("    -v, --version    Show version information");
    println!("        --background Start minimized to system tray (for autostart)");
    println!("        --settings   Open settings window on startup");
    println!("        --emoji      Open with emoji picker tab selected");
    println!();
    println!("SHORTCUTS:");
    println!("    Super+V          Open clipboard history");
    println!("    Super+.          Open emoji picker");
    println!("    Ctrl+Alt+V       Alternative shortcut");
}
