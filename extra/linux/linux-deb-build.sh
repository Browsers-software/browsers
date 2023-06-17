#!/usr/bin/env bash

# exit when any command fails
set -e

PROJECT_ROOT="../.."
EXTRA_LINUX_DIST_PATH="./dist/"

DEB_ARCH="$1"
DEB_SRC_DIR="deb_source_${DEB_ARCH}"

# e.g "target/aarch64-unknown-linux-gnu"
TARGET_PATH="$2"
# e.g "$PROJECT_ROOT/target/aarch64-unknown-linux-gnu"
LINUX_TARGET_DIR="$PROJECT_ROOT/$TARGET_PATH"

create_deb_content() {
  local deb_arch="$1"
  local linux_target_dir="$2"
  local PROJECT_RESOURCES_DIR="$PROJECT_ROOT/resources"

  echo "Creating deb content"

  rm -rf "$DEB_SRC_DIR"
  mkdir -p "$DEB_SRC_DIR"

  mkdir -p "$DEB_SRC_DIR/usr/share/icons/hicolor"

  # /usr/share/icons/hicolor/512x512/apps/software.Browsers.png
  local sizes=("16x16" "32x32" "64x64" "128x128" "256x256" "512x512")

  for size in "${sizes[@]}"; do
    mkdir -p "$DEB_SRC_DIR/usr/share/icons/hicolor/$size/apps"
    cp "$PROJECT_RESOURCES_DIR/icons/$size/software.Browsers.png" "$DEB_SRC_DIR/usr/share/icons/hicolor/$size/apps/software.Browsers.png"
  done

  mkdir -p "$DEB_SRC_DIR/usr/share/applications"

  local TARGET_INSTALL_BINARY_PATH="/usr/bin/browsers"

  local TEMPLATE_DESKTOP_FILE_PATH="$EXTRA_LINUX_DIST_PATH/software.Browsers.template.desktop"

  mkdir -p "$DEB_SRC_DIR/usr/share/applications/xfce4/helpers"

  local DESKTOP_FILE_PATH="$DEB_SRC_DIR/usr/share/applications/software.Browsers.desktop"
  sed "s|€ExecCommand€|$TARGET_INSTALL_BINARY_PATH %u|g" "$TEMPLATE_DESKTOP_FILE_PATH" > "$DESKTOP_FILE_PATH"


  local TEMPLATE_XFCE4_DESKTOP_FILE_PATH="$EXTRA_LINUX_DIST_PATH/xfce4/helpers/software.Browsers.template.desktop"
  local XFCE4_DESKTOP_FILE_PATH="$DEB_SRC_DIR/usr/share/applications/xfce4/helpers/software.Browsers.desktop"

  sed "s|€XFCEBinaries€|browsers;$TARGET_INSTALL_BINARY_PATH;|g" "$TEMPLATE_XFCE4_DESKTOP_FILE_PATH" > "$XFCE4_DESKTOP_FILE_PATH"

  local DEB_DATA_DIR="$DEB_SRC_DIR/usr/share/software.Browsers"

  mkdir -p "$DEB_DATA_DIR"

  mkdir -p "$DEB_DATA_DIR/resources"

  mkdir -p "$DEB_DATA_DIR/resources/i18n/en-US"
  cp "$PROJECT_RESOURCES_DIR/i18n/en-US/builtin.ftl" "$DEB_DATA_DIR/resources/i18n/en-US/builtin.ftl"

  mkdir -p "$DEB_DATA_DIR/resources/icons/512x512"
  cp "$PROJECT_RESOURCES_DIR/icons/512x512/software.Browsers.png" "$DEB_DATA_DIR/resources/icons/512x512/software.Browsers.png"

  mkdir -p "$DEB_DATA_DIR/bin"

  # architecture and build specific part (linux_target_dir)
  # armv7l, aarch64, x86_64

  cp "$linux_target_dir/release/browsers" "$DEB_DATA_DIR/bin/browsers"

  # Create symlink
  mkdir -p $DEB_SRC_DIR/usr/bin
  ln -s "../share/software.Browsers/bin/browsers" "$DEB_SRC_DIR/usr/bin/browsers"

  mkdir -p "$DEB_SRC_DIR/DEBIAN"
  cp "$linux_target_dir/meta/deb_control/control" "$DEB_SRC_DIR/DEBIAN/control"
}

# creates deb file to e.g "$PROJECT_ROOT/target/aarch64-unknown-linux-gnu/release/browsers_${deb_arch}.deb"
create_deb() {
  local deb_arch="$1"
  local deb_filename="browsers_${deb_arch}.deb"
  dpkg-deb --root-owner-group --build "$DEB_SRC_DIR" "$deb_filename"
  cp "$deb_filename" "$LINUX_TARGET_DIR/release/"
}

create_deb_content "$DEB_ARCH" "$LINUX_TARGET_DIR"
create_deb "$DEB_ARCH"

