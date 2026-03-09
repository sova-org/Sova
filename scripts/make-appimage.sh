#!/usr/bin/env bash
set -euo pipefail

# Usage: scripts/make-appimage.sh <binary-path> <arch> <output-dir>
# Produces an AppImage from a Linux binary.
# On native Linux with matching arch: uses linuxdeploy.
# Otherwise (cross-compilation): builds AppImage via mksquashfs in Docker.

if [[ $# -ne 3 ]]; then
    echo "Usage: $0 <binary-path> <arch> <output-dir>"
    exit 1
fi

BINARY="$1"
ARCH="$2"
OUTDIR="$3"

REPO_ROOT="$(git rev-parse --show-toplevel)"
CACHE_DIR="$REPO_ROOT/.cache"
APP_NAME="$(basename "$BINARY")"

RUNTIME_URL="https://github.com/AppImage/type2-runtime/releases/download/continuous/runtime-${ARCH}"
RUNTIME="$CACHE_DIR/runtime-${ARCH}"

build_appdir() {
    local appdir="$1"
    mkdir -p "$appdir/usr/bin"
    cp "$BINARY" "$appdir/usr/bin/$APP_NAME"
    chmod +x "$appdir/usr/bin/$APP_NAME"

    mkdir -p "$appdir/usr/share/icons/hicolor/512x512/apps"
    cp "$REPO_ROOT/desktop/assets/icon.png" "$appdir/usr/share/icons/hicolor/512x512/apps/sova.png"

    cp "$REPO_ROOT/desktop/assets/sova.desktop" "$appdir/sova.desktop"

    cat > "$appdir/AppRun" <<APPRUN
#!/bin/sh
SELF="\$(readlink -f "\$0")"
HERE="\$(dirname "\$SELF")"
exec "\$HERE/usr/bin/$APP_NAME" "\$@"
APPRUN
    chmod +x "$appdir/AppRun"

    ln -sf usr/share/icons/hicolor/512x512/apps/sova.png "$appdir/sova.png"
    ln -sf sova.desktop "$appdir/.DirIcon" 2>/dev/null || true
}

download_runtime() {
    mkdir -p "$CACHE_DIR"
    if [[ ! -f "$RUNTIME" ]]; then
        echo "  Downloading AppImage runtime for $ARCH..."
        curl -fSL "$RUNTIME_URL" -o "$RUNTIME"
    fi
}

run_native() {
    local linuxdeploy_url="https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-${ARCH}.AppImage"
    local linuxdeploy="$CACHE_DIR/linuxdeploy-${ARCH}.AppImage"

    mkdir -p "$CACHE_DIR"
    if [[ ! -f "$linuxdeploy" ]]; then
        echo "  Downloading linuxdeploy for $ARCH..."
        curl -fSL "$linuxdeploy_url" -o "$linuxdeploy"
        chmod +x "$linuxdeploy"
    fi

    local appdir
    appdir="$(mktemp -d)/AppDir"
    build_appdir "$appdir"

    export ARCH
    export LDAI_RUNTIME_FILE="$RUNTIME"
    "$linuxdeploy" \
        --appimage-extract-and-run \
        --appdir "$appdir" \
        --desktop-file "$appdir/sova.desktop" \
        --icon-file "$appdir/usr/share/icons/hicolor/512x512/apps/sova.png" \
        --output appimage

    local appimage
    appimage=$(ls -1t ./*.AppImage 2>/dev/null | head -1 || true)
    if [[ -z "$appimage" ]]; then
        echo "  ERROR: No AppImage produced"
        exit 1
    fi
    mkdir -p "$OUTDIR"
    mv "$appimage" "$OUTDIR/${APP_NAME}-linux-${ARCH}.AppImage"
    echo "  AppImage -> $OUTDIR/${APP_NAME}-linux-${ARCH}.AppImage"
}

run_docker() {
    local platform
    case "$ARCH" in
        x86_64)  platform="linux/amd64" ;;
        aarch64) platform="linux/arm64" ;;
        *) echo "Unsupported arch: $ARCH"; exit 1 ;;
    esac

    local appdir
    appdir="$(mktemp -d)/AppDir"
    build_appdir "$appdir"

    local image_tag="sova-appimage-${ARCH}"

    echo "  Building Docker image $image_tag ($platform)..."
    docker build --platform "$platform" -q -t "$image_tag" - <<'DOCKERFILE'
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y --no-install-recommends \
        squashfs-tools \
    && rm -rf /var/lib/apt/lists/*
DOCKERFILE

    echo "  Creating squashfs via Docker ($image_tag)..."
    docker run --rm --platform "$platform" \
        -v "$appdir:/appdir:ro" \
        -v "$CACHE_DIR:/cache" \
        "$image_tag" \
        mksquashfs /appdir /cache/appimage-${ARCH}.squashfs \
            -root-owned -noappend -comp gzip -no-progress

    mkdir -p "$OUTDIR"
    local final="$OUTDIR/${APP_NAME}-linux-${ARCH}.AppImage"
    cat "$RUNTIME" "$CACHE_DIR/appimage-${ARCH}.squashfs" > "$final"
    chmod +x "$final"
    rm -f "$CACHE_DIR/appimage-${ARCH}.squashfs"
    echo "  AppImage -> $final"
}

HOST_ARCH="$(uname -m)"

download_runtime

echo "  Building AppImage for ${APP_NAME} ($ARCH)..."

if [[ "$HOST_ARCH" == "$ARCH" ]] && [[ "$(uname -s)" == "Linux" ]]; then
    run_native
else
    run_docker
fi
