#!/bin/bash
# Build and install the Flatpak from the official manifest.
# ساخت و نصب فلت‌پک از مانیفست رسمی.
#
# Usage / استفاده:
#   ./packaging/flatpak/build.sh            # build + install to the user
#   FLATPAK_INSTALL=0 ./packaging/flatpak/build.sh   # build only / فقط ساخت
#
# Requirements / پیش‌نیازها (installed once / نصب یک‌باره):
#   flatpak install --user flathub org.gnome.Sdk//46 org.gnome.Platform//46 \
#     org.freedesktop.Sdk.Extension.rust-stable org.freedesktop.Sdk.Extension.node20

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MANIFEST="$REPO_ROOT/packaging/flatpak/io.github.mahdi-arts.clipboard-history.yml"
BUILD_DIR="$REPO_ROOT/.flatpak-builder"
APP_ID="io.github.mahdi-arts.clipboard-history"

if ! command -v flatpak-builder >/dev/null 2>&1; then
  echo "error: flatpak-builder not found. Install flatpak + flatpak-builder first." >&2
  exit 1
fi

cd "$REPO_ROOT"

echo "[*] Building $APP_ID ..."
flatpak-builder --force-clean --user --install-deps-from=flathub \
  "$BUILD_DIR" "$MANIFEST"

if [ "${FLATPAK_INSTALL:-1}" = "1" ]; then
  echo "[*] Installing $APP_ID for the current user ..."
  flatpak-builder --user --install "$BUILD_DIR" "$MANIFEST"
  echo "[✓] Installed. Launch with:"
  echo "    flatpak run $APP_ID"
  echo "    # paste simulation override (optional):"
  echo "    flatpak override --user --device=all $APP_ID"
fi
