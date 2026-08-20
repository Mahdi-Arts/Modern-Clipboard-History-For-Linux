# =============================================================================
# Modern Clipboard History for Linux — Docker Build Environment
# =============================================================================
# This provides a reproducible environment for CI builds and testing.
# It is NOT for distribution — use native DEB/RPM/AppImage for that.
#
# BUILD:
#   docker build -t clipboard-history:latest .
#
# RUN TESTS:
#   docker run --rm clipboard-history:latest cargo test --manifest-path src-tauri/Cargo.toml
#
# RUN LINT:
#   docker run --rm clipboard-history:latest bash -c "cd src-tauri && cargo clippy"
# =============================================================================

FROM ghcr.io/tauri-apps/tauri:ubuntu-24.04

WORKDIR /app

# Install Node.js + build essentials
RUN apt-get update && apt-get install -y --no-install-recommends \
    nodejs \
    npm \
    xclip \
    wl-clipboard \
    && rm -rf /var/lib/apt/lists/*

# Cache npm dependencies (before source for layer caching)
COPY package.json package-lock.json ./
RUN npm ci --no-audit --no-fund 2>/dev/null || true

# Copy source
COPY . .

# Build frontend (required for Tauri build, even in test mode)
RUN npm run build 2>/dev/null || true

# Default: run Rust tests
CMD ["cargo", "test", "--manifest-path", "src-tauri/Cargo.toml", "--all-features"]