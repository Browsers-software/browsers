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

## Or build via docker image

    cargo install cross --git https://github.com/cross-rs/cross

    cd cross
    ./build-cross-images.sh
    cd ..
    ./build-linux.sh
