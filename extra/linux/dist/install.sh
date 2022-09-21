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

# Use XDG_DATA_HOME or default to $HOME/.local/share/applications if it's missing
TARGET_DESKTOP_DIR_PATH="${XDG_DATA_HOME:-$HOME/.local/share/applications}"

# Copy .desktop file to $HOME/.local/share/applications/
desktop-file-install --dir="$TARGET_DESKTOP_DIR_PATH" --rebuild-mime-info-cache "$SOURCE_DESKTOP_FILE_PATH"

# Refresh desktop database
update-desktop-database "$TARGET_DESKTOP_DIR_PATH"
