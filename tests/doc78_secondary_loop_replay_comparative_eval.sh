#!/bin/bash
# Replay-driven comparative proof harness for Doc-78 §15.1 baseline-vs-bounded evidence.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

run_case() {
  local label="$1"
  local logfile="$2"
  shift 2
  if (cd "$ROOT_DIR" && "$@") >"$logfile" 2>&1; then
    log_pass "$label"
  else
    log_fail "$label"
    tail -n 60 "$logfile" || true
  fi
}

run_case \
  "replay summary derives comparative-improvement pair from persisted secondary outcomes" \
  "/tmp/doc78_secondary_loop_replay_core.log" \
  cargo test -q -p focusa-core test_secondary_loop_comparative_summary_from_replay_log

run_case \
  "replay summary enforces session-bound filtering for comparative evidence" \
  "/tmp/doc78_secondary_loop_replay_filter.log" \
  cargo test -q -p focusa-core test_secondary_loop_comparative_summary_respects_session_filter

run_case \
  "strict replay export path remains fail-fast on malformed legacy payload rows" \
  "/tmp/doc78_secondary_loop_replay_strict.log" \
  cargo test -q -p focusa-core test_replay_events_fail_fast_on_legacy_payload_without_handle

run_case \
  "closure reporting consumes replay comparative evidence for the active task pair" \
  "/tmp/doc78_secondary_loop_replay_closure.log" \
  cargo test -q -p focusa-core secondary_loop_closure_replay_evidence_reads_persisted_pairs

run_case \
  "status surfaces per-task closure replay comparative proof payload" \
  "/tmp/doc78_secondary_loop_replay_status.log" \
  cargo test -q -p focusa-api secondary_loop_closure_replay_evidence_surfaces_current_task_pair

run_case \
  "status closure replay evidence stays fail-closed without matching task pair" \
  "/tmp/doc78_secondary_loop_replay_status_fail_closed.log" \
  cargo test -q -p focusa-api secondary_loop_closure_replay_evidence_defaults_fail_closed_without_match

run_case \
  "consumer replay payload exposes comparative summary and closure evidence in ok mode" \
  "/tmp/doc78_secondary_loop_replay_status_consumer_ok.log" \
  cargo test -q -p focusa-api secondary_loop_replay_consumer_payload_surfaces_ok_state

run_case \
  "consumer replay payload remains fail-closed when replay summary is unavailable" \
  "/tmp/doc78_secondary_loop_replay_status_consumer_error.log" \
  cargo test -q -p focusa-api secondary_loop_replay_consumer_payload_surfaces_error_state_fail_closed

echo "=== DOC78 SECONDARY LOOP REPLAY COMPARATIVE EVAL RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then
  exit 1
fi
