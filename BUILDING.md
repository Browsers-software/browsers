# Build Universal macOS binary

    rustup target add x86_64-apple-darwin
    rustup target add aarch64-apple-darwin

    ./build-mac.sh

# Build Linux binary
    
    cd cross
    ./build-cross-images.sh
    cd..

    ./build-linux.sh
