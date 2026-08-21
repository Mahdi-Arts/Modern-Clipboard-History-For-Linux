# Archived workflow patches / پچ‌های آرشیوشدهٔ ورک‌فلو

> **Status:** every patch in this directory is historical/superseded — do
> not apply. The hardened, canonical-named pipelines are **active in-repo**
> (`.github/workflows/` is the single source of truth); the final activation
> patch (`hardened-ci-workflows.patch` below) was applied on 2026-08-21 and
> is kept here as the audit record of what was shipped.
>
> **وضعیت:** همهٔ پچ‌های این پوشه تاریخی/منسوخ‌اند — اعمال نکنید. خطوط
> لولهٔ hardened و نام‌گذاری‌شدهٔ رسمی **هم‌اکنون فعال‌اند** (`.github/workflows/`
> منبع حقیقت واحد است)؛ پچ فعال‌سازی نهایی (`hardened-ci-workflows.patch`
> در همین‌جا) در ۲۰۲۶-۰۸-۲۱ اعمال شد و به‌عنوان سند حسابرسیِ آنچه
> منتشر شده نگهداری می‌شود.

## Contents / محتوا

- `workflows-rename-*.patch`, `enterprise-workflow-upgrade.patch` — earlier,
  superseded iterations. **Do not apply.**
  تکرارهای پیشین و منسوخ. **اعمال نکنید.**
- `hardened-ci-workflows.patch` — the **applied** final iteration (2026-08-21):
  canonical `ci.yml` (rustup-only, packaging job incl. `flatpak-builder-lint`),
  `release.yml` (canonical artifact names/URLs/AUR, GPG/SBOM/SLSA), a new
  manual `e2e.yml`, and `actions/stale` pinned to a full commit SHA.
  Historical record — **do not apply**; `.github/workflows/` already contains
  its result.
  تکرار نهاییِ **اعمال‌شده** (۲۰۲۶-۰۸-۲۱): `ci.yml` رسمی (فقط rustup، job
  بسته‌بندی شامل `flatpak-builder-lint`)، `release.yml` (نام‌های رسمی
  آرتیفکت/URL/AUR، GPG/SBOM/SLSA)، `e2e.yml` دستی جدید و پین‌شدن
  `actions/stale` به SHA کامل. سند تاریخی — **اعمال نکنید**؛ نتیجه‌اش
  همین حالا در `.github/workflows/` است.
