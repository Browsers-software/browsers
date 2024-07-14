#!/usr/bin/env bash

# exit when any command fails
set -e

# bool function to test if the user is root or not (POSIX only)
# though doesn't work if there is no root, but user still has correct permissions
is_user_root_or_sudoer() {
  [ "$(id -u)" -eq 0 ]
}

if [[ $* == *--system* ]]; then
  if ! is_user_root_or_sudoer; then
    echo "You must run this installer with sudo when using --system flag"
    echo "Please run again as:"
    echo ""
    echo "sudo ./uninstall.sh --system"
    echo ""
    exit 1
  fi

  IS_LOCAL_INSTALL=false
else
  IS_LOCAL_INSTALL=true
fi

XDG_CACHE_HOME="${XDG_CACHE_HOME:-$HOME/.cache}"
XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
XDG_STATE_HOME="${XDG_STATE_HOME:-$HOME/.local/state}"
XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"

# INSTALL_DIR will keep a symlink to real binary
if [ "$IS_LOCAL_INSTALL" = true ]; then
  #  ~/.local/bin
  INSTALL_DIR="$HOME/.local/bin"

  # ~/.local/share
  DATA_PARENT_DIR="$XDG_DATA_HOME"

  TARGET_XFCE4_HELPERS_DIR="$XDG_DATA_HOME/xfce4/helpers"

  ICONS_DIR="$XDG_DATA_HOME/icons"

  # ~/.local/share/applications
  TARGET_DESKTOP_DIR_PATH="$XDG_DATA_HOME/applications"
else
  # /usr/local/bin is for binaries not managed by package manager
  # (otherwise should use /usr/bin if using package manager)
  INSTALL_DIR="/usr/local/bin"

  # /usr/local/share is for files not managed by package manager
  # (otherwise should use /usr/share if using package manager)
  DATA_PARENT_DIR="/usr/local/share"

  TARGET_XFCE4_HELPERS_DIR="/usr/share/xfce4/helpers"

  ICONS_DIR="/usr/share/icons"

  TARGET_DESKTOP_DIR_PATH="/usr/share/applications"
fi

# Print all rm commands for observability
set -x

rm -rf "$XDG_CACHE_HOME/software.Browsers/"
rm -rf "$XDG_CONFIG_HOME/software.Browsers/"
rm -rf "$XDG_STATE_HOME/software.Browsers/"

rm -f "$INSTALL_DIR/browsers"

rm -rf "$DATA_PARENT_DIR/software.Browsers/"
rm -f "$TARGET_XFCE4_HELPERS_DIR/software.Browsers.desktop"

for size in 16 32 64 128 256 512; do
  rm -f "$ICONS_DIR/hicolor/${size}x${size}/apps/software.Browsers.png"
done

rm -f "$TARGET_DESKTOP_DIR_PATH/software.Browsers.desktop"

set +x

update-desktop-database "$TARGET_DESKTOP_DIR_PATH"

echo "Browsers has been uninstalled."
