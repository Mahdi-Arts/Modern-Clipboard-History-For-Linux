# ADR-0005: Blocking CI quality gates and signed releases

- **Status:** Accepted (v2.3.0)
- **Date:** 2026-08-20

## Context / زمینه

README and CHANGELOG previously claimed blocking `cargo audit` / `npm audit`,
published `SHA256SUMS`, SPDX SBOMs and SLSA provenance. The workflows still
had `continue-on-error: true` on audits, ran no tests, and the release notes
pointed at the upstream fork.

مستندات ادعا می‌کرد گیت‌های امنیتی الزامی‌اند، اما workflowها audit را
با `continue-on-error` اجرا می‌کردند، تست نداشتند و Release به فورک بالادستی
اشاره می‌کرد.

## Decision / تصمیم

1. CI job `quality` runs `npm run lint`, `npm run test:coverage`,
   `cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test`.
2. CI job `security` runs `cargo audit` and `npm audit --audit-level=high`
   **without** `continue-on-error`.
3. `build-linux` depends on both jobs and smokes `--version` / `--help`.
4. Tagged releases publish `SHA256SUMS`, an SPDX SBOM, and SLSA provenance
   (`actions/attest-build-provenance`). All URLs target
   `Mahdi-Arts/Modern-Clipboard-History-For-Linux`.

## Consequences / پیامدها

- A known high-severity advisory turns the PR red.
- The installer can require checksums because releases actually attach
  `SHA256SUMS`.
- Cloudsmith upload remains best-effort (`continue-on-error`) because it
  needs a secret that forks may not have.
