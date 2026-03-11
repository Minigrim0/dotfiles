#!/usr/bin/env bash
# Read current brightness as a percentage.
# The backend is set by dots at install time via machine profile.
# Symlink: ~/.config/waybar/scripts/brightness-backend.sh -> laptop.sh or desktop.sh

BACKEND="$(dirname "$0")/brightness-backend.sh"

if [[ -x "$BACKEND" ]]; then
    source "$BACKEND"
    get_brightness
else
    echo "?"
fi
