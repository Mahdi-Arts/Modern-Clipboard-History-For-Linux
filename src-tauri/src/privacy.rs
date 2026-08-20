//! Privacy filters for clipboard history.
//! Secrets, password-manager windows, and optional image capture.

use serde::{Deserialize, Serialize};

/// Runtime privacy policy loaded from user settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrivacyPolicy {
    /// Drop clipboard text that looks like a credential or key.
    pub filter_secrets: bool,
    /// Persist captured images (full PNG on disk + thumbnail in history).
    pub save_images: bool,
    /// Skip capture when the focused app looks like a password manager / incognito window.
    pub exclude_sensitive_apps: bool,
    /// Extra WM_CLASS / title fragments supplied by the user.
    pub extra_excluded_apps: Vec<String>,
}

impl Default for PrivacyPolicy {
    fn default() -> Self {
        Self {
            filter_secrets: true,
            save_images: true,
            exclude_sensitive_apps: true,
            extra_excluded_apps: Vec::new(),
        }
    }
}

const DEFAULT_EXCLUDED_APPS: &[&str] = &[
    "keepass",
    "keepassxc",
    "keepassx",
    "1password",
    "1password-linux",
    "bitwarden",
    "bitwarden-desktop",
    "vaultwarden",
    "lastpass",
    "enpass",
    "protonpass",
    "proton-pass",
    "protonmail",
    "authy",
    "seahorse",
    "gnome-keyring",
    "kwallet",
    "kwalletmanager",
    "secret-service",
    "snap.1password",
    "org.keepassxc.keepassxc",
    "com.bitwarden.desktop",
    "org.gnome.seahorse",
];

const SENSITIVE_TITLE_FRAGMENTS: &[&str] = &[
    "private browsing",
    "incognito",
    "inprivate",
    "password",
    "passkey",
    "unlock vault",
    "master password",
];

/// True when `text` looks like a secret that must never be stored.
pub fn looks_like_secret(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }

    if looks_like_private_key(trimmed) {
        return true;
    }
    if looks_like_known_token(trimmed) {
        return true;
    }
    if looks_like_jwt(trimmed) {
        return true;
    }
    if looks_like_password_assignment(trimmed) {
        return true;
    }
    false
}

fn looks_like_private_key(text: &str) -> bool {
    text.contains("BEGIN OPENSSH PRIVATE KEY")
        || text.contains("BEGIN RSA PRIVATE KEY")
        || text.contains("BEGIN PRIVATE KEY")
        || text.contains("BEGIN EC PRIVATE KEY")
        || text.contains("BEGIN DSA PRIVATE KEY")
        || text.contains("BEGIN PGP PRIVATE KEY BLOCK")
}

fn looks_like_known_token(text: &str) -> bool {
    let compact = text.trim();
    const PREFIXES: &[&str] = &[
        "ghp_",
        "gho_",
        "ghu_",
        "ghs_",
        "ghr_",
        "github_pat_",
        "xoxb-",
        "xoxp-",
        "xoxa-",
        "xoxr-",
        "sk_live_",
        "rk_live_",
        "sk_test_",
        "AIza",
        "ya29.",
    ];
    if PREFIXES.iter().any(|p| compact.starts_with(p)) {
        return compact.len() >= 20;
    }
    if compact.starts_with("sk-") && compact.len() >= 24 && compact.is_ascii() {
        return compact
            .bytes()
            .skip(3)
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
    }
    if let Some(idx) = compact.find("AKIA") {
        let slice = compact.get(idx..idx + 20).unwrap_or("");
        return slice.len() == 20
            && slice.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit());
    }
    false
}

fn looks_like_jwt(text: &str) -> bool {
    let compact = text.trim();
    if !compact.starts_with("eyJ") {
        return false;
    }
    let mut parts = compact.split('.');
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some(h), Some(p), Some(s), None)
            if h.len() > 10 && p.len() > 10 && s.len() > 10
    )
}

fn looks_like_password_assignment(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    const KEYS: &[&str] = &[
        "password=",
        "passwd=",
        "pwd=",
        "secret=",
        "client_secret=",
        "api_secret=",
    ];
    KEYS.iter().any(|k| lower.contains(k)) && text.len() < 4096
}

/// True when the focused window should not contribute clipboard history.
pub fn is_sensitive_source(class: &str, title: &str, extra: &[String]) -> bool {
    let class_l = class.to_ascii_lowercase();
    let title_l = title.to_ascii_lowercase();

    if DEFAULT_EXCLUDED_APPS
        .iter()
        .any(|frag| class_l.contains(frag) || title_l.contains(frag))
    {
        return true;
    }
    if SENSITIVE_TITLE_FRAGMENTS
        .iter()
        .any(|frag| title_l.contains(frag))
    {
        return true;
    }
    extra.iter().any(|frag| {
        let f = frag.trim().to_ascii_lowercase();
        !f.is_empty() && (class_l.contains(&f) || title_l.contains(&f))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_pem_and_tokens() {
        assert!(looks_like_secret(
            "-----BEGIN OPENSSH PRIVATE KEY-----\nabc\n-----END OPENSSH PRIVATE KEY-----"
        ));
        assert!(looks_like_secret("ghp_abcdefghijklmnopqrstuvwxyz0123456789"));
        assert!(looks_like_secret("sk-abcdefghijklmnopqrstuvwxyz012345"));
        assert!(looks_like_secret(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4ifQ.s3cr3tSignatureHere"
        ));
        assert!(looks_like_secret("https://example.com/login?password=hunter2"));
        assert!(!looks_like_secret("hello world"));
        assert!(!looks_like_secret("https://example.com/docs"));
    }

    #[test]
    fn detects_password_managers() {
        assert!(is_sensitive_source("keepassxc", "KeePassXC", &[]));
        assert!(is_sensitive_source("firefox", "Private Browsing", &[]));
        assert!(is_sensitive_source("Code", "main.rs", &["code".into()]));
        assert!(!is_sensitive_source("firefox", "Example Domain", &[]));
    }
}
