#!/bin/sh

INSTALL_DIR="$HOME/.local/bin"
if [ ! -d "$INSTALL_DIR" ]; then
    echo "$INSTALL_DIR does not exist. Please install manually"
    exit 1
fi

THIS_DIR="$(dirname "$0")"

# armv7l, (arm64 on macos), aarch64, x86_64
ARCH="$(uname -m)"

SRC_BINARY_PATH="$THIS_DIR/$ARCH/browsers"
if [ ! -f "$SRC_BINARY_PATH" ]; then
    echo "$SRC_BINARY_PATH does not exist. Please install manually"
    exit 1
fi

SOURCE_DESKTOP_FILE_PATH="$THIS_DIR/software.Browsers.desktop"
if [ ! -f "$SOURCE_DESKTOP_FILE_PATH" ]; then
    echo "$SOURCE_DESKTOP_FILE_PATH does not exist. Please install manually"
    exit 1
fi

# Copy binary to $HOME/.local/bin
cp "$SRC_BINARY_PATH" "$INSTALL_DIR/browsers"

# Installs to /.local/share/icons/hicolor/512x512/apps/software.Browsers.png
xdg-icon-resource install --novendor --size 16 icons/16x16/software.Browsers.png
xdg-icon-resource install --novendor --size 32 icons/32x32/software.Browsers.png
xdg-icon-resource install --novendor --size 64 icons/64x64/software.Browsers.png
xdg-icon-resource install --novendor --size 128 icons/128x128/software.Browsers.png
xdg-icon-resource install --novendor --size 256 icons/256x256/software.Browsers.png
xdg-icon-resource install --novendor --size 512 icons/512x512/software.Browsers.png

# Use XDG_DATA_HOME or default to $HOME/.local/share if it's missing
XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
# $HOME/.local/share/applications
TARGET_DESKTOP_DIR_PATH="$XDG_DATA_HOME/applications"

# Copy .desktop file to $HOME/.local/share/applications/
desktop-file-install --dir="$TARGET_DESKTOP_DIR_PATH" --rebuild-mime-info-cache "$SOURCE_DESKTOP_FILE_PATH"

# Refresh desktop database
update-desktop-database "$TARGET_DESKTOP_DIR_PATH"

# Use XDG_DATA_HOME or default to $HOME/.local/share if it's missing
TARGET_ICON_DIR_PATH="${XDG_DATA_HOME:-$HOME/.local/share}"