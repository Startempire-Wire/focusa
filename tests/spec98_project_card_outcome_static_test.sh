#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
API="$ROOT/crates/focusa-api/src/routes/project.rs"
CLI="$ROOT/crates/focusa-cli/src/commands/project.rs"
TOOLS="$ROOT/apps/pi-extension/src/tools.ts"
CONTRACTS="$ROOT/apps/pi-extension/src/tool-contracts.ts"
DOC="$ROOT/docs/current/PREDICTION_ALGORITHMS_IMPLEMENTED.md"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

rg -n 'ProjectCardOutcomeRequest|/v1/project/card/outcome|project_card_algorithm_outcomes\.jsonl|append_project_card_algorithm_outcome|project_card_run_exists|update_weights_from_algorithm_outcome|projected_project_card_weights|TAIL_BYTES|project_card_outcome_stats|outcome_learning|outcome_bias|shortest_path_to_success|eliminated_candidates|crosswire_health|effective_ontology_objects|trajectory_report_card|prior_session_context|recent_decisions|focus_goal_signals' "$API" >/dev/null \
  || fail "project card outcome API/persistence/prior-context/outcome ranking/shortest-path/crosswire reporting is missing"

rg -n 'CardOutcome|/v1/project/card/outcome|algorithm_run_id|evidence_refs' "$CLI" >/dev/null \
  || fail "project card outcome CLI is missing"

rg -n 'focusa_project_card_outcome|/project/card/outcome|write_project_card_outcome|project.card_outcome' "$TOOLS" "$CONTRACTS" "$ROOT/docs/focusa-tools/tools/focusa_project_card_outcome.md" >/dev/null \
  || fail "project card outcome Pi tool/contract/docs are missing"

rg -n 'project_card_algorithm_outcomes\.jsonl|card-outcome|algorithm_run_id|POST /v1/project/card/outcome' "$DOC" "$ROOT/docs/current/API_REFERENCE_CURRENT.md" "$ROOT/docs/current/CLI_REFERENCE_CURRENT.md" >/dev/null \
  || fail "project card outcome docs are missing"

pass "project card algorithm outcomes attach to algorithm_run_id"
