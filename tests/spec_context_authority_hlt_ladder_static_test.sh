#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
FILE="crates/focusa-api/src/routes/trajectory.rs"

rg -q 'fn is_generic_bootstrap_hlt' "$FILE"
rg -q 'fn latest_valid_historical_trajectory' "$FILE"
rg -q 'hlt_history_fallback' "$FILE"
rg -q 'bootstrap_degraded_placeholder' "$FILE"
rg -q 'effective_long_term_goal_present' "$FILE"
rg -q 'Workpoint/current_focus cannot populate MLG/STG when HLT is invalid or generic' "$FILE"
rg -q 'Trajectory definition required before ladder projection' "$FILE"

if rg -n 'Maintain and improve \{project_label\} within verified project scope' "$FILE" | grep -q .; then
  rg -q 'hlt_degraded_placeholder = true' "$FILE"
fi

echo "hlt ladder static test passed"
