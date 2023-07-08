#!/usr/bin/env bash

# exit when any command fails
set -e

PROJECT_ROOT="../.."

DEB_ARCH="$1"

# e.g "target/aarch64-unknown-linux-gnu"
TARGET_PATH="$2"
# e.g "$PROJECT_ROOT/target/aarch64-unknown-linux-gnu"
LINUX_TARGET_DIR="$PROJECT_ROOT/$TARGET_PATH"

copy_deb_control() {
  local deb_src_dir="$1"
  local linux_target_dir="$2"

  mkdir -p "$deb_src_dir/DEBIAN"
  cp "$linux_target_dir/meta/deb_control/control" "$deb_src_dir/DEBIAN/control"
}

# creates deb file to e.g "$PROJECT_ROOT/target/aarch64-unknown-linux-gnu/release/browsers_${deb_arch}.deb"
create_deb() {
  local deb_arch="$1"
  local deb_filename="browsers_${deb_arch}.deb"
  # using xz compression instead of the new default zstd, which can't be opened by older distros
  dpkg-deb -Zxz --root-owner-group --build "$PACKAGE_SRC_DIR" "$deb_filename"
  cp "$deb_filename" "$LINUX_TARGET_DIR/release/"
}

source linux-build-file-tree.sh
create_package_content "$LINUX_TARGET_DIR" "deb_source_${DEB_ARCH}"
copy_deb_control "$PACKAGE_SRC_DIR" "$LINUX_TARGET_DIR"
create_deb "$DEB_ARCH"

