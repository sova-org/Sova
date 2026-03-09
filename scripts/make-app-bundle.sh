#!/usr/bin/env bash
set -euo pipefail

# Usage: scripts/make-app-bundle.sh <target>
# Creates a macOS .app bundle at target/<target>/release/bundle/osx/Sova.app

if [[ $# -lt 1 || $# -gt 2 ]]; then
    echo "Usage: $0 <target> [--native]"
    exit 1
fi

TARGET="$1"
NATIVE=false
[[ "${2:-}" == "--native" ]] && NATIVE=true

REPO_ROOT="$(git rev-parse --show-toplevel)"
ICON="$REPO_ROOT/desktop/assets/Sova.icns"
VERSION="0.1.0"

if $NATIVE; then
    BINARY="$REPO_ROOT/target/release/sova-frontend"
    APP_DIR="$REPO_ROOT/target/release/bundle/osx/Sova.app"
else
    BINARY="$REPO_ROOT/target/$TARGET/release/sova-frontend"
    APP_DIR="$REPO_ROOT/target/$TARGET/release/bundle/osx/Sova.app"
fi

if [[ ! -f "$BINARY" ]]; then
    echo "ERROR: binary not found at $BINARY"
    exit 1
fi
CONTENTS="$APP_DIR/Contents"
rm -rf "$APP_DIR"
mkdir -p "$CONTENTS/MacOS" "$CONTENTS/Resources"

cp "$BINARY" "$CONTENTS/MacOS/sova-frontend"
[[ -f "$ICON" ]] && cp "$ICON" "$CONTENTS/Resources/Sova.icns"

cat > "$CONTENTS/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>Sova</string>
    <key>CFBundleDisplayName</key>
    <string>Sova</string>
    <key>CFBundleIdentifier</key>
    <string>com.sova.sova</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleExecutable</key>
    <string>sova-frontend</string>
    <key>CFBundleIconFile</key>
    <string>Sova.icns</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.music</string>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright (c) 2025 Raphaël Forment</string>
    <key>NSMicrophoneUsageDescription</key>
    <string>Sova needs microphone access for audio input.</string>
</dict>
</plist>
PLIST

echo "  APP -> $APP_DIR"
