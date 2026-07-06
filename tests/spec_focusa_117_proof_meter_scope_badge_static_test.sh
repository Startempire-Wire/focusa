#!/usr/bin/env bash
# Spec 117 §6.7 / §14.2 — Proof Meter + Scope Badge static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

PROOF="$ROOT_DIR/crates/focusa-tui/src/views/proof_status.rs"
MAIN="$ROOT_DIR/crates/focusa-tui/src/main.rs"
HOME="$ROOT_DIR/crates/focusa-tui/src/views/deck_home.rs"
MOD="$ROOT_DIR/crates/focusa-tui/src/views/mod.rs"

[[ -f "$PROOF" ]] || fail "proof_status.rs missing"

for needle in \
  'ProofMeter' \
  'ScopeBadge' \
  'PROOF_METER_STATES' \
  'none:[-----]' \
  'linked:[##---]' \
  'verified:[#####]' \
  'SCOPE_BADGE_STATES' \
  'canonical' \
  'advisory' \
  'blocked' \
  'unbound'; do
  grep -qF -- "$needle" "$PROOF" || fail "proof/scope model missing: $needle"
done
pass "proof meter and scope badge model covers required states"

grep -qF 'pub mod proof_status;' "$MOD" || fail "views mod missing proof_status export"
grep -qF 'proof_status::proof_meter(app)' "$HOME" || fail "Deck Home missing proof meter render"
grep -qF 'proof_status::scope_badge(app)' "$HOME" || fail "Deck Home missing scope badge render"
grep -qF 'proof_meter_states' "$MAIN" || fail "headless proof missing proof_meter_states"
grep -qF 'scope_badge_states' "$MAIN" || fail "headless proof missing scope_badge_states"
pass "proof meter/scope badge wired into Deck Home and headless proof"

echo "focusa-117 proof-meter-scope-badge static test: PASS"
