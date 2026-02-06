#!/bin/bash
set -e

TARGET=$(rustc -vV | grep host | cut -d' ' -f2)
EXT=""
[[ "$TARGET" == *"windows"* ]] && EXT=".exe"

cd "$(dirname "$0")/../.."

# Skip if binary is newer than source
BINARY="gui/src-tauri/binaries/sova_server-${TARGET}${EXT}"
if [ -f "$BINARY" ]; then
  NEWEST_CORE=$(find core/src -name "*.rs" -newer "$BINARY" 2>/dev/null | head -1)
  NEWEST_SERVER=$(find server/src -name "*.rs" -newer "$BINARY" 2>/dev/null | head -1)
  NEWEST_LANGS=$(find langs/src -name "*.rs" -newer "$BINARY" 2>/dev/null | head -1)
  NEWEST_CARGO=$(find server -name "Cargo.toml" -newer "$BINARY" 2>/dev/null | head -1)
  if [ -z "$NEWEST_CORE" ] && [ -z "$NEWEST_SERVER" ] && [ -z "$NEWEST_LANGS" ] && [ -z "$NEWEST_CARGO" ]; then
    echo "Sidecar up-to-date, skipping"
    exit 0
  fi
fi

cargo build -p sova-server --bin sova_server
mkdir -p gui/src-tauri/binaries
cp target/debug/sova_server${EXT} gui/src-tauri/binaries/sova_server-${TARGET}${EXT}
echo "Built sova_server-${TARGET}${EXT} (debug)"
