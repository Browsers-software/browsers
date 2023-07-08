#!/usr/bin/env bash

# exit when any command fails
set -e

PROJECT_ROOT="../.."

RPM_ARCH="$1"

RPM_TOP_DIR="rpm_source_${RPM_ARCH}/rpmbuild"

THIS_DIR_ABSOLUTE="$( cd -- "$(dirname "$0")" >/dev/null 2>&1 ; pwd -P )"

RPM_TOP_DIR_ABSOLUTE="$THIS_DIR_ABSOLUTE/$RPM_TOP_DIR"

# e.g "target/aarch64-unknown-linux-gnu"
TARGET_PATH="$2"
# e.g "$PROJECT_ROOT/target/aarch64-unknown-linux-gnu"
LINUX_TARGET_DIR="$PROJECT_ROOT/$TARGET_PATH"

copy_rpm_spec() {
  local rpm_top_dir="$1"
  local linux_target_dir="$2"

  mkdir -p "$rpm_top_dir/SPECS"
  cp "$linux_target_dir/meta/rpm_spec/browsers.spec" "$rpm_top_dir/SPECS/browsers.spec"
}

# creates rpm file to e.g "$PROJECT_ROOT/target/aarch64-unknown-linux-gnu/release/browsers_${rpm_arch}.rpm"
create_rpm() {
  local rpm_arch="$1"
  local rpm_filename="browsers.${rpm_arch}.rpm"
  # --buildroot=DIRECTORY      override build root
  # --build-in-place           run build in current directory
  # --target=CPU-VENDOR-OS     override target platform
  # --target aarch64-fedora-linux
  #rpmbuild --target --buildroot arm64 -bb "$PACKAGE_SRC_DIR/rpmbuild/SPECS/browsers.spec"

  echo "RPM_TOP_DIR_ABSOLUTE is ${RPM_TOP_DIR_ABSOLUTE}"
  rpmbuild --target "${rpm_arch}-linux" --define "_topdir ${RPM_TOP_DIR_ABSOLUTE}" -bb "$RPM_TOP_DIR_ABSOLUTE/SPECS/browsers.spec"

  #dpkg-deb -Zxz --root-owner-group --build "$DEB_SRC_DIR" "$rpm_filename"
  cp "${RPM_TOP_DIR_ABSOLUTE}/RPMS/$rpm_filename" "$LINUX_TARGET_DIR/release/"
}

source linux-build-file-tree.sh
create_package_content "$LINUX_TARGET_DIR" "${RPM_TOP_DIR}/tree"
copy_rpm_spec "$RPM_TOP_DIR" "$LINUX_TARGET_DIR"
create_rpm "$RPM_ARCH"
