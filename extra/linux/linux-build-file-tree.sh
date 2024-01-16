#!/usr/bin/env bash

# exit when any command fails
set -e

PROJECT_ROOT="../.."
EXTRA_LINUX_DIST_PATH="./dist/"

PACKAGE_SRC_DIR="$1"

# e.g "target/aarch64-unknown-linux-gnu"
TARGET_PATH="$2"
# e.g "$PROJECT_ROOT/target/aarch64-unknown-linux-gnu"
LINUX_TARGET_DIR="$PROJECT_ROOT/$TARGET_PATH"

create_package_content() {
  local linux_target_dir="$1"
  local package_src_dir="$2"
  local project_resources_dir="$PROJECT_ROOT/resources"

  echo "Creating package content"

  rm -rf "$package_src_dir"
  mkdir -p "$package_src_dir"

  mkdir -p "$package_src_dir/usr/share/icons/hicolor"

  # /usr/share/icons/hicolor/512x512/apps/software.Browsers.png
  local sizes=("16x16" "32x32" "64x64" "128x128" "256x256" "512x512")

  for size in "${sizes[@]}"; do
    mkdir -p "$package_src_dir/usr/share/icons/hicolor/$size/apps"
    cp "$project_resources_dir/icons/$size/software.Browsers.png" "$package_src_dir/usr/share/icons/hicolor/$size/apps/software.Browsers.png"
  done

  mkdir -p "$package_src_dir/usr/share/applications"

  local target_install_binary_path="/usr/bin/browsers"

  local template_desktop_file_path="$EXTRA_LINUX_DIST_PATH/software.Browsers.template.desktop"

  mkdir -p "$package_src_dir/usr/share/applications/xfce4/helpers"

  local desktop_file_path="$package_src_dir/usr/share/applications/software.Browsers.desktop"
  sed "s|€ExecCommand€|$target_install_binary_path %u|g" "$template_desktop_file_path" > "$desktop_file_path"

  local template_xfce4_desktop_file_path="$EXTRA_LINUX_DIST_PATH/xfce4/helpers/software.Browsers.template.desktop"
  local xfce4_desktop_file_path="$package_src_dir/usr/share/applications/xfce4/helpers/software.Browsers.desktop"

  sed "s|€XFCEBinaries€|browsers;$target_install_binary_path;|g" "$template_xfce4_desktop_file_path" > "$xfce4_desktop_file_path"

  local package_data_dir="$package_src_dir/usr/share/software.Browsers"

  mkdir -p "$package_data_dir"

  mkdir -p "$package_data_dir/resources"

  mkdir -p "$package_data_dir/resources/i18n/en-US"
  cp "$project_resources_dir/i18n/en-US/builtin.ftl" "$package_data_dir/resources/i18n/en-US/builtin.ftl"

  mkdir -p "$package_data_dir/resources/icons/512x512"
  cp "$project_resources_dir/icons/512x512/software.Browsers.png" "$package_data_dir/resources/icons/512x512/software.Browsers.png"

  mkdir -p "$package_data_dir/resources/lib"
  cp "$project_resources_dir/lib/application-repository.toml" "$package_data_dir/resources/lib/application-repository.toml"

  mkdir -p "$package_data_dir/bin"

  # architecture and build specific part (linux_target_dir)
  # armv7l, aarch64, x86_64

  cp "$linux_target_dir/release/browsers" "$package_data_dir/bin/browsers"

  # Create symlink
  mkdir -p "$package_src_dir/usr/bin"
  ln -s "../share/software.Browsers/bin/browsers" "$package_src_dir/usr/bin/browsers"
}

create_package_content "$LINUX_TARGET_DIR" "$PACKAGE_SRC_DIR"
