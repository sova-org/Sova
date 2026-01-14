#!/bin/bash
set -e

TARGET=$(rustc -vV | grep host | cut -d' ' -f2)
EXT=""
[[ "$TARGET" == *"windows"* ]] && EXT=".exe"

cd "$(dirname "$0")/../.."
cargo build --release -p sova-server --bin sova_server

mkdir -p gui/src-tauri/binaries
cp target/release/sova_server${EXT} gui/src-tauri/binaries/sova_server-${TARGET}${EXT}

echo "Built sova_server-${TARGET}${EXT}"
