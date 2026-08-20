# 📦 Packaging & Deployment Guide / راهنمای بسته‌بندی و استقرار

<div dir="rtl">

## نمای کلی

این سند راهنمای کامل ساخت، امضا و انتشار برنامه در دو کانال اصلی است:
**۱) بستهٔ `.deb` برای GitHub Release** و **۲) بستهٔ Flatpak**.

</div>

This document is the complete guide for building, verifying, and shipping the
app through its two main channels: **1) the `.deb` package published on
GitHub Releases**, and **2) Flatpak**.

---

## 1. Packaging layout / ساختار دایرکتوری بسته‌بندی

```
packaging/
├── README.md                  ← This guide / همین راهنما
├── apparmor/                  ← Optional AppArmor hardening / سخت‌سازی اختیاری
│   ├── win11-clipboard-history
│   └── install.sh             (--enforce to switch from complain mode)
├── debian/                    ← Classic Debian source package / بستهٔ سورس دبیان
│   ├── control                (deps, description)
│   ├── rules                  (dh build steps)
│   ├── changelog              (dpkg changelog — bump version here)
│   ├── copyright              (MIT + upstream attribution)
│   ├── postinst / postrm      (uinput module, udev reload, caches)
│   └── source/format          (3.0 quilt)
├── flatpak/                   ← Flathub-style manifest / مانیفست Flatpak
│   ├── io.github.mahdi-arts.clipboard-history.yml        (build manifest)
│   └── io.github.mahdi-arts.clipboard-history.metainfo.xml (AppStream releases)
└── (repo root)
    ├── aur/PKGBUILD           ← AUR `-bin` package (auto-updated by CI)
    ├── src-tauri/bundle/linux/
    │   ├── 99-win11-clipboard-input.rules   (udev: uaccess, NOT 0666)
    │   ├── postinst.sh / postrm.sh          (Tauri deb/rpm hooks)
    │   ├── wrapper.sh                       (clean env, NVIDIA workaround)
    │   └── win11-clipboard-history.desktop
    ├── scripts/install.sh     ← Checksum-verifying convenience installer
    └── scripts/release.sh     ← Maintainer release helper
```

<div dir="rtl">هر کانال از **همان** باینری و فایل‌های دسکتاپ استفاده می‌کند؛ فقط لایهٔ بسته‌بندی متفاوت است.</div>

Every channel uses the **same** binary and desktop files; only the packaging
layer differs.

---

## 2. Channel 1 — `.deb` for GitHub Release / کانال ۱ — بستهٔ `.deb`

<div dir="rtl">کانال توصیه‌شده: دسترسی کامل به قابلیت paste (شبیه‌سازی Ctrl+V).</div>

Preferred channel: full paste capability (Ctrl+V simulation).

### 2.1 Build / ساخت

```bash
# a) Tauri bundle (used by CI) / باندل Tauri (همان مسیر CI)
make deps && npm ci
npm run tauri:build
# → src-tauri/target/release/bundle/deb/win11-clipboard-history_<ver>_amd64.deb
# → src-tauri/target/release/bundle/rpm/*.rpm
# → src-tauri/target/release/bundle/appimage/*.AppImage

# b) Classic Debian source package / بستهٔ سورس کلاسیک دبیان
cp -a packaging/debian debian
dpkg-buildpackage -us -uc
```

### 2.2 Post-install (paste permission) / پس از نصب

```bash
sudo apt install ./win11-clipboard-history_<ver>_amd64.deb
sudo setfacl -m u:$USER:rw /dev/uinput   # paste injection access
```

<div dir="rtl">قانون udev نصب‌شده از `TAG+="uaccess"` استفاده می‌کند (فقط کاربر نشست فعال)، نه `0666`.</div>

The shipped udev rule uses `TAG+="uaccess"` (active-session user only), never `0666`.

### 2.3 Release with verification / انتشار با راستی‌آزمایی

Tagging `vX.Y.Z` triggers `.github/workflows/release.yml`, which:

| Step / گام | What it does / کاری که انجام می‌دهد |
| --- | --- |
| Build matrix | x86_64 + aarch64 `.deb` / `.rpm` / AppImage |
| `SHA256SUMS` | checksums of every artifact, published as a release asset |
| SPDX SBOM | `syft` scan **per artifact** → `<file>.spdx.json` |
| SLSA provenance | `actions/attest-build-provenance` attestations |
| AUR (optional) | PKGBUILD checksums updated when `AUR_SSH_KEY` is set |
| Cloudsmith (optional) | repo upload when `CLOUDSMITH_API_KEY` is set |

<div dir="rtl">نصاب `scripts/install.sh` تطبیق checksum با `SHA256SUMS` را **اجباری** می‌داند (`ALLOW_UNVERIFIED=1` فقط برای موارد استثنایی).</div>

`scripts/install.sh` **requires** a matching `SHA256SUMS` entry
(`ALLOW_UNVERIFIED=1` skips — not recommended).

---

## 3. Channel 2 — Flatpak / کانال ۲ — فلت‌پک

### 3.1 Local build / ساخت محلی

```bash
flatpak install --user flathub org.gnome.Sdk//46 org.gnome.Platform//46 \
  org.freedesktop.Sdk.Extension.rust-stable org.freedesktop.Sdk.Extension.node20
flatpak-builder --user --install --force-clean \
  build-dir packaging/flatpak/io.github.mahdi-arts.clipboard-history.yml
```

### 3.2 Sandbox policy / سیاست سندباکس

| Permission | Granted? | Why / چرا |
| --- | --- | --- |
| `--socket=wayland` + `--socket=fallback-x11` | ✅ | windowing / نمایش پنجره |
| `--share=ipc` | ✅ | X11 shared memory / حافظهٔ مشترک X11 |
| Portals (`Settings`, `Desktop`) | ✅ | theme detection / تشخیص تم |
| StatusNotifierWatcher | ✅ | system tray / تِرِی سیستم |
| XDG data/config dirs (create) | ✅ | history DB + settings / دیتابیس و تنظیمات |
| `--device=all` (`/dev/uinput`) | ❌ default | Flathub policy; paste needs the override below |
| `--share=network` | ❌ default | optional GIF search only / فقط جستجوی اختیاری GIF |

```bash
# Enable paste simulation / فعال‌سازی شبیه‌سازی paste
flatpak override --user --device=all io.github.mahdi-arts.clipboard-history
# Optional GIF search / جستجوی GIF اختیاری
flatpak override --user --share=network io.github.mahdi-arts.clipboard-history
```

### 3.3 Flathub submission / ارسال به Flathub

<div dir="rtl">برای انتشار عمومی در Flathub: یک pull request به مخزن `flathub/io.github.mahdi-arts.clipboard-history` با همین مانیفست باز کنید. پیش‌نیازها: رکورد `<releases>` در metainfo به‌روز باشد (نسخه + تاریخ)، اسکرین‌شات‌ها و metadata کامل.</div>

For a public Flathub listing, open a PR to
`flathub/io.github.mahdi-arts.clipboard-history` with this manifest. The
metainfo `<releases>` entry must be bumped for every version, and screenshots
must be attached.

### 3.4 Flatpak limitations / محدودیت‌های فلت‌پک

<div dir="rtl">۱) paste تا قبل از `--device=all` غیرفعال است. ۲) کلیدهای سراسری (Super+V) از داخل سندباکس قابل ثبت نیستند — میانبر جایگزین Ctrl+Alt+V یا نسخهٔ بومی. ۳) تنظیمات udev نصب نمی‌شود.</div>

1. Paste is disabled until `--device=all` is granted.
2. Global shortcuts (Super+V) cannot be registered from inside the sandbox —
   use Ctrl+Alt+V or the native packages.
3. udev rules are not installed (native channels only).

---

## 4. Optional hardening / سخت‌سازی اختیاری

```bash
# AppArmor (complain mode by default; test before enforcing)
sudo /usr/share/doc/win11-clipboard-history/apparmor/install.sh --enforce
```

---

## 5. Deployment checklist / چک‌لیست استقرار

- [ ] Version bumped in `package.json`, `src-tauri/tauri.conf.json`,
      `src-tauri/Cargo.toml`, `packaging/debian/changelog`, and the Flatpak
      metainfo `<releases>` (the release workflow syncs the first three from
      the git tag automatically).
- [ ] `make lint && make test` green locally / سبز بودن گیت‌ها به‌صورت محلی
- [ ] CI green on the release commit / سبز بودن CI
- [ ] Tag pushed → release assets verified (`sha256sum -c SHA256SUMS`)
- [ ] AUR/Cloudsmith updated (if secrets configured)
- [ ] CHANGELOG entry + metainfo `<releases>` entry added
