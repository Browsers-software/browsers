# Build Universal macOS binary

    rustup target add x86_64-apple-darwin
    rustup target add aarch64-apple-darwin

    ./build-mac.sh

# Build Linux binary

## Setup (e.g Ubuntu)

    sudo apt install build-essential

## Setup (e.g Fedora)

    sudo dnf groupinstall "Development Tools"

## Build Natively

    cargo build --release

## Or build via zigbuild

    brew install zig
    cargo install --locked cargo-zigbuild
    ./build-linux.sh
