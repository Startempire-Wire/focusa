#!/usr/bin/env bash
# Guard: release.yml must publish signed Tauri menubar .app artifacts for tag releases.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKFLOW="$ROOT_DIR/.github/workflows/release.yml"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[ -f "$WORKFLOW" ] || fail "release workflow missing"

for token in \
  'Package signed macOS app bundles' \
  'ditto -c -k --keepParent' \
  '.app.zip' \
  'bundle/dmg/*.dmg' \
  'Upload signed menubar artifacts to workflow' \
  'actions/upload-artifact@v4' \
  'Upload signed menubar artifacts to release' \
  'softprops/action-gh-release@v2' \
  'dist/menubar/*' \
  'if-no-files-found: error'; do
  grep -Fq "$token" "$WORKFLOW" || fail "missing release artifact token: $token"
done

python3 - <<'PY' "$WORKFLOW"
import sys
from pathlib import Path
text = Path(sys.argv[1]).read_text()
package = text.index('Package signed macOS app bundles')
workflow_upload = text.index('Upload signed menubar artifacts to workflow')
release_upload = text.index('Upload signed menubar artifacts to release')
rust_release = text.index('# Publish Rust binaries')
assert package < workflow_upload < release_upload < rust_release, 'signed app packaging/upload steps must stay in tauri-build before rust-release'
PY

pass "release.yml publishes signed menubar .app artifacts"
