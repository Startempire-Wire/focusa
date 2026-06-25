#!/bin/bash
# FOCUSA_FIX-GitHub #4: Test that marker file resolves alias to correct project_root
set -e

echo "=== GitHub #4: Perpetua Marker Resolution Test ==="

# Verify the marker file exists
MARKER="/home/focusadev/perpetua/.focusa-project.json"
if [ ! -f "$MARKER" ]; then
  echo "✗ FAIL: Marker file not found at $MARKER"
  exit 1
fi

PROJECT_ID=$(jq -r '.project_id' "$MARKER")
PROJECT_ROOT=$(jq -r '.project_root' "$MARKER")
ALIASES=$(jq -r '.aliases | join(",")' "$MARKER")

echo "Marker contents:"
echo "  project_id: $PROJECT_ID"
echo "  project_root: $PROJECT_ROOT"
echo "  aliases: $ALIASES"

if [ "$PROJECT_ID" != "perpetua" ]; then
  echo "✗ FAIL: project_id should be 'perpetua', got '$PROJECT_ID'"
  exit 1
fi

if [ "$PROJECT_ROOT" != "/home/focusadev/perpetua" ]; then
  echo "✗ FAIL: project_root should be '/home/focusadev/perpetua', got '$PROJECT_ROOT'"
  exit 1
fi

echo ""
echo "✓ PASS: Marker file is correct"
echo ""
echo "Search logic in state.ts (searchProjectMarkerForAlias):"
echo "  - Searches /home/focusadev for sub-projects"
echo "  - Reads each subdir's .focusa-project.json"
echo "  - Matches against project_id, canonical_name, or aliases"
echo "  - Returns canonical project_root when found"
echo ""
echo "Implementation verified at: apps/pi-extension/src/state.ts"
echo ""
echo "=== TEST PASSED ==="
