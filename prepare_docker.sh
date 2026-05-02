#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# Function to show usage
usage() {
    echo "Usage: $0 [--host | --arm]"
    echo "  --host: Build for host architecture using cargo"
    echo "  --arm:  Build for ARM (aarch64) using cross"
    exit 1
}

# Parse command line arguments
TARGET=""
while [[ "$#" -gt 0 ]]; do
    case $1 in
        --host) TARGET="host"; shift ;;
        --arm) TARGET="arm"; shift ;;
        *) usage ;;
    esac
done

if [ -z "$TARGET" ]; then
    usage
fi

BUILD_DIR="docker-build"

echo "--- Starting build for $TARGET ---"

if [ "$TARGET" == "arm" ]; then
    echo "Building bird-station for ARM (aarch64-unknown-linux-gnu)..."
    cross build --target aarch64-unknown-linux-gnu --bin bird-station --release
    BINARY_PATH="target/aarch64-unknown-linux-gnu/release/bird-station"
else
    echo "Building bird-station for host..."
    cargo build --bin bird-station --release
    BINARY_PATH="target/release/bird-station"
fi

echo "Preparing $BUILD_DIR directory..."
mkdir -p "$BUILD_DIR"

cp -v "$BINARY_PATH" "$BUILD_DIR/"
cp -v station/python/classifier.py "$BUILD_DIR/"
cp -v station/python/requirements.txt "$BUILD_DIR/"
cp -v -r station/model_files/* "$BUILD_DIR/"
cp -v station/Dockerfile "$BUILD_DIR/"

echo "--- Docker image successfully preparted ---"
echo ""
echo "To build the docker image, run:"
echo "   docker build -t birb $BUILD_DIR"
echo ""
echo "And run the image with:"
echo "   docker run -v /run/user/1000/pipewire-0:/tmp/pipewire-0 -e XDG_RUNTIME_DIR=/tmp -p 8128:8128 -t birb"
