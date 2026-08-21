#!/bin/bash
# Post-removal script for modern-clipboard-history-for-linux
set -e

case "$1" in
    purge)
        # Remove module configuration (handle both possible filenames for compatibility)
        rm -f /etc/modules-load.d/modern-clipboard-history-for-linux.conf
        rm -f /etc/modules-load.d/uinput.conf
        
        # Update caches
        update-desktop-database -q /usr/share/applications 2>/dev/null || true
        gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor 2>/dev/null || true
        ;;
esac

exit 0
