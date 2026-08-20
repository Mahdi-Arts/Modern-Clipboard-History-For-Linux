# CI / release workflows — intended source of truth
# گردش‌کارهای CI و انتشار — منبع حقیقتِ مورد نظر

> **Status (2026-08-21):** the files in this directory are the **intended
> blocking pipelines** (SHA-pinned actions, blocking audits, AUR
> fail-closed, optional GPG signing). GitHub Apps without the `workflows`
> permission cannot push `.github/workflows/*`, so activation is a single
> maintainer step — apply the pre-built patch:
> **وضعیت (۲۰۲۶-۰۸-۲۱):** فایل‌های این پوشه، خطوط لولهٔ **مسدودکنندهٔ
> مورد نظر** هستند (پین SHA، گیت‌های امنیتی مسدودکننده، AUR fail-closed و
> امضای GPG اختیاری). GitHub App بدون مجوز `workflows` نمی‌تواند
> `.github/workflows/*` پوش کند؛ بنابراین فعال‌سازی یک گام دستی است —
> اعمال پچ آماده:

```bash
git am docs/patches/hardened-ci-workflows.patch
git push   # needs a classic PAT or a direct maintainer push
```

After activation, keep the two copies in sync:
پس از فعال‌سازی، دو نسخه را همگام نگه دارید:

```bash
cp .github/workflows/ci.yml docs/github-workflows/ci.yml
cp .github/workflows/release.yml docs/github-workflows/release.yml
git add docs/github-workflows .github/workflows
```

## What is enforced / چه چیزهایی الزامی است

- **CI (`.github/workflows/ci.yml`)** — every gate blocks:
  - ESLint (type-aware) + `tsc` with zero warnings
  - Frontend tests with coverage thresholds (`npm run test:coverage`)
  - Fast Rust syntax gate (`node scripts/check-rust-syntax.mjs`)
  - `cargo fmt --check`, `clippy -D warnings`, `cargo test` + `--all-features`
  - `cargo audit`, `cargo deny check advisories bans licenses sources`,
    `npm audit --audit-level=high` — no `continue-on-error`
  - Release build + CLI smoke (`--version` / `--help`, bare and `xvfb-run`)

- **Release (`.github/workflows/release.yml`)** — runs on `v*` tags:
  - Multi-arch builds (x86_64 + aarch64) → `.deb` / `.rpm` / AppImage
  - `SHA256SUMS` + optional GPG `SHA256SUMS.sig` (`RELEASE_GPG_PRIVATE_KEY` secret)
  - Per-artifact SPDX SBOM (`syft`) + SLSA build-provenance attestations
  - Optional Cloudsmith upload (`CLOUDSMITH_API_KEY`) and AUR update
    (`AUR_SSH_KEY` + `AUR_KNOWN_HOSTS`) — both fail closed when unset.
    The AUR SSH connection uses `StrictHostKeyChecking yes` with pinned
    `known_hosts` from the secret; trust-on-first-use is never allowed.

- **Supply chain:** every `uses:` is pinned to a full commit SHA (OpenSSF
  recommendation). Tags are kept in `# comments` for review only.
- **زنجیرهٔ تأمین:** هر `uses:` به SHA کامل کامیت پین شده است (توصیهٔ
  OpenSSF). تگ‌ها فقط در کامنت برای بازبینی مانده‌اند.

See the contract at [`docs/CI.md`](../CI.md).
قرارداد کامل در [`docs/CI.md`](../CI.md).
