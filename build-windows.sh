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
  cp "extra/windows/dist/install.bat" "$target_dir/install.bat"
  cp "extra/windows/dist/announce_default.ps1" "$target_dir/announce_default.ps1"
  cp "extra/windows/dist/uninstall.bat" "$target_dir/uninstall.bat"

  mkdir -p "$target_dir/resources/icons/"

  for size in 16 32 64 128 256 512; do
    mkdir -p "$target_dir/resources/icons/${size}x${size}"
    cp "resources/icons/${size}x${size}/software.Browsers.png" "$target_dir/resources/icons/${size}x${size}/software.Browsers.png"
  done

  mkdir -p "$target_dir/resources/i18n"
  mkdir -p "$target_dir/resources/i18n/en-US"
  cp "resources/i18n/en-US/builtin.ftl" "$target_dir/resources/i18n/en-US/builtin.ftl"

  mkdir -p "$target_dir/resources/repository"
  cp "resources/repository/application-repository.toml" "$target_dir/resources/repository/application-repository.toml"

  mkdir -p "$target_dir/startmenu"
  mkdir -p "$target_dir/startmenu/user"
  cp "extra/windows/dist/startmenu/user/Browsers.lnk" "$target_dir/startmenu/user/Browsers.lnk"

  mkdir -p "$target_dir/startmenu/system"
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
      './resources/icons/16x16/software.Browsers.png'
      './resources/icons/32x32/software.Browsers.png'
      './resources/icons/64x64/software.Browsers.png'
      './resources/icons/128x128/software.Browsers.png'
      './resources/icons/256x256/software.Browsers.png'
      './resources/icons/512x512/software.Browsers.png'
      './resources/i18n/en-US/builtin.ftl'
      './resources/repository/application-repository.toml'
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