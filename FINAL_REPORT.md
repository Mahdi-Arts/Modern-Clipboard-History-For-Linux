# Final report — enterprise hardening (v2.2.0)

## What shipped

1. **At-rest encryption** — `history_crypto.rs` encrypts SQLite text columns with ChaCha20-Poly1305. Key: `~/.local/share/win11-clipboard-history/history.key` (`0600`). Legacy plaintext still loads. ADR 0004.
2. **Persistence split** — schema/SQL lives in `history_store.rs`; policy stays in `clipboard_manager.rs`. Paste-ticket unit tests in `lib.rs`.
3. **CI** — `cargo audit` and `npm audit --audit-level=high` are **blocking**. Coverage, Clippy `-D warnings`, Xvfb smoke after build.
4. **Releases** — `SHA256SUMS` generated and uploaded; install URLs point at `Mahdi-Arts/Modern-Clipboard-History-For-Linux`.
5. **Packaging** — Debian `control`/`rules` and Flatpak manifest remain the source of truth for `.deb` and Flathub-style builds. AppArmor documents `history.key`.
6. **UI** — slightly deeper acrylic glass; privacy copy explains encryption (EN/FA).

## How to ship a `.deb`

```bash
make deps && npm ci && npm run tauri:build
# src-tauri/target/release/bundle/deb/*.deb
```

Classic:

```bash
cp -a packaging/debian debian
dpkg-buildpackage -us -uc
```

## Flatpak

`packaging/flatpak/io.github.mahdi-arts.clipboard-history.yml`  
Paste via `/dev/uinput` still needs `--device=all` or a native deb.

## Tests added

- History crypto round-trip
- Encrypted-on-disk vs decrypted-in-memory
- One-shot paste tickets
