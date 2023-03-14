#!/usr/bin/env bash

# Load in some secrets
source .env || true

build_binary() {
  # Set minimum macOS version to support older OS versions
  export MACOSX_DEPLOYMENT_TARGET=10.7

  # Clean universal binary and app bundle
  rm -rf target/universal-apple-darwin/release/

  # Build ARM64 binary (also re-creates target/universal-apple-darwin/release/Browsers.app/Contents/Info.plist)
  cargo build --target aarch64-apple-darwin --release

  # Build x86_64 binary (also re-creates target/universal-apple-darwin/release/Browsers.app/Contents/Info.plist)
  cargo build --target x86_64-apple-darwin --release

  # Build universal binary
  mkdir -p target/universal-apple-darwin/release/
  lipo -create -output target/universal-apple-darwin/release/Browsers target/x86_64-apple-darwin/release/browsers target/aarch64-apple-darwin/release/browsers
}

build_app_bundle() {
  # Build .app bundle
  mkdir -p target/universal-apple-darwin/release/Browsers.app/Contents
  mkdir -p target/universal-apple-darwin/release/Browsers.app/Contents/MacOS
  mkdir -p target/universal-apple-darwin/release/Browsers.app/Contents/Resources
  mkdir -p target/universal-apple-darwin/release/Browsers.app/Contents/Resources/i18n/en-US
  mkdir -p target/universal-apple-darwin/release/Browsers.app/Contents/Resources/icons
  mkdir -p target/universal-apple-darwin/release/Browsers.app/Contents/Resources/icons/512x512
  # FYI: extra/macos/Info.plist is copied
  #      to target/universal-apple-darwin/release/Browsers.app/Contents/Info.plist
  #      by build.rs (because it also sets version from Cargo.toml)
  cp extra/macos/icons/Browsers.icns target/universal-apple-darwin/release/Browsers.app/Contents/Resources/Browsers.icns
  cp resources/icons/512x512/software.Browsers.png target/universal-apple-darwin/release/Browsers.app/Contents/Resources/icons/512x512/software.Browsers.png
  cp resources/i18n/en-US/builtin.ftl target/universal-apple-darwin/release/Browsers.app/Contents/Resources/i18n/en-US/builtin.ftl
  cp target/universal-apple-darwin/release/Browsers target/universal-apple-darwin/release/Browsers.app/Contents/MacOS/Browsers
}

sign_app_bundle() {
  # Sign with hardened runtime (hardened runtime is required for notarization)
  rcodesign sign \
    --p12-file "$P12_FILE" \
    --p12-password "$P12_PASSWORD" \
    --code-signature-flags runtime \
    "./target/universal-apple-darwin/release/Browsers.app"
}

notarize_app_bundle() {
  rcodesign notary-submit \
    --api-key-path "$NOTARY_API_KEY_JSON_FILE" \
    --staple \
    "./target/universal-apple-darwin/release/Browsers.app"
}

build_dmg() {
  # Build .dmg disk image installer
  cd extra/macos/dmg || exit
  ./mac-dmg-build.sh
  cd ../../../
}

# This is a good format for auto-updating
make_tar_gz() {
  rm -f ./target/universal-apple-darwin/release/browsers_mac.tar.gz

  tar -zcf ./target/universal-apple-darwin/release/browsers_mac.tar.gz \
    -C ./target/universal-apple-darwin/release \
    ./Browsers.app
}

make_tar_gz

build_binary
build_app_bundle
sign_app_bundle
notarize_app_bundle
build_dmg
make_tar_gz