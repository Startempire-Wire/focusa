#!/usr/bin/env bash
# Spec 111 Slices 7+8 — Pi/tool contracts and docs/snapshot acceptance static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

TOOLS_DIR="$ROOT_DIR/docs/focusa-tools/tools"
PI_TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"
PRELOAD_ROUTE="$ROOT_DIR/crates/focusa-api/src/routes/preload.rs"
DOCS_DIR="$ROOT_DIR/docs"
[[ -d "$TOOLS_DIR" ]] || fail "tools dir missing"

for tc in focusa_preload_profiles focusa_preload_build focusa_preload_write focusa_preload_receipt_preview; do
  [[ -f "$TOOLS_DIR/$tc.md" ]] || fail "tool contract missing: $tc.md"
done
pass "Pi tool contracts present for focusa_preload_{profiles,build,write,receipt_preview}"

for profile in rules_only rules_and_context budget_light budget_deep; do
  grep -qF "Type.Literal(\"$profile\")" "$PI_TOOLS" || fail "Pi preload schema missing profile literal: $profile"
  grep -qF "$profile" "$PRELOAD_ROUTE" || fail "preload route missing profile id: $profile"
done
grep -qF 'functionallyFailed' "$PI_TOOLS" || fail "Pi preload tools ignore body-level failed status"
grep -qF 'human_readable' "$PRELOAD_ROUTE" || fail "preload API missing human-readable diagnostics"
pass "preload profile schema and body-level failure semantics are explicit"

[[ -f "$DOCS_DIR/SPEC_111_AGENT_BOOTSTRAP.md" ]] || fail "SPEC_111_AGENT_BOOTSTRAP.md reference doc missing"
grep -qF 'focusa.preload.v1' "$DOCS_DIR/SPEC_111_AGENT_BOOTSTRAP.md" || fail "ref doc missing preload schema"
grep -qF 'FOCUSA_PRELOAD_FAIL' "$DOCS_DIR/SPEC_111_AGENT_BOOTSTRAP.md" || fail "ref doc missing FOCUSA_PRELOAD_FAIL"
pass "Spec 111 reference doc present and aligned"

echo "focusa-111 preload slice7_8 static test: PASS"
