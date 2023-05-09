#!/bin/sh

# exit when any command fails
set -e

THIS_DIR="$(dirname "$0")"

INSTALL_DIR="$HOME/.local/bin"
if [ ! -d "$INSTALL_DIR" ]; then
    mkdir -p "$INSTALL_DIR"
    echo "$INSTALL_DIR did not exist. We created it for you."
fi

# Use XDG_DATA_HOME or default to $HOME/.local/share if it's missing
XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
if [ ! -d "$XDG_DATA_HOME" ]; then
    mkdir -p "$XDG_DATA_HOME"
    echo "$XDG_DATA_HOME did not exist. We created it for you."
fi

# Holds binary, icon, translations
DATA_DIR="$XDG_DATA_HOME/software.Browsers"
if [ ! -d "$DATA_DIR" ]; then
    mkdir -p "$DATA_DIR"
    echo "$DATA_DIR did not exist. We created it for you."
fi

RESOURCES_DIR="$DATA_DIR/resources"
if [ ! -d "$RESOURCES_DIR" ]; then
    mkdir -p "$RESOURCES_DIR"
    echo "$RESOURCES_DIR did not exist. We created it for you."
fi

# Where we store the real binary, which will be symlinked from .local/bin
# Useful for in-place upgrades
TARGET_BINARY_DIR="$DATA_DIR/bin"
if [ ! -d "$TARGET_BINARY_DIR" ]; then
    mkdir -p "$TARGET_BINARY_DIR"
    echo "$TARGET_BINARY_DIR did not exist. We created it for you."
fi

ICONS_DIR="$RESOURCES_DIR/icons"
if [ ! -d "$ICONS_DIR" ]; then
    mkdir -p "$ICONS_DIR"
    echo "$ICONS_DIR did not exist. We created it for you."
fi

# We store the icon also here, which is shown in About dialog
ICONS_512_DIR="$ICONS_DIR/512x512"
if [ ! -d "$ICONS_512_DIR" ]; then
    mkdir -p "$ICONS_512_DIR"
    echo "$ICONS_512_DIR did not exist. We created it for you."
fi

SRC_ICONS_DIR="$THIS_DIR/icons"
cp "$SRC_ICONS_DIR/512x512/software.Browsers.png" "$ICONS_DIR/512x512/software.Browsers.png"

I18N_DIR="$RESOURCES_DIR/i18n"
if [ ! -d "$I18N_DIR" ]; then
    mkdir -p "$I18N_DIR"
    echo "$I18N_DIR did not exist. We created it for you."
fi

I18N_EN_US_DIR="$I18N_DIR/en-US"
if [ ! -d "$I18N_EN_US_DIR" ]; then
    mkdir -p "$I18N_EN_US_DIR"
    echo "$I18N_EN_US_DIR did not exist. We created it for you."
fi

SRC_I18N_DIR="$THIS_DIR/i18n"
cp "$SRC_I18N_DIR/en-US/builtin.ftl" "$I18N_DIR/en-US/builtin.ftl"

# armv7l, (arm64 on macos), aarch64, x86_64
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

SOURCE_DESKTOP_FILE_PATH="$THIS_DIR/software.Browsers.desktop"

sed "s|€ExecCommand€|$TARGET_BINARY_PATH %u|g" "$TEMPLATE_DESKTOP_FILE_PATH" > "$SOURCE_DESKTOP_FILE_PATH"

TEMPLATE_XFCE4_DESKTOP_FILE_PATH="$THIS_DIR/xfce4/helpers/software.Browsers.template.desktop"
if [ ! -f "$TEMPLATE_XFCE4_DESKTOP_FILE_PATH" ]; then
    echo "$TEMPLATE_XFCE4_DESKTOP_FILE_PATH does not exist. Please install manually"
    exit 1
fi

SOURCE_XFCE4_DESKTOP_FILE_PATH="$THIS_DIR/xfce4/helpers/software.Browsers.desktop"

sed "s|€XFCEBinaries€|browsers;$TARGET_BINARY_PATH;|g" "$TEMPLATE_XFCE4_DESKTOP_FILE_PATH" > "$SOURCE_XFCE4_DESKTOP_FILE_PATH"

# ~/.local/share/xfce4/helpers
TARGET_XFCE4_HELPERS_DIR="$XDG_DATA_HOME/xfce4/helpers"
if [ ! -d "$TARGET_XFCE4_HELPERS_DIR" ]; then
    mkdir -p "$TARGET_XFCE4_HELPERS_DIR"
    echo "$TARGET_XFCE4_HELPERS_DIR did not exist. We created it for you."
fi

TARGET_XFCE4_DESKTOP_FILE_PATH="$TARGET_XFCE4_HELPERS_DIR/software.Browsers.desktop"

# XFCE4 .desktop file adds Browsers as an option in Default Browsers select list
# ~/.local/share/xfce4/helpers/software.Browsers.desktop
cp "$SOURCE_XFCE4_DESKTOP_FILE_PATH" "$TARGET_XFCE4_DESKTOP_FILE_PATH"

# $HOME/.local/share/software.Browsers/bin/browsers
TARGET_BINARY_PATH="$TARGET_BINARY_DIR/browsers"
cp "$SRC_BINARY_PATH" "$TARGET_BINARY_PATH"

TARGET_INSTALL_BINARY_PATH="$INSTALL_DIR/browsers"
# Symlink binary to $HOME/.local/bin
ln -sf "$TARGET_BINARY_PATH" "$TARGET_INSTALL_BINARY_PATH"

# Installs to /.local/share/icons/hicolor/512x512/apps/software.Browsers.png
xdg-icon-resource install --novendor --size 16 icons/16x16/software.Browsers.png
xdg-icon-resource install --novendor --size 32 icons/32x32/software.Browsers.png
xdg-icon-resource install --novendor --size 64 icons/64x64/software.Browsers.png
xdg-icon-resource install --novendor --size 128 icons/128x128/software.Browsers.png
xdg-icon-resource install --novendor --size 256 icons/256x256/software.Browsers.png
xdg-icon-resource install --novendor --size 512 icons/512x512/software.Browsers.png

# $HOME/.local/share/applications
TARGET_DESKTOP_DIR_PATH="$XDG_DATA_HOME/applications"

# Copy .desktop file to $HOME/.local/share/applications/
desktop-file-install --dir="$TARGET_DESKTOP_DIR_PATH" --rebuild-mime-info-cache "$SOURCE_DESKTOP_FILE_PATH"

# Refresh desktop database
update-desktop-database "$TARGET_DESKTOP_DIR_PATH"

echo "Browsers has been installed. Enjoy!"
echo "Please report any issues at https://github.com/Browsers-software/browsers/issues"