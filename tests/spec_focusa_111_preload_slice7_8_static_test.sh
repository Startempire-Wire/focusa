#!/usr/bin/env bash
# Spec 111 Slices 7+8 — Pi/tool contracts and docs/snapshot acceptance static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

TOOLS_DIR="$ROOT_DIR/docs/focusa-tools/tools"
DOCS_DIR="$ROOT_DIR/docs"
[[ -d "$TOOLS_DIR" ]] || fail "tools dir missing"

for tc in focusa_preload_profiles focusa_preload_build focusa_preload_write focusa_preload_receipt_preview; do
  [[ -f "$TOOLS_DIR/$tc.md" ]] || fail "tool contract missing: $tc.md"
done
pass "Pi tool contracts present for focusa_preload_{profiles,build,write,receipt_preview}"

[[ -f "$DOCS_DIR/SPEC_111_AGENT_BOOTSTRAP.md" ]] || fail "SPEC_111_AGENT_BOOTSTRAP.md reference doc missing"
grep -qF 'focusa.preload.v1' "$DOCS_DIR/SPEC_111_AGENT_BOOTSTRAP.md" || fail "ref doc missing preload schema"
grep -qF 'FOCUSA_PRELOAD_FAIL' "$DOCS_DIR/SPEC_111_AGENT_BOOTSTRAP.md" || fail "ref doc missing FOCUSA_PRELOAD_FAIL"
pass "Spec 111 reference doc present and aligned"

echo "focusa-111 preload slice7_8 static test: PASS"
