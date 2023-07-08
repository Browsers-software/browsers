#!/usr/bin/env bash

# exit when any command fails
set -e

PROJECT_ROOT="../.."
EXTRA_LINUX_DIST_PATH="./dist/"

RPM_ARCH="$1"

RPM_TOP_DIR="rpm_source_${RPM_ARCH}/rpmbuild"

THIS_DIR_ABSOLUTE="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"

RPM_TOP_DIR_ABSOLUTE="$THIS_DIR_ABSOLUTE/$RPM_TOP_DIR"

RPM_SRC_DIR="${RPM_TOP_DIR}/tree"

# e.g "target/aarch64-unknown-linux-gnu"
TARGET_PATH="$2"
# e.g "$PROJECT_ROOT/target/aarch64-unknown-linux-gnu"
LINUX_TARGET_DIR="$PROJECT_ROOT/$TARGET_PATH"

# TODO: externalize common part for deb and rpm (mostly building the tree)
create_rpm_content() {
  local rpm_arch="$1"
  local linux_target_dir="$2"
  local PROJECT_RESOURCES_DIR="$PROJECT_ROOT/resources"

  echo "Creating rpm content"

  rm -rf "$RPM_SRC_DIR"
  mkdir -p "$RPM_SRC_DIR"

  mkdir -p "$RPM_SRC_DIR/usr/share/icons/hicolor"

  # /usr/share/icons/hicolor/512x512/apps/software.Browsers.png
  local sizes=("16x16" "32x32" "64x64" "128x128" "256x256" "512x512")

  for size in "${sizes[@]}"; do
    mkdir -p "$RPM_SRC_DIR/usr/share/icons/hicolor/$size/apps"
    cp "$PROJECT_RESOURCES_DIR/icons/$size/software.Browsers.png" "$RPM_SRC_DIR/usr/share/icons/hicolor/$size/apps/software.Browsers.png"
  done

  mkdir -p "$RPM_SRC_DIR/usr/share/applications"

  local TARGET_INSTALL_BINARY_PATH="/usr/bin/browsers"

  local TEMPLATE_DESKTOP_FILE_PATH="$EXTRA_LINUX_DIST_PATH/software.Browsers.template.desktop"

  mkdir -p "$RPM_SRC_DIR/usr/share/applications/xfce4/helpers"

  local DESKTOP_FILE_PATH="$RPM_SRC_DIR/usr/share/applications/software.Browsers.desktop"
  sed "s|€ExecCommand€|$TARGET_INSTALL_BINARY_PATH %u|g" "$TEMPLATE_DESKTOP_FILE_PATH" > "$DESKTOP_FILE_PATH"


  local TEMPLATE_XFCE4_DESKTOP_FILE_PATH="$EXTRA_LINUX_DIST_PATH/xfce4/helpers/software.Browsers.template.desktop"
  local XFCE4_DESKTOP_FILE_PATH="$RPM_SRC_DIR/usr/share/applications/xfce4/helpers/software.Browsers.desktop"

  sed "s|€XFCEBinaries€|browsers;$TARGET_INSTALL_BINARY_PATH;|g" "$TEMPLATE_XFCE4_DESKTOP_FILE_PATH" > "$XFCE4_DESKTOP_FILE_PATH"

  local RPM_DATA_DIR="$RPM_SRC_DIR/usr/share/software.Browsers"

  mkdir -p "$RPM_DATA_DIR"

  mkdir -p "$RPM_DATA_DIR/resources"

  mkdir -p "$RPM_DATA_DIR/resources/i18n/en-US"
  cp "$PROJECT_RESOURCES_DIR/i18n/en-US/builtin.ftl" "$RPM_DATA_DIR/resources/i18n/en-US/builtin.ftl"

  mkdir -p "$RPM_DATA_DIR/resources/icons/512x512"
  cp "$PROJECT_RESOURCES_DIR/icons/512x512/software.Browsers.png" "$RPM_DATA_DIR/resources/icons/512x512/software.Browsers.png"

  mkdir -p "$RPM_DATA_DIR/bin"

  # architecture and build specific part (linux_target_dir)
  # armv7l, aarch64, x86_64

  cp "$linux_target_dir/release/browsers" "$RPM_DATA_DIR/bin/browsers"

  # Create symlink
  mkdir -p $RPM_SRC_DIR/usr/bin
  ln -s "../share/software.Browsers/bin/browsers" "$RPM_SRC_DIR/usr/bin/browsers"

  mkdir -p "$RPM_TOP_DIR/SPECS"
  cp "$linux_target_dir/meta/rpm_spec/browsers.spec" "$RPM_TOP_DIR/SPECS/browsers.spec"
}

# creates rpm file to e.g "$PROJECT_ROOT/target/aarch64-unknown-linux-gnu/release/browsers_${rpm_arch}.rpm"
create_rpm() {
  local rpm_arch="$1"
  local rpm_filename="browsers.${rpm_arch}.rpm"
  # --buildroot=DIRECTORY      override build root
  # --build-in-place           run build in current directory
  # --target=CPU-VENDOR-OS     override target platform
  # --target aarch64-fedora-linux
  #rpmbuild --target --buildroot arm64 -bb "$RPM_SRC_DIR/rpmbuild/SPECS/browsers.spec"

  echo "RPM_TOP_DIR_ABSOLUTE is ${RPM_TOP_DIR_ABSOLUTE}"
  rpmbuild --target "${rpm_arch}-linux" --define "_topdir ${RPM_TOP_DIR_ABSOLUTE}" -bb "$RPM_TOP_DIR_ABSOLUTE/SPECS/browsers.spec"

  #dpkg-deb -Zxz --root-owner-group --build "$DEB_SRC_DIR" "$rpm_filename"
  cp "${RPM_TOP_DIR_ABSOLUTE}/RPMS/$rpm_filename" "$LINUX_TARGET_DIR/release/"
}

create_rpm_content "$RPM_ARCH" "$LINUX_TARGET_DIR"
create_rpm "$RPM_ARCH"
