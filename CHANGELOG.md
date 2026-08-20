# 📦 Changelog

<div dir="rtl">

همه تغییرات قابل توجه این پروژه در این فایل ثبت می‌شود.

</div>

All notable changes to this project will be documented in this file.

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