#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
API="$ROOT/crates/focusa-api/src/routes/project.rs"
CLI="$ROOT/crates/focusa-cli/src/commands/project.rs"
TOOLS="$ROOT/apps/pi-extension/src/tools.ts"
CONTRACTS="$ROOT/apps/pi-extension/src/tool-contracts.ts"
DOC="$ROOT/docs/current/PROJECT_INTELLIGENCE_FLYWHEEL.md"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

rg -n '/v1/project/card|focusa.project_card.v1|trajectory_define_goal|prediction_stats_card|read_predictions|ontology|metacognition' "$API" >/dev/null \
  || fail "project card API route does not fuse project/ontology/trajectory/prediction/metacog guidance"

rg -n 'Card|/v1/project/card|current_ask' "$CLI" >/dev/null \
  || fail "project card CLI parity missing"

rg -n 'focusa_project_card|/project/card|focusa.project_card.v1|focusa_trajectory_define_goal|prior_session_context|success_sequence|recommended_first_event' "$TOOLS" "$CONTRACTS" >/dev/null \
  || fail "project card Pi tool/contract parity missing"

rg -n 'focusa project card|GET /v1/project/card|ontology objects → trajectory hierarchy → prediction|advisory-only' "$DOC" "$ROOT/docs/current/API_REFERENCE_CURRENT.md" "$ROOT/docs/current/CLI_REFERENCE_CURRENT.md" >/dev/null \
  || fail "project card public docs missing API/CLI/flywheel contract"

pass "project card flywheel API/CLI/docs are wired"
