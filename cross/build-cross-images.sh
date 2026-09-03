docker buildx build --platform linux/amd64 -f Dockerfile.linux_x86_64 -t browsers.software/x86_64-unknown-linux-gnu:local --load .
docker buildx build --platform linux/amd64 -f Dockerfile.linux_aarch64 -t browsers.software/aarch64-unknown-linux-gnu:local --load .
docker buildx build --platform linux/amd64 -f Dockerfile.linux_armv7 -t browsers.software/armv7-unknown-linux-gnueabihf:local --load .
