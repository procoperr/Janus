#!/usr/bin/env bash
set -e

# Basic Janus Sync Example
# This script demonstrates common Janus workflows

echo "Janus Basic Sync Example"
echo "============================"
echo ""

# Create temporary directories for demonstration
DEMO_DIR=$(mktemp -d -t janus-demo-XXXXXX)
SOURCE_DIR="$DEMO_DIR/source"
DEST_DIR="$DEMO_DIR/dest"

echo "Creating demo directories:"
echo "   Source: $SOURCE_DIR"
echo "   Dest:   $DEST_DIR"
echo ""

mkdir -p "$SOURCE_DIR"
mkdir -p "$DEST_DIR"

# Cleanup function
cleanup() {
    echo ""
    echo "Cleaning up demo directories..."
    rm -rf "$DEMO_DIR"
    echo "Cleanup complete"
}

trap cleanup EXIT

# Create sample files in source
echo "Creating sample files in source..."
echo "Hello, Janus!" > "$SOURCE_DIR/file1.txt"
echo "This is a test file" > "$SOURCE_DIR/file2.txt"
mkdir -p "$SOURCE_DIR/subdir"
echo "Nested content" > "$SOURCE_DIR/subdir/nested.txt"

# Create a larger file to demonstrate streaming
dd if=/dev/zero of="$SOURCE_DIR/largefile.bin" bs=1M count=10 2>/dev/null
echo "   - Created 10MB test file"
echo ""

# Step 1: Scan source directory
echo "Step 1: Scanning source directory"
echo "-------------------------------------"
janus scan "$SOURCE_DIR"
echo ""

# Step 2: Initial sync (destination is empty)
echo "Step 2: Initial sync (destination is empty)"
echo "-----------------------------------------------"
janus diff "$SOURCE_DIR" "$DEST_DIR"
echo ""
echo "Syncing..."
janus sync "$SOURCE_DIR" "$DEST_DIR" -y --no-progress
echo "Initial sync complete"
echo ""

# Step 3: Verify sync
echo "Step 3: Verify sync (should show no changes)"
echo "------------------------------------------------"
janus diff "$SOURCE_DIR" "$DEST_DIR"
echo ""

# Step 4: Modify a file
echo "Step 4: Modifying a file"
echo "----------------------------"
echo "Updated content" > "$SOURCE_DIR/file1.txt"
echo "   - Modified file1.txt"
echo ""

janus diff "$SOURCE_DIR" "$DEST_DIR"
echo ""
echo "Syncing changes..."
janus sync "$SOURCE_DIR" "$DEST_DIR" -y --no-progress
echo "Changes synced"
echo ""

# Step 5: Add a new file
echo "Step 5: Adding a new file"
echo "----------------------------"
echo "Brand new file" > "$SOURCE_DIR/new_file.txt"
echo "   - Created new_file.txt"
echo ""

janus diff "$SOURCE_DIR" "$DEST_DIR"
echo ""
echo "Syncing new file..."
janus sync "$SOURCE_DIR" "$DEST_DIR" -y --no-progress
echo "New file synced"
echo ""

# Step 6: Rename detection
echo "Step 6: Rename detection"
echo "---------------------------"
mv "$SOURCE_DIR/file2.txt" "$SOURCE_DIR/file2_renamed.txt"
echo "   - Renamed file2.txt -> file2_renamed.txt"
echo ""

janus diff "$SOURCE_DIR" "$DEST_DIR" --detailed
echo ""
echo "Syncing rename..."
janus sync "$SOURCE_DIR" "$DEST_DIR" -y --no-progress
echo "Rename synced (no data copied!)"
echo ""

# Step 7: Remove a file (with delete option)
echo "Step 7: Deleting files"
echo "--------------------------"
rm "$SOURCE_DIR/new_file.txt"
echo "   - Removed new_file.txt from source"
echo ""

janus diff "$SOURCE_DIR" "$DEST_DIR"
echo ""
echo "Syncing with --delete option..."
janus sync "$SOURCE_DIR" "$DEST_DIR" --delete -y --no-progress
echo "Removed files deleted from destination"
echo ""

# Step 8: Dry run demonstration
echo "Step 8: Dry run demonstration"
echo "--------------------------------"
echo "New content for dry run" > "$SOURCE_DIR/file1.txt"
echo "   - Modified file1.txt"
echo ""

echo "Running dry-run (no changes will be made)..."
janus dry-run "$SOURCE_DIR" "$DEST_DIR"
echo ""

echo "Verify file1.txt is still the old version in destination:"
cat "$DEST_DIR/file1.txt"
echo ""

# Step 9: Final sync
echo "Step 9: Final sync"
echo "---------------------"
janus sync "$SOURCE_DIR" "$DEST_DIR" -y --no-progress
echo "Final sync complete"
echo ""

# Summary
echo "=============================="
echo "Demo Complete!"
echo "=============================="
echo ""
echo "This demo covered:"
echo "  Initial sync of empty destination"
echo "  Detecting and syncing modifications"
echo "  Adding new files"
echo "  Rename detection (saves copying!)"
echo "  Deleting removed files"
echo "  Dry-run mode"
echo ""
echo "You're ready to use Janus for real synchronization!"
echo ""
echo "Tip: Use 'janus --help' to see all available options"
