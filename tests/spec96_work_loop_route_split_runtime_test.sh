#!/bin/bash
set -euo pipefail
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
TMP_DIR="${TMPDIR:-/tmp}/focusa-spec96-work-loop-routes.$$"
COLD_MAX_TIME="${FOCUSA_COLD_ROUTE_MAX_TIME:-15}"
mkdir -p "$TMP_DIR"
trap 'rm -rf "$TMP_DIR"' EXIT

curl -fsS --max-time 3 "${BASE_URL}/v1/work-loop/health" >"$TMP_DIR/health.json"
curl -fsS --max-time 3 "${BASE_URL}/v1/work-loop/status?summary_only=true" >"$TMP_DIR/summary.json"
curl -fsS --max-time "$COLD_MAX_TIME" "${BASE_URL}/v1/work-loop/status/deep" >"$TMP_DIR/deep.json"

jq -e '.status=="ok" and .route_tier=="hot" and .summary_only==true and .deep_status_route=="/v1/work-loop/status/deep"' "$TMP_DIR/health.json" >/dev/null
jq -e '.route_tier=="hot" and .summary_only==true and .bounds.summary_only==true and .deep_status_route=="/v1/work-loop/status/deep" and (.cold_omitted|length > 0)' "$TMP_DIR/summary.json" >/dev/null
jq -e '.route_tier=="cold" and .summary_only==false and (.cold_omitted|length == 0) and has("policy") and has("worktree")' "$TMP_DIR/deep.json" >/dev/null

echo "SPEC96 work-loop route split runtime test: PASS"
