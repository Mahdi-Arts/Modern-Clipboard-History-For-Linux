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

Manifest: `packaging/flatpak/io.github.mahdi-arts.clipboard-history.yml`

The sandbox **does not** request `--device=all` (Flathub policy). Clipboard
history and UI work; simulated Ctrl+V needs a native `.deb`/`.rpm` or:

```bash
flatpak override --user --device=all io.github.mahdi-arts.clipboard-history
```

GIF search additionally needs `--share=network`.

## Checksums

GitHub Releases include `SHA256SUMS`. `scripts/install.sh` verifies downloads
when that file is present.

## AppArmor (optional hardening)

A profile is shipped with the deb under
`/usr/share/doc/win11-clipboard-history/apparmor/win11-clipboard-history`
and lives in the repo at `packaging/apparmor/`. It is installed in
**complain mode** by default (logs only, never blocks). To enforce:

```bash
sudo /usr/share/doc/win11-clipboard-history/apparmor/install.sh --enforce
```

Test it on your desktop environment first — see the header of the profile
for the access list (XDG dirs, /dev/uinput, helper binaries, display sockets).
