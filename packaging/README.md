# Linux packaging

## Debian / Ubuntu (`.deb`)

Preferred path for GitHub Releases is the Tauri bundle:

```bash
make deps
npm ci
npm run tauri:build
# artifacts:
#   src-tauri/target/release/bundle/deb/*.deb
#   src-tauri/target/release/bundle/rpm/*.rpm
#   src-tauri/target/release/bundle/appimage/*.AppImage
```

This repository also ships a classic Debian source layout under `packaging/debian/`
(`control`, `rules`, `changelog`, `copyright`, `postinst`, `postrm`) for
`dpkg-buildpackage` / Salsa-style builds.

```bash
# from the repository root, after copying debian/ into place:
cp -a packaging/debian debian
dpkg-buildpackage -us -uc
```

Post-install still needs `/dev/uinput` access for paste:

```bash
sudo setfacl -m u:$USER:rw /dev/uinput
```

## Flatpak

Manifest: `packaging/flatpak/dev.gustavosett.ClipboardHistory.yml`

The sandbox **does not** request `--device=all` (Flathub policy). Clipboard
history and UI work; simulated Ctrl+V needs a native `.deb`/`.rpm` or:

```bash
flatpak override --user --device=all dev.gustavosett.ClipboardHistory
```

GIF search additionally needs `--share=network`.

## Checksums

GitHub Releases include `SHA256SUMS`. `scripts/install.sh` verifies downloads
when that file is present.
