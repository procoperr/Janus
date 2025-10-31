#!/usr/bin/env bash
set -e

# Build release artifacts for Janus
# This script builds release binaries for all supported platforms

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ARTIFACTS_DIR="$PROJECT_ROOT/release-artifacts"

# Colors for output
# No colors needed for cleaner output
RED=''
GREEN=''
YELLOW=''
BLUE=''
NC=''

# Supported targets
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-msvc"
)

echo "Janus Release Builder"
echo "================================"
echo ""

# Parse arguments
BUILD_ALL=false
SIGN_ARTIFACTS=false
SPECIFIC_TARGET=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --all)
            BUILD_ALL=true
            shift
            ;;
        --sign)
            SIGN_ARTIFACTS=true
            shift
            ;;
        --target)
            SPECIFIC_TARGET="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --all           Build for all supported targets"
            echo "  --target TARGET Build for specific target"
            echo "  --sign          Sign artifacts (requires GPG)"
            echo "  --help          Show this help message"
            echo ""
            echo "Supported targets:"
            for target in "${TARGETS[@]}"; do
                echo "  - $target"
            done
            exit 0
            ;;
        *)
            echo "ERROR: Unknown option: $1"
            exit 1
            ;;
    esac
done

# Determine which targets to build
if [ -n "$SPECIFIC_TARGET" ]; then
    TARGETS_TO_BUILD=("$SPECIFIC_TARGET")
elif [ "$BUILD_ALL" = true ]; then
    TARGETS_TO_BUILD=("${TARGETS[@]}")
else
    # Detect current platform
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux*)
            if [ "$ARCH" = "x86_64" ]; then
                TARGETS_TO_BUILD=("x86_64-unknown-linux-gnu")
            fi
            ;;
        Darwin*)
            if [ "$ARCH" = "x86_64" ]; then
                TARGETS_TO_BUILD=("x86_64-apple-darwin")
            elif [ "$ARCH" = "arm64" ]; then
                TARGETS_TO_BUILD=("aarch64-apple-darwin")
            fi
            ;;
        MINGW*|MSYS*|CYGWIN*)
            TARGETS_TO_BUILD=("x86_64-pc-windows-msvc")
            ;;
        *)
            echo "ERROR: Unsupported platform: $OS"
            exit 1
            ;;
    esac
fi

# Clean and create artifacts directory
echo "Preparing artifacts directory..."
rm -rf "$ARTIFACTS_DIR"
mkdir -p "$ARTIFACTS_DIR"

# Get current version from Cargo.toml
VERSION=$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | head -n 1 | cut -d '"' -f 2)
echo "Building version: $VERSION"
echo ""

# Function to build for a target
build_target() {
    local target=$1
    local binary_name="janus"
    local asset_name="janus-$VERSION-$target"

    if [[ "$target" == *"windows"* ]]; then
        binary_name="janus.exe"
    fi

    echo "Building for $target..."

    # Install target if not present
    if ! rustup target list --installed | grep -q "^$target$"; then
        echo "  Installing target $target..."
        rustup target add "$target"
    fi

    # Special handling for musl
    if [[ "$target" == *"musl"* ]]; then
        if ! command -v musl-gcc &> /dev/null; then
            echo "  WARNING: musl-gcc not found. Install musl-tools or musl-dev"
            echo "  Ubuntu/Debian: sudo apt-get install musl-tools"
            echo "  Alpine: apk add musl-dev"
            return 1
        fi
    fi

    # Build
    cd "$PROJECT_ROOT"
    cargo build --release --locked --target "$target" 2>&1 | grep -E "Compiling|Finished|error" || true

    if [ ${PIPESTATUS[0]} -ne 0 ]; then
        echo "  ERROR: Build failed for $target"
        return 1
    fi

    local binary_path="$PROJECT_ROOT/target/$target/release/$binary_name"

    if [ ! -f "$binary_path" ]; then
        echo "  ERROR: Binary not found: $binary_path"
        return 1
    fi

    # Strip binary (if not Windows)
    if [[ "$target" != *"windows"* ]]; then
        echo "  Stripping binary..."
        strip "$binary_path" 2>/dev/null || true
    fi

    # Get binary size
    local size=$(du -h "$binary_path" | cut -f1)
    echo "  SUCCESS: Built successfully (size: $size)"

    # Create archive
    echo "  Creating archive..."
    cd "$PROJECT_ROOT/target/$target/release"

    if [[ "$target" == *"windows"* ]]; then
        # Create zip for Windows
        if command -v zip &> /dev/null; then
            zip -q "$ARTIFACTS_DIR/$asset_name.zip" "$binary_name"
            echo "  SUCCESS: Created $asset_name.zip"
        else
            echo "  WARNING: zip not found, copying binary only"
            cp "$binary_name" "$ARTIFACTS_DIR/$asset_name.exe"
        fi
    else
        # Create tar.gz for Unix
        tar czf "$ARTIFACTS_DIR/$asset_name.tar.gz" "$binary_name"
        echo "  SUCCESS: Created $asset_name.tar.gz"
    fi

    # Sign if requested
    if [ "$SIGN_ARTIFACTS" = true ]; then
        echo "  Signing artifact..."
        cd "$ARTIFACTS_DIR"

        if command -v gpg &> /dev/null; then
            local archive_file
            if [[ "$target" == *"windows"* ]]; then
                archive_file="$asset_name.zip"
            else
                archive_file="$asset_name.tar.gz"
            fi

            gpg --detach-sign --armor "$archive_file" 2>/dev/null || {
                echo "  WARNING: Signing failed"
            }

            if [ -f "$archive_file.asc" ]; then
                echo "  SUCCESS: Signed: $archive_file.asc"
            fi
        else
            echo "  WARNING: GPG not found, skipping signing"
        fi
    fi

    echo ""
    return 0
}

# Build for each target
SUCCESS_COUNT=0
FAIL_COUNT=0

for target in "${TARGETS_TO_BUILD[@]}"; do
    if build_target "$target"; then
        ((SUCCESS_COUNT++))
    else
        ((FAIL_COUNT++))
    fi
done

# Summary
echo "================================"
echo "Build Summary"
echo "================================"
echo "SUCCESS: $SUCCESS_COUNT"
if [ $FAIL_COUNT -gt 0 ]; then
    echo "FAILED: $FAIL_COUNT"
fi
echo ""

if [ $SUCCESS_COUNT -gt 0 ]; then
    echo "Artifacts created in:"
    echo "  $ARTIFACTS_DIR"
    echo ""
    echo "Contents:"
    ls -lh "$ARTIFACTS_DIR" | tail -n +2 | awk '{print "  " $9 " (" $5 ")"}'
    echo ""

    if [ "$SIGN_ARTIFACTS" = true ]; then
        echo "Signatures created"
    fi
fi

if [ $FAIL_COUNT -gt 0 ]; then
    echo "WARNING: Some builds failed. Check the output above for details."
    exit 1
fi

echo "SUCCESS: Release build complete!"
