#!/usr/bin/env bash
# Spec 117 .32 — Proof Meter + Scope Badge aligned with Spec 119 §30 + §31.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

PS="$ROOT_DIR/crates/focusa-tui/src/views/proof_status.rs"
MAIN="$ROOT_DIR/crates/focusa-tui/src/main.rs"
[[ -f "$PS" ]] || fail "proof_status.rs missing"
for needle in \
  'affordance_reality: &' \
  'precedence_frame: &' \
  'AFFORDANCE_REALITY_POSSIBLE' \
  'AFFORDANCE_REALITY_LIMITED' \
  'AFFORDANCE_REALITY_UNAVAILABLE' \
  'PRECEDENCE_FRAME_PROJECT' \
  'PRECEDENCE_FRAME_AUTHORITY' \
  'PRECEDENCE_FRAME_OPERATOR' \
  'affordance_reality_matches_status' \
  'scope_badge_carries_precedence_frame'; do
  grep -qF -- "$needle" "$PS" || fail "proof_status missing: $needle"
done
pass "proof_status exposes Affordance Reality + Precedence Frame per Spec 119 §30/§31"
grep -qF 'affordance_reality_states' "$MAIN" || fail "headless proof missing affordance_reality_states"
grep -qF 'precedence_frames' "$MAIN" || fail "headless proof missing precedence_frames"
pass "headless proof exposes affordance_reality_states + precedence_frames"
echo "focusa-117 proof-status-spec119 static test: PASS"
