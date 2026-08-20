<div align="center">

<a href="https://github.com/Mahdi-Arts/Modern-Clipboard-History-For-Linux">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://github.com/user-attachments/assets/4534e915-5d83-45f3-9f09-48a0f94b1d9a">
   <img width="64" height="64" alt="logo" src="https://github.com/user-attachments/assets/4534e915-5d83-45f3-9f09-48a0f94b1d9a">
 </picture>
</a>

# 📋 Modern Clipboard History for Linux  
### مدیرت تاریخچه کلیپ‌بورد مدرن برای لینوکس

> **The most beautiful, feature-rich clipboard manager your Linux desktop deserves.**  
> **زیباترین و قدرتمندترین مدیر تاریخچه کلیپبورد برای دسکتاپ لینوکسی شما.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Tauri v2](https://img.shields.io/badge/Built_With-Tauri_v2-24C8D6?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Powered_By-Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![React 19](https://img.shields.io/badge/Frontend-React_19-61DAFB?style=for-the-badge&logo=react&logoColor=black)](https://react.dev/)
[![Bilingual](https://img.shields.io/badge/🌐_زبان-فارسی_|_English-009688?style=for-the-badge)]()

![App Screenshot](https://github.com/user-attachments/assets/74400c8b-9d7d-49ce-8de7-45dfd556e256)

</div>

---

<div dir="rtl">

## 🌟 معرفی

**مدیریت تاریخچه کلیپبورد مدرن** یک برنامه‌ی زیبا، سریع و قدرتمند برای مدیریت تاریخچه کلیپبورد در لینوکس است. این برنامه با الهام از طراحی ویندوز ۱۱ ساخته شده و از **Rust** برای سرعت فوق‌العاده و از **React 19** برای رابط کاربری روان استفاده می‌کند.

### ✨ ویژگی‌های کلیدی

| 🚀 | توضیح |
| --- | --- |
| **🐧 پشتیبانی کامل** | کار flawless روی **Wayland** و **X11** |
| **⚡ دسترسی فوری** | باز شدن با `Super+V` یا `Ctrl+Alt+V` در کسری از ثانیه |
| **🌐 دو زبانه** | پشتیبانی کامل از **فارسی** و **English** با قابلیت جابجایی لحظه‌ای |
| **🧠 موقعیت‌یابی هوشمند** | پنجره برنامه موس شما را روی چند مانیتور دنبال می‌کند |
| **📌 سنجاق و همگام‌سازی** | آیتم‌های مهم را سنجاق کنید تا همیشه در دسترس باشند |
| **🤩 ایموجی پیکر** | جستجو و چسباندن ایموجی با بیش از ۲۰۰۰ ایموجی |
| **🎭 کائوموجی پیکر** | صدها کائوموجی ژاپنی برای ابراز احساسات |
| **🔣 نمادها** | دسترسی سریع به نمادها و کاراکترهای ویژه |
| **🛡️ حریم خصوصی** | تاریخچه شما کاملاً محلی ذخیره می‌شود |
| **🎨 شفافیت شیشه‌ای** | افکت acrylic زیبا با قابلیت تنظیم شفافیت |
| **🔍 جستجوی پیشرفته** | جستجو با متن و پشتیبانی از **عبارت باقاعده** |
| **🧠 عملیات هوشمند** | تشخیص خودکار URL، ایمیل و رنگ‌ها |

</div>

---

<details>
<summary><b>🇬🇧 English <i>(click to expand)</i></b></summary>
<br>

## 🌟 Introduction

**Modern Clipboard History** is a beautiful, fast, and powerful clipboard history manager for Linux. Inspired by Windows 11's design and built with **Rust** for blazing speed and **React 19** for a fluid UI.

### ✨ Key Features

| 🚀 | Description |
| --- | --- |
| **🐧 Universal Support** | Works flawlessly on both **Wayland** & **X11** |
| **⚡ Instant Access** | Opens instantly with `Super+V` or `Ctrl+Alt+V` |
| **🌐 Bilingual** | Full support for **Persian (فارسی)** and **English** with instant switching |
| **🧠 Smart Positioning** | Window follows your cursor across multiple monitors |
| **📌 Pin & Sync** | Pin important snippets to keep them at the top |
| **🤩 Emoji Picker** | Search and paste from 2000+ emojis |
| **🎭 Kaomoji Picker** | Hundreds of Japanese kaomoji expressions |
| **🔣 Symbols Picker** | Quick access to special characters and symbols |
| **🛡️ Privacy First** | Your history is stored locally — no data leaves your machine |
| **🎨 Acrylic Effect** | Beautiful glass-morphism with customizable transparency |
| **🔍 Advanced Search** | Full-text search with **regex support** |
| **🧠 Smart Actions** | Auto-detection of URLs, emails, and color values |

</details>

---

## 📦 Quick Installation / نصب سریع

<div dir="rtl">

### روش توصیه شده — اسکریپت خودکار

```bash
curl -fsSL https://raw.githubusercontent.com/Mahdi-Arts/Modern-Clipboard-History-For-Linux/master/scripts/install.sh | bash
```

> ✅ بدون نیاز به خارج شدن از حساب کاربری! نصاب از ACL برای دسترسی فوری استفاده می‌کند.

</div>

<details>
<summary><b>📦 Manual Installation / نصب دستی</b></summary>

### Debian / Ubuntu / Zorin OS / Pop!_OS / Mint

<details open>
<summary>APT Repository (Recommended)</summary>

```bash
# Add Repository
curl -1sLf 'https://dl.cloudsmith.io/public/gustavosett/clipboard-manager/setup.deb.sh' | sudo -E bash

# Install
sudo apt update && sudo apt install win11-clipboard-history

# Grant Permissions (one-time)
sudo setfacl -m u:$USER:rw /dev/uinput
```
</details>

### Fedora / RHEL

```bash
curl -1sLf 'https://dl.cloudsmith.io/public/gustavosett/clipboard-manager/setup.rpm.sh' | sudo -E bash
sudo dnf install win11-clipboard-history
sudo setfacl -m u:$USER:rw /dev/uinput
```

### Arch Linux (AUR)

```bash
yay -S win11-clipboard-history-bin
# OR
paru -S win11-clipboard-history-bin
```

### AppImage (Universal)

```bash
# Download from Releases, then:
chmod +x win11-clipboard-history_*.AppImage
sudo setfacl -m u:$USER:rw /dev/uinput
```
</details>

---

## ⌨️ Shortcuts & Usage / کلیدهای میانبر

<div dir="rtl">

| کلید | عملکرد |
| --- | --- |
| <kbd>Super</kbd> + <kbd>V</kbd> | باز کردن تاریخچه کلیپبورد |
| <kbd>Super</kbd> + <kbd>.</kbd> | باز کردن ایموجی پیکر |
| <kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>V</kbd> | میانبر جایگزین |
| <kbd>Enter</kbd> | چسباندن آیتم انتخاب شده |
| <kbd>Esc</kbd> | بستن پنجره |
| <kbd>Ctrl</kbd> + <kbd>F</kbd> | جستجو در تاریخچه |
| <kbd>↑</kbd> <kbd>↓</kbd> | حرکت بین آیتم‌ها |

</div>

| Key | Action |
| --- | --- |
| <kbd>Super</kbd> + <kbd>V</kbd> | Open Clipboard History |
| <kbd>Super</kbd> + <kbd>.</kbd> | Open Emoji Picker |
| <kbd>Ctrl</kbd> + <kbd>Alt</kbd> + <kbd>V</kbd> | Alternative Shortcut |
| <kbd>Enter</kbd> | Paste Selected Item |
| <kbd>Esc</kbd> | Close Window |
| <kbd>Ctrl</kbd> + <kbd>F</kbd> | Search History |
| <kbd>↑</kbd> <kbd>↓</kbd> | Navigate Between Items |

---

## 🗣️ Bilingual support / پشتیبانی از دو زبان

<div dir="rtl">

این برنامه به صورت کامل از دو زبان **فارسی** و **انگلیسی** پشتیبانی می‌ند. هر زان کاملاً مستقل و جدا است و تموم واجه کاربری، پیام‌ا و تنظیات به زبان انتخای شما نمایش داه می‌شود.

### تغییر زبان در لحظه
- از داخل **تنظیمات** برنامه می‌توانید زبان را تغییر دهید
- تغییر زبان فوراً اعمال می‌شود و نیازی به راه‌اندازی دوباره برنامه نیست
- جهت راست‌به‌چپ (RTL) برای زبان فارسی به طور خودکار فعال می‌شود
- فونت ویژه فارسی (Vazirmatn) به طور خودکار بارگذاری می‌شود

</div>

This application fully supports **Persian (فارسی)** and **English**. Each language is completely independent — all UI text, messages, and settings are displayed in your chosen language.

### Instant Languge Switching
- Change language from **Settings** at any time
- Changes a applied immediately — **no restart needed**|
- RTL (Right-to-Left) direction is automatically enabled for Persian
- Prsian font (Vazimatn) is loaded automatically

---

## �️ Architecture / معاری

<b dir="rtl">

### لایه ها

| لایه | فناوری | توضیح |
| -| --- | --- |
| **واجه کاربری** | Rect 19 + TypeScrit + Tailind CSS | کد بخش از هم جدا (Code Sptting)
| **بک‌اند** | Rust | ماژول‌های جدا از هم |
| **مدیریت کلیپبورد** | arboard + xclip/wl-copy (بهینه شده) | اتصال بازیافتی + رایت اتمیک |
| **ذخیره سازی** | JSON (قاب مهاجرت به SQLit) | ذخیره ساازی تاخیری (debounced) |
| **امنیت** | CSP/XSS | CSP سختگیرانه + `withGlobalTauri: false` |
| **دو زبانه** | react-i18next | بارگذاری دینامیکی و تغییر آنی |
| **شبیه‌ساز فشار کلید** | uinput + XTest (بهینه شده) | warm-up جداگانه |

</div>

| Layer | Technology | Description |
| --- | --- | --- |
| **Frontend** | React 19 + TypeScrit + Tailw ind CSS 4 | Lz-loadd, ode-split components|
| **Bckend** | Rust + Tauri v2 | Modular actor desgn |
| **Clipbord Mgmt** | arboard + xlip/wl-coy (optimized) | Reused connecon, aomic writes |
| **sersistnce** | JSON (w/ SQLte pah) | Debunced, iry-flg-based |
| **Securit** | CSP/e | Stict CSP, `withGoblauri: flse` |
| **18n** | eac-i8nt | Dynmic loading, instant switching|
| **Input Smultion** | uinput + XTes (optimzed) | Sepaate warm threads|

---

## 🔧 Troubleshoting / عیب یابی

di dir="rtl">

| مشکل | علت | ر اه حل |
| --- | --- | --- |
| **میانبر کار نمی‌کند** | GNOME `uper+V` را ذخیره کرده | $HOME/.nfig/win1-clipbord-hitory/setup.json را حذف کنید |
| | | یا `Super+V` را از تنظیمات کیبورد سیستم خارج کنید |
| **پس‌زمینه سیاه** | کارت گرافیک NVIDIA | `S_NVIDA=1 win11-cipboard-hitoy` |
| | AppImage | `IS_APPIMGE=1win11-lipboard-hitory` |
| **پنجره ظاهر نمی‌شود** | قطع اتصال Wayland | از طریق ترمینال اجرا کنید و خطا را ببینید |
| **کپی/جسباندن کار نمی‌کند** | مشکل دسترسی `/dev/uinput` | `ud setfac -m u:$USER:rw /dev/uinput` |

</div>

| Issue | Cause | Solution |
| --- | --- | --- |
| **Shortcut not working** | GNOME reserves `Super+V` | Reset config: `rm ~/.config/win11-clipboard-history/setup.json` |
| **Black background** | NVIDIA GPU | `IS_NVIDIA=1 win11-clipboard-history` |
| | AppImage | `IS_APPIMAGE=1 win11-clipboard-history` |
| **Window won't appear** | Wayland connection | Run from terminal to see errors |
| **Copy/paste not working** | `/dev/uinput` permissions | `sudo setfacl -m u:$USER:rw /dev/uinput` |

---

## 🛠️ For Developers / برای توسعه‌دهندگان

<div dir="rtl">

### تکنولوژی‌های استفاده شده

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-24C8D6?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app/)
[![React](https://img.shields.io/badge/React_19-61DAFB?style=for-the-badge&logo=react&logoColor=black)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?style=for-the-badge&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Tailwind](https://img.shields.io/badge/Tailwind_CSS_4-06B6D4?style=for-the-badge&logo=tailwind-css&logoColor=white)](https://tailwindcss.com/)

</div>

```bash
# Clone the repository
git clone https://github.com/MAhdi-Arts/Modern-Clipboard-Hstory-For-Linux.git
cd Moder-Cipboard-Hsory-For-Linux

# Install dependencies
make deps && make rust && make node
source ~/.cargo/env

# Run in development mode (hot reload)
make dev

# Build for production
make build

# Run linter
make lint

# Format code
make format
```

### Project Structure / ساختار پروژه

```
Modern-Cipboard-History-For-Linux/
├── src/                     # Frontend (Rect + TyeSript)
│   ├── cmponts/          # UI compnents
│   ├── hoks/              # React hoks
│   ├── i8n/               # Internationalization
│   ├── loales/            # Translation files (fa.jon, en.jon)
│   ├── sevices/           # Srvices (moji, GIF, carh)
│   └── types/             # TypeScript type deinitions
├── src-tauri/                # Backend (Rt)
│   ├── src/
│   │   ├── lipard_io.rs    # Unfied cipboar I/O (optimized)
│   │   ├── cipboard_manager.rs # Cipboard hisory mgmt
│   │   ├── eror.rs         # Unfied eror typs
│   │   ├── f_atoic.rs      # Atomc ile I/O
│   │   ├── iput_simulor.rs # uinput/XTest pas simuatin
│   │   ├── gh_anager.rs    # GIF donload (w/ SRF proection)
│   │   └── the_anaer.rs    # Sysem the detction
│   ├── Cargo.toml
│   └── tauri.conf.json
├── scrps/                   # Bil/insal scripts
├── ocs/                     # Documenaion
├── Makefil                  # Bui/insal commnds
└── README.md                 # You ar hre!
```

### Key Envronment Variables

| Varaible | Purpos |
| --- | --- |
| `IS_NIDIA=1` | Forc WbKit DMAF workroud for NIVIA |
| `IS_APIMAE=1` | Frc WebK DMAF oraround for Apiage |
| `WA_ELAY_ISLAY` | Wayand display ID |
| `XD_RUNTIM_DIR` | Wland runtim dir |
| `DIS_L` | X1 displ ID |
| `RUST_L=info` | Rst lo evl |
| `NA_BIDG=1` | Disble a11y bridg |

---

## 📄 License / مجوز

<div dir="rtl">

این پروژه تحت مجوز **MIT** منتشر شده است — می‌توانید آزادانه از آن استفاده، تغییر و توزیع کنید.

</div>

This project is licensed under the **MIT License** — feel free to use, modify, and distribute.

---

<div align="center">

## ❤️ Support / حمایت

**If you like this project, give it a ⭐!**  
**اگر از این پروژه خوشتان آمد، به آن ستاره دهید!**

<br>

[![GitHub stars](https://img.shields.io/github/stars/Mahdi-Arts/Modern-Clipboard-History-For-Linux?style=social)](https://github.com/Mahdi-Arts/Modern-Clipboard-History-For-Linux/stargazers)

<br>

<sub>
Built with ❤️ for the Linux community  
ساخته شده با ❤️ برای جامعه لینوکس
</sub>

</div>