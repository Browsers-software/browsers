#!/usr/bin/env bash

# exit when any command fails
set -e

# Load in some secrets
if test -f .env; then
  source .env
fi

target_dir='target/universal-apple-darwin/release'

build_binary() {
  # Set minimum macOS version to support older OS versions
  export MACOSX_DEPLOYMENT_TARGET=10.7

  # Build x86_64 binary (also re-creates target/universal-apple-darwin/meta/Info.plist)
  cargo build --target x86_64-apple-darwin --release

  # Build ARM64 binary (also re-creates target/universal-apple-darwin/meta/Info.plist)
  cargo build --target aarch64-apple-darwin --release

  # Clean universal binary and app bundle
  rm -rf "${target_dir:?}/"

  # Build universal binary
  mkdir -p "$target_dir/"
  lipo -create -output "$target_dir/Browsers" target/x86_64-apple-darwin/release/browsers target/aarch64-apple-darwin/release/browsers
}

build_app_bundle() {
  # Build .app bundle
  mkdir -p "$target_dir/Browsers.app/Contents"
  mkdir -p "$target_dir/Browsers.app/Contents/MacOS"
  mkdir -p "$target_dir/Browsers.app/Contents/Resources"
  mkdir -p "$target_dir/Browsers.app/Contents/Resources/i18n/en-US"
  mkdir -p "$target_dir/Browsers.app/Contents/Resources/icons"
  mkdir -p "$target_dir/Browsers.app/Contents/Resources/icons/512x512"
  # FYI: extra/macos/Info.plist is copied
  #      to target/universal-apple-darwin/meta/Info.plist
  #      by build.rs (because it also sets version from Cargo.toml)
  cp target/universal-apple-darwin/meta/Info.plist target/universal-apple-darwin/release/Browsers.app/Contents/Info.plist
  cp extra/macos/icons/Browsers.icns "$target_dir/Browsers.app/Contents/Resources/Browsers.icns"
  cp resources/icons/512x512/software.Browsers.png "$target_dir/Browsers.app/Contents/Resources/icons/512x512/software.Browsers.png"
  cp resources/i18n/en-US/builtin.ftl "$target_dir/Browsers.app/Contents/Resources/i18n/en-US/builtin.ftl"
  cp target/universal-apple-darwin/release/Browsers "$target_dir/Browsers.app/Contents/MacOS/Browsers"
}

sign_app_bundle() {
  # Sign with hardened runtime (hardened runtime is required for notarization)
  rcodesign sign \
    --p12-file "$P12_FILE" \
    --p12-password "$P12_PASSWORD" \
    --code-signature-flags runtime \
    "./$target_dir/Browsers.app"
}

notarize_app_bundle() {
  rcodesign notary-submit \
    --api-key-path "$NOTARY_API_KEY_JSON_FILE" \
    --staple \
    "./$target_dir/Browsers.app"
}

build_dmg() {
  # Build .dmg disk image installer
  cd extra/macos/dmg || exit
  ./mac-dmg-build.sh
  cd ../../../
}

# This is a good format for auto-updating
make_archives() {
  rm -f "./${target_dir:?}/browsers_mac.tar.gz"
  rm -f "./${target_dir:?}/browsers_mac.tar.gz.sig"

  rm -f "./${target_dir:?}/browsers_mac.tar.xz"
  rm -f "./${target_dir:?}/browsers_mac.tar.xz.sig"

  # .tar.gz
  tar -zcf "./$target_dir/browsers_mac.tar.gz" \
    -C "./$target_dir" \
    ./Browsers.app

  # creates browsers_mac.tar.gz.sig
  signify -S -s "$APPCAST_SECRET_KEY_FILE" -m "./$target_dir/browsers_mac.tar.gz"

  # .tar.xz
  tar -Jcf "./$target_dir/browsers_mac.tar.xz" \
    -C "./$target_dir" \
    ./Browsers.app

  # creates browsers_mac.tar.xz.sig
  signify -S -s "$APPCAST_SECRET_KEY_FILE" -m "./$target_dir/browsers_mac.tar.xz"
}

build_binary
build_app_bundle
sign_app_bundle
notarize_app_bundle
build_dmg
make_archives