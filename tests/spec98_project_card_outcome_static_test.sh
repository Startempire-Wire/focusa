#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
API="$ROOT/crates/focusa-api/src/routes/project.rs"
CLI="$ROOT/crates/focusa-cli/src/commands/project.rs"
DOC="$ROOT/docs/current/PREDICTION_ALGORITHMS_IMPLEMENTED.md"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

rg -n 'ProjectCardOutcomeRequest|/v1/project/card/outcome|project_card_algorithm_outcomes\.jsonl|append_project_card_algorithm_outcome|project_card_run_exists|update_weights_from_algorithm_outcome|prior_session_context|recent_decisions|focus_goal_signals' "$API" >/dev/null \
  || fail "project card outcome API/persistence/prior-context is missing"

rg -n 'CardOutcome|/v1/project/card/outcome|algorithm_run_id|evidence_refs' "$CLI" >/dev/null \
  || fail "project card outcome CLI is missing"

rg -n 'project_card_algorithm_outcomes\.jsonl|card-outcome|algorithm_run_id|POST /v1/project/card/outcome' "$DOC" "$ROOT/docs/current/API_REFERENCE_CURRENT.md" "$ROOT/docs/current/CLI_REFERENCE_CURRENT.md" >/dev/null \
  || fail "project card outcome docs are missing"

pass "project card algorithm outcomes attach to algorithm_run_id"
