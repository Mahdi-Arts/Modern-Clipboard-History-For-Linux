# CI contract / قرارداد CI

> **English** below the Persian section. / بخش انگلیسی پایین‌تر است.

<div dir="rtl">

این سند قرارداد گیت‌های کیفیت است. نسخهٔ اجراییِ مورد نظر در
[`docs/github-workflows/`](github-workflows/) نگهداری می‌شود و فعال‌سازی آن
یک گام دستی است (GitHub App بدون مجوز `workflows` نمی‌تواند
`.github/workflows/` را پوش کند):

```bash
git am docs/patches/hardened-ci-workflows.patch && git push
```

همهٔ actionها به SHA کامل کامیت پین شده‌اند (توصیهٔ OpenSSF).

## گیت‌های مسدودکننده (`docs/github-workflows/ci.yml`)

| Job | بررسی | مسدودکننده؟ |
| --- | --- | --- |
| quality | `npm run lint` (tsc + ESLint، صفر هشدار) | بله |
| quality | `npm run test:coverage` (آستانه‌های Vitest) | بله |
| quality | `cargo fmt --all -- --check` | بله |
| quality | `cargo clippy --all-targets -- -D warnings` | بله |
| quality | `cargo test` (feature پیش‌فرض، بدون HTTP) | بله |
| quality | `cargo test --all-features` | بله |
| security | `cargo audit` | بله |
| security | `cargo deny check advisories bans licenses sources` | بله |
| security | `npm audit --audit-level=high` | بله |
| build-linux | بیلد Tauri + `--version` / `--help` روی باینری (با و بدون xvfb) | بله |

`continue-on-error` روی هیچ گیت امنیتی نیست.

## انتشار (`docs/github-workflows/release.yml`)

با تگ `v*` ساخته می‌شود:

- `.deb` / `.rpm` / AppImage برای x86_64 و aarch64
- `SHA256SUMS` (+ امضای اختیاری GPG با secret ی `RELEASE_GPG_PRIVATE_KEY`)
- SPDX SBOM به ازای هر آرتیفکت (`syft`)
- گواهی SLSA build-provenance
- همهٔ URLها به `Mahdi-Arts/Modern-Clipboard-History-For-Linux`

Cloudsmith و AUR فقط وقتی secretهای مخزن تنظیم شده باشند فعال می‌شوند.
اتصال AUR با `StrictHostKeyChecking yes` و `known_hosts` پین‌شده از secret ی
`AUR_KNOWN_HOSTS` انجام می‌شود — هرگز trust-on-first-use.

</div>

---

This document is the quality-gate contract. The intended blocking
workflows live in [`docs/github-workflows/`](github-workflows/) with a
ready-to-apply patch (`.github/workflows/*` cannot be pushed by GitHub Apps
without the `workflows` permission):

```bash
git am docs/patches/hardened-ci-workflows.patch && git push
```

All actions are pinned to full commit SHAs (OpenSSF recommendation).

## Blocking gates (`docs/github-workflows/ci.yml`)

Every row in the table above is a hard failure. Audits do **not** use
`continue-on-error`. Default-feature `cargo test` proves the release
binary compiles **without** our optional `reqwest` / GIF search feature.

## Releases (`docs/github-workflows/release.yml`)

Tag `v*` publishes checksums (plus an optional GPG `SHA256SUMS.sig` driven
by `RELEASE_GPG_PRIVATE_KEY`), per-artifact SPDX SBOMs, and SLSA
attestations. Optional channels (Cloudsmith, AUR) require repository
secrets; they never silently point at a third-party fork. The AUR SSH
connection is fail-closed: `StrictHostKeyChecking yes` with `known_hosts`
pinned via the `AUR_KNOWN_HOSTS` secret — trust-on-first-use is never
accepted.
