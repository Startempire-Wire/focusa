#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPACTION="$ROOT/apps/pi-extension/src/compaction.ts"
CONTRACTS="$ROOT/apps/pi-extension/src/tool-contracts.ts"
DOC="$ROOT/docs/current/END_OF_TASK_LEARNING_LOOP.md"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

[[ -f "$DOC" ]] || fail "missing END_OF_TASK_LEARNING_LOOP doc"

rg -n '## Task Summary|## Predictive Context|## Metacog Context|## Possibilities|End-of-task Report Contract|focusa_predict_recent|focusa_predict_stats|focusa_predict_evaluate|focusa_metacog_doctor|focusa_metacog_retrieve|focusa_metacog_capture' "$COMPACTION" >/dev/null \
  || fail "compaction card lacks task summary/prediction/metacog/possibility/end-of-task learning loop surfaces"

rg -n 'prediction/metacog|learning-loop context|task-boundary reviews|final task completion|compaction cards, trajectory reviews, and work reports' "$CONTRACTS" >/dev/null \
  || fail "tool contracts lack prediction/metacog trajectory and report cross-references"

rg -n 'Close: focusa_predict_recent/stats → focusa_predict_evaluate → focusa_metacog_capture/retrieve|mandatory at task boundaries|compaction cards|trajectory reviews|Work reports' "$DOC" >/dev/null \
  || fail "end-of-task learning loop doc lacks required route and boundary policy"

pass "end-of-task learning loop is surfaced in compaction cards, tool contracts, and docs"
