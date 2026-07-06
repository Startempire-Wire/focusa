#!/usr/bin/env bash
# Spec 117 .34 — Beginner Mode aligned with Spec 119 §30 affordance reality.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

BM="$ROOT_DIR/crates/focusa-tui/src/beginner_mode.rs"
MAIN="$ROOT_DIR/crates/focusa-tui/src/main.rs"
[[ -f "$BM" ]] || fail "beginner_mode.rs missing"
for needle in \
  'AFFORDANCE_REALITY_BY_BEGINNER_STATE' \
  'affordance_reality_for' \
  '"disconnected", "unavailable"' \
  '"unbound", "limited"' \
  '"no_workpoint", "limited"' \
  '"no_evidence", "limited"' \
  '"resumable", "possible"' \
  'affordance_reality_matches_spec119'; do
  grep -qF -- "$needle" "$BM" || fail "beginner_mode missing: $needle"
done
pass "beginner_mode map covers all 5 states with affordance reality per Spec 119 §30"
grep -qF 'beginner_mode_affordance_by_state' "$MAIN" || fail "headless proof missing beginner_mode_affordance_by_state"
pass "headless proof exposes beginner_mode_affordance_by_state"
echo "focusa-117 beginner-mode-spec119 static test: PASS"
