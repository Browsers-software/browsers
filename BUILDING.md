# Build Universal macOS binary

    rustup target add x86_64-apple-darwin
    rustup target add aarch64-apple-darwin

    ./build-mac.sh

# Build Linux binary

## Setup (e.g Ubuntu)
    sudo apt install build-essential
    sudo apt install libpango-1.0-0 libpango1.0-dev libgtk-3-dev

## Setup (e.g Fedora)
    sudo dnf groupinstall "Development Tools"
    sudo dnf install glib2-devel pango-devel cairo-gobject-devel atk-devel gtk3-devel

## Build Natively

    cargo build --release

## Or build via docker image

    cd cross
    ./build-cross-images.sh
    cd ..
    ./build-linux.sh
