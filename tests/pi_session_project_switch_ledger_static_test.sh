#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE_TS="$ROOT_DIR/apps/pi-extension/src/state.ts"
TURNS_TS="$ROOT_DIR/apps/pi-extension/src/turns.ts"
COMPACTION_TS="$ROOT_DIR/apps/pi-extension/src/compaction.ts"
SESSION_TS="$ROOT_DIR/apps/pi-extension/src/session.ts"
SPEC="$ROOT_DIR/docs/current/PROJECT_SCOPE_OVERRIDE_INCIDENT_AND_GUARD_SPEC_2026-05-31.md"

fail() { echo "✗ FAIL: $1" >&2; exit 1; }
pass() { echo "✓ PASS: $1"; }

rg -n 'interface PiProjectThreadObservation|PROJECT_SWITCH_LEDGER_MAX_OBSERVATIONS|projectSwitchLedger' "$STATE_TS" >/dev/null \
  || fail "Project-switch ledger state/schema missing"
pass "Project-switch ledger state/schema exists"

rg -n 'observeProjectThreadEvidence|observeProjectThreadHintsFromText|formatProjectSwitchLedgerLines|projectSwitchLedgerCandidateForAsk' "$STATE_TS" >/dev/null \
  || fail "Project-switch ledger helper path missing"
pass "Project-switch ledger helper path exists"

rg -n 'boundedAttentionText|recent_actions.*slice|appendEntry\("focusa-project-switch-ledger"|slice\(0, 2000\)' "$STATE_TS" >/dev/null \
  || fail "Ledger is not bounded to summaries/evidence handles"
pass "Ledger stores bounded summaries/evidence, not raw transcript blobs"

rg -n 'observeProjectThreadHintsFromText\(newTaskText|observeProjectThreadHintsFromText\(projectHintText|PROJECT_SWITCH_LEDGER' "$TURNS_TS" >/dev/null \
  || fail "Input/tool evidence does not feed project-switch ledger into Focus Slice"
pass "Input/tool evidence feeds project-switch ledger into Focus Slice"

rg -n 'projectSwitchLedger.*slice|projectSwitchLedger: S\.projectSwitchLedger|formatProjectSwitchLedgerLines' "$STATE_TS" "$SESSION_TS" "$COMPACTION_TS" >/dev/null \
  || fail "Project-switch ledger does not persist/appear in compaction"
pass "Project-switch ledger persists and appears in compaction"

rg -n 'PTM|planmarr|same Pi session|project-switch ledger|project_thread_observation' "$SPEC" >/dev/null \
  || fail "Spec lacks PTM/same-session project-switch ledger expectation"
pass "Spec documents PTM same-session project-switch expectation"

echo "Pi session project-switch ledger static test: PASS"
