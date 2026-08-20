# CI contract / قرارداد CI

> **English** below the Persian section. / بخش انگلیسی پایین‌تر است.

<div dir="rtl">

این سند قرارداد گیت‌های کیفیت است. منبع گردش‌کارهای سخت‌شده
[`docs/github-workflows/`](github-workflows/) است. فعال‌سازی روی
`.github/workflows/` به توکن با مجوز `workflows` نیاز دارد
([راهنما](github-workflows/README.md)).

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
- `SHA256SUMS`
- SPDX SBOM به ازای هر آرتیفکت (`syft`)
- گواهی SLSA build-provenance
- همهٔ URLها به `Mahdi-Arts/Modern-Clipboard-History-For-Linux`

Cloudsmith و AUR فقط وقتی secret مخزن تنظیم شده باشد فعال می‌شوند.

</div>

---

This document is the quality-gate contract. The hardened workflow **source**
is [`docs/github-workflows/`](github-workflows/). Copying it onto
`.github/workflows/` requires a token with the `workflows` scope
([instructions](github-workflows/README.md)).

## Blocking gates (`docs/github-workflows/ci.yml`)

Every row in the table above is a hard failure. Audits do **not** use
`continue-on-error`. Default-feature `cargo test` proves the release
binary compiles **without** our optional `reqwest` / GIF search feature.

## Releases (`docs/github-workflows/release.yml`)

Tag `v*` publishes checksums, per-artifact SPDX SBOMs, and SLSA
attestations. Optional channels (Cloudsmith, AUR) require repository
secrets; they never silently point at a third-party fork.
