# ADR 0004 — Field-level encryption at rest

- **Status:** Accepted
- **Date:** 2026-08-20

## Context

Clipboard history may contain credentials that the secret filter missed.
SQLite files with mode `0600` still leak to anyone who can read the user
home directory (stolen disk, another UID with backups). SQLCipher would
encrypt the whole database but adds a C toolchain and ABI surface.

## Decision

Encrypt `text`, `html`, `preview`, and `thumb_base64` columns with
ChaCha20-Poly1305. A 256-bit key is stored in `history.key` (mode `0600`)
beside `history.db`. On-disk format: Base64(`W11E1` || nonce || ciphertext).
Legacy plaintext rows remain readable and are rewritten encrypted on persist.

## Consequences

- Other local users and idle disk images cannot read history without the key file.
- A process already running as the same UID can still read `history.key`.
- Optional future work: wrap the key with libsecret / KWallet.
