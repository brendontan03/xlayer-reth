#!/usr/bin/env bash
set -euo pipefail

# Quick Compile Time Test
# This script does a quick single compile time measurement
# without cleaning between builds - useful for incremental compile testing

# Parse arguments
USE_DEFAULT_LINKER=false
PROFILE="release"
PACKAGE="xlayer-reth-node"

while [[ $# -gt 0 ]]; do
    case $1 in
        --default-linker)
            USE_DEFAULT_LINKER=true
            shift
            ;;
        *)
            if [ "$PROFILE" == "release" ] && [[ ! "$1" =~ ^-- ]]; then
                PROFILE="$1"
            elif [ "$PACKAGE" == "xlayer-reth-node" ] && [[ ! "$1" =~ ^-- ]]; then
                PACKAGE="$1"
            fi
            shift
            ;;
    esac
done

CARGO_CONFIG=".cargo/config.toml"
TEMP_BACKUP=".cargo/config.toml.tmp"

echo "=========================================="
echo "Quick Compile Test"
echo "Package: $PACKAGE"
echo "Profile: $PROFILE"
if [ "$USE_DEFAULT_LINKER" = true ]; then
    echo "Linker: DEFAULT (system linker)"
else
    echo "Linker: CUSTOM (from .cargo/config.toml)"
fi
echo "=========================================="
echo ""

# Temporarily disable custom linker if requested
if [ "$USE_DEFAULT_LINKER" = true ] && [ -f "$CARGO_CONFIG" ]; then
    echo "Temporarily disabling custom linker configuration..."
    mv "$CARGO_CONFIG" "$TEMP_BACKUP"
    echo ""
fi

# Cleanup function to restore config on exit
cleanup() {
    if [ -f "$TEMP_BACKUP" ]; then
        mv "$TEMP_BACKUP" "$CARGO_CONFIG"
        echo "Restored .cargo/config.toml"
    fi
}
trap cleanup EXIT

# Show which linker is configured
if [ -f "$CARGO_CONFIG" ]; then
    echo "Current linker configuration:"
    grep -A 2 "\[target\." "$CARGO_CONFIG" | head -20
    echo ""
else
    echo "Using system default linker (no .cargo/config.toml)"
    echo ""
fi

echo "Starting compilation..."
echo ""

# Use GNU time if available
if command -v gtime &> /dev/null; then
    gtime -f "\nCompilation completed!\nWall time: %E\nCPU usage: %P\nMax memory: %M KB" \
        cargo build --profile "$PROFILE" -p "$PACKAGE"
else
    { time cargo build --profile "$PROFILE" -p "$PACKAGE"; } 2>&1
fi
