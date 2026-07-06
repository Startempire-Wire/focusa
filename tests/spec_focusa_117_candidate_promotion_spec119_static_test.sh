#!/usr/bin/env bash
# Spec 117 .31 — Workpoint Candidate Promotion preview-before-commit per Spec 119 §7.11.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

REC="$ROOT_DIR/crates/focusa-tui/src/views/recall.rs"
MAIN="$ROOT_DIR/crates/focusa-tui/src/main.rs"
[[ -f "$REC" ]] || fail "recall.rs missing"
for needle in \
  'pub preview_state: &' \
  'is_preview_only' \
  'preview_only_until_operator_approval' \
  'workpoint_candidate_preview_state_blocks_canonical_write' \
  'render_workpoint_candidate' \
  'operator_approval' \
  'canonical_workpoint_checkpoint'; do
  grep -qF -- "$needle" "$REC" || fail "recall missing: $needle"
done
pass "recall surfaces preview_state and is_preview_only invariant per Spec 119 §7.11"
grep -qF 'workpoint_candidate_preview_state' "$MAIN" || fail "headless proof missing workpoint_candidate_preview_state"
grep -qF 'workpoint_candidate_preview_only' "$MAIN" || fail "headless proof missing workpoint_candidate_preview_only"
pass "headless proof exposes preview_state + preview_only"
echo "focusa-117 candidate-promotion-spec119 static test: PASS"
