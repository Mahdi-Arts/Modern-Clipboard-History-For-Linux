# Status — 1.1.0

The 2026-08-20 optimization audit was implemented in **v1.1.0**. See `CHANGELOG.md` for the full list.

Still intentionally deferred (not 1.0 blockers):

- Full event-driven X11 clipboard via XFixes (adaptive 200/800ms polling is in place)
- Splitting `linux_shortcut_manager.rs` into per-DE files (behavior changed: WM rewrite is opt-in)
- Flatpak / xdg-desktop-portal input instead of raw uinput (documented as long-term)

If you are reading this file from an older checkout, treat the checklist as **done**, not as a backlog.
