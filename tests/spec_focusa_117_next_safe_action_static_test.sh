#!/usr/bin/env bash
# Spec 117 §6.5 — Next Safe Action static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

NSA="$ROOT_DIR/crates/focusa-tui/src/next_safe_action.rs"
MAIN="$ROOT_DIR/crates/focusa-tui/src/main.rs"
HOME="$ROOT_DIR/crates/focusa-tui/src/views/deck_home.rs"

[[ -f "$NSA" ]] || fail "next_safe_action.rs missing"

for needle in \
  'NextSafeAction' \
  'pub fn recommend' \
  'authority_posture' \
  'walkthrough_context' \
  'HEADLESS_PROOF_STATES' \
  'disconnected:start_daemon' \
  'unbound:bind_project' \
  'no_workpoint:create_workpoint' \
  'no_evidence:attach_evidence' \
  'resumable:resume_mission' \
  'blocked:review_scope_before_acting'; do
  grep -qF -- "$needle" "$NSA" || fail "next safe action module missing: $needle"
done
pass "next safe action model covers state + authority + walkthrough context"

grep -qF 'mod next_safe_action;' "$MAIN" || fail "main.rs missing next_safe_action module"
grep -qF 'next_safe_action_model' "$MAIN" || fail "headless proof missing next_safe_action_model"
grep -qF "KeyCode::Char('d') | KeyCode::Char('n')" "$MAIN" || fail "main.rs missing n next-safe-action shortcut"
grep -qF 'primary_action:' "$HOME" || fail "Deck Home missing primary_action render"
grep -qF 'authority:' "$HOME" || fail "Deck Home missing authority render"
grep -qF 'why:' "$HOME" || fail "Deck Home missing why render"
pass "next safe action surfaced as one primary action in Deck Home"

echo "focusa-117 next-safe-action static test: PASS"