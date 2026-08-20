//! Linux Desktop Environment Shortcut Manager.
//!
//! Detects the active desktop environment and registers/unregisters the
//! application shortcuts through the appropriate backend:
//!
//! - `shortcut_config`    — the shared shortcut table and bindings
//! - `shortcut_error`     — the shared error type
//! - `shortcut_utils`     — process/file helpers (no shell, atomic writes)
//! - `shortcut_gsettings` — GNOME / Cinnamon / MATE backends
//! - `shortcut_tiling`    — i3 / Sway / Hyprland backends
//!
//! KDE Plasma, XFCE, COSMIC, LXQt and LXDE are implemented in this module
//! because they are single-handler backends.

mod shortcut_config;
mod shortcut_error;
mod shortcut_gsettings;
mod shortcut_tiling;
mod shortcut_utils;

use self::shortcut_config::{escape_xml, get_command_path, INI_SECTION_ENCODE, ShortcutConfig, SHORTCUTS};
use self::shortcut_error::{Result, ShortcutError};
use self::shortcut_gsettings::{CinnamonHandler, GnomeHandler, MateHandler};
use self::shortcut_tiling::{HyprlandHandler, I3Handler, SwayHandler};
use self::shortcut_utils::Utils;
use percent_encoding::utf8_percent_encode;
use std::env;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;

/// When false (default), tiling WM handlers never comment out the user's existing bindings.
static ALLOW_WM_CONFIG_REWRITE: AtomicBool = AtomicBool::new(false);

/// Opt-in: allow commenting out existing Super+V bindings in i3/Sway/Hyprland configs.
pub fn set_allow_wm_config_rewrite(allow: bool) {
    ALLOW_WM_CONFIG_REWRITE.store(allow, Ordering::Relaxed);
}

fn allow_wm_config_rewrite() -> bool {
    ALLOW_WM_CONFIG_REWRITE.load(Ordering::Relaxed)
}

// =============================================================================
// Error Handling (re-exported from `shortcut_error`)
// =============================================================================

pub use self::shortcut_error::{Result as ShortcutResult, ShortcutError};

// =============================================================================
// Public API
// =============================================================================

pub fn register_global_shortcut() {
    let handler = detect_handler();
    tracing::info!("[ShortcutManager] Detected Environment: {}", handler.name());

    let command_path = get_command_path();
    tracing::info!("[ShortcutManager] Using command path: {command_path}");

    for shortcut in SHORTCUTS {
        // Create a new config with the correct command path
        let mut config = shortcut.clone();
        config.command = command_path;

        match handler.register(&config) {
            Ok(_) => tracing::info!("[ShortcutManager] ✓ Registered '{}'", config.name),
            Err(e) => tracing::warn!("[ShortcutManager] ✗ Failed '{}': {e}", config.name),
        }
    }
}

pub fn unregister_global_shortcut() {
    let handler = detect_handler();
    tracing::info!("[ShortcutManager] Environment: {}", handler.name());

    let command_path = get_command_path();

    for shortcut in SHORTCUTS {
        // Create a new config with the correct command path
        let mut config = shortcut.clone();
        config.command = command_path;

        match handler.unregister(&config) {
            Ok(_) => tracing::info!("[ShortcutManager] ✓ Unregistered '{}'", config.name),
            Err(e) => tracing::warn!("[ShortcutManager] ✗ Failed '{}': {e}", config.name),
        }
    }
}

// =============================================================================
// Traits & Abstractions
// =============================================================================

trait ShortcutHandler {
    fn name(&self) -> &str;
    fn register(&self, shortcut: &ShortcutConfig) -> Result<()>;
    fn unregister(&self, shortcut: &ShortcutConfig) -> Result<()>;
}

fn detect_handler() -> Box<dyn ShortcutHandler> {
    let xdg_current = env_var("XDG_CURRENT_DESKTOP").to_lowercase();
    let xdg_session = env_var("XDG_SESSION_DESKTOP").to_lowercase();
    let combined = format!("{xdg_current} {xdg_session}");

    if combined.contains("gnome") || combined.contains("unity") || combined.contains("pantheon") {
        return Box::new(GnomeHandler);
    }
    if combined.contains("cinnamon") {
        return Box::new(CinnamonHandler);
    }
    // KDE Plasma 5 or 6
    if combined.contains("kde") || combined.contains("plasma") {
        return Box::new(KdeHandler);
    }
    if combined.contains("xfce") {
        return Box::new(XfceHandler);
    }
    if combined.contains("mate") {
        return Box::new(MateHandler);
    }
    if combined.contains("cosmic") {
        return Box::new(CosmicHandler);
    }
    if combined.contains("lxqt") {
        return Box::new(LxqtHandler);
    }
    if combined.contains("lxde") {
        return Box::new(LxdeHandler);
    }
    if combined.contains("budgie") {
        return Box::new(GnomeHandler); // Budgie uses gsettings like GNOME
    }
    if combined.contains("deepin") {
        return Box::new(GnomeHandler); // Deepin uses gsettings like GNOME
    }
    // Tiling Window Managers
    if combined.contains("i3") {
        return Box::new(I3Handler);
    }
    if combined.contains("sway") {
        return Box::new(SwayHandler);
    }
    if combined.contains("hyprland") {
        return Box::new(HyprlandHandler);
    }

    // Heuristic Fallback - check running processes for tiling WMs
    if is_process_running("i3") {
        return Box::new(I3Handler);
    }
    if is_process_running("sway") {
        return Box::new(SwayHandler);
    }
    if is_process_running("hyprland") || is_process_running("Hyprland") {
        return Box::new(HyprlandHandler);
    }

    // Heuristic Fallback for traditional DEs
    if Utils::command_exists("kwriteconfig5") || Utils::command_exists("kwriteconfig6") {
        return Box::new(KdeHandler);
    }
    if Utils::command_exists("xfconf-query") {
        return Box::new(XfceHandler);
    }

    // Default fallback
    Box::new(GnomeHandler)
}

fn is_process_running(name: &str) -> bool {
    Command::new("pgrep")
        .arg("-x")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn env_var(key: &str) -> String {
    env::var(key).unwrap_or_default()
}

// =============================================================================
// Implementations
// =============================================================================

// --- KDE Plasma Logic ---

struct KdeHandler;
impl KdeHandler {
    fn get_config_path() -> Result<PathBuf> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;
        Ok(PathBuf::from(home).join(".config/khotkeysrc"))
    }

    fn reload_kde() {
        // Try both Plasma 5 and modern methods
        let _ = Utils::run(
            "qdbus",
            &[
                "org.kde.kglobalaccel",
                "/kglobalaccel",
                "org.kde.KGlobalAccel.reloadConfig",
            ],
        );
    }
}

impl ShortcutHandler for KdeHandler {
    fn name(&self) -> &str {
        "KDE Plasma"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        let path = Self::get_config_path()?;
        let section_name = format!("Data_{}", s.id.replace('-', "_"));

        Utils::modify_file_atomic(&path, |content| {
            if content.contains(&format!("[{section_name}]")) {
                return Ok(None); // Already exists
            }

            let mut lines: Vec<String> = content.lines().map(String::from).collect();
            let mut data_count_idx = None;
            let mut data_count = 0;

            let mut in_data_group = false;

            for (i, line) in lines.iter().enumerate() {
                if line.trim() == "[Data]" {
                    in_data_group = true;
                } else if line.starts_with('[') && in_data_group {
                    in_data_group = false;
                }

                if in_data_group && line.starts_with("DataCount=") {
                    data_count_idx = Some(i);
                    if let Ok(c) = line.split('=').nth(1).unwrap_or("0").trim().parse::<u32>() {
                        data_count = c;
                    }
                    break;
                }
            }

            // Update Count
            if let Some(idx) = data_count_idx {
                lines[idx] = format!("DataCount={}", data_count + 1);
            } else {
                lines.push("[Data]".to_string());
                lines.push("DataCount=1".to_string());
            }

            // Append New Entry
            // Generate deterministic UUID v5 based on shortcut ID to ensure uniqueness per
            // shortcut but consistency across runs (idempotency)
            let namespace = Uuid::NAMESPACE_DNS;
            let uuid = Uuid::new_v5(&namespace, s.id.as_bytes()).to_string();
            let full_cmd = s.full_command();

            let entry = format!(
                "\n[{0}]\nComment={1}\nEnabled=true\nName={1}\nType=SIMPLE_ACTION_DATA\n\n[{0}/Actions]\nActionsCount=1\n\n[{0}/Actions/Action0]\nCommandURL={2}\nType=COMMAND_URL\n\n[{0}/Conditions]\nComment=\nConditionsCount=0\n\n[{0}/Triggers]\nTriggersCount=1\n\n[{0}/Triggers/Trigger0]\nKey={3}\nType=SHORTCUT\nUuid={{{4}}}\n",
                section_name, s.name, full_cmd, s.kde_binding, uuid
            );

            lines.push(entry);
            Ok(Some(lines.join("\n")))
        })?;

        Self::reload_kde();
        Ok(())
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        let path = Self::get_config_path()?;
        let section_name = format!("Data_{}", s.id.replace('-', "_"));

        Utils::modify_file_atomic(&path, |content| {
            if !content.contains(&section_name) {
                return Ok(None);
            }

            let lines: Vec<&str> = content.lines().collect();
            let mut new_lines = Vec::new();
            let mut skip_block = false;

            for line in lines {
                if line.starts_with(&format!("[{section_name}]")) {
                    skip_block = true;
                } else if line.starts_with('[') && skip_block {
                    // Check if it's a child subsection (start with same prefix) or new section
                    if !line.starts_with(&format!("[{section_name}/")) {
                        skip_block = false;
                    }
                }

                if !skip_block {
                    new_lines.push(line.to_string());
                }
            }
            Ok(Some(new_lines.join("\n")))
        })?;

        Self::reload_kde();
        Ok(())
    }
}

// --- XFCE ---

struct XfceHandler;
impl ShortcutHandler for XfceHandler {
    fn name(&self) -> &str {
        "XFCE"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        if !Utils::command_exists("xfconf-query") {
            return Err(ShortcutError::DependencyMissing("xfconf-query".into()));
        }
        let property = format!("/commands/custom/{}", s.xfce_binding);

        // Check if exists to avoid error spam
        let exists = Command::new("xfconf-query")
            .args(["-c", "xfce4-keyboard-shortcuts", "-p", &property])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !exists {
            Utils::run(
                "xfconf-query",
                &[
                    "-c",
                    "xfce4-keyboard-shortcuts",
                    "-p",
                    &property,
                    "-n",
                    "-t",
                    "string",
                    "-s",
                    &s.full_command(),
                ],
            )?;
        }
        Ok(())
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        if !Utils::command_exists("xfconf-query") {
            return Ok(());
        }
        let property = format!("/commands/custom/{}", s.xfce_binding);
        // Ignore error on unregister if it doesn't exist
        let _ = Utils::run(
            "xfconf-query",
            &["-c", "xfce4-keyboard-shortcuts", "-p", &property, "-r"],
        );
        Ok(())
    }
}

// --- COSMIC (Epoch 1.0+) ---

// Indentation constants for COSMIC RON format
const COSMIC_ENTRY_INDENT: &str = "    ";
const COSMIC_FIELD_INDENT: &str = "        ";
const COSMIC_MODIFIER_INDENT: &str = "            ";

struct CosmicHandler;
impl CosmicHandler {
    /// Escape special characters for RON string format
    fn escape_ron_string(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    /// Format modifiers for COSMIC RON format - each on its own line
    /// Input: "Super" or "Ctrl, Alt" -> properly formatted RON array entries
    fn format_modifiers(mods: &str) -> String {
        let formatted: Vec<String> = mods
            .split(',')
            .map(|m| m.trim())
            .filter(|m| !m.is_empty())
            .map(|m| {
                // Normalize modifier names to COSMIC's expected format
                let normalized: String = match m.to_lowercase().as_str() {
                    "ctrl" | "control" => "Ctrl".to_string(),
                    "alt" => "Alt".to_string(),
                    "super" | "meta" => "Super".to_string(),
                    "shift" => "Shift".to_string(),
                    _ => {
                        // Fallback: normalize capitalization (First letter uppercase + rest lowercase)
                        let mut chars = m.chars();
                        match chars.next() {
                            Some(first) => {
                                let mut result = first.to_uppercase().to_string();
                                result.push_str(&chars.as_str().to_lowercase());
                                result
                            }
                            None => String::new(),
                        }
                    }
                };
                format!("{COSMIC_MODIFIER_INDENT}{normalized},")
            })
            .collect();
        formatted.join("\n")
    }

    /// Build a COSMIC shortcut entry in proper RON format
    fn build_entry(s: &ShortcutConfig) -> String {
        let mods_formatted = Self::format_modifiers(s.cosmic_mods);
        let full_cmd = Self::escape_ron_string(&s.full_command());
        let name = Self::escape_ron_string(s.name);
        let key = Self::escape_ron_string(s.cosmic_key);

        format!(
            r#"{}(\n{}modifiers: [\n{}\n{}],\n{}key: "{}",\n{}description: Some("{}"),\n{}): Spawn("{}"),"#,
            COSMIC_ENTRY_INDENT,
            COSMIC_FIELD_INDENT,
            mods_formatted,
            COSMIC_FIELD_INDENT,
            COSMIC_FIELD_INDENT,
            key,
            COSMIC_FIELD_INDENT,
            name,
            COSMIC_ENTRY_INDENT,
            full_cmd
        )
    }
}

impl ShortcutHandler for CosmicHandler {
    fn name(&self) -> &str {
        "COSMIC (Epoch)"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;
        let path = PathBuf::from(home)
            .join(".config/cosmic/com.system76.CosmicSettings.Shortcuts/v1/custom");

        let full_cmd = s.full_command();
        let entry = Self::build_entry(s);

        Utils::modify_file_atomic(&path, |content| {
            // Check if this command is already registered to avoid duplicates
            if content.contains(&format!("Spawn(\"{full_cmd}\")")) {
                return Ok(None);
            }

            let trimmed = content.trim();

            // If file is empty or doesn't start with '{', create new structure
            if trimmed.is_empty() {
                return Ok(Some(format!("{{\n{entry}\n}}")));
            }

            // File should be a RON map: { ... }
            if !trimmed.starts_with('{') {
                // Reject unexpected formats instead of trying to wrap potentially malformed content
                return Err(ShortcutError::ParseError(
                    "Invalid COSMIC config format - expected RON map starting with '{'".into(),
                ));
            }

            // Find the last '}' and insert before it
            if let Some(pos) = content.rfind('}') {
                let mut new_content = content.to_string();
                new_content.insert_str(pos, &format!("{entry}\n"));
                return Ok(Some(new_content));
            }

            Err(ShortcutError::ParseError(
                "Invalid COSMIC config format - missing closing brace".into(),
            ))
        })?;
        Ok(())
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        let home = env::var("HOME").unwrap_or_default();
        let path = PathBuf::from(home)
            .join(".config/cosmic/com.system76.CosmicSettings.Shortcuts/v1/custom");

        if !path.exists() {
            return Ok(());
        }

        let full_cmd = s.full_command();
        let spawn_pattern = format!("Spawn(\"{full_cmd}\")");

        Utils::modify_file_atomic(&path, |content| {
            if !content.contains(&spawn_pattern) {
                return Ok(None);
            }

            // Parse and remove the entry block containing our command
            // RON format: (key_tuple): Value, - we track depth to find entry boundaries
            // depth starts at 0 before the opening '{'; depth 1 = inside outer map {}, depth 2+ = inside an entry
            let mut result = String::new();
            let mut depth = 0;
            let mut in_entry = false;
            let mut entry_start = 0;
            let mut prev_depth: i32;

            for c in content.chars() {
                prev_depth = depth;

                // Update depth first
                if c == '{' || c == '(' {
                    depth += 1;
                } else if c == '}' || c == ')' {
                    depth -= 1;
                }

                // Detect entry start: '(' that takes us from depth 1 to depth 2
                if c == '(' && prev_depth == 1 && depth == 2 {
                    entry_start = result.len();
                    in_entry = true;
                }

                result.push(c);

                // Detect entry end: ',' when we're at depth 1 (after the Spawn(...) closed)
                if in_entry && depth == 1 && c == ',' {
                    // Check if this entry contains our command
                    let entry_content = &result[entry_start..];
                    if entry_content.contains(&spawn_pattern) {
                        // Remove this entry (including leading whitespace)
                        let trim_start = result[..entry_start].trim_end().len();
                        result.truncate(trim_start);
                        result.push('\n');
                    }
                    in_entry = false;
                }
            }

            // Clean up sequences of more than two consecutive newlines in a single pass
            let mut cleaned = String::with_capacity(result.len());
            let mut newline_count = 0;
            for ch in result.chars() {
                if ch == '\n' {
                    if newline_count < 2 {
                        cleaned.push('\n');
                    }
                    newline_count += 1;
                } else {
                    newline_count = 0;
                    cleaned.push(ch);
                }
            }

            Ok(Some(cleaned))
        })?;
        Ok(())
    }
}

// --- LXQt ---

struct LxqtHandler;
impl ShortcutHandler for LxqtHandler {
    fn name(&self) -> &str {
        "LXQt"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;
        let path = PathBuf::from(home).join(".config/lxqt/globalkeyshortcuts.conf");

        let full_cmd = s.full_command();
        // LXQt uses INI format for shortcuts
        // Section name is URL-encoded keybinding followed by shortcut ID
        // Only encode characters problematic for INI format: / \ [ ] = ; # and spaces
        let encoded_binding = utf8_percent_encode(s.kde_binding, INI_SECTION_ENCODE).to_string();
        let section = format!("{encoded_binding}/{}", s.id);
        let entry = format!("\n[{section}]\nComment={}\nEnabled=true\nExec={full_cmd}", s.name);

        Utils::modify_file_atomic(&path, |content| {
            if content.contains(&format!("[{section}]")) {
                return Ok(None); // Already exists
            }

            let mut new_content = content.clone();
            new_content.push_str(&entry);
            Ok(Some(new_content))
        })?;
        Ok(())
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;
        let path = PathBuf::from(home).join(".config/lxqt/globalkeyshortcuts.conf");

        if !path.exists() {
            return Ok(());
        }

        // Use same encoding as register for consistency
        let encoded_binding = utf8_percent_encode(s.kde_binding, INI_SECTION_ENCODE).to_string();
        let section = format!("{encoded_binding}/{}", s.id);

        Utils::modify_file_atomic(&path, |content| {
            if !content.contains(&format!("[{section}]")) {
                return Ok(None);
            }

            let lines: Vec<&str> = content.lines().collect();
            let mut new_lines = Vec::new();
            let mut skip_block = false;

            for line in lines {
                if line.trim() == format!("[{section}]") {
                    skip_block = true;
                    continue;
                }
                if line.starts_with('[') && skip_block {
                    skip_block = false;
                }
                if !skip_block {
                    new_lines.push(line.to_string());
                }
            }
            Ok(Some(new_lines.join("\n")))
        })?;
        Ok(())
    }
}

// --- LXDE (Openbox) ---

struct LxdeHandler;
impl ShortcutHandler for LxdeHandler {
    fn name(&self) -> &str {
        "LXDE/Openbox"
    }

    fn register(&self, s: &ShortcutConfig) -> Result<()> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;

        // LXDE uses Openbox for window management
        let path = PathBuf::from(&home).join(".config/openbox/lxde-rc.xml");

        // Fallback to default openbox config if LXDE-specific doesn't exist
        let path = if path.exists() {
            path
        } else {
            PathBuf::from(&home).join(".config/openbox/rc.xml")
        };

        if !path.exists() {
            return Err(ShortcutError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "Openbox config not found",
            )));
        }

        let full_cmd = s.full_command();
        // The keybind XML to add - use the LXDE/Openbox-specific binding
        // Escape XML special characters to prevent XML injection
        let escaped_binding = escape_xml(s.lxde_binding);
        let escaped_cmd = escape_xml(&full_cmd);
        let keybind = format!(
            r#"    <keybind key="{escaped_binding}">
      <action name="Execute">
        <command>{escaped_cmd}</command>
      </action>
    </keybind>"#
        );

        Utils::modify_file_atomic(&path, |content| {
            if content.contains(&format!("<command>{escaped_cmd}</command>")) {
                return Ok(None); // Already exists
            }

            // Find the </keyboard> closing tag and insert before it
            if let Some(pos) = content.find("</keyboard>") {
                let mut new_content = content.clone();
                new_content.insert_str(pos, &format!("{keybind}\n  "));

                // Trigger openbox reconfigure
                let _ = Utils::run("openbox", &["--reconfigure"]);

                return Ok(Some(new_content));
            }

            Err(ShortcutError::ParseError(
                "Could not find </keyboard> in Openbox config".into(),
            ))
        })?;
        Ok(())
    }

    fn unregister(&self, s: &ShortcutConfig) -> Result<()> {
        let home = env::var("HOME")
            .map_err(|_| ShortcutError::UnsupportedEnvironment("HOME not set".into()))?;

        let path = PathBuf::from(&home).join(".config/openbox/lxde-rc.xml");
        let path = if path.exists() {
            path
        } else {
            PathBuf::from(&home).join(".config/openbox/rc.xml")
        };

        if !path.exists() {
            return Ok(());
        }

        let full_cmd = s.full_command();
        let escaped_binding = escape_xml(s.lxde_binding);
        let escaped_cmd = escape_xml(&full_cmd);

        Utils::modify_file_atomic(&path, |content| {
            if !content.contains(&format!("<command>{escaped_cmd}</command>")) {
                return Ok(None);
            }

            // Remove the keybind block - this is a simplified approach
            // A proper XML parser would be better but adds dependency
            let pattern = format!(
                r#"    <keybind key="{escaped_binding}">
      <action name="Execute">
        <command>{escaped_cmd}</command>
      </action>
    </keybind>"#
            );

            let new_content = content.replace(&pattern, "");

            // Trigger openbox reconfigure
            let _ = Utils::run("openbox", &["--reconfigure"]);

            Ok(Some(new_content))
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wm_rewrite_defaults_off() {
        assert!(!allow_wm_config_rewrite());
        set_allow_wm_config_rewrite(true);
        assert!(allow_wm_config_rewrite());
        set_allow_wm_config_rewrite(false);
        assert!(!allow_wm_config_rewrite());
    }
}
