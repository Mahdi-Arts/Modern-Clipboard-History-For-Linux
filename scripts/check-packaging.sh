#!/usr/bin/env bash
# Validate release-package contracts without building a GUI bundle.
# اعتبارسنجی قراردادهای بستهٔ انتشار بدون ساخت bundle گرافیکی.
set -euo pipefail

readonly ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

readonly BINARY="win11-clipboard-history-bin"
readonly WRAPPER="win11-clipboard-history"
readonly APP_ID="io.github.mahdi-arts.clipboard-history"

fail() {
    printf 'packaging check failed: %s\n' "$*" >&2
    exit 1
}

[[ "$(sed -n '/^\[\[bin\]\]/,/^$/s/^name = "\([^"]*\)"/\1/p' src-tauri/Cargo.toml)" == "$BINARY" ]] \
    || fail "Cargo binary must be $BINARY"

grep -Fq "src-tauri/target/release/$BINARY" packaging/debian/rules \
    || fail "Debian rules do not install the canonical binary"
grep -Fq "src-tauri/target/release/$BINARY" packaging/flatpak/$APP_ID.yml \
    || fail "Flatpak manifest does not install the canonical binary"
grep -Fq "/usr/lib/$WRAPPER/$BINARY" src-tauri/bundle/linux/wrapper.sh \
    || fail "wrapper does not search the Debian/RPM library path"
grep -Fq "/app/lib/$WRAPPER/$BINARY" src-tauri/bundle/linux/wrapper.sh \
    || fail "wrapper does not search the Flatpak library path"
grep -Fq 'Icon=io.github.mahdi-arts.clipboard-history' \
    src-tauri/bundle/linux/modern-clipboard-history-for-linux.desktop \
    || fail "desktop entry and installed icon ID have drifted"
grep -Fq 'modern-clipboard-history-for-linux.desktop' Makefile \
    || fail "Makefile references a missing desktop source"

for required in \
    src-tauri/bundle/linux/wrapper.sh \
    src-tauri/bundle/linux/modern-clipboard-history-for-linux.desktop \
    src-tauri/bundle/linux/99-modern-clipboard-history-input.rules \
    packaging/flatpak/$APP_ID.yml \
    packaging/flatpak/$APP_ID.metainfo.xml \
    packaging/debian/control \
    packaging/debian/rules; do
    [[ -f "$required" ]] || fail "missing $required"
done

if command -v desktop-file-validate >/dev/null 2>&1; then
    desktop-file-validate src-tauri/bundle/linux/modern-clipboard-history-for-linux.desktop
fi
if command -v appstreamcli >/dev/null 2>&1; then
    appstreamcli validate --no-net packaging/flatpak/$APP_ID.metainfo.xml
fi

printf 'Packaging contracts are consistent. / قراردادهای بسته‌بندی سازگارند.\n'
