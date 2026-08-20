# =============================================================================
# Modern Clipboard History for Linux — Docker Build Environment
# =============================================================================
# Reproducible environment for CI builds and tests.
# Not for distribution — use native DEB/RPM/AppImage for that.
#
#   docker build -t clipboard-history:latest .
#   docker run --rm clipboard-history:latest
# =============================================================================

FROM ghcr.io/tauri-apps/tauri:ubuntu-24.04

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    nodejs \
    npm \
    xclip \
    wl-clipboard \
    && rm -rf /var/lib/apt/lists/*

COPY package.json package-lock.json ./
RUN npm ci --no-audit --no-fund

COPY . .

RUN npm run lint
RUN npm test
RUN npm run build

CMD ["cargo", "test", "--manifest-path", "src-tauri/Cargo.toml", "--all-features"]
