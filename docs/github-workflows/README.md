# Hardened CI / release workflows
# گردش‌کارهای سخت‌شدهٔ CI و انتشار

GitHub Apps without the `workflows` permission cannot push
`.github/workflows/*.yml`. The files in this directory are the intended
blocking pipelines (audits, coverage, cargo test, SHA256SUMS, SPDX, SLSA).

برنامهٔ GitHub بدون مجوز `workflows` نمی‌تواند فایل‌های
`.github/workflows` را پوش کند. این پوشه منبع گردش‌کارهای مورد نظر است.

Apply after merge (needs a token with the `workflows` scope):

```bash
cp docs/github-workflows/ci.yml .github/workflows/ci.yml
cp docs/github-workflows/release.yml .github/workflows/release.yml
git add .github/workflows
git commit -m "ci: activate hardened CI and release workflows"
git push
```
