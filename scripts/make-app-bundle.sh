#!/usr/bin/env bash
set -euo pipefail

# Usage: scripts/make-app-bundle.sh <target>
# Creates a macOS .app bundle at target/<target>/release/bundle/osx/Sova.app

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 <target>"
    exit 1
fi

TARGET="$1"
REPO_ROOT="$(git rev-parse --show-toplevel)"
BINARY="$REPO_ROOT/target/$TARGET/release/sova-frontend"
ICON="$REPO_ROOT/desktop/assets/Sova.icns"
VERSION="$(grep -m1 '^version' "$REPO_ROOT/Cargo.toml" | sed 's/.*"\(.*\)"/\1/')"

if [[ ! -f "$BINARY" ]]; then
    echo "ERROR: binary not found at $BINARY"
    exit 1
fi

APP_DIR="$REPO_ROOT/target/$TARGET/release/bundle/osx/Sova.app"
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
