#!/bin/bash
# Doc78 sustained production-runtime smoke:
# - runs the production governance/replay harness repeatedly against an already-running daemon
# - captures per-run artifacts plus a series summary for sustained evidence review
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:18799}"
RUNS_RAW="${FOCUSA_DOC78_PROD_SERIES_RUNS:-3}"
SERIES_DIR="${FOCUSA_DOC78_PROD_SERIES_DIR:-${ROOT_DIR}/docs/evidence/doc78-production-runtime-series-latest}"
PARENT_WORK_ITEM_ID="${FOCUSA_DOC78_PROD_PARENT_WORK_ITEM_ID:-focusa-o8vn}"
WRITER_PREFIX="${FOCUSA_DOC78_PROD_SERIES_WRITER_PREFIX:-doc78-production-series}"
SEED_NON_CLOSURE="${FOCUSA_DOC78_PROD_SERIES_SEED_NON_CLOSURE:-1}"
HARNESS_PATH="${ROOT_DIR}/tests/doc78_production_runtime_governance_replay_smoke.sh"

if ! [[ "$RUNS_RAW" =~ ^[0-9]+$ ]] || [ "$RUNS_RAW" -lt 1 ]; then
  echo "invalid FOCUSA_DOC78_PROD_SERIES_RUNS value: ${RUNS_RAW}" >&2
  exit 1
fi
RUNS="$RUNS_RAW"

if [ ! -x "$HARNESS_PATH" ]; then
  echo "missing executable harness: ${HARNESS_PATH}" >&2
  exit 1
fi

FAILED=0
PASSED=0
STARTED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
RUN_RECORDS_FILE="${SERIES_DIR}/run-records.jsonl"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
log_info(){ echo -e "${YELLOW}INFO${NC}: $1"; }

seed_non_closure_traces(){
  local run_index="$1"
  if [ "$SEED_NON_CLOSURE" != "1" ]; then
    return 0
  fi
  curl -sS -X POST "${BASE_URL}/v1/telemetry/trace" -H 'Content-Type: application/json' \
    -d "{\"event_type\":\"verification_result\",\"verification_kind\":\"secondary_loop_quality\",\"loop_objective\":\"governance_pause_resolution\",\"continuation_decision\":\"defer\",\"turn_id\":\"doc78-prod-series-${run_index}-seed-1\",\"quality\":\"production-series-seed\"}" >/dev/null
  curl -sS -X POST "${BASE_URL}/v1/telemetry/trace" -H 'Content-Type: application/json' \
    -d "{\"event_type\":\"verification_result\",\"verification_kind\":\"secondary_loop_quality\",\"loop_objective\":\"continuity_handoff_validation\",\"continuation_decision\":\"continue\",\"turn_id\":\"doc78-prod-series-${run_index}-seed-2\",\"quality\":\"production-series-seed\"}" >/dev/null
}

mkdir -p "$SERIES_DIR"
: > "$RUN_RECORDS_FILE"

{
  echo "base_url=${BASE_URL}"
  echo "runs_requested=${RUNS}"
  echo "parent_work_item_id=${PARENT_WORK_ITEM_ID}"
  echo "writer_prefix=${WRITER_PREFIX}"
  echo "seed_non_closure=${SEED_NON_CLOSURE}"
  echo "started_at=${STARTED_AT}"
} > "${SERIES_DIR}/series.meta"

log_info "series artifact capture enabled: ${SERIES_DIR}"
log_info "base URL: ${BASE_URL}"
log_info "runs requested: ${RUNS}"

echo "=== DOC78 PRODUCTION RUNTIME SERIES SMOKE ==="
echo ""

for run_index in $(seq 1 "$RUNS"); do
  run_name="$(printf 'run-%02d' "$run_index")"
  run_dir="${SERIES_DIR}/${run_name}"
  run_started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  mkdir -p "$run_dir"

  seed_non_closure_traces "$run_index"

  run_exit_code=0
  if FOCUSA_BASE_URL="$BASE_URL" \
    FOCUSA_DOC78_PROD_WRITER="${WRITER_PREFIX}-${run_name}" \
    FOCUSA_DOC78_PROD_PARENT_WORK_ITEM_ID="$PARENT_WORK_ITEM_ID" \
    FOCUSA_DOC78_PROD_ARTIFACT_DIR="$run_dir" \
    bash "$HARNESS_PATH" > "${run_dir}/harness-output.log" 2>&1; then
    run_exit_code=0
  else
    run_exit_code=$?
  fi

  tests_passed="-1"
  tests_failed="-1"
  if [ -f "${run_dir}/result.meta" ]; then
    tests_passed="$(grep '^tests_passed=' "${run_dir}/result.meta" | head -n1 | cut -d'=' -f2 || echo -1)"
    tests_failed="$(grep '^tests_failed=' "${run_dir}/result.meta" | head -n1 | cut -d'=' -f2 || echo -1)"
  fi

  non_closure_objective_events="0"
  if [ -f "${run_dir}/status_after.json" ]; then
    non_closure_objective_events="$(jq -r '.secondary_loop_eval_bundle.secondary_loop_objective_profile.non_closure_objective_events // 0' "${run_dir}/status_after.json" 2>/dev/null || echo 0)"
  fi

  governance_marker_events="0"
  if [ -f "${run_dir}/scope_failure_trace_after.json" ]; then
    governance_marker_events="$(jq -r '[.events[] | select(.payload.reason == "governance decision pending" and .payload.path == "select_next_continuous_subtask")] | length' "${run_dir}/scope_failure_trace_after.json" 2>/dev/null || echo 0)"
  fi

  run_ok=false
  if [ "$run_exit_code" -eq 0 ] && [ "$tests_failed" = "0" ]; then
    run_ok=true
  fi

  run_record="$(jq -n \
    --arg run_name "$run_name" \
    --arg run_dir "$run_dir" \
    --arg run_started_at "$run_started_at" \
    --arg run_finished_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --argjson run_index "$run_index" \
    --argjson exit_code "$run_exit_code" \
    --argjson tests_passed "${tests_passed:-0}" \
    --argjson tests_failed "${tests_failed:-0}" \
    --argjson non_closure_objective_events "${non_closure_objective_events:-0}" \
    --argjson governance_marker_events "${governance_marker_events:-0}" \
    --argjson ok "$run_ok" \
    '{
      run_index: $run_index,
      run_name: $run_name,
      run_dir: $run_dir,
      run_started_at: $run_started_at,
      run_finished_at: $run_finished_at,
      exit_code: $exit_code,
      tests_passed: $tests_passed,
      tests_failed: $tests_failed,
      non_closure_objective_events: $non_closure_objective_events,
      governance_marker_events: $governance_marker_events,
      ok: $ok
    }')"
  echo "$run_record" >> "$RUN_RECORDS_FILE"

  if [ "$run_ok" = true ]; then
    log_pass "${run_name} passed (tests_passed=${tests_passed}, non_closure_objective_events=${non_closure_objective_events}, governance_marker_events=${governance_marker_events})"
  else
    log_fail "${run_name} failed (exit=${run_exit_code}, tests_failed=${tests_failed})"
  fi

  cp "$RUN_RECORDS_FILE" "${run_dir}/run-records-snapshot.jsonl"
done

SUMMARY_JSON="$(jq -s \
  --arg doc "78" \
  --arg kind "production_runtime_series" \
  --arg base_url "$BASE_URL" \
  --arg parent_work_item_id "$PARENT_WORK_ITEM_ID" \
  --arg started_at "$STARTED_AT" \
  --arg finished_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --argjson runs_requested "$RUNS" \
  '{
    doc: $doc,
    kind: $kind,
    base_url: $base_url,
    parent_work_item_id: $parent_work_item_id,
    runs_requested: $runs_requested,
    runs_completed: length,
    runs_passed: ([.[] | select(.ok == true)] | length),
    runs_failed: ([.[] | select(.ok != true)] | length),
    started_at: $started_at,
    finished_at: $finished_at,
    runs: .
  }' "$RUN_RECORDS_FILE")"

printf '%s\n' "$SUMMARY_JSON" > "${SERIES_DIR}/series-summary.json"

jq -r '
  [
    "# Doc78 Production Runtime Series Summary",
    "",
    "- runs requested: \(.runs_requested)",
    "- runs completed: \(.runs_completed)",
    "- runs passed: \(.runs_passed)",
    "- runs failed: \(.runs_failed)",
    "- base URL: \(.base_url)",
    "- started at: \(.started_at)",
    "- finished at: \(.finished_at)",
    "",
    "## Runs",
    (.runs[] | "- \(.run_name): ok=\(.ok) tests_failed=\(.tests_failed) non_closure_objective_events=\(.non_closure_objective_events) governance_marker_events=\(.governance_marker_events)")
  ] | join("\n")
' "${SERIES_DIR}/series-summary.json" > "${SERIES_DIR}/series-summary.md"

echo ""
echo "=== DOC78 PRODUCTION RUNTIME SERIES RESULTS ==="
echo "Runs passed: ${PASSED}"
echo "Runs failed: ${FAILED}"
echo "Series summary: ${SERIES_DIR}/series-summary.json"

if [ "$FAILED" -ne 0 ]; then
  exit 1
fi
