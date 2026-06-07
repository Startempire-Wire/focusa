#!/usr/bin/env bash
set -euo pipefail
BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for term in command_palette 'Resume work' 'Link proof' 'Explain conflict' 'Start next bead' 'Make repair report' 'Run clean-repair check' args_preview full_palette; do
  rg -F "$term" crates/focusa-api/src/routes/traverse.rs >/dev/null || fail "traverse command palette missing $term"
done
pass "traverse declares command palette terms"

curl -fsS --max-time 10 "$BASE/v1/health" >/dev/null || { echo "daemon unavailable; static checks only"; exit 0; }

curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d '{"surface":"command_palette","selector":"top","limit":3}' \
  "$BASE/v1/traverse" >/tmp/spec102-command-top.json
jq -e '
  .traversal.command_palette.mode == "top"
  and (.traversal.command_palette.commands | length) == 3
  and (.traversal.command_palette.commands[] | select(.label == "Resume work" and .tool == "focusa_workpoint_resume" and .args_preview != null and .when != null))
  and (.traversal.command_palette.commands[] | select(.label == "Link proof" and .tool == "focusa_evidence_capture"))
  and (.traversal.command_palette.commands[] | select(.label == "Start next bead" and .tool == "focusa_workpoint_checkpoint"))
  and .traversal.command_palette.full_palette_available == true
' /tmp/spec102-command-top.json >/dev/null || fail "top command palette missing 3 actionable commands"
pass "top 3 command palette is actionable"

curl -fsS --max-time 15 -H 'Content-Type: application/json' \
  -d '{"surface":"command_palette","selector":"full","limit":10}' \
  "$BASE/v1/traverse" >/tmp/spec102-command-full.json
jq -e '
  .traversal.command_palette.mode == "full"
  and (.traversal.command_palette.commands | length) >= 6
  and (.traversal.command_palette.commands[] | select(.label == "Explain conflict"))
  and (.traversal.command_palette.commands[] | select(.label == "Make repair report"))
  and (.traversal.command_palette.commands[] | select(.label == "Run clean-repair check"))
' /tmp/spec102-command-full.json >/dev/null || fail "full command palette missing expected routines"
pass "full command palette available on request"

echo "SPEC102 agent command palette test: PASS"
