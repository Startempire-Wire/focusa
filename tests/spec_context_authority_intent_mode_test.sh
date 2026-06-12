#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CARGO_BIN="${CARGO:-cargo}"
OUT="${TMPDIR:-/tmp}/focusa-intent-mode-$$.json"
trap 'rm -f "$OUT"' EXIT

"$CARGO_BIN" test -q -p focusa-cli classifies_maybe_prompt_as_planning_without_mutation --locked
"$CARGO_BIN" run -q -p focusa-cli --locked -- --json action classify-intent \
  --prompt "Maybe we can add a flag for install context" > "$OUT"

jq -e '.schema == "focusa.intent_mode_gate.v1"' "$OUT" >/dev/null
jq -e '.mode == "planning_discussion"' "$OUT" >/dev/null
jq -e '.mutation_allowed == false' "$OUT" >/dev/null
jq -e '.recommended_action | contains("do not mutate")' "$OUT" >/dev/null

echo "intent mode test passed"
