#!/usr/bin/env bash

# exit when any command fails
set -e

target_dir='target/universal-unknown-linux-gnu/release'

build_binary() {
  cross build --target x86_64-unknown-linux-gnu --release
  cross build --target aarch64-unknown-linux-gnu --release
  cross build --target armv7-unknown-linux-gnueabihf --release

  # Clean universal binary
  rm -rf "${target_dir:?}/"

  # Build universal binary
  mkdir -p "$target_dir/"
  mkdir -p "$target_dir/x86_64/"
  mkdir -p "$target_dir/aarch64/"
  mkdir -p "$target_dir/armv7l/"

  cp "target/x86_64-unknown-linux-gnu/release/browsers" "$target_dir/x86_64/browsers"
  cp "target/aarch64-unknown-linux-gnu/release/browsers" "$target_dir/aarch64/browsers"
  cp "target/armv7-unknown-linux-gnueabihf/release/browsers" "$target_dir/armv7l/browsers"
}

build_app_bundle() {
  mkdir -p "$target_dir/icons/"
  mkdir -p "$target_dir/icons/16x16"
  mkdir -p "$target_dir/icons/32x32"
  mkdir -p "$target_dir/icons/64x64"
  mkdir -p "$target_dir/icons/128x128"
  mkdir -p "$target_dir/icons/256x256"
  mkdir -p "$target_dir/icons/512x512"
  mkdir -p "$target_dir/i18n"
  mkdir -p "$target_dir/i18n/en-US"

  mkdir -p "$target_dir/xfce4/helpers"

  cp "resources/icons/16x16/software.Browsers.png" "$target_dir/icons/16x16/software.Browsers.png"
  cp "resources/icons/32x32/software.Browsers.png" "$target_dir/icons/32x32/software.Browsers.png"
  cp "resources/icons/64x64/software.Browsers.png" "$target_dir/icons/64x64/software.Browsers.png"
  cp "resources/icons/128x128/software.Browsers.png" "$target_dir/icons/128x128/software.Browsers.png"
  cp "resources/icons/256x256/software.Browsers.png" "$target_dir/icons/256x256/software.Browsers.png"
  cp "resources/icons/512x512/software.Browsers.png" "$target_dir/icons/512x512/software.Browsers.png"
  cp "resources/i18n/en-US/builtin.ftl" "$target_dir/i18n/en-US/builtin.ftl"

  cp "extra/linux/dist/install.sh" "$target_dir/install.sh"
  cp "extra/linux/dist/uninstall.sh" "$target_dir/uninstall.sh"
  cp "extra/linux/dist/software.Browsers.template.desktop" "$target_dir/software.Browsers.template.desktop"
  cp "extra/linux/dist/xfce4/helpers/software.Browsers.template.desktop" "$target_dir/xfce4/helpers/software.Browsers.template.desktop"
}

make_archives() {
  rm -f "./${target_dir:?}/browsers_linux.tar.gz"
  rm -f "./${target_dir:?}/browsers_linux.tar.gz.sha256"
  rm -f "./${target_dir:?}/browsers_linux.tar.gz.sig"

  rm -f "./${target_dir:?}/browsers_linux.tar.xz"
  rm -f "./${target_dir:?}/browsers_linux.tar.xz.sha256"
  rm -f "./${target_dir:?}/browsers_linux.tar.xz.sig"

  local filelist=(
    './x86_64/browsers'
    './aarch64/browsers'
    './armv7l/browsers'
    './icons/16x16/software.Browsers.png'
    './icons/32x32/software.Browsers.png'
    './icons/64x64/software.Browsers.png'
    './icons/128x128/software.Browsers.png'
    './icons/256x256/software.Browsers.png'
    './icons/512x512/software.Browsers.png'
    './software.Browsers.template.desktop'
    './xfce4/helpers/software.Browsers.template.desktop'
    './i18n/en-US/builtin.ftl'
    './install.sh'
    './uninstall.sh'
  )

  tar -zcf "./$target_dir/browsers_linux.tar.gz" \
    -C "./$target_dir" \
    "${filelist[@]}"

  create_signatures "$target_dir" "browsers_linux.tar.gz"

  tar -Jcf "./$target_dir/browsers_linux.tar.xz" \
    -C "./$target_dir" \
    "${filelist[@]}"

  create_signatures "$target_dir" "browsers_linux.tar.xz"
}

create_signatures() {
  local target_dir="$1"
  local file_name="$2"

  # creates $filename.sha256
  shasum --algorithm 256 "./$target_dir/$file_name" | cut -f1 -d' ' > "./$target_dir/$file_name.sha256"

  # creates $filename.sig
  signify -S -s "$APPCAST_SECRET_KEY_FILE" -m "./$target_dir/$file_name"
}

build_deb() {
  # Build deb package
  cd extra/linux || exit
  ./linux-deb-build.sh "$1" "$2"
  cd ../../
}

make_deb_packages() {
  # creating deb does not depend on universal directory at all, but will store the deb there
  build_deb "amd64" "target/x86_64-unknown-linux-gnu"
  build_deb "arm64" "target/aarch64-unknown-linux-gnu"
  build_deb "armhf" "target/armv7-unknown-linux-gnueabihf"

  cp "target/x86_64-unknown-linux-gnu/release/browsers_amd64.deb" "$target_dir/x86_64/browsers_amd64.deb"
  cp "target/aarch64-unknown-linux-gnu/release/browsers_arm64.deb" "$target_dir/aarch64/browsers_arm64.deb"
  cp "target/armv7-unknown-linux-gnueabihf/release/browsers_armhf.deb" "$target_dir/armv7l/browsers_armhf.deb"
}

build_rpm() {
  # Build rpm package
  cd extra/linux || exit
  ./linux-rpm-build.sh "$1" "$2"
  cd ../../
}

make_rpm_packages() {
  # creating rpm does not depend on universal directory at all, but will store the rpm there
  build_rpm "x86_64" "target/x86_64-unknown-linux-gnu"
  build_rpm "aarch64" "target/aarch64-unknown-linux-gnu"
  build_rpm "armhfp" "target/armv7-unknown-linux-gnueabihf"

  cp "target/x86_64-unknown-linux-gnu/release/browsers.x86_64.rpm" "$target_dir/x86_64/browsers.x86_64.rpm"
  cp "target/aarch64-unknown-linux-gnu/release/browsers.aarch64.rpm" "$target_dir/aarch64/browsers.aarch64.rpm"
  cp "target/armv7-unknown-linux-gnueabihf/release/browsers.armhfp.rpm" "$target_dir/armv7l/browsers.armhfp.rpm"
}

build_binary
build_app_bundle
make_archives
make_deb_packages
make_rpm_packages
