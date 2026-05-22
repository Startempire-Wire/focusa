#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BASE_URL="${FOCUSA_API_BASE_URL:-http://127.0.0.1:8787}"
TMP_DIR="$(mktemp -d /tmp/spec96-lowmem-tool-deps.XXXXXX)"
ACTIVATE_BODY="${TMP_DIR}/activate.json"
STATUS_BODY="${TMP_DIR}/status.json"
VALIDATE_JSON="${TMP_DIR}/validate.json"
PROOF_JSON="${TMP_DIR}/proof.json"
FINAL_BODY="${TMP_DIR}/final.json"
HEALTH_BODY="${TMP_DIR}/health.json"
DEACTIVATE_BODY="${TMP_DIR}/deactivate.json"
DEACTIVATE_ERR="${TMP_DIR}/deactivate.err"

cleanup() {
  curl -fsS --max-time 15 -X POST "${BASE_URL}/v1/resource/mode" \
    -H 'Content-Type: application/json' \
    --data '{"action":"deactivate_lowmem","reason":"spec96 lowmem tool dependency proof cleanup"}' \
    >"$DEACTIVATE_BODY" 2>"$DEACTIVATE_ERR" || true
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

cd "$ROOT_DIR"

curl -fsS --max-time 5 "${BASE_URL}/v1/health" >"$HEALTH_BODY"
node scripts/validate-focusa-tool-contracts.mjs --json >"$VALIDATE_JSON"
initial_tools="$(jq -r '.tools' "$VALIDATE_JSON")"
initial_contracts="$(jq -r '.contracts' "$VALIDATE_JSON")"
if [[ "$initial_tools" -le 0 || "$initial_tools" != "$initial_contracts" ]]; then
  echo "✗ FAIL: static tool/contract dependency counts invalid before LowMem" >&2
  cat "$VALIDATE_JSON" >&2
  exit 1
fi

echo "✓ PASS: static tool dependency graph valid before LowMem tools=${initial_tools}"

curl -fsS --max-time 15 -X POST "${BASE_URL}/v1/resource/mode" \
  -H 'Content-Type: application/json' \
  --data '{"action":"activate_lowmem","reason":"spec96 lowmem tool dependency proof"}' \
  >"$ACTIVATE_BODY"

mode="$(jq -r '.resource_mode.mode' "$ACTIVATE_BODY")"
policy="$(jq -r '.resource_mode.tool_availability_policy' "$ACTIVATE_BODY")"
if [[ "$mode" != "lowmem" || "$policy" != "all_tools_callable_with_bounded_or_degraded_envelopes" ]]; then
  echo "✗ FAIL: LowMem activation did not preserve all-tools callable policy" >&2
  cat "$ACTIVATE_BODY" >&2
  exit 1
fi

echo "✓ PASS: LowMem forced and all-tools callable policy surfaced"

curl -fsS --max-time 10 "${BASE_URL}/v1/resource/mode" >"$STATUS_BODY"
if ! jq -e '.resource_mode.retention_policy.retain_order | index("liveness") and index("workpoint") and index("evidence_handles")' "$STATUS_BODY" >/dev/null; then
  echo "✗ FAIL: LowMem status missing retention/dependency preservation posture" >&2
  cat "$STATUS_BODY" >&2
  exit 1
fi

echo "✓ PASS: LowMem status exposes retention posture for core tool dependencies"

node scripts/validate-focusa-tool-contracts.mjs --json >"$VALIDATE_JSON"
lowmem_tools="$(jq -r '.tools' "$VALIDATE_JSON")"
lowmem_contracts="$(jq -r '.contracts' "$VALIDATE_JSON")"
if [[ "$lowmem_tools" != "$initial_tools" || "$lowmem_contracts" != "$initial_contracts" ]]; then
  echo "✗ FAIL: static tool/contract counts changed under LowMem" >&2
  cat "$VALIDATE_JSON" >&2
  exit 1
fi

if ! jq -e '.failures | length == 0' "$VALIDATE_JSON" >/dev/null; then
  echo "✗ FAIL: static tool dependencies failed under LowMem" >&2
  cat "$VALIDATE_JSON" >&2
  exit 1
fi

echo "✓ PASS: static tool dependencies unchanged under LowMem"

set +e
node scripts/prove-focusa-tool-contracts-live.mjs --json --safe-fixtures >"$PROOF_JSON"
proof_rc=$?
set -e
if [[ "$proof_rc" -ne 0 ]] || ! jq -e --argjson expected "$initial_contracts" '.status == "passed" and .static_count == $expected and .live_count == $expected and .payload_equal == true and (.fixture_checks | all(.status == "passed"))' "$PROOF_JSON" >/dev/null; then
  echo "✗ FAIL: live tool contract/dependency proof failed under LowMem" >&2
  cat "$PROOF_JSON" >&2
  exit 1
fi

if ! jq -e '.checked_endpoints | index("/v1/ontology/tool-contracts") and index("/v1/health")' "$PROOF_JSON" >/dev/null; then
  echo "✗ FAIL: live proof did not probe required dependency endpoints" >&2
  cat "$PROOF_JSON" >&2
  exit 1
fi

echo "✓ PASS: live tool contracts and representative read dependencies available under LowMem"

curl -fsS --max-time 15 -X POST "${BASE_URL}/v1/resource/mode" \
  -H 'Content-Type: application/json' \
  --data '{"action":"deactivate_lowmem","reason":"spec96 lowmem tool dependency proof complete"}' \
  >"$FINAL_BODY"

echo "SPEC96 LowMem tool dependency runtime test: PASS"
