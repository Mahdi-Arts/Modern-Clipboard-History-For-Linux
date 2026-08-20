<div align="center">

# 📋 Modern Clipboard History for Linux
### مدیر تاریخچه کلیپ‌بورد مدرن برای لینوکس

> A fast, bilingual clipboard manager for Linux (Wayland & X11), inspired by Windows 11 Win+V.
> مدیر کلیپ‌بورد سریع و دوزبانه برای لینوکس، با الهام از Win+V ویندوز ۱۱.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Tauri v2](https://img.shields.io/badge/Built_With-Tauri_v2-24C8D6?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Powered_By-Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![React 19](https://img.shields.io/badge/Frontend-React_19-61DAFB?style=for-the-badge&logo=react&logoColor=black)](https://react.dev/)
[![Bilingual](https://img.shields.io/badge/🌐_زبان-فارسی_|_English-009688?style=for-the-badge)]()

</div>

---

## Features / ویژگی‌ها

| | English | فارسی |
| --- | --- | --- |
| 🐧 | Wayland and X11 | پشتیبانی Wayland و X11 |
| ⚡ | `Super+V` / `Ctrl+Alt+V` | میانبر فوری |
| 🌐 | Persian + English, instant RTL | فارسی و انگلیسی با RTL آنی |
| 📌 | Pin important items | سنجاق آیتم‌های مهم |
| 🤩 | Emoji, kaomoji, symbols | ایموجی، کائوموجی، نماد |
| 🛡️ | Local SQLite history, secret filter | تاریخچه محلی، فیلتر اسرار |
| 🎨 | Acrylic UI, light/dark/system | ظاهر شیشه‌ای، تم سیستم |
| 🔍 | Search with optional regex | جستجو با regex اختیاری |

---

## Privacy / حریم خصوصی

Clipboard history is stored **only on this machine**:

- Database: `~/.local/share/win11-clipboard-history/history.db` (mode `0600`)
- Images: `~/.local/share/win11-clipboard-history/images/` (full PNG; UI gets a thumbnail)
- Settings: `~/.config/win11-clipboard-history/user_settings.json`

**Defaults (on):**

- Skip private keys, API tokens, JWTs, and `password=` values
- Skip password-manager and private-browsing windows on **X11** (Wayland compositors do not expose the focused app)
- Save images (can be turned off)

History size is capped at **2000** items. Persistence is incremental (upsert/delete), not a full rewrite on every copy.

**Network:** the app does not upload clipboard contents. GIF search (optional, currently hidden) requires `TENOR_API_KEY` in the environment and only talks to Tenor over pinned HTTPS.

This project needs `/dev/uinput` (or XTest) to simulate Ctrl+V. That is a powerful permission — treat the binary like a trusted input device.

Tiling window managers (i3 / Sway / Hyprland) are **not** rewritten unless you enable *Allow rewriting tiling WM configs* in Settings.

---

## Installation / نصب

Prefer your package manager over piping scripts to `bash`. Verify `SHA256SUMS` from GitHub Releases.

### Debian / Ubuntu

```bash
sudo apt install ./win11-clipboard-history_2.0.0_amd64.deb
sudo setfacl -m u:$USER:rw /dev/uinput
```

Download the `.deb` from [GitHub Releases](https://github.com/Mahdi-Arts/Modern-Clipboard-History-For-Linux/releases) after verifying the published checksum.

### Fedora

```bash
sudo dnf install ./win11-clipboard-history-2.0.0-1.x86_64.rpm
sudo setfacl -m u:$USER:rw /dev/uinput
```

### Arch Linux (AUR)

```bash
yay -S win11-clipboard-history-bin
```

### Flatpak

See `packaging/README.md`. The Flatpak sandbox does not grant `/dev/uinput`; use the `.deb`/`.rpm` for paste simulation, or override with `--device=all`.

### Convenience installer (review before running)

```bash
curl -fsSL https://raw.githubusercontent.com/Mahdi-Arts/Modern-Clipboard-History-For-Linux/master/scripts/install.sh -o install-clipboard.sh
less install-clipboard.sh
bash install-clipboard.sh
```

The installer verifies `SHA256SUMS` when GitHub Releases publish that file.

---

## Shortcuts / میانبرها

| Key | Action |
| --- | --- |
| <kbd>Super</kbd>+<kbd>V</kbd> | Open clipboard history |
| <kbd>Super</kbd>+<kbd>.</kbd> | Open emoji picker |
| <kbd>Ctrl</kbd>+<kbd>Alt</kbd>+<kbd>V</kbd> | Alternative shortcut |
| <kbd>Enter</kbd> | Paste selected item |
| <kbd>Esc</kbd> | Close |
| <kbd>Ctrl</kbd>+<kbd>F</kbd> | Search |

---

## Architecture / معماری

| Layer | Stack |
| --- | --- |
| UI | React 19, TypeScript, Tailwind CSS 4, lazy-loaded pickers |
| Backend | Rust, Tauri v2 |
| Clipboard I/O | arboard + `wl-copy` / `xclip` |
| Persistence | SQLite (WAL) incremental upsert, PNG files, atomic JSON for settings |
| Input | Persistent uinput device (Wayland) / XTest (X11) |
| Security | CSP, `withGlobalTauri: false`, SSRF allowlist + DNS pin, scoped `shell:allow-open` |

---

## Troubleshooting / عیب‌یابی

| Issue | Fix |
| --- | --- |
| Shortcut does nothing on GNOME | GNOME reserves Super+V. Rebind notification tray, or use Ctrl+Alt+V. Reset: `rm ~/.config/win11-clipboard-history/setup.json` |
| Black / opaque window on NVIDIA | `IS_NVIDIA=1 win11-clipboard-history` |
| Paste does nothing | `sudo setfacl -m u:$USER:rw /dev/uinput` then log out/in |
| Window missing on Wayland | Run from a terminal and read the log in `~/.local/share/win11-clipboard-history/logs/` |
| Password manager still captured | On Wayland, focused-app skip is unavailable. Keep *Skip secrets* on. |

---

## Development / توسعه

```bash
git clone https://github.com/Mahdi-Arts/Modern-Clipboard-History-For-Linux.git
cd Modern-Clipboard-History-For-Linux
make deps && make rust && make node
source ~/.cargo/env
make dev          # hot reload
make test         # frontend + Rust unit tests
make lint
make build
```

CI runs `npm test`, `cargo test`, Clippy (`-D warnings`), and `npm audit --audit-level=high`.

### Environment

| Variable | Purpose |
| --- | --- |
| `IS_NVIDIA=1` | WebKit DMA-BUF workaround |
| `IS_APPIMAGE=1` | Same workaround for AppImage |
| `TENOR_API_KEY` | Optional GIF search (not bundled) |
| `RUST_LOG=info` | Tracing level |

Packaging notes: `packaging/README.md`.

---

## License / مجوز

MIT. Based on the original [Windows 11 Clipboard History For Linux](https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux) by gustavosett.

Security reports: use [GitHub private advisories](https://github.com/Mahdi-Arts/Modern-Clipboard-History-For-Linux/security/advisories/new) — do not open a public issue.
