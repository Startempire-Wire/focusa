#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
API="$ROOT/crates/focusa-api/src/routes/project.rs"
DOC="$ROOT/docs/current/PREDICTION_ALGORITHMS_IMPLEMENTED.md"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

rg -n 'fn sigmoid|fn logit|fn softmax|fn expected_value|fn exponential_decay|fn ema|fn z_score|fn brier_score|fn log_loss|fn normalized_weighted_score|algorithmic_intelligence' "$API" >/dev/null \
  || fail "prediction math formulas are not implemented/surfaced in project card"

rg -n 'readiness_to_execute|need_to_bootstrap_or_rebootstrap|need_to_learn_or_evaluate|expected_utility|Brier score|Softmax' "$DOC" >/dev/null \
  || fail "prediction algorithm public doc missing formulas or project-card usage"

pass "prediction algorithms implemented and documented"
