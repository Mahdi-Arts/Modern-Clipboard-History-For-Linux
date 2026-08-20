# Final report — v2.2.0 production hardening

This release applies the review findings from the architecture/security audit
and aligns CI, installer, packaging, and runtime controls with the README.

## What changed

1. **Compile / quality**
   - Replaced invalid `etracing::` calls with `tracing::`.
   - Restored missing `AtomicBool` / `Mutex` imports so the Rust crate type-checks.
2. **Security**
   - `finish_paste` requires a one-shot ticket after a clipboard write.
   - Smart Actions call `open_safe_url` (Rust `xdg-open` + allowlist). Tauri
     `shell:allow-open` was removed.
   - History IPC strips HTML and caps text at 2048 chars.
   - SQLite: `secure_delete`, `0600` on db + WAL/SHM.
3. **Supply chain**
   - CI blocks on tests, coverage, Clippy, `cargo audit`, `npm audit`.
   - Releases emit SHA256SUMS, SPDX SBOM, SLSA provenance for this repo only.
   - Installer verifies GitHub artifacts by default; Cloudsmith is opt-in.
4. **Packaging**
   - Debian `packaging/debian/` installs AppArmor docs; Flatpak manifest kept
     conservative (no `--device=all`).
5. **UI**
   - Empty-state Super+V hint; loading spinner instead of a blank window.

## Verification (this workspace)

- `npm test` — 73 tests, all passing
- `npm run lint` — passing
- `npm run test:coverage` — thresholds met
- `bash -n scripts/install.sh` — passing

Rust `cargo test` / Clippy run in GitHub Actions (GTK/WebKit sysroot).

## Residual accepted risk

- History remains unencrypted at rest (`0600` + OS disk encryption).
- `/dev/uinput` can type into the focused window; treat the binary as a
  trusted input device.
- Wayland cannot skip password-manager windows by identity.
