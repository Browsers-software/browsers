#!/usr/bin/env bash

# exit when any command fails
set -e

create_dir_if_not_exists() {
  local dir_path="$1"
  local hide_notice="$2"

  if [ ! -d "$dir_path" ]; then
      mkdir -p "$dir_path"
      if [ ! "$hide_notice" = true ]; then
        echo "$dir_path did not exist. We created it for you."
      fi
  fi
}

# bool function to test if the user is root or not (POSIX only)
# though doesn't work if there is no root, but user still has correct permissions
is_user_root_or_sudoer() {
  [ "$(id -u)" -eq 0 ]
}

THIS_DIR="$(dirname "$0")"

if [[ $* == *--system* ]]; then
  IS_LOCAL_INSTALL=false

  if ! is_user_root_or_sudoer; then
    echo "You must run this installer with sudo when using --system flag"
    echo "Please run again as:"
    echo ""
    echo "sudo ./install.sh --system"
    echo ""
    exit 1
  fi
else
  IS_LOCAL_INSTALL=true
fi

# INSTALL_DIR will keep a symlink to real binary
if [ "$IS_LOCAL_INSTALL" = true ]; then
  #  ~/.local/bin
  INSTALL_DIR="$HOME/.local/bin"
else
  # /usr/local/bin is for binaries not managed by package manager
  # (otherwise should use /usr/bin if using package manager)
  INSTALL_DIR="/usr/local/bin"
fi
create_dir_if_not_exists "$INSTALL_DIR"

TARGET_INSTALL_BINARY_PATH="$INSTALL_DIR/browsers"

# Use XDG_DATA_HOME or default to $HOME/.local/share if it's missing
XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
create_dir_if_not_exists "$XDG_DATA_HOME"

if [ "$IS_LOCAL_INSTALL" = true ]; then
  # ~/.local/share/software.Browsers
  DATA_DIR="$XDG_DATA_HOME/software.Browsers"
else
  # /usr/local/share is for files not managed by package manager
  # (otherwise should use /usr/share if using package manager)
  DATA_DIR="/usr/local/share/software.Browsers"
fi

# Holds binary, icon, translations
create_dir_if_not_exists "$DATA_DIR"

RESOURCES_DIR="$DATA_DIR/resources"
create_dir_if_not_exists "$RESOURCES_DIR" true

# Where we store the real binary, which will be symlinked from .local/bin
# Useful for in-place upgrades
TARGET_BINARY_DIR="$DATA_DIR/bin"
create_dir_if_not_exists "$TARGET_BINARY_DIR" true

# $HOME/.local/share/software.Browsers/bin/browsers
TARGET_BINARY_PATH="$TARGET_BINARY_DIR/browsers"

ICONS_DIR="$RESOURCES_DIR/icons"
create_dir_if_not_exists "$ICONS_DIR" true

# We store the icon also here, which is shown in About dialog
ICONS_512_DIR="$ICONS_DIR/512x512"
create_dir_if_not_exists "$ICONS_512_DIR" true

SRC_ICONS_DIR="$THIS_DIR/icons"
cp "$SRC_ICONS_DIR/512x512/software.Browsers.png" "$ICONS_DIR/512x512/software.Browsers.png"

I18N_DIR="$RESOURCES_DIR/i18n"
create_dir_if_not_exists "$I18N_DIR" true

I18N_EN_US_DIR="$I18N_DIR/en-US"
create_dir_if_not_exists "$I18N_EN_US_DIR" true

SRC_I18N_DIR="$THIS_DIR/i18n"
cp "$SRC_I18N_DIR/en-US/builtin.ftl" "$I18N_DIR/en-US/builtin.ftl"

# armv7l, aarch64, x86_64
ARCH="$(uname -m)"

SRC_BINARY_PATH="$THIS_DIR/$ARCH/browsers"
if [ ! -f "$SRC_BINARY_PATH" ]; then
    echo "$SRC_BINARY_PATH does not exist. Please install manually"
    exit 1
fi

TEMPLATE_DESKTOP_FILE_PATH="$THIS_DIR/software.Browsers.template.desktop"
if [ ! -f "$TEMPLATE_DESKTOP_FILE_PATH" ]; then
    echo "$TEMPLATE_DESKTOP_FILE_PATH does not exist. Please install manually"
    exit 1
fi

# Write to different paths, so that if you ran this script first with sudo
# You can run it again without sudo (otherwise you can edit the file created by sudo)
if [ "$IS_LOCAL_INSTALL" = true ]; then
  create_dir_if_not_exists "$THIS_DIR/user" true
  SOURCE_DESKTOP_FILE_PATH="$THIS_DIR/user/software.Browsers.desktop"
else
  create_dir_if_not_exists "$THIS_DIR/system" true
  SOURCE_DESKTOP_FILE_PATH="$THIS_DIR/system/software.Browsers.desktop"
fi

sed "s|€ExecCommand€|$TARGET_INSTALL_BINARY_PATH %u|g" "$TEMPLATE_DESKTOP_FILE_PATH" > "$SOURCE_DESKTOP_FILE_PATH"

TEMPLATE_XFCE4_DESKTOP_FILE_PATH="$THIS_DIR/xfce4/helpers/software.Browsers.template.desktop"
if [ ! -f "$TEMPLATE_XFCE4_DESKTOP_FILE_PATH" ]; then
    echo "$TEMPLATE_XFCE4_DESKTOP_FILE_PATH does not exist. Please install manually"
    exit 1
fi

if [ "$IS_LOCAL_INSTALL" = true ]; then
  create_dir_if_not_exists "$THIS_DIR/user/xfce4/helpers" true
  SOURCE_XFCE4_DESKTOP_FILE_PATH="$THIS_DIR/user/xfce4/helpers/software.Browsers.desktop"
else
  create_dir_if_not_exists "$THIS_DIR/system/xfce4/helpers" true
  SOURCE_XFCE4_DESKTOP_FILE_PATH="$THIS_DIR/system/xfce4/helpers/software.Browsers.desktop"
fi

sed "s|€XFCEBinaries€|browsers;$TARGET_INSTALL_BINARY_PATH;|g" "$TEMPLATE_XFCE4_DESKTOP_FILE_PATH" > "$SOURCE_XFCE4_DESKTOP_FILE_PATH"

if [ "$IS_LOCAL_INSTALL" = true ]; then
  # ~/.local/share/xfce4/helpers
  TARGET_XFCE4_HELPERS_DIR="$XDG_DATA_HOME/xfce4/helpers"
else
  TARGET_XFCE4_HELPERS_DIR="/usr/share/xfce4/helpers"
fi
create_dir_if_not_exists "$TARGET_XFCE4_HELPERS_DIR"

TARGET_XFCE4_DESKTOP_FILE_PATH="$TARGET_XFCE4_HELPERS_DIR/software.Browsers.desktop"

# XFCE4 .desktop file adds Browsers as an option in Default Browsers select list
# ~/.local/share/xfce4/helpers/software.Browsers.desktop
cp "$SOURCE_XFCE4_DESKTOP_FILE_PATH" "$TARGET_XFCE4_DESKTOP_FILE_PATH"

# $HOME/.local/share/software.Browsers/bin/browsers
cp "$SRC_BINARY_PATH" "$TARGET_BINARY_PATH"

# Symlink binary to $HOME/.local/bin
ln -sf "$TARGET_BINARY_PATH" "$TARGET_INSTALL_BINARY_PATH"

# Installs to ~/.local/share/icons/hicolor/512x512/apps/software.Browsers.png
#          or /usr/share/icons/hicolor/512x512/apps/software.Browsers.png
# --mode user|system
# The default is to use system mode when called by root and to use user mode
# when called by a non-root user.
# Could also consider symlinking from application directory
# (but we don't need all those icons for the app itself)
xdg-icon-resource install --novendor --size 16 icons/16x16/software.Browsers.png
xdg-icon-resource install --novendor --size 32 icons/32x32/software.Browsers.png
xdg-icon-resource install --novendor --size 64 icons/64x64/software.Browsers.png
xdg-icon-resource install --novendor --size 128 icons/128x128/software.Browsers.png
xdg-icon-resource install --novendor --size 256 icons/256x256/software.Browsers.png
xdg-icon-resource install --novendor --size 512 icons/512x512/software.Browsers.png

if [ "$IS_LOCAL_INSTALL" = true ]; then
  # ~/.local/share/applications
  TARGET_DESKTOP_DIR_PATH="$XDG_DATA_HOME/applications"
else
  TARGET_DESKTOP_DIR_PATH="/usr/share/applications"
fi

# Copy .desktop file to $HOME/.local/share/applications/ or /usr/share/applications
desktop-file-install --dir="$TARGET_DESKTOP_DIR_PATH" --rebuild-mime-info-cache "$SOURCE_DESKTOP_FILE_PATH"

# Refresh desktop database
update-desktop-database "$TARGET_DESKTOP_DIR_PATH"

echo "Browsers has been installed. Enjoy!"
echo "Please report any issues at https://github.com/Browsers-software/browsers/issues"