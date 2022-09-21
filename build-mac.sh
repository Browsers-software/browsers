#!/usr/bin/env bash

# Load in some secrets
source .env

build_binary() {
  # Set minimum macOS version to support older OS versions
  export MACOSX_DEPLOYMENT_TARGET=10.7

  # Build ARM64 binary
  cargo build --target aarch64-apple-darwin --release

  # Build x86_64 binary
  cargo build --target x86_64-apple-darwin --release

  # Clean universal binary and app bundle
  rm -r target/universal-apple-darwin/release/

  # Build universal binary
  mkdir -p target/universal-apple-darwin/release/
  lipo -create -output target/universal-apple-darwin/release/Browsers target/x86_64-apple-darwin/release/browsers target/aarch64-apple-darwin/release/browsers
}

build_app_bundle() {
  # Build .app bundle
  mkdir -p target/universal-apple-darwin/release/Browsers.app/Contents
  mkdir -p target/universal-apple-darwin/release/Browsers.app/Contents/MacOS
  mkdir -p target/universal-apple-darwin/release/Browsers.app/Contents/Resources
  cp extra/macos/Info.plist target/universal-apple-darwin/release/Browsers.app/Contents/Info.plist
  cp extra/macos/icons/Browsers.icns target/universal-apple-darwin/release/Browsers.app/Contents/Resources/Browsers.icns
  cp target/universal-apple-darwin/release/Browsers target/universal-apple-darwin/release/Browsers.app/Contents/MacOS/Browsers
}

sign() {
  # Sign with hardened runtime (hardened runtime is required for notarization)
  rcodesign sign \
    --p12-file "$P12_FILE" \
    --p12-password "$P12_PASSWORD" \
    --code-signature-flags runtime \
    "./target/universal-apple-darwin/release/Browsers.app"
}

notarize() {
  rcodesign notary-submit \
    --api-key-path ~/.appstoreconnect/key.json \
    --staple \
    "./target/universal-apple-darwin/release/Browsers.app"
}

build_dmg() {
  # Build .dmg disk image installer
  cd extra/macos/dmg || exit
  ./mac-dmg-build.sh
  cd ../../../
}

build_binary
build_app_bundle
sign
notarize
build_dmg