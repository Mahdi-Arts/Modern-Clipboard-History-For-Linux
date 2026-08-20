# 📦 Changelog

<div dir="rtl">

همه تغییرات قابل توجه این پروژه در این فایل ثبت می‌شود.

</div>

All notable changes to this project will be documented in this file.

---

## [2.2.0] - 2026-08-20

### Security / امنیت

- Paste injection (`finish_paste`) now requires a one-shot ticket issued only after a real clipboard write.
- Smart Actions open URLs via a Rust `xdg-open` helper with the same allowlist as the TypeScript sanitizer. The Tauri shell plugin permission (`http://*` / `https://*`) was removed.
- IPC history payloads strip HTML and cap text at 2048 characters; full items stay in SQLite.
- SQLite uses `secure_delete` and `0600` on the db plus `-wal`/`-shm` sidecars.
- Installer downloads from this repository's GitHub Releases by default (checksum required). Cloudsmith `curl | sudo bash` is opt-in (`USE_CLOUDSMITH=1`).
- CI now **blocks** on `npm test` + coverage, `cargo test`, Clippy `-D warnings`, and `cargo audit` / `npm audit`.
- Releases publish `SHA256SUMS`, an SPDX SBOM, and SLSA provenance; notes no longer point at the upstream fork.

### UI / UX

- Empty-state panel with Super+V hint, loading spinner instead of a blank window, tighter glass cards.

### Packaging / بسته‌بندی

- Debian `rules` ships the AppArmor profile; version 2.2.0 across npm / Cargo / Tauri / AUR / changelog.

---

## [2.1.0] - 2026-08-20

### Security / امنیت

- **Fully offline**: Vazirmatn font bundled locally; Google Fonts removed; CSP tightened to `font-src 'self' data:` — the app makes zero network calls at runtime.
- **Hardened URL sanitizer**: blocks IPv6 loopback/ULA/link-local/mapped, private/CGNAT/benchmark/multicast IPv4, raw control characters, and `.internal` hosts (tests caught the original gaps).
- **Mandatory supply-chain verification**: `install.sh` now *implements* `verify_downloaded_file` (previously referenced but undefined — every GitHub-releases fallback path was broken) and requires SHA256SUMS matches; optional GPG verification via `WIN11_CLIPBOARD_TRUST_KEY`.
- **Blocking security audits in CI**: `cargo audit` and `npm audit` are hard gates (previously `continue-on-error: true`).
- **Release hardening**: SLSA build provenance attestation, SPDX SBOM, SHA256SUMS published per release; all release URLs fixed to the Mahdi-Arts repository (previously pointed at the upstream fork's install scripts and Cloudsmith/AUR sources).
- Optional **AppArmor profile** (complain mode) shipped in `packaging/apparmor/` and installed by the deb to `/usr/share/doc/`.
- Identifier migrated to `io.github.mahdi-arts.clipboard-history` (Flatpak ID, developer metadata).

### Quality / کیفیت

- **Settings UI refactored**: 1106-line `SettingsApp.tsx` split into single-responsibility sections under `src/components/settings/` with a shared `SectionCard`; added `:focus-visible` rings, `prefers-reduced-motion` support, and fixed the undefined `custom-scrollbar` class.
- **Rust refactor**: 1622-line `linux_shortcut_manager.rs` split into `shortcut_config`, `shortcut_error`, `shortcut_utils`, `shortcut_gsettings`, and `shortcut_tiling` submodules (i3/Sway/Hyprland now share idempotent atomic helpers); all `println!/eprintln!` replaced with `tracing`.
- **Testing**: component tests (Testing Library + jsdom), real hook tests with `renderHook`, SSRF edge-case tests (28 urlSafety cases), keyboard-nav tests; **67→72+ tests**; Vitest coverage gate (85%+ lines on covered units).
- `rust-toolchain.toml` added for reproducible Rust builds.

### Packaging / بسته‌بندی

- Desktop file localized (fa); AppStream metainfo updated with 2.1.0 release entry; Flatpak manifest installs metainfo and uses the new app-id.
- Debian metadata: maintainer/author fields point to the project owner; AppArmor profile shipped in the deb.

### Documentation / مستندات

- New `docs/THREAT_MODEL.md` and `docs/adr/` (SQLite persistence, paste injection, SSRF DNS pinning).


### Reliability / پایداری

- Incremental SQLite persistence (upsert / delete / sort-index updates) instead of rewriting the whole table on every copy.
- History hard-cap lowered to **2000** items (was 100 000 on a full-rewrite store).
- Clipboard watcher no longer re-reads `user_settings.json` every poll.

### Security / امنیت

- Tiling WM config rewrite (`i3` / Sway / Hyprland) is **actually gated** on `allow_wm_config_rewrite` (default off).
- Outbound HTTPS clients pin DNS to already-validated public addresses (DNS-rebinding window closed).
- Conflict resolver no longer shells out via `sh -c`; only `gsettings` / `xfconf-query` argv.
- `install.sh` verifies `SHA256SUMS` when GitHub Releases publish them.
- CSP tightened (`img-src` no longer `https:`); Google Fonts allowed only for Vazirmatn.
- Smart-action URL sanitizer rejects loopback and link-local metadata IPs.
- Settings UI warns that password-manager skip is X11-only.

### Quality / کیفیت

- Fixed compile breaks from the 2.0 module split (`Ordering`, `Mutex`, command module paths).
- CI now **requires** `npm test` and `cargo test` before Linux builds.
- History / paste / pin commands return `Result<_, AppError>`.
- Bilingual strings for header, empty state, tray (fa/en), and smart actions.

### Packaging / بسته‌بندی

- Debian `changelog` / `copyright` / `source/format`; `postrm` no longer walks `/home/*`.
- Flatpak manifest no longer requests `--device=all` or `--share=network` by default.
- GitHub Releases publish `SHA256SUMS`.

### Testing / تست

- **New frontend tests**: `urlSafety.test.ts` (13 test cases), `useClipboardHistory.test.ts` (5 test cases)
- **Enhanced `historySearch.test.ts`**: comprehensive tests for regex safety, text extraction, and filtering
- **All tests use Vitest** with proper Tauri API mocking

### Packaging / بسته‌بندی

- **Debian packaging structure** (`packaging/debian/`):
  - `control` — package metadata with proper dependencies
  - `rules` — build rules using dh + Tauri
  - `postinst` — post-install script (udev, input group, desktop database)
  - `postrm` — post-remove script (cleanup)
- **Flatpak manifest** (`packaging/flatpak/`):
  - `dev.gustavosett.ClipboardHistory.yml` — full Flatpak build manifest
  - `dev.gustavosett.ClipboardHistory.metainfo.xml` — AppStream metadata for Flathub

### Quality / کیفیت

- Replaced scattered `eprintln!` calls with `tracing::info!`, `tracing::warn!`, `tracing::error!`
- Named threads (`clipboard-watcher`, `xtest-paste-warmup`, etc.) for better debugging
- Improved code documentation with module-level doc comments

---

## [1.1.0] - 2026-08-20

### Reliability / پایداری

- Clipboard history is persisted to SQLite (`history.db`) on every mutation, on Drop, and on Quit. The previous dirty-flag path never flushed to disk.
- Legacy `history.json` is migrated once, then renamed to `history.json.bak`.
- Images are stored as PNG files; the UI only receives a thumbnail.

### Privacy / حریم خصوصی

- Secret filter (private keys, tokens, JWTs, `password=`) on by default.
- Skip password-manager and private-browsing windows on X11 (on by default).
- Optional “don’t save images”.
- History/settings files are chmod `0600`.
- Tiling WM config rewrite is **opt-in**.

### Security / امنیت

- Real SSRF controls for GIF downloads: HTTPS-only, host allowlist, DNS/IP checks, no redirects.
- Tenor query strings are URL-encoded; API key comes only from `TENOR_API_KEY` (no hardcoded fallback).
- `finish_paste` refuses Ctrl+V unless a clipboard write happened in the last 5 seconds.
- `shell:allow-open` is scoped to `http(s)` and `mailto`.
- Tracing is initialized and the worker guard is kept alive.

### UI / UX

- Pinned items no longer render twice.
- Privacy section and language picker in Settings.
- Incremental `clipboard-changed` updates (no full history refetch).
- Safer regex search (length + nested-quantifier guard).

### Quality / کیفیت

- Frontend Vitest tests and Rust unit tests in CI.
- README rebuilt (the 1.0 architecture section was corrupted).
- Dockerfile no longer swallows build failures.

---

## [1.0.0] - 2026-08-20

### ✅ Production Release Readiness / آماده‌سازی انتشار پایدار

- Promoted the project to the first stable production release line.
- Synchronized application version across npm, Tauri, Cargo, Cargo.lock, and AUR packaging metadata.
- Updated release/package references for the current repository.
- Verified frontend production build, TypeScript, ESLint, Prettier formatting, lockfile reproducibility, and npm security audit.
- Prepared release workflow and Linux packaging metadata for generating `.deb`, `.rpm`, and AppImage artifacts in a full Tauri build environment.

---

## [0.8.0] - 2026-08-20

### 🌐 Bilingual Support / پشتیبانی دو زبانه (NEW)

- **Full Persian (فارسی) + English support** with `react-i18next`
- Instant language switching — **no restart required** (تمام برنامه بدون ریستارت تغییر می‌کند)
- Automatic **RTL (Right-to-Left)** layout for Persian
- **Vazirmatn** Persian font support
- Language selection in Settings UI + persistence in user settings
- Cross-window synchronization via `app-language-changed` event
- `set_app_language` Tauri command with validation
- All documentation bilingual: README, CONTRIBUTING, SECURITY, issue/PR templates
- `docs/BILINGUAL.md` — i18n development guide

### 🚀 Performance / کارایی

- **Unified clipboard I/O module** (`clipboard_io.rs`): single cached X11/Wayland connection reused across all reads — eliminates connection churn in the watcher
- **Watcher optimized**: clipboard read now happens *outside* the history mutex (shorter lock window), and one `arboard::Clipboard` instance is reused instead of 3 new ones per 500ms tick
- **O(1) duplicate detection**: `HashSet<u64>` text-hash index replaces linear scans
- **Debounced persistence**: dirty-flag based saving instead of full-file rewrite on every change
- **Atomic file writes** (`fs_atomic.rs`): `.tmp` + `rename` — crash-safe history/settings
- **Virtualized clipboard list**: `react-window` List replaces flat `Array.map()` — renders only visible items (performance for 100k+ items)

### 🔒 Security / امنیت

- **CSP enabled** (was `null`): strict `Content-Security-Policy` in `tauri.conf.json`
- **`withGlobalTauri: false`**: Tauri API no longer exposed to global scope
- **SSRF protection** for GIF downloads: HTTPS-only, private/loopback IPs blocked
- **10 MB download limit** for GIFs (streamed, not buffered)
- **Unified error types** (`error.rs` with `thiserror`) replacing `Result<(), String>`
- **Tenor API key moved to backend** (`tenor_api.rs`): key no longer in frontend bundle — client calls `search_tenor` Tauri command instead of direct API
- **CSP tightened**: Tenor CDN removed from CSP (now proxied through backend)

### 🧹 Code Quality / کیفیت کد

- `tracing` + `tracing-subscriber` logging with daily rotating log files (replaces scattered `eprintln!`)
- `Cargo.lock` now tracked in git (reproducible builds)
- Stable FNV hash for GIF cache filenames (was randomized `DefaultHasher`)
- GIF cache TTL (24h) to avoid stale content
- Lazy-loaded tabs (Emoji/Kaomoji Symbol pickers) with `React.lazy` + `Suspense`

### 🐳 DevOps & CI

- **Dockerfile** (multi-stage): reproducible build environment for CI
- **CI updated**: Rust tests (`cargo test`) added to pipeline; Docker build check job added
- **Build pipeline**: test job blocks build-linux (quality gate)

### 📚 Documentation / مستندات

- **Fully bilingual README** (فارسی/English) — 359 lines
- **Bilingual CONTRIBUTING.md** with i18n guidelines (349 lines)
- **Bilingual SECURITY.md**, issue templates, PR template
- New `docs/BILINGUAL.md` — i18n development guide
- New `CHANGELOG.md` — structured release history
- `OPTIMIZATION_REPORT.md` — full architecture analysis with prioritized action plan

---

## [0.7.1] - Previous Release

- Bug fixes and dependency updates (see git history for details)