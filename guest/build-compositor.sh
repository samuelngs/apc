#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORKSPACE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
ARCH="${1:-aarch64}"

case "$ARCH" in
    aarch64|arm64) DOCKER_PLATFORM="linux/arm64" ;;
    x86_64|amd64)  DOCKER_PLATFORM="linux/amd64" ;;
    *) echo "Usage: $0 [aarch64|x86_64]"; exit 1 ;;
esac

OUT_DIR="$SCRIPT_DIR/out/$ARCH"
mkdir -p "$OUT_DIR"

echo "==> Building agentos-compositor (arch=$ARCH)"

docker run --rm \
    --platform "$DOCKER_PLATFORM" \
    -v "$WORKSPACE_DIR:/work" \
    -v cargo-registry:/usr/local/cargo/registry \
    -w /work \
    rust:alpine3.21 \
    sh -c '
set -eux
apk add --no-cache \
    musl-dev \
    pkgconf \
    libdrm-dev \
    mesa-dev \
    libinput-dev \
    libseat-dev \
    eudev-dev \
    wayland-dev \
    libxkbcommon-dev \
    pixman-dev \
    linux-headers

RUSTFLAGS="-C target-feature=-crt-static" cargo build --release -p agentos-compositor 2>&1
cp target/release/agentos-compositor /work/guest/out/'"$ARCH"'/
'

echo "==> Compositor built: $OUT_DIR/agentos-compositor"
ls -lh "$OUT_DIR/agentos-compositor"
