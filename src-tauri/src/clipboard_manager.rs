//! Clipboard Manager Module
//! Handles clipboard monitoring, history storage, and paste injection.

use arboard::{Clipboard, ImageData};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use crate::history_crypto::HistoryCrypto;
use crate::history_store::{self, PersistRow};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::privacy::{self, PrivacyPolicy};

pub const DEFAULT_MAX_HISTORY_SIZE: usize = 50;
pub const MAX_HISTORY_HARD_CAP: usize = 2_000;
const PREVIEW_TEXT_MAX_LEN: usize = 100;
const GIF_CACHE_MARKER: &str = "win11-clipboard-history/gifs/";
const FILE_URI_PREFIX: &str = "file://";
const CLIPBOARD_HELPER_READY_TIMEOUT: Duration = Duration::from_secs(2);
const CLIPBOARD_HELPER_POLL_INTERVAL: Duration = Duration::from_millis(2);

pub use crate::content_hash::calculate_hash;

fn truncate_chars(s: &str, max: usize) -> String {
    let mut chars = s.chars();
    let head: String = chars.by_ref().take(max).collect();
    if chars.next().is_none() {
        s.to_string()
    } else {
        head
    }
}

fn get_system_clipboard() -> Result<Clipboard, String> {
    Clipboard::new().map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum ClipboardContent {
    Text(String),
    RichText { plain: String, html: String },
    Image {
        base64: String,
        width: u32,
        height: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: String,
    pub content: ClipboardContent,
    pub timestamp: DateTime<Utc>,
    pub pinned: bool,
    pub preview: String,
}

impl ClipboardItem {
    pub fn new_text(text: String) -> Self {
        let preview = if text.chars().count() > PREVIEW_TEXT_MAX_LEN {
            format!("{}...", text.chars().take(PREVIEW_TEXT_MAX_LEN).collect::<String>())
        } else {
            text.clone()
        };
        Self::create(ClipboardContent::Text(text), preview)
    }

    pub fn new_rich_text(plain: String, html: String) -> Self {
        let preview = if plain.chars().count() > PREVIEW_TEXT_MAX_LEN {
            format!("{}...", plain.chars().take(PREVIEW_TEXT_MAX_LEN).collect::<String>())
        } else {
            plain.clone()
        };
        Self::create(ClipboardContent::RichText { plain, html }, preview)
    }

    pub fn new_image(base64: String, width: u32, height: u32, hash: u64) -> Self {
        let preview = format!("Image ({}x{}) #{}", width, height, hash);
        Self::create(ClipboardContent::Image { base64, width, height }, preview)
    }

    fn create(content: ClipboardContent, preview: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content,
            timestamp: Utc::now(),
            pinned: false,
            preview,
        }
    }

    /// Strip HTML and cap text so IPC never carries the full clipboard payload.
    pub fn for_ipc(&self) -> Self {
        const UI_TEXT_MAX: usize = 2048;
        let content = match &self.content {
            ClipboardContent::Text(text) => ClipboardContent::Text(truncate_chars(text, UI_TEXT_MAX)),
            ClipboardContent::RichText { plain, .. } => ClipboardContent::RichText {
                plain: truncate_chars(plain, UI_TEXT_MAX),
                html: String::new(),
            },
            other => other.clone(),
        };
        Self {
            id: self.id.clone(),
            content,
            timestamp: self.timestamp,
            pinned: self.pinned,
            preview: self.preview.clone(),
        }
    }

    pub fn extract_image_hash(&self) -> Option<u64> {
        if !matches!(self.content, ClipboardContent::Image { .. }) {
            return None;
        }
        self.preview
            .split('#')
            .nth(1)
            .and_then(|h| h.parse::<u64>().ok())
    }
}

pub struct ClipboardManager {
    history: Vec<ClipboardItem>,
    text_hashes: HashSet<u64>,
    last_pasted_text: Option<String>,
    last_pasted_image_hash: Option<u64>,
    last_added_text_hash: Option<u64>,
    db_path: PathBuf,
    json_legacy_path: PathBuf,
    images_dir: PathBuf,
    conn: Connection,
    crypto: HistoryCrypto,
    image_paths: HashMap<String, PathBuf>,
    max_history_size: usize,
    dirty: bool,
    privacy: PrivacyPolicy,
    auto_delete_interval_minutes: u64,
}

impl ClipboardManager {
    fn clamp_max_history_size(size: usize) -> usize {
        match size {
            0 => DEFAULT_MAX_HISTORY_SIZE,
            1..=MAX_HISTORY_HARD_CAP => size,
            _ => MAX_HISTORY_HARD_CAP,
        }
    }

    pub fn new(persistence_path: PathBuf, max_history_size: usize) -> Self {
        let max_size = Self::clamp_max_history_size(max_history_size);
        let base_dir = persistence_path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let _ = fs::create_dir_all(&base_dir);
        crate::fs_atomic::restrict_permissions(&base_dir);

        let db_path = base_dir.join("history.db");
        let images_dir = base_dir.join("images");
        let _ = fs::create_dir_all(&images_dir);
        crate::fs_atomic::restrict_permissions(&images_dir);

        let conn = history_store::open_database(&db_path).unwrap_or_else(|e| {
            error!("[ClipboardManager] Failed to open SQLite ({e}); using in-memory fallback");
            Connection::open_in_memory().expect("in-memory sqlite")
        });
        let crypto = HistoryCrypto::load_or_create(&base_dir).unwrap_or_else(|e| {
            error!("[ClipboardManager] history.key unavailable ({e}); generating ephemeral key");
            HistoryCrypto::load_or_create(&std::env::temp_dir().join("win11-clipboard-ephemeral"))
                .expect("ephemeral history crypto")
        });

        let mut manager = Self {
            history: Vec::with_capacity(max_size),
            text_hashes: HashSet::new(),
            last_pasted_text: None,
            last_pasted_image_hash: None,
            last_added_text_hash: None,
            db_path,
            json_legacy_path: persistence_path,
            images_dir,
            conn,
            crypto,
            image_paths: HashMap::new(),
            max_history_size: max_size,
            dirty: false,
            privacy: PrivacyPolicy::default(),
            auto_delete_interval_minutes: 0,
        };
        manager.migrate_legacy_json();
        manager.load_from_db();
        manager.rebuild_hash_index();
        manager
    }

    pub fn set_privacy_policy(&mut self, policy: PrivacyPolicy) {
        self.privacy = policy;
    }

    pub fn privacy_policy(&self) -> PrivacyPolicy {
        self.privacy.clone()
    }

    pub fn set_auto_delete_interval_minutes(&mut self, minutes: u64) {
        self.auto_delete_interval_minutes = minutes;
    }

    pub fn auto_delete_interval_minutes(&self) -> u64 {
        self.auto_delete_interval_minutes
    }

    pub fn set_max_history_size(&mut self, new_size: usize) {
        let mut clamped = Self::clamp_max_history_size(new_size);
        let pinned_count = self.history.iter().filter(|i| i.pinned).count();
        if clamped < pinned_count {
            clamped = pinned_count;
        }
        self.max_history_size = clamped;
        let trimmed = self.enforce_history_limit();
        if trimmed {
            self.save_history();
        }
    }

    pub fn get_max_history_size(&self) -> usize {
        self.max_history_size
    }

    fn migrate_legacy_json(&mut self) {
        if !self.json_legacy_path.exists() {
            return;
        }
        // Only migrate when the database is empty.
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM items", [], |r| r.get(0))
            .unwrap_or(0);
        if count > 0 {
            return;
        }

        let Ok(content) = fs::read_to_string(&self.json_legacy_path) else {
            return;
        };
        let Ok(items) = serde_json::from_str::<Vec<ClipboardItem>>(&content) else {
            warn!("[ClipboardManager] Legacy history.json is unreadable; leaving in place");
            return;
        };

        info!(
            "[ClipboardManager] Migrating {} items from history.json → SQLite",
            items.len()
        );
        for mut item in items {
            if let ClipboardContent::Image { base64, width, height } = &item.content {
                if let Ok(png) = BASE64.decode(base64) {
                    if let Ok(stored) =
                        crate::image_store::store_png_bytes(&self.images_dir, &item.id, &png)
                    {
                        self.image_paths
                            .insert(item.id.clone(), stored.full_path.clone());
                        item.content = ClipboardContent::Image {
                            base64: stored.thumb_base64,
                            width: *width,
                            height: *height,
                        };
                    }
                }
            }
            self.history.push(item);
        }
        let _ = self.enforce_history_limit();
        self.save_history();

        let bak = self.json_legacy_path.with_extension("json.bak");
        if fs::rename(&self.json_legacy_path, &bak).is_err() {
            let _ = fs::remove_file(&self.json_legacy_path);
        }
        info!("[ClipboardManager] Legacy JSON migrated");
    }

    fn load_from_db(&mut self) {
        if !self.history.is_empty() {
            return;
        }
        let rows = match history_store::load_rows(&self.conn) {
            Ok(r) => r,
            Err(e) => {
                warn!("[ClipboardManager] Failed to load: {e}");
                return;
            }
        };

        for row in rows {
            let text = self.crypto.decrypt_optional(row.text);
            let html = self.crypto.decrypt_optional(row.html);
            let mut preview = self.crypto.decrypt_str(&row.preview);
            let thumb_base64 = self.crypto.decrypt_optional(row.thumb_base64);
            let content = match row.kind.as_str() {
                "richtext" => ClipboardContent::RichText {
                    plain: text.unwrap_or_default(),
                    html: html.unwrap_or_default(),
                },
                "image" => {
                    if let Some(path) = row.image_path.clone() {
                        self.image_paths
                            .insert(row.id.clone(), PathBuf::from(&path));
                    }
                    ClipboardContent::Image {
                        base64: thumb_base64.unwrap_or_default(),
                        width: row.width.unwrap_or(0) as u32,
                        height: row.height.unwrap_or(0) as u32,
                    }
                }
                _ => ClipboardContent::Text(text.unwrap_or_default()),
            };

            let timestamp = DateTime::<Utc>::from_timestamp_millis(row.created_at)
                .unwrap_or_else(Utc::now);

            if matches!(content, ClipboardContent::Image { .. }) {
                if let Some(hash) = row.image_hash {
                    if !preview.contains('#') {
                        preview = format!("{preview} #{hash}");
                    }
                }
            }

            self.history.push(ClipboardItem {
                id: row.id,
                content,
                timestamp,
                pinned: row.pinned,
                preview,
            });
        }

        let _ = self.enforce_history_limit();
        if let Some(first) = self.history.first() {
            match &first.content {
                ClipboardContent::Text(text) => {
                    self.last_added_text_hash = Some(calculate_hash(text));
                }
                ClipboardContent::RichText { plain, .. } => {
                    self.last_added_text_hash = Some(calculate_hash(plain));
                }
                ClipboardContent::Image { .. } => {
                    self.last_added_text_hash = None;
                }
            }
        }
        debug!(
            "[ClipboardManager] Loaded {} items from SQLite",
            self.history.len()
        );
    }

    pub fn save_history(&mut self) {
        if let Err(e) = self.persist_sqlite() {
            error!("[ClipboardManager] Failed to save history: {e}");
        } else {
            self.dirty = false;
        }
    }

    fn persist_sqlite(&mut self) -> Result<(), String> {
        let rows = self.collect_persist_rows()?;
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM items", []).map_err(|e| e.to_string())?;
        {
            let mut stmt = tx
                .prepare(history_store::INSERT_ITEM_SQL)
                .map_err(|e| e.to_string())?;
            for row in &rows {
                stmt.execute(params![
                    row.id,
                    row.kind,
                    row.text,
                    row.html,
                    row.image_path,
                    row.image_hash,
                    row.width,
                    row.height,
                    row.preview,
                    row.pinned,
                    row.created_at,
                    row.thumb,
                    row.sort_index,
                ])
                .map_err(|e| e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        crate::fs_atomic::restrict_sqlite_files(&self.db_path);
        Ok(())
    }

    fn persist_upsert_item(&mut self, item: &ClipboardItem, sort_index: i64) -> Result<(), String> {
        let row = persist_row_from_item(
            sort_index as usize,
            item,
            self.image_paths
                .get(&item.id)
                .map(|p| p.to_string_lossy().into_owned()),
            &self.crypto,
        )?;
        history_store::execute_insert(&self.conn, &row)?;
        crate::fs_atomic::restrict_sqlite_files(&self.db_path);
        Ok(())
    }

    fn persist_delete_ids(&mut self, ids: &[String]) -> Result<(), String> {
        if ids.is_empty() {
            return Ok(());
        }
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        for id in ids {
            tx.execute("DELETE FROM items WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn persist_meta(&mut self) -> Result<(), String> {
        let meta: Vec<(String, i64, i64)> = self
            .history
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                (
                    item.id.clone(),
                    idx as i64,
                    if item.pinned { 1 } else { 0 },
                )
            })
            .collect();
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        {
            let mut stmt = tx
                .prepare("UPDATE items SET sort_index = ?1, pinned = ?2 WHERE id = ?3")
                .map_err(|e| e.to_string())?;
            for (id, idx, pinned) in &meta {
                stmt.execute(params![idx, pinned, id])
                    .map_err(|e| e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn persist_mutation(&mut self) {
        if let Err(e) = self.persist_meta() {
            tracing::warn!("[ClipboardManager] Incremental persist failed ({e}); rewriting");
            if let Err(e) = self.persist_sqlite() {
                error!("[ClipboardManager] Failed to save history: {e}");
                return;
            }
        }
        self.dirty = false;
    }

    fn collect_persist_rows(&self) -> Result<Vec<PersistRow>, String> {
        self.history
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                persist_row_from_item(
                    idx,
                    item,
                    self.image_paths
                        .get(&item.id)
                        .map(|p| p.to_string_lossy().into_owned()),
                    &self.crypto,
                )
            })
            .collect()
    }

    pub fn add_text(&mut self, text: String, html: Option<String>) -> Option<ClipboardItem> {
        if self.privacy.filter_secrets && privacy::looks_like_secret(&text) {
            debug!("[ClipboardManager] Skipping secret-looking clipboard text");
            return None;
        }
        if self.should_skip_text(&text) {
            return None;
        }

        let text_hash = calculate_hash(&text);
        if Some(text_hash) == self.last_added_text_hash {
            return None;
        }
        if self.is_duplicate_text(&text) {
            self.last_added_text_hash = Some(text_hash);
            return None;
        }
        self.remove_duplicate_text_from_history(&text);

        let item = match html {
            Some(html_content) if !html_content.trim().is_empty() => {
                ClipboardItem::new_rich_text(text, html_content)
            }
            _ => ClipboardItem::new_text(text),
        };
        self.insert_item(item.clone());
        self.last_added_text_hash = Some(text_hash);
        Some(item)
    }

    pub fn add_image(&mut self, image_data: ImageData<'_>, hash: u64) -> Option<ClipboardItem> {
        if !self.privacy.save_images {
            debug!("[ClipboardManager] Image capture disabled by privacy settings");
            return None;
        }
        if self.should_skip_image(hash) {
            return None;
        }

        let id = Uuid::new_v4().to_string();
        let stored = match crate::image_store::store_rgba(&self.images_dir, &id, &image_data) {
            Ok(s) => s,
            Err(e) => {
                warn!("[ClipboardManager] Failed to store image: {e}");
                return None;
            }
        };

        let mut item = ClipboardItem::new_image(
            stored.thumb_base64,
            stored.width,
            stored.height,
            hash,
        );
        item.id = id.clone();
        self.image_paths.insert(id, stored.full_path);
        self.insert_item(item.clone());
        Some(item)
    }

    fn should_skip_text(&mut self, text: &str) -> bool {
        if text.trim().is_empty() {
            return true;
        }
        if text.contains(FILE_URI_PREFIX) && text.contains(GIF_CACHE_MARKER) {
            return true;
        }
        if let Some(ref pasted) = self.last_pasted_text {
            if pasted == text {
                self.last_pasted_text = None;
                return true;
            }
            self.last_pasted_text = None;
        }
        false
    }

    fn should_skip_image(&mut self, hash: u64) -> bool {
        if let Some(pasted_hash) = self.last_pasted_image_hash {
            if pasted_hash == hash {
                self.last_pasted_image_hash = None;
                return true;
            }
        }
        if let Some(item) = self.history.iter().find(|item| !item.pinned) {
            if let Some(item_hash) = item.extract_image_hash() {
                if item_hash == hash {
                    return true;
                }
            }
        }
        false
    }

    fn is_duplicate_text(&self, text: &str) -> bool {
        if let Some(item) = self.history.iter().find(|item| !item.pinned) {
            match &item.content {
                ClipboardContent::Text(t) if t == text => return true,
                ClipboardContent::RichText { plain, .. } if plain == text => return true,
                _ => {}
            }
        }
        false
    }

    fn remove_duplicate_text_from_history(&mut self, text: &str) {
        let hash = calculate_hash(text);
        if !self.text_hashes.contains(&hash) {
            return;
        }
        if let Some(pos) = self.history.iter().position(|item| {
            if item.pinned {
                return false;
            }
            match &item.content {
                ClipboardContent::Text(t) => t == text,
                ClipboardContent::RichText { plain, .. } => plain == text,
                _ => false,
            }
        }) {
            let removed = self.history.remove(pos);
            self.remove_image_file(&removed.id);
            self.rebuild_hash_index();
        }
    }

    fn rebuild_hash_index(&mut self) {
        self.text_hashes.clear();
        for item in &self.history {
            if let Some(text) = match &item.content {
                ClipboardContent::Text(t) => Some(t),
                ClipboardContent::RichText { plain, .. } => Some(plain),
                _ => None,
            } {
                self.text_hashes.insert(calculate_hash(text));
            }
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    fn insert_item(&mut self, item: ClipboardItem) {
        let insert_pos = self
            .history
            .iter()
            .position(|i| !i.pinned)
            .unwrap_or(self.history.len());
        let inserted_id = item.id.clone();
        self.history.insert(insert_pos, item);
        self.dirty = true;
        let overflow: Vec<String> = {
            let mut ids = Vec::new();
            while self.history.len() > self.max_history_size {
                if let Some(pos) = self.history.iter().rposition(|i| !i.pinned) {
                    let removed = self.history.remove(pos);
                    ids.push(removed.id.clone());
                    self.remove_image_file(&removed.id);
                } else {
                    break;
                }
            }
            ids
        };
        self.rebuild_hash_index();
        if let Some(item) = self.history.iter().find(|i| i.id == inserted_id).cloned() {
            if let Err(e) = self.persist_upsert_item(&item, insert_pos as i64) {
                tracing::warn!("[ClipboardManager] Upsert failed ({e}); rewriting");
                self.save_history();
                return;
            }
        }
        if let Err(e) = self.persist_delete_ids(&overflow) {
            tracing::warn!("[ClipboardManager] Overflow delete failed ({e})");
        }
        self.persist_mutation();
    }

    fn enforce_history_limit(&mut self) -> bool {
        let before = self.history.len();
        while self.history.len() > self.max_history_size {
            if let Some(pos) = self.history.iter().rposition(|i| !i.pinned) {
                let removed = self.history.remove(pos);
                self.remove_image_file(&removed.id);
            } else {
                break;
            }
        }
        self.history.len() != before
    }

    fn remove_image_file(&mut self, id: &str) {
        if let Some(path) = self.image_paths.remove(id) {
            crate::image_store::remove_image(&path);
        }
    }

    pub fn get_history(&self) -> Vec<ClipboardItem> {
        self.history.clone()
    }

    /// History payload for the webview: preview + truncated plain text, no HTML.
    pub fn get_history_for_ui(&self) -> Vec<ClipboardItem> {
        self.history.iter().map(ClipboardItem::for_ipc).collect()
    }

    pub fn get_item(&self, id: &str) -> Option<&ClipboardItem> {
        self.history.iter().find(|item| item.id == id)
    }

    pub fn clear(&mut self) {
        let removed: Vec<String> = self
            .history
            .iter()
            .filter(|i| !i.pinned)
            .map(|i| i.id.clone())
            .collect();
        self.history.retain(|item| item.pinned);
        for id in &removed {
            self.remove_image_file(id);
        }
        self.dirty = true;
        self.rebuild_hash_index();
        if let Err(e) = self.persist_delete_ids(&removed) {
            tracing::warn!("[ClipboardManager] Clear delete failed ({e}); rewriting");
            self.save_history();
            return;
        }
        self.persist_mutation();
    }

    pub fn remove_item(&mut self, id: &str) {
        self.history.retain(|item| item.id != id);
        self.remove_image_file(id);
        self.dirty = true;
        self.rebuild_hash_index();
        if let Err(e) = self.persist_delete_ids(&[id.to_string()]) {
            tracing::warn!("[ClipboardManager] Delete failed ({e}); rewriting");
            self.save_history();
            return;
        }
        self.persist_mutation();
    }

    pub fn toggle_pin(&mut self, id: &str) -> Option<ClipboardItem> {
        let pos = self.history.iter().position(|i| i.id == id)?;
        self.history[pos].pinned = !self.history[pos].pinned;
        let item = self.history.remove(pos);
        let insert_pos = self
            .history
            .iter()
            .position(|i| !i.pinned)
            .unwrap_or(self.history.len());
        self.history.insert(insert_pos, item);
        let item_clone = self.history[insert_pos].clone();
        self.dirty = true;
        self.persist_mutation();
        Some(item_clone)
    }

    pub fn move_item_to_top(&mut self, id: &str) -> bool {
        let current_pos = match self.history.iter().position(|i| i.id == id) {
            Some(pos) => pos,
            None => return false,
        };
        let item_pinned = self.history[current_pos].pinned;
        let insert_pos = if item_pinned {
            0
        } else {
            self.history
                .iter()
                .position(|i| !i.pinned)
                .unwrap_or(self.history.len())
        };
        if insert_pos == current_pos {
            return true;
        }
        let item = self.history.remove(current_pos);
        self.history.insert(insert_pos, item);
        self.dirty = true;
        self.persist_mutation();
        true
    }

    pub fn cleanup_old_items(&mut self, interval_minutes: u64) -> bool {
        if interval_minutes == 0 {
            return false;
        }
        let now = Utc::now();
        let interval_seconds = (interval_minutes * 60) as i64;
        let mut removed_ids = Vec::new();
        self.history.retain(|item| {
            if item.pinned {
                return true;
            }
            let age_seconds = now.signed_duration_since(item.timestamp).num_seconds();
            let keep = age_seconds < interval_seconds;
            if !keep {
                removed_ids.push(item.id.clone());
            }
            keep
        });
        for id in &removed_ids {
            self.remove_image_file(id);
        }
        if !removed_ids.is_empty() {
            self.dirty = true;
            self.rebuild_hash_index();
            self.save_history();
            return true;
        }
        false
    }

    pub fn mark_as_pasted(&mut self, item: &ClipboardItem) {
        match &item.content {
            ClipboardContent::Text(text) => {
                self.last_pasted_text = Some(text.clone());
                self.last_pasted_image_hash = None;
            }
            ClipboardContent::RichText { plain, html: _ } => {
                self.last_pasted_text = Some(plain.clone());
                self.last_pasted_image_hash = None;
            }
            ClipboardContent::Image { .. } => {
                if let Some(hash) = item.extract_image_hash() {
                    self.last_pasted_image_hash = Some(hash);
                }
                self.last_pasted_text = None;
            }
        }
    }

    pub fn mark_text_as_pasted(&mut self, text: &str) {
        self.last_pasted_text = Some(text.to_string());
        self.last_added_text_hash = Some(calculate_hash(&text));
    }

    /// Write `item` onto the OS clipboard without injecting Ctrl+V.
    /// / نوشتن آیتم روی کلیپ‌بورد سیستم بدون تزریق Ctrl+V.
    ///
    /// Callers (`commands::paste_item`) then authorize keystroke injection
    /// via the paste ticket + `wrote_recently` gate.
    /// فراخواننده پس از آن تزریق را با بلیت paste و گیت `wrote_recently` مجاز می‌کند.
    pub fn write_item_to_clipboard(&mut self, item: &ClipboardItem) -> Result<(), String> {
        self.mark_as_pasted(item);
        match &item.content {
            ClipboardContent::Text(text) => self.set_text_robust(text)?,
            ClipboardContent::RichText { plain, html } => self.set_html_robust(html, plain)?,
            ClipboardContent::Image {
                base64,
                width,
                height,
            } => {
                if let Some(path) = self.image_paths.get(&item.id) {
                    if let Ok(png) = crate::image_store::read_png(path) {
                        self.set_image_png_bytes(&png)?;
                    } else {
                        self.set_image_robust(base64, *width, *height)?;
                    }
                } else {
                    self.set_image_robust(base64, *width, *height)?;
                }
            }
        }
        // Stamp so inject_authorized_paste's wrote_recently gate can pass
        // for image payloads that go through xclip/wl-copy without clipboard_io::write.
        crate::clipboard_io::notify_write();
        self.move_item_to_top(&item.id);
        Ok(())
    }

    pub fn paste_item(&mut self, item: &ClipboardItem) -> Result<(), String> {
        self.write_item_to_clipboard(item)?;
        self.simulate_paste_action()?;
        Ok(())
    }

    fn set_image_png_bytes(&self, png: &[u8]) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            if crate::session::is_wayland() {
                if self
                    .set_clipboard_external("wl-copy", &["--type", "image/png"], png)
                    .is_ok()
                {
                    return Ok(());
                }
            } else if self
                .set_clipboard_external(
                    "xclip",
                    &["-selection", "clipboard", "-t", "image/png"],
                    png,
                )
                .is_ok()
            {
                return Ok(());
            }
        }
        let img = image::load_from_memory(png).map_err(|e| format!("Image load failed: {e}"))?;
        let rgba = img.to_rgba8();
        let (width, height) = (rgba.width(), rgba.height());
        self.set_image_from_rgba(rgba.into_raw(), width, height)
    }

    fn set_image_from_rgba(&self, bytes: Vec<u8>, width: u32, height: u32) -> Result<(), String> {
        let mut clipboard = get_system_clipboard()?;
        let image_data = ImageData {
            width: width as usize,
            height: height as usize,
            bytes: bytes.clone().into(),
        };
        clipboard.set_image(image_data).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn set_image_robust(&self, base64_str: &str, width: u32, height: u32) -> Result<(), String> {
        let bytes = BASE64
            .decode(base64_str)
            .map_err(|e| format!("Base64 decode failed: {}", e))?;

        #[cfg(target_os = "linux")]
        {
            if crate::session::is_wayland() {
                if self
                    .set_clipboard_external("wl-copy", &["--type", "image/png"], &bytes)
                    .is_ok()
                {
                    return Ok(());
                }
            } else if self
                .set_clipboard_external(
                    "xclip",
                    &["-selection", "clipboard", "-t", "image/png"],
                    &bytes,
                )
                .is_ok()
            {
                return Ok(());
            }
        }

        let img =
            image::load_from_memory(&bytes).map_err(|e| format!("Image load failed: {}", e))?;
        let rgba = img.to_rgba8();
        self.set_image_from_rgba(rgba.into_raw(), width, height)
    }

    fn simulate_paste_action(&self) -> Result<(), String> {
        crate::input_simulator::simulate_paste_keystroke()
    }

    pub fn set_text_robust(&self, text: &str) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            if crate::session::is_wayland() {
                if let Ok(()) = self.set_clipboard_external(
                    "wl-copy",
                    &["--type", "text/plain;charset=utf-8"],
                    text.as_bytes(),
                ) {
                    crate::clipboard_io::write(&crate::clipboard_io::Payload::Text(text)).ok();
                    return Ok(());
                }
            } else if let Ok(()) = self.set_clipboard_external(
                "xclip",
                &["-selection", "clipboard", "-t", "UTF8_STRING"],
                text.as_bytes(),
            ) {
                crate::clipboard_io::notify_write();
                return Ok(());
            }
        }

        let mut clipboard = get_system_clipboard()?;
        clipboard.set_text(text).map_err(|e| e.to_string())?;
        let observed = clipboard.get_text().map_err(|e| e.to_string())?;
        if observed != text {
            return Err("Clipboard text verification returned different data".to_string());
        }
        crate::clipboard_io::notify_write();
        Ok(())
    }

    pub fn set_html_robust(&self, html: &str, plain: &str) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            if crate::session::is_wayland() {
                if let Ok(()) =
                    self.set_clipboard_external("wl-copy", &["--type", "text/html"], html.as_bytes())
                {
                    let _ = self.set_text_robust(plain);
                    return Ok(());
                }
            } else if let Ok(()) = self.set_clipboard_external(
                "xclip",
                &["-selection", "clipboard", "-t", "text/html"],
                html.as_bytes(),
            ) {
                let _ = self.set_text_robust(plain);
                return Ok(());
            }
        }

        let mut clipboard = get_system_clipboard()?;
        clipboard
            .set_html(html, Some(plain))
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn set_clipboard_external(&self, cmd: &str, args: &[&str], data: &[u8]) -> Result<(), String> {
        use std::io::{Read, Write};
        use std::process::{Command, Stdio};

        let owner_before = crate::paste_sync::clipboard_owner();

        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn {}: {}", cmd, e))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(data)
                .map_err(|e| format!("Pipe write error: {}", e))?;
        }

        if cmd == "wl-copy" {
            return wait_for_clipboard_helper_ready(&mut child, cmd);
        }

        let handoff_confirmed = crate::paste_sync::settle_clipboard_handoff(
            owner_before,
            CLIPBOARD_HELPER_READY_TIMEOUT,
        );
        if !handoff_confirmed {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "{} did not acquire the clipboard selection within {:?}",
                cmd, CLIPBOARD_HELPER_READY_TIMEOUT
            ));
        }

        match child.try_wait() {
            Ok(Some(status)) if !status.success() => {
                let mut stderr = String::new();
                if let Some(mut stderr_pipe) = child.stderr.take() {
                    let _ = stderr_pipe.read_to_string(&mut stderr);
                }
                Err(format!(
                    "{} exited with status {}. Stderr: {}",
                    cmd,
                    status,
                    stderr.trim()
                ))
            }
            Ok(_) => {
                thread::spawn(move || {
                    let _ = child.wait();
                });
                Ok(())
            }
            Err(e) => Err(format!("Process status check failed: {}", e)),
        }
    }
}

impl Drop for ClipboardManager {
    fn drop(&mut self) {
        if self.dirty {
            let _ = self.persist_sqlite();
        }
    }
}

fn persist_row_from_item(
    idx: usize,
    item: &ClipboardItem,
    image_path: Option<String>,
    crypto: &HistoryCrypto,
) -> Result<PersistRow, String> {
    let (kind, text, html, image_hash, width, height, thumb) = match &item.content {
        ClipboardContent::Text(t) => ("text", Some(t.clone()), None, None, None, None, None),
        ClipboardContent::RichText { plain, html } => (
            "richtext",
            Some(plain.clone()),
            Some(html.clone()),
            None,
            None,
            None,
            None,
        ),
        ClipboardContent::Image {
            base64,
            width,
            height,
        } => (
            "image",
            None,
            None,
            item.extract_image_hash().map(|h| h as i64),
            Some(*width as i64),
            Some(*height as i64),
            Some(base64.clone()),
        ),
    };
    Ok(PersistRow {
        id: item.id.clone(),
        kind,
        text: crypto.encrypt_optional(text.as_deref())?,
        html: crypto.encrypt_optional(html.as_deref())?,
        image_path,
        image_hash,
        width,
        height,
        preview: crypto.encrypt_str(&item.preview)?,
        pinned: if item.pinned { 1 } else { 0 },
        created_at: item.timestamp.timestamp_millis(),
        thumb: crypto.encrypt_optional(thumb.as_deref())?,
        sort_index: idx as i64,
    })
}

fn wait_for_clipboard_helper_ready(
    child: &mut std::process::Child,
    command: &str,
) -> Result<(), String> {
    use std::io::Read;

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(status)) => {
                let mut stderr = String::new();
                if let Some(mut pipe) = child.stderr.take() {
                    let _ = pipe.read_to_string(&mut stderr);
                }
                return Err(format!(
                    "{} exited with status {}. Stderr: {}",
                    command,
                    status,
                    stderr.trim()
                ));
            }
            Ok(None) if start.elapsed() < CLIPBOARD_HELPER_READY_TIMEOUT => {
                thread::sleep(CLIPBOARD_HELPER_POLL_INTERVAL);
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "{} did not confirm clipboard readiness within {:?}",
                    command, CLIPBOARD_HELPER_READY_TIMEOUT
                ));
            }
            Err(error) => {
                return Err(format!("Failed to inspect {} status: {}", command, error));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    fn temp_manager(name: &str) -> ClipboardManager {
        let dir = temp_dir().join(format!("clip-hist-{name}-{}", Uuid::new_v4()));
        let _ = fs::create_dir_all(&dir);
        ClipboardManager::new(dir.join("history.json"), 10)
    }

    #[test]
    fn persists_text_across_reload() {
        let dir = temp_dir().join(format!("clip-reload-{}", Uuid::new_v4()));
        let path = dir.join("history.json");
        {
            let mut mgr = ClipboardManager::new(path.clone(), 10);
            assert!(mgr.add_text("hello persistence".into(), None).is_some());
        }
        let mgr2 = ClipboardManager::new(path, 10);
        let hist = mgr2.get_history();
        assert_eq!(hist.len(), 1);
        match &hist[0].content {
            ClipboardContent::Text(t) => assert_eq!(t, "hello persistence"),
            _ => panic!("expected text"),
        }
    }

    #[test]
    fn secrets_are_not_stored() {
        let mut mgr = temp_manager("secrets");
        assert!(mgr
            .add_text("ghp_abcdefghijklmnopqrstuvwxyz0123456789".into(), None)
            .is_none());
        assert!(mgr.get_history().is_empty());
    }

    #[test]
    fn duplicate_text_is_not_reinserted() {
        let mut mgr = temp_manager("dup");
        assert!(mgr.add_text("same".into(), None).is_some());
        assert!(mgr.add_text("same".into(), None).is_none());
        assert_eq!(mgr.get_history().len(), 1);
    }

    #[test]
    fn secrets_and_disk_are_encrypted() {
        let dir = temp_dir().join(format!("clip-enc-{}", Uuid::new_v4()));
        let path = dir.join("history.json");
        {
            let mut mgr = ClipboardManager::new(path.clone(), 10);
            assert!(mgr.add_text("encrypt-me-please".into(), None).is_some());
        }
        let db = dir.join("history.db");
        let conn = rusqlite::Connection::open(&db).unwrap();
        let stored: String = conn
            .query_row("SELECT text FROM items LIMIT 1", [], |r| r.get(0))
            .unwrap();
        assert_ne!(stored, "encrypt-me-please");
        let mgr2 = ClipboardManager::new(path, 10);
        match &mgr2.get_history()[0].content {
            ClipboardContent::Text(t) => assert_eq!(t, "encrypt-me-please"),
            _ => panic!("expected text"),
        }
    }

    #[test]
    fn incremental_persist_keeps_order() {
        let dir = temp_dir().join(format!("clip-inc-{}", Uuid::new_v4()));
        let path = dir.join("history.json");
        {
            let mut mgr = ClipboardManager::new(path.clone(), 10);
            assert!(mgr.add_text("one".into(), None).is_some());
            assert!(mgr.add_text("two".into(), None).is_some());
            mgr.remove_item(&mgr.get_history()[1].id.clone());
        }
        let mgr2 = ClipboardManager::new(path, 10);
        let hist = mgr2.get_history();
        assert_eq!(hist.len(), 1);
        match &hist[0].content {
            ClipboardContent::Text(t) => assert_eq!(t, "two"),
            _ => panic!("expected remaining text"),
        }
    }
}
