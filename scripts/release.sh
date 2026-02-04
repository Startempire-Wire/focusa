#!/usr/bin/env bash
# Focusa release build script.
#
# Builds: daemon binary, CLI binary, (optionally) Tauri app.

set -euo pipefail
cd "$(dirname "$0")/.."

echo "Building release binaries..."
cargo build --release --workspace

echo ""
echo "Built artifacts:"
ls -la target/release/focusa target/release/focusa-daemon 2>/dev/null || echo "(binaries will appear after first build)"

echo ""
echo "✓ Release build complete"
