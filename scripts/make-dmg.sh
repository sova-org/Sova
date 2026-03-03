#!/usr/bin/env bash
set -euo pipefail

# Usage: scripts/make-dmg.sh <app-path> <output-dir>
# Produces a .dmg from a macOS .app bundle using only hdiutil.

if [[ $# -ne 2 ]]; then
    echo "Usage: $0 <app-path> <output-dir>"
    exit 1
fi

APP_PATH="$1"
OUTDIR="$2"

if [[ ! -d "$APP_PATH" ]]; then
    echo "ERROR: $APP_PATH is not a directory"
    exit 1
fi

LIPO_OUTPUT=$(lipo -info "$APP_PATH/Contents/MacOS/sova-frontend" 2>/dev/null)

if [[ -z "$LIPO_OUTPUT" ]]; then
    echo "ERROR: could not determine architecture from $APP_PATH"
    exit 1
fi

if echo "$LIPO_OUTPUT" | grep -q "Architectures in the fat file"; then
    ARCH="universal"
else
    ARCH=$(echo "$LIPO_OUTPUT" | awk '{print $NF}')
    case "$ARCH" in
        arm64) ARCH="aarch64" ;;
    esac
fi

STAGING="$(mktemp -d)"
trap 'rm -rf "$STAGING"' EXIT

cp -R "$APP_PATH" "$STAGING/Sova.app"
ln -s /Applications "$STAGING/Applications"

DMG_NAME="Sova-${ARCH}.dmg"
mkdir -p "$OUTDIR"

hdiutil create -volname "Sova" \
    -srcfolder "$STAGING" \
    -ov -format UDZO \
    "$OUTDIR/$DMG_NAME"

echo "  DMG -> $OUTDIR/$DMG_NAME"
