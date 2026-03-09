#!/usr/bin/env bash
set -euo pipefail

export MACOSX_DEPLOYMENT_TARGET="12.0"

cd "$(git rev-parse --show-toplevel)"

OUT="releases"

PLATFORMS=(
    "aarch64-apple-darwin"
    "x86_64-apple-darwin"
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "x86_64-pc-windows-gnu"
)

PLATFORM_LABELS=(
    "macOS aarch64 (native)"
    "macOS x86_64 (native)"
    "Linux x86_64 (cross)"
    "Linux aarch64 (cross)"
    "Windows x86_64 (cross)"
)

PLATFORM_ALIASES=(
    "macos-arm64"
    "macos-x86_64"
    "linux-x86_64"
    "linux-aarch64"
    "windows-x86_64"
)

# --- CLI argument parsing ---

cli_platforms=""
cli_targets=""
cli_yes=false
cli_all=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --platforms) cli_platforms="$2"; shift 2 ;;
        --targets)   cli_targets="$2"; shift 2 ;;
        --yes)       cli_yes=true; shift ;;
        --all)       cli_all=true; shift ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --platforms <list>  Comma-separated: macos-arm64,macos-x86_64,linux-x86_64,linux-aarch64,windows-x86_64"
            echo "  --targets <list>    Comma-separated: server,desktop"
            echo "  --all               Build all platforms and targets"
            echo "  --yes               Skip confirmation prompt"
            echo ""
            echo "Without options, runs interactively."
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

resolve_platform_alias() {
    local alias="$1"
    for i in "${!PLATFORM_ALIASES[@]}"; do
        if [[ "${PLATFORM_ALIASES[$i]}" == "$alias" ]]; then
            echo "$i"
            return
        fi
    done
    echo "Unknown platform: $alias" >&2
    exit 1
}

# --- Helpers ---

prompt_platforms() {
    echo "Select platform (0=all, comma-separated):"
    echo "  0) All"
    for i in "${!PLATFORMS[@]}"; do
        echo "  $((i+1))) ${PLATFORM_LABELS[$i]}"
    done
    read -rp "> " choice

    if [[ "$choice" == "0" || -z "$choice" ]]; then
        selected_platforms=("${PLATFORMS[@]}")
        selected_labels=("${PLATFORM_LABELS[@]}")
    else
        IFS=',' read -ra indices <<< "$choice"
        selected_platforms=()
        selected_labels=()
        for idx in "${indices[@]}"; do
            idx="${idx// /}"
            idx=$((idx - 1))
            if (( idx < 0 || idx >= ${#PLATFORMS[@]} )); then
                echo "Invalid platform index: $((idx+1))"
                exit 1
            fi
            selected_platforms+=("${PLATFORMS[$idx]}")
            selected_labels+=("${PLATFORM_LABELS[$idx]}")
        done
    fi
}

prompt_targets() {
    echo ""
    echo "Select targets (0=all, comma-separated):"
    echo "  0) All"
    echo "  1) sova-server"
    echo "  2) sova-desktop"
    read -rp "> " choice

    build_server=false
    build_desktop=false

    if [[ "$choice" == "0" || -z "$choice" ]]; then
        build_server=true
        build_desktop=true
    else
        IFS=',' read -ra targets <<< "$choice"
        for t in "${targets[@]}"; do
            t="${t// /}"
            case "$t" in
                1) build_server=true ;;
                2) build_desktop=true ;;
                *) echo "Invalid target: $t"; exit 1 ;;
            esac
        done
    fi
}

confirm_summary() {
    echo ""
    echo "=== Build Summary ==="
    echo ""
    echo "Platforms:"
    for label in "${selected_labels[@]}"; do
        echo "  - $label"
    done
    echo ""
    echo "Targets:"
    $build_server  && echo "  - sova-server"
    $build_desktop && echo "  - sova-desktop"
    echo ""
    read -rp "Proceed? [Y/n] " yn
    case "${yn,,}" in
        n|no) echo "Aborted."; exit 0 ;;
    esac
}

platform_os() {
    case "$1" in
        *windows*) echo "windows" ;;
        *linux*)   echo "linux" ;;
        *apple*)   echo "macos" ;;
    esac
}

platform_arch() {
    case "$1" in
        aarch64*) echo "aarch64" ;;
        x86_64*)  echo "x86_64" ;;
    esac
}

platform_suffix() {
    case "$1" in
        *windows*) echo ".exe" ;;
        *)         echo "" ;;
    esac
}

is_cross_target() {
    case "$1" in
        *linux*|*windows*) return 0 ;;
        *)                 return 1 ;;
    esac
}

native_target() {
    [[ "$1" == "aarch64-apple-darwin" ]]
}

release_dir() {
    if native_target "$1"; then
        echo "target/release"
    else
        echo "target/$1/release"
    fi
}

target_flag() {
    if native_target "$1"; then
        echo ""
    else
        echo "--target $1"
    fi
}

builder_for() {
    if is_cross_target "$1"; then
        echo "cross"
    else
        echo "cargo"
    fi
}

build_binary() {
    local platform="$1"
    shift
    local builder
    builder=$(builder_for "$platform")
    local tf
    tf=$(target_flag "$platform")
    # shellcheck disable=SC2086
    $builder build --release $tf "$@"
}

copy_artifacts() {
    local platform="$1"
    local rd
    rd=$(release_dir "$platform")
    local os
    os=$(platform_os "$platform")
    local arch
    arch=$(platform_arch "$platform")
    local suffix
    suffix=$(platform_suffix "$platform")

    if $build_server; then
        local src="$rd/sova_server${suffix}"
        local dst="$OUT/sova_server-${os}-${arch}${suffix}"
        cp "$src" "$dst"
        echo "    sova_server -> $dst"
    fi

    if $build_desktop; then
        local src="$rd/sova-frontend${suffix}"
        local dst="$OUT/sova-frontend-${os}-${arch}${suffix}"
        cp "$src" "$dst"
        echo "    sova-frontend -> $dst"

        if [[ "$os" == "macos" ]]; then
            local native_flag=""
            native_target "$platform" && native_flag="--native"
            # shellcheck disable=SC2086
            scripts/make-app-bundle.sh "$platform" $native_flag
            local app_src="$rd/bundle/osx/Sova.app"
            if [[ ! -d "$app_src" ]]; then
                echo "    ERROR: .app bundle not found at $app_src"
                return 1
            fi
            local app_dst="$OUT/Sova-${arch}.app"
            rm -rf "$app_dst"
            cp -R "$app_src" "$app_dst"
            echo "    Sova.app -> $app_dst"
            scripts/make-dmg.sh "$app_dst" "$OUT"
        fi
    fi

    if [[ "$os" == "linux" ]]; then
        if $build_server; then
            scripts/make-appimage.sh "$rd/sova_server" "$arch" "$OUT"
        fi
        if $build_desktop; then
            scripts/make-appimage.sh "$rd/sova-frontend" "$arch" "$OUT"
        fi
    fi
}

# --- Main ---

if $cli_all; then
    selected_platforms=("${PLATFORMS[@]}")
    selected_labels=("${PLATFORM_LABELS[@]}")
    build_server=true
    build_desktop=true
elif [[ -n "$cli_platforms" || -n "$cli_targets" ]]; then
    if [[ -n "$cli_platforms" ]]; then
        selected_platforms=()
        selected_labels=()
        IFS=',' read -ra aliases <<< "$cli_platforms"
        for alias in "${aliases[@]}"; do
            alias="${alias// /}"
            idx=$(resolve_platform_alias "$alias")
            selected_platforms+=("${PLATFORMS[$idx]}")
            selected_labels+=("${PLATFORM_LABELS[$idx]}")
        done
    else
        selected_platforms=("${PLATFORMS[@]}")
        selected_labels=("${PLATFORM_LABELS[@]}")
    fi

    build_server=false
    build_desktop=false
    if [[ -n "$cli_targets" ]]; then
        IFS=',' read -ra tgts <<< "$cli_targets"
        for t in "${tgts[@]}"; do
            t="${t// /}"
            case "$t" in
                server)  build_server=true ;;
                desktop) build_desktop=true ;;
                *) echo "Unknown target: $t (expected: server, desktop)"; exit 1 ;;
            esac
        done
    else
        build_server=true
        build_desktop=true
    fi
else
    prompt_platforms
    prompt_targets
fi

if ! $cli_yes && [[ -z "$cli_platforms" ]] && ! $cli_all; then
    confirm_summary
fi

mkdir -p "$OUT"

step=0
total=${#selected_platforms[@]}

for platform in "${selected_platforms[@]}"; do
    step=$((step + 1))
    echo ""
    echo "=== [$step/$total] $platform ==="

    if $build_server; then
        echo "  -> sova-server"
        build_binary "$platform" -p sova-server
    fi

    if $build_desktop; then
        echo "  -> sova-desktop"
        build_binary "$platform" -p sova-desktop
    fi

    echo "  Copying artifacts..."
    copy_artifacts "$platform"
done

echo ""
echo "=== Done ==="
echo ""
ls -lhR "$OUT/"
