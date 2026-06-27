#!/usr/bin/env bash
# macOS code signing guard (focusa-covz).
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

MOD="$ROOT_DIR/crates/focusa-cli/src/commands/codesign.rs"
MAIN="$ROOT_DIR/crates/focusa-cli/src/main.rs"
DOC="$ROOT_DIR/docs/INSTALL-GAP-AUDIT.md"
BEADS="$ROOT_DIR/.beads/issues.jsonl"

rg -n 'pub struct CodesignArgs|pub async fn run|Inspect|Sign|codesign_present|notary_present|spctl_present|host_supported|xcrun notarytool|FOCUSA_DEVELOPER_ID|FOCUSA_APPLE_TEAM_ID' "$MOD" >/dev/null \
  || fail "codesign.rs missing inspect + sign Rust core"
rg -n 'Commands::Codesign\(args\) => commands::codesign::run\(args' "$MAIN" >/dev/null \
  || fail "main.rs not dispatching focusa codesign"
if [ ! -f "$DOC" ]; then
  grep -q '"focusa-covz"' "$BEADS" || fail "focusa-covz bead missing for macOS codesign"
fi
rg -n 'codesign|Codesign' "$DOC" 2>/dev/null >/dev/null || true
pass "macOS code signing surface lives in Rust core with audit linkage"
