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
  mkdir -p "$target_dir/lib"

  mkdir -p "$target_dir/startmenu"
  mkdir -p "$target_dir/startmenu/user"
  mkdir -p "$target_dir/startmenu/system"

  cp "resources/icons/16x16/software.Browsers.png" "$target_dir/icons/16x16/software.Browsers.png"
  cp "resources/icons/32x32/software.Browsers.png" "$target_dir/icons/32x32/software.Browsers.png"
  cp "resources/icons/64x64/software.Browsers.png" "$target_dir/icons/64x64/software.Browsers.png"
  cp "resources/icons/128x128/software.Browsers.png" "$target_dir/icons/128x128/software.Browsers.png"
  cp "resources/icons/256x256/software.Browsers.png" "$target_dir/icons/256x256/software.Browsers.png"
  cp "resources/icons/512x512/software.Browsers.png" "$target_dir/icons/512x512/software.Browsers.png"
  cp "resources/i18n/en-US/builtin.ftl" "$target_dir/i18n/en-US/builtin.ftl"
  cp "resources/lib/application-repository.toml" "$target_dir/lib/application-repository.toml"

  cp "extra/windows/dist/install.bat" "$target_dir/install.bat"
  cp "extra/windows/dist/uninstall.bat" "$target_dir/uninstall.bat"
  cp "extra/windows/dist/announce_default.ps1" "$target_dir/announce_default.ps1"
  cp "extra/windows/dist/startmenu/user/Browsers.lnk" "$target_dir/startmenu/user/Browsers.lnk"
  cp "extra/windows/dist/startmenu/system/Browsers.lnk" "$target_dir/startmenu/system/Browsers.lnk"
}

make_archives() {
    rm -f "./${target_dir:?}/Browsers_windows.zip"

    rm -f "./${target_dir:?}/browsers_windows.tar.gz"
    rm -f "./${target_dir:?}/browsers_windows.tar.gz.sha256"
    rm -f "./${target_dir:?}/browsers_windows.tar.gz.sig"

    rm -f "./${target_dir:?}/browsers_windows.tar.xz"
    rm -f "./${target_dir:?}/browsers_windows.tar.xz.sha256"
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
      './lib/application-repository.toml'
      './install.bat'
      './uninstall.bat'
      './announce_default.ps1'
      './startmenu/user/Browsers.lnk'
      './startmenu/system/Browsers.lnk'
    )

  cd "./$target_dir"
  zip "./Browsers_windows.zip" "${filelist[@]}"
  cd -

  tar -zcf "./$target_dir/browsers_windows.tar.gz" \
    -C "./$target_dir" \
    "${filelist[@]}"

  create_signatures "$target_dir" "browsers_windows.tar.gz"

  tar -Jcf "./$target_dir/browsers_windows.tar.xz" \
    -C "./$target_dir" \
    "${filelist[@]}"

  create_signatures "$target_dir" "browsers_windows.tar.xz"
}

create_signatures() {
  local target_dir="$1"
  local file_name="$2"

  # creates $filename.sha256
  certutil -hashfile packages.txt SHA256 | sed '2q;d' | sed -z "s/\n//g" > "./$target_dir/$file_name.sha256"

  # creates $filename.sig
  signify -S -s "$APPCAST_SECRET_KEY_FILE" -m "./$target_dir/$file_name"
}

build_binary
build_app_bundle
make_archives