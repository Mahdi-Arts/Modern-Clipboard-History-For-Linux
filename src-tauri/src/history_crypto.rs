//! Field-level at-rest encryption for SQLite history text.
//!
//! Sensitive columns (`text`, `html`, `preview`, `thumb_base64`) are stored as
//! `W11E1` || nonce(12) || ChaCha20-Poly1305 ciphertext. Legacy plaintext rows
//! are still readable and are rewritten encrypted on the next persist.
//!
//! The 256-bit key lives in `history.key` next to the database (mode `0600`).
//! This protects idle disk images and other local users; it does **not**
//! protect against a process already running as the same UID.

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use rand::rngs::OsRng;
use rand::RngCore;
use std::fs;
use std::path::{Path, PathBuf};

const MAGIC: &[u8] = b"W11E1";
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const KEY_FILE: &str = "history.key";

pub struct HistoryCrypto {
    cipher: ChaCha20Poly1305,
}

impl HistoryCrypto {
    pub fn load_or_create(data_dir: &Path) -> Result<Self, String> {
        let path = key_path(data_dir);
        crate::fs_atomic::ensure_parent(&path).map_err(|e| e.to_string())?;
        let key_bytes = if path.exists() {
            let bytes = fs::read(&path).map_err(|e| format!("read history.key: {e}"))?;
            if bytes.len() != KEY_LEN {
                return Err("history.key has unexpected length".into());
            }
            bytes
        } else {
            let mut bytes = vec![0u8; KEY_LEN];
            rand::rngs::OsRng.fill_bytes(&mut bytes);
            crate::fs_atomic::write_atomic(&path, &bytes).map_err(|e| e.to_string())?;
            crate::fs_atomic::restrict_permissions(&path);
            bytes
        };
        let key = Key::from_slice(&key_bytes);
        Ok(Self {
            cipher: ChaCha20Poly1305::new(key),
        })
    }

    pub fn encrypt_optional(&self, plain: Option<&str>) -> Option<String> {
        plain.map(|p| self.encrypt_str(p))
    }

    pub fn encrypt_str(&self, plain: &str) -> String {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ct = self
            .cipher
            .encrypt(nonce, plain.as_bytes())
            .unwrap_or_else(|_| plain.as_bytes().to_vec());
        let mut out = Vec::with_capacity(MAGIC.len() + NONCE_LEN + ct.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&nonce_bytes);
        out.extend_from_slice(&ct);
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, out)
    }

    pub fn decrypt_optional(&self, stored: Option<String>) -> Option<String> {
        stored.map(|s| self.decrypt_str(&s))
    }

    pub fn decrypt_str(&self, stored: &str) -> String {
        if !looks_encrypted(stored) {
            return stored.to_string();
        }
        let Ok(raw) = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, stored)
        else {
            return stored.to_string();
        };
        if raw.len() < MAGIC.len() + NONCE_LEN + 16 || &raw[..MAGIC.len()] != MAGIC {
            return stored.to_string();
        }
        let nonce = Nonce::from_slice(&raw[MAGIC.len()..MAGIC.len() + NONCE_LEN]);
        let ct = &raw[MAGIC.len() + NONCE_LEN..];
        match self.cipher.decrypt(nonce, ct) {
            Ok(pt) => String::from_utf8(pt).unwrap_or_else(|_| stored.to_string()),
            Err(_) => stored.to_string(),
        }
    }
}

fn key_path(data_dir: &Path) -> PathBuf {
    data_dir.join(KEY_FILE)
}

fn looks_encrypted(s: &str) -> bool {
    s.len() > 16 && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn roundtrip_and_legacy_plaintext() {
        let dir = std::env::temp_dir().join(format!("hist-crypto-{}", Uuid::new_v4()));
        let _ = fs::create_dir_all(&dir);
        let crypto = HistoryCrypto::load_or_create(&dir).unwrap();
        let enc = crypto.encrypt_str("secret clipboard");
        assert_ne!(enc, "secret clipboard");
        assert_eq!(crypto.decrypt_str(&enc), "secret clipboard");
        assert_eq!(crypto.decrypt_str("legacy plaintext"), "legacy plaintext");
    }

    #[test]
    fn same_key_file_reused() {
        let dir = std::env::temp_dir().join(format!("hist-crypto-k-{}", Uuid::new_v4()));
        let _ = fs::create_dir_all(&dir);
        let a = HistoryCrypto::load_or_create(&dir).unwrap();
        let blob = a.encrypt_str("hello");
        let b = HistoryCrypto::load_or_create(&dir).unwrap();
        assert_eq!(b.decrypt_str(&blob), "hello");
    }
}
