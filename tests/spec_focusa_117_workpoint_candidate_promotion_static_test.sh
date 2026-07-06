#!/usr/bin/env bash
# Spec 117 §15.3 — Recall → Workpoint candidate promotion static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

RECALL="$ROOT_DIR/crates/focusa-tui/src/views/recall.rs"
MAIN="$ROOT_DIR/crates/focusa-tui/src/main.rs"
[[ -f "$RECALL" ]] || fail "recall.rs missing"

for needle in \
  'WorkpointCandidatePromotion' \
  'WORKPOINT_CANDIDATE_PROMOTION_FLOW' \
  'recall_search' \
  'recall_deck_card' \
  'verify_project_root_and_continuity_id' \
  'context_authority_preflight' \
  'proof_check' \
  'render_workpoint_candidate' \
  'operator_approval' \
  'canonical_workpoint_checkpoint'; do
  grep -qF -- "$needle" "$RECALL" || fail "candidate promotion flow missing: $needle"
done
pass "candidate promotion flow has required Context Authority gates"

for needle in \
  'WORKPOINT_CANDIDATE_FORBIDDEN' \
  'recall_direct_canonical_write' \
  'promotion_without_scope_verification' \
  'promotion_without_operator_approval' \
  'promotion_without_proof_or_explicit_gap' \
  'Recall cannot directly create canonical Workpoint authority'; do
  grep -qF -- "$needle" "$RECALL" || fail "candidate promotion forbidden rule missing: $needle"
done
pass "candidate promotion forbids direct canonical Recall writes"

grep -qF 'workpoint_candidate_promotion_flow' "$MAIN" || fail "headless proof missing workpoint_candidate_promotion_flow"
grep -qF 'workpoint_candidate_forbidden' "$MAIN" || fail "headless proof missing workpoint_candidate_forbidden"
pass "candidate promotion exposed in headless proof"

echo "focusa-117 workpoint-candidate-promotion static test: PASS"
