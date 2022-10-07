#!/usr/bin/env bash

cross build --target x86_64-unknown-linux-gnu --release
cross build --target aarch64-unknown-linux-gnu --release

# Build universal binary
mkdir -p target/universal-unknown-linux-gnu/release/
mkdir -p target/universal-unknown-linux-gnu/release/x86_64/
mkdir -p target/universal-unknown-linux-gnu/release/aarch64/

cp "target/x86_64-unknown-linux-gnu/release/browsers" "target/universal-unknown-linux-gnu/release/x86_64/browsers"
cp "target/aarch64-unknown-linux-gnu/release/browsers" "target/universal-unknown-linux-gnu/release/aarch64/browsers"

cp "extra/linux/dist/install.sh" "target/universal-unknown-linux-gnu/release/install.sh"
cp "extra/linux/dist/software.Browsers.desktop" "target/universal-unknown-linux-gnu/release/software.Browsers.desktop"

rm -f ./target/universal-unknown-linux-gnu/release/browsers_linux.tar.gz

tar -zcf ./target/universal-unknown-linux-gnu/release/browsers_linux.tar.gz -C ./target/universal-unknown-linux-gnu/release ./x86_64/browsers \
  ./aarch64/browsers \
  ./software.Browsers.desktop \
  ./install.sh
