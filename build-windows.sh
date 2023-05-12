#!/usr/bin/env bash

# exit when any command fails
set -e

target_dir='target/universal-pc-windows-msvc/release'

build_binary() {
  # Build x86_64 binary
  cargo build --target x86_64-pc-windows-msvc --release

  # Build ARM64 binary
  cargo build --target aarch64-pc-windows-msvc --release

  # Clean universal binary and app bundle
  rm -rf "${target_dir:?}/"

  # Build universal binary
  mkdir -p "$target_dir/"
  mkdir -p "$target_dir/x86_64/"
  mkdir -p "$target_dir/aarch64/"

  cp "target/x86_64-pc-windows-msvc/release/browsers.exe" "$target_dir/x86_64/browsers.exe"
  cp "target/aarch64-pc-windows-msvc/release/browsers.exe" "$target_dir/aarch64/browsers.exe"
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

  cp "resources/icons/16x16/software.Browsers.png" "$target_dir/icons/16x16/software.Browsers.png"
  cp "resources/icons/32x32/software.Browsers.png" "$target_dir/icons/32x32/software.Browsers.png"
  cp "resources/icons/64x64/software.Browsers.png" "$target_dir/icons/64x64/software.Browsers.png"
  cp "resources/icons/128x128/software.Browsers.png" "$target_dir/icons/128x128/software.Browsers.png"
  cp "resources/icons/256x256/software.Browsers.png" "$target_dir/icons/256x256/software.Browsers.png"
  cp "resources/icons/512x512/software.Browsers.png" "$target_dir/icons/512x512/software.Browsers.png"
  cp "resources/i18n/en-US/builtin.ftl" "$target_dir/i18n/en-US/builtin.ftl"
}

make_archives() {
    rm -f "./${target_dir:?}/browsers_windows.tar.gz"
    rm -f "./${target_dir:?}/browsers_windows.tar.gz.sig"

    rm -f "./${target_dir:?}/browsers_windows.tar.xz"
    rm -f "./${target_dir:?}/browsers_windows.tar.xz.sig"

    local filelist=(
      './x86_64/browsers.exe'
      './aarch64/browsers.exe'
      './icons/16x16/software.Browsers.png'
      './icons/32x32/software.Browsers.png'
      './icons/64x64/software.Browsers.png'
      './icons/128x128/software.Browsers.png'
      './icons/256x256/software.Browsers.png'
      './icons/512x512/software.Browsers.png'
      './i18n/en-US/builtin.ftl'
    )

  tar -zcf "./$target_dir/browsers_windows.tar.gz" \
    -C "./$target_dir" \
    "${filelist[@]}"

  # creates browsers_windows.tar.gz.sig
  signify -S -s "$APPCAST_SECRET_KEY_FILE" -m "./$target_dir/browsers_windows.tar.gz"

  tar -Jcf "./$target_dir/browsers_windows.tar.xz" \
    -C "./$target_dir" \
    "${filelist[@]}"

  # creates browsers_windows.tar.xz.sig
  signify -S -s "$APPCAST_SECRET_KEY_FILE" -m "./$target_dir/browsers_windows.tar.xz"
}

build_binary
build_app_bundle
make_archives