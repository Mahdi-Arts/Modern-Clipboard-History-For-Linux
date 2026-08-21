# ADR-0006: Encryption-key backend (file ↔ Secret Service)

- **Status:** Accepted (v2.3.0)
- **Date:** 2026-08-21

## Context / زمینه

The history encryption key was a 32-byte file (`history.key`, mode `0600`)
next to the SQLite database. That protects other local users and idle disk
images, but the key itself sits unencrypted on disk. Desktop environments
already ship an encrypted, login-unlocked store: the freedesktop **Secret
Service** (GNOME Keyring / KWallet).

کلید رمزنگاری تاریخچه فایلی ۳۲ بایتی کنار دیتابیس بود. این برای کاربران
دیگر و تصاویر دیسک کافی است، اما خود کلید رمزنشده روی دیسک است؛ در حالی
که دسکتاپ‌ها فروشگاه رمزنگاری‌شدهٔ بازشونده با ورود کاربر دارند: Secret
Service (GNOME Keyring / KWallet).

## Decision / تصمیم

1. `history_crypto::KeyBackend` abstracts the storage location:
   `File` (default, unchanged) and `SecretService`.
2. The Secret Service backend v1 talks to the keyring through the
   `secret-tool` helper (`libsecret-tools`), storing the base64 key under
   attributes `application=modern-clipboard-history-for-linux`, `purpose=history.key`.
   No new Rust dependency; native zbus integration remains a future option.
3. **Key-integrity marker.** `history.key.check` stores
   `ChaCha20-Poly1305("modern-clipboard-history-for-linux:key-check:v1")` under the
   adopted key. A backend may only be adopted when it decrypts the marker —
   otherwise loading **fails closed** rather than risking history encrypted
   under the wrong key.
   نشانگر یکپارچگی: بک‌اند فقط وقتی پذیرفته می‌شود که نشانگر را رمزگشایی
   کند؛ در غیر این صورت بارگذاری fail-closed می‌شود.
4. Databases written before the marker exist keep their file key on first
   launch (never silently re-keyed). Migration is explicit:
   `migrate_history_key_to_secret_service` stores the key, verifies the
   read-back, then renames `history.key` → `history.key.migrated`;
   `migrate_history_key_to_file` reverses it.
5. When the requested backend is unavailable (headless session, missing
   `secret-tool`), the loader falls back to the file key **only** if the
   file key proves itself against the marker.

## Consequences / پیامدها

- With the keyring backend the key never touches the disk; losing access to
  the keyring loses the history (pinned items included) — surfaced in the
  Settings UI before switching.
- The `secret-tool` exec happens once at startup and once per migration;
  capture/paste paths are unaffected.
- A same-UID adversary remains out of scope (see THREAT_MODEL §3, T1).
