#!/usr/bin/env bash

# exit when any command fails
set -e

target_dir='target/universal-unknown-linux-gnu/release'

build() {
  cross build --target x86_64-unknown-linux-gnu --release
  cross build --target aarch64-unknown-linux-gnu --release

  # Clean universal binary
  rm -rf "${target_dir:?}/"

  # Build universal binary
  mkdir -p "$target_dir/"
  mkdir -p "$target_dir/x86_64/"
  mkdir -p "$target_dir/aarch64/"
  mkdir -p "$target_dir/icons/"
  mkdir -p "$target_dir/icons/16x16"
  mkdir -p "$target_dir/icons/32x32"
  mkdir -p "$target_dir/icons/64x64"
  mkdir -p "$target_dir/icons/128x128"
  mkdir -p "$target_dir/icons/256x256"
  mkdir -p "$target_dir/icons/512x512"
  mkdir -p "$target_dir/i18n"
  mkdir -p "$target_dir/i18n/en-US"

  cp "target/x86_64-unknown-linux-gnu/release/browsers" "$target_dir/x86_64/browsers"
  cp "target/aarch64-unknown-linux-gnu/release/browsers" "$target_dir/aarch64/browsers"

  cp "extra/linux/dist/install.sh" "$target_dir/install.sh"
  cp "extra/linux/dist/software.Browsers.template.desktop" "$target_dir/software.Browsers.desktop"

  cp "resources/icons/16x16/software.Browsers.png" "$target_dir/icons/16x16/software.Browsers.png"
  cp "resources/icons/32x32/software.Browsers.png" "$target_dir/icons/32x32/software.Browsers.png"
  cp "resources/icons/64x64/software.Browsers.png" "$target_dir/icons/64x64/software.Browsers.png"
  cp "resources/icons/128x128/software.Browsers.png" "$target_dir/icons/128x128/software.Browsers.png"
  cp "resources/icons/256x256/software.Browsers.png" "$target_dir/icons/256x256/software.Browsers.png"
  cp "resources/icons/512x512/software.Browsers.png" "$target_dir/icons/512x512/software.Browsers.png"

  cp "resources/i18n/en-US/builtin.ftl" "$target_dir/i18n/en-US/builtin.ftl"
}

make_archives() {
  rm -f "./${target_dir:?}/browsers_linux.tar.gz"
  rm -f "./${target_dir:?}/browsers_linux.tar.gz.sig"

  rm -f "./${target_dir:?}/browsers_linux.tar.xz"
  rm -f "./${target_dir:?}/browsers_linux.tar.xz.sig"

  local filelist=(
    './x86_64/browsers'
    './aarch64/browsers'
    './icons/16x16/software.Browsers.png'
    './icons/32x32/software.Browsers.png'
    './icons/64x64/software.Browsers.png'
    './icons/128x128/software.Browsers.png'
    './icons/256x256/software.Browsers.png'
    './icons/512x512/software.Browsers.png'
    './software.Browsers.template.desktop'
    './install.sh'
  )

  tar -zcf "./$target_dir/browsers_linux.tar.gz" \
    -C "./$target_dir" \
    "${filelist[@]}"

  # creates browsers_linux.tar.gz.sig
  signify -S -s "$APPCAST_SECRET_KEY_FILE" -m "./$target_dir/browsers_linux.tar.gz"

  tar -Jcf "./$target_dir/browsers_linux.tar.xz" \
    -C "./$target_dir" \
    "${filelist[@]}"

  # creates browsers_linux.tar.xz.sig
  signify -S -s "$APPCAST_SECRET_KEY_FILE" -m "./$target_dir/browsers_linux.tar.xz"
}

build
make_archives