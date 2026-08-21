# Deployment Guide — .deb → GitHub Release → Flatpak
# راهنمای استقرار — بستهٔ `.deb` → انتشار گیت‌هاب → Flatpak

This document is the **operational** companion to [`packaging/README.md`](README.md).
It describes how the project is packaged and published end-to-end, plus the
Enterprise-critical concerns (key backup/recovery, signature verification,
AppArmor) that must be understood before a release.

این سند، همراه **عملیاتی** [`packaging/README.md`](README.md) است و چگونگی
بسته‌بندی و انتشار سراسری پروژه، به‌همراه نگرانی‌های حیاتی سطح Enterprise
(پشتیبان‌گیری/بازیابی کلید، راستی‌آزمایی امضا، AppArmor) را شرح می‌دهد.

---

## ۱. هدف از این سند / Purpose

The project ships to Linux users through three primary channels:

1. **`.deb` (Debian/Ubuntu)** — the **primary** distribution artifact, published
   to each GitHub Release and consumed by `install.sh`, `.deb` installs, and the
   AUR (which repackages the `.deb`).
2. **`.rpm` (Fedora/RHEL)** and **AppImage** — secondary artifacts in the same
   release.
3. **Flatpak** — a sandboxed channel built from the same source via
   `packaging/flatpak/build.sh`.

پروژه از سه کانال اصلی به کاربران لینوکس می‌رسد:

1. **`.deb` (دبیان/اوبونتو)** — مهم‌ترین artifact، که در هر GitHub Release منتشر و
   توسط `install.sh`، نصب `.deb` و AUR مصرف می‌شود.
2. **`.rpm` (فدورا/RHEL)** و **AppImage** — artifactهای ثانویه در همان release.
3. **Flatpak** — کانال sandbox که از همان سورس با `packaging/flatpak/build.sh` ساخته می‌شود.

---

## ۲. ساختار دایرکتوری بسته‌بندی / Packaging layout

```
packaging/
├── README.md                 # Canonical binary & build instructions (دوزبانه)
├── DEPLOYMENT.md             # ← این سند / this document
├── apparmor/
│   ├── modern-clipboard-history-for-linux   # AppArmor profile (complain default)
│   └── install.sh            # installs profile; `--enforce` to harden
├── debian/
│   ├── control               # package metadata (Source/Package/Depends/Description)
│   ├── rules                 # dh_* build/install rules
│   ├── postinst / postrm     # post-install / post-remove hooks (udev, uinput)
│   ├── changelog
│   ├── copyright
│   └── source/format
└── flatpak/
    ├── build.sh
    ├── io.github.mahdi-arts.clipboard-history.yml
    └── io.github.mahdi-arts.clipboard-history.metainfo.xml
```

Key installation paths produced by the `.deb` (see `debian/rules`):

| Path | Purpose |
| --- | --- |
| `/usr/bin/modern-clipboard-history-for-linux` | launcher wrapper |
| `/usr/lib/modern-clipboard-history-for-linux/…-bin` | host-linked binary |
| `/etc/udev/rules.d/99-modern-clipboard-history-input.rules` | uinput permissions |
| `/etc/modules-load.d/modern-clipboard-history-for-linux.conf` | load `uinput` module |
| `/usr/share/applications/io.github.mahdi-arts.clipboard-history.desktop` | menu entry |
| `/usr/share/doc/modern-clipboard-history-for-linux/apparmor/…` | AppArmor profile + installer |

مسیرهای کلیدی تولیدشده توسط `.deb` (نگاه کنید به `debian/rules`):

| مسیر | کاربرد |
| --- | --- |
| `/usr/bin/modern-clipboard-history-for-linux` | راه‌انداز wrapper |
| `/usr/lib/modern-clipboard-history-for-linux/…-bin` | باینری متصل به میزبان |
| `/etc/udev/rules.d/99-modern-clipboard-history-input.rules` | مجوزهای uinput |
| `/etc/modules-load.d/modern-clipboard-history-for-linux.conf` | بارگذاری ماژول `uinput` |
| `/usr/share/applications/io.github.mahdi-arts.clipboard-history.desktop` | ورودی منو |
| `/usr/share/doc/modern-clipboard-history-for-linux/apparmor/…` | پروفایل AppArmor + نصب‌کننده |

---

## ۳. زنجیرهٔ انتشار / Release pipeline

The release is produced by `.github/workflows/release.yml`:

1. **Version bump** — reads the tag and writes `version` into
   `src-tauri/tauri.conf.json` + `src-tauri/Cargo.toml`.
2. **Build** — `npm run tauri:build` produces `.deb`, `.rpm`, and AppImage.
3. **Supply-chain** — publishes `SHA256SUMS` (+ optional GPG `SHA256SUMS.sig`),
   per-artifact SPDX SBOM (syft), and SLSA provenance (attest-build-provenance).
4. **Optional channels** — Cloudsmith and AUR are opt-in via secrets and
   **fail closed** on real errors.

انتشار توسط `.github/workflows/release.yml` تولید می‌شود:

1. **افزایش نسخه** — نسخه از برچسب خوانده و در `tauri.conf.json` + `Cargo.toml` نوشته می‌شود.
2. **ساخت** — `npm run tauri:build` خروجی `.deb`، `.rpm` و AppImage می‌سازد.
3. **زنجیرهٔ تأمین** — `SHA256SUMS` (+ امضای GPG اختیاری `SHA256SUMS.sig`)،
   SBOM SPDX و provenance نوع SLSA منتشر می‌شود.
4. **کانال‌های اختیاری** — Cloudsmith و AUR اختیاری‌اند و روی خطای واقعی **fail-closed** می‌شوند.

---

## ۴. داده‌ها و کلید رمزنگاری (بازیابی و پشتیبان‌گیری) / Data & encryption key

> **Enterprise-critical.** The clipboard history is encrypted at rest with a key
> that lives either in a **key file** (`~/.local/share/modern-clipboard-history-for-linux/history.key`)
> or in the **Secret Service keyring**. **Losing the key means losing the
> history** — there is no backdoor and no master password reset.
>
> **حیاتی.** تاریخچهٔ کلیپ‌بورد با کلیدی رمز می‌شود که یا در **فایل کلید**
> (`~/.local/share/modern-clipboard-history-for-linux/history.key`) یا در
> **کلید-ring دسکتاپ** است. **از دست دادن کلید یعنی از دست دادن تاریخچه**؛ هیچ
> درِ پشتی و هیچ ریست master password وجود ندارد.

**Backup strategy / راهبرد پشتیبان‌گیری:**

- **File backend:** back up `history.db`, `history.db-wal`, `history.db-shm`,
  `history.key`, and `history.key.check` **together** (they must stay in sync).
  Back up فایل کلید: `history.db`، `history.db-wal`، `history.db-shm`،
  `history.key` و `history.key.check` را **با هم** نگه دارید (باید هم‌اهنگ بمانند).
- **Secret Service backend:** the key is in GNOME Keyring / KWallet. Back up
  the keyring, or **export** the key and switch to the file backend via
  Settings → Privacy before losing the keyring.
  بک‌اند Secret Service: کلید در GNOME Keyring / KWallet است. از کلید-ring
  پشتیبان بگیرید، یا پیش از از دست دادن آن، کلید را **خروجی** گرفته و از
  Settings → Privacy به بک‌اند فایل سوئیچ کنید.
- **Recommendation / توصیه:** prefer **Secret Service** on supported desktops;
  it avoids an on-disk key. Prefer دریافت کلید از Secret Service در
  دسکتاپ‌های پشتیبانی‌شده؛ چون کلید روی دیسک نمی‌ماند.

**Quarantine / قرنطینه:** undecryptable history rows are recorded in
`~/.local/share/modern-clipboard-history-for-linux/quarantine.log` (ids + reasons)
instead of being silently dropped. Rows that fail to decrypt are **never**
surfaced as partial items.

ردیف‌های قابل‌رمزگشایی‌نشدن در `quarantine.log` ثبت می‌شوند (شناسه‌ها + دلایل)
به‌جای حذف بی‌صدا. ردیف‌های رمزگشایی‌نشده هرگز به‌صورت آیتم ناقص نمایش داده نمی‌شوند.

---

## ۵. AppArmor / محدودسازی

The shipped AppArmor profile (`packaging/apparmor/…`) installs in **complain**
mode by default (logs violations, blocks nothing). To harden:

```bash
sudo ./packaging/apparmor/install.sh --enforce
sudo aa-status | grep modern   # confirm it is loaded
```

Because `/dev/uinput` is a powerful capability, **enforce mode should be
validated** on your desktop before rolling it out widely.

پروفایل AppArmor به‌صورت پیش‌فرض در حالت **complain** نصب می‌شود (فقط ثبت، بدون
مسدودکردن). برای سخت‌کردن:

```bash
sudo ./packaging/apparmor/install.sh --enforce
sudo aa-status | grep modern   # تأیید بارگذاری
```

چون `/dev/uinput` قابلیت قدرتمندی است، **پیش از گسترش، حالت enforce باید** روی
دسکتاپ خودتان اعتبارسنجی شود.

---

## ۶. تست نصب بسته / Package install smoke test

After building, verify the `.deb` installs and runs (see `scripts/verify-deb.sh`):

```bash
sudo apt install ./modern-clipboard-history-for-linux_2.5.0_amd64.deb
command -v modern-clipboard-history-for-linux          # launcher on PATH
/usr/lib/modern-clipboard-history-for-linux/modern-clipboard-history-for-linux-bin --version
```

And the Flatpak build:

```bash
bash packaging/flatpak/build.sh
```

---

## ۷. فهرست وظایف انتشار / Release checklist

- [ ] `git tag vX.Y.Z` and push (release workflow runs).
- [ ] `.deb`, `.rpm`, AppImage uploaded + `SHA256SUMS` (+ `.sig`).
- [ ] SBOM + SLSA provenance present on the release.
- [ ] AUR/Cloudsmith channels updated (or confirmed skipped safely).
- [ ] CHANGELOG updated in both languages.

- [ ] برچسب `vX.Y.Z` و push (workflow انتشار اجرا شود).
- [ ] بارگذاری `.deb`، `.rpm`، AppImage + `SHA256SUMS` (+ `.sig`).
- [ ] وجود SBOM و provenance نوع SLSA در release.
- [ ] به‌روزرسانی کانال‌های AUR/Cloudsmith (یا skip امن تأیید شود).
- [ ] به‌روزرسانی CHANGELOG به هر دو زبان.

**یا علی مدد.**
