#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BASE_URL="${FOCUSA_API_BASE_URL:-http://127.0.0.1:8787}"
TMP_DIR="$(mktemp -d /tmp/spec96-lowmem-surgical.XXXXXX)"

cleanup() {
  curl -fsS --max-time 15 -X POST "${BASE_URL}/v1/resource/mode" \
    -H 'Content-Type: application/json' \
    --data '{"action":"deactivate_lowmem","reason":"spec96 lowmem surgical stress cleanup"}' \
    >"${TMP_DIR}/deactivate.json" 2>"${TMP_DIR}/deactivate.err" || true
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

cd "$ROOT_DIR"

curl -fsS --max-time 5 "${BASE_URL}/v1/health" >"${TMP_DIR}/health-before.json"
uptime_before="$(jq -r '.uptime_ms // 0' "${TMP_DIR}/health-before.json")"
node scripts/validate-focusa-tool-contracts.mjs --json >"${TMP_DIR}/contracts-before.json"
tools_before="$(jq -r '.tools' "${TMP_DIR}/contracts-before.json")"
contracts_before="$(jq -r '.contracts' "${TMP_DIR}/contracts-before.json")"

curl -fsS --max-time 15 -X POST "${BASE_URL}/v1/resource/mode" \
  -H 'Content-Type: application/json' \
  --data '{"action":"activate_lowmem","reason":"spec96 lowmem surgical stress"}' \
  >"${TMP_DIR}/activate.json"

if ! jq -e '.resource_mode.mode == "lowmem" and .resource_mode.tool_availability_policy == "all_tools_callable_with_bounded_or_degraded_envelopes"' "${TMP_DIR}/activate.json" >/dev/null; then
  echo "✗ FAIL: LowMem not activated with all-tools callable policy" >&2
  cat "${TMP_DIR}/activate.json" >&2
  exit 1
fi

echo "✓ PASS: LowMem forced with all-tools callable policy"

node scripts/validate-focusa-tool-contracts.mjs --json >"${TMP_DIR}/contracts-lowmem.json"
if ! jq -e --argjson tools "$tools_before" --argjson contracts "$contracts_before" '.tools == $tools and .contracts == $contracts and (.failures|length == 0)' "${TMP_DIR}/contracts-lowmem.json" >/dev/null; then
  echo "✗ FAIL: tool/contract registry changed or failed under LowMem" >&2
  cat "${TMP_DIR}/contracts-lowmem.json" >&2
  exit 1
fi

echo "✓ PASS: no official tool disappears under LowMem (${tools_before})"

hot_routes=(
  "/v1/health"
  "/v1/status?summary_only=true"
  "/v1/resource/mode"
  "/v1/workpoint/current"
  "/v1/trajectory/view?mode=summary&project_root=${ROOT_DIR}"
)
for route in "${hot_routes[@]}"; do
  code="$(curl -sS -o "${TMP_DIR}/hot.json" -w '%{http_code}' --max-time 5 "${BASE_URL}${route}" || true)"
  if [[ "$code" != "200" ]]; then
    echo "✗ FAIL: hot LowMem route ${route} returned ${code}" >&2
    cat "${TMP_DIR}/hot.json" >&2 || true
    exit 1
  fi
done

echo "✓ PASS: representative hot routes remain callable under LowMem"

# Cold pressure may timeout; hot health must recover and daemon uptime must not reset.
set +e
curl -fsS --max-time 1 "${BASE_URL}/v1/status/deep" >"${TMP_DIR}/cold-status.json" 2>"${TMP_DIR}/cold-status.err"
cold_rc=$?
set -e
for _ in 1 2 3; do
  if curl -fsS --max-time 5 "${BASE_URL}/v1/health" >"${TMP_DIR}/health-after-cold.json"; then
    break
  fi
  sleep 1
done
if ! jq -e '.ok == true' "${TMP_DIR}/health-after-cold.json" >/dev/null; then
  echo "✗ FAIL: health unavailable after cold pressure probe rc=${cold_rc}" >&2
  cat "${TMP_DIR}/cold-status.err" >&2 || true
  exit 1
fi
uptime_after="$(jq -r '.uptime_ms // 0' "${TMP_DIR}/health-after-cold.json")"
if [[ "$uptime_after" -lt "$uptime_before" ]]; then
  echo "✗ FAIL: daemon uptime reset under LowMem stress before=${uptime_before} after=${uptime_after}" >&2
  exit 1
fi

echo "✓ PASS: no healthcheck restart storm after cold route pressure rc=${cold_rc}"

curl -fsS --max-time 10 "${BASE_URL}/v1/ontology/world?include_full_payload=true&limit_objects=5&limit_links=5" >"${TMP_DIR}/full-payload.json"
if ! jq -e 'has("degraded") and has("full_payload_blocked_by_pressure") and .bounds.objects.include_full_payload != null' "${TMP_DIR}/full-payload.json" >/dev/null; then
  echo "✗ FAIL: full payload route lacks explicit degradation/opt-in metadata" >&2
  cat "${TMP_DIR}/full-payload.json" >&2
  exit 1
fi

echo "✓ PASS: full-payload cold route exposes explicit degradation/opt-in metadata"

writer_id="spec96-lowmem-surgical-$$"
checkpoint_key="spec96-lowmem-surgical-${writer_id}"
jq -n \
  --arg root "$ROOT_DIR" \
  --arg key "$checkpoint_key" \
  '{project_root:$root, continuity_id:"spec96-lowmem-surgical", session_id:"spec96-lowmem-surgical", checkpoint_reason:"manual", canonical:true, mission:"Spec96 LowMem surgical stress Workpoint", next_slice:"prove evidence link remains bounded under LowMem", source_turn_id:"spec96-lowmem-surgical", idempotency_key:$key}' \
  >"${TMP_DIR}/checkpoint-payload.json"
checkpoint_code="$(curl -sS -o "${TMP_DIR}/workpoint-checkpoint.json" -w '%{http_code}' --max-time 5 -X POST "${BASE_URL}/v1/workpoint/checkpoint" \
  -H 'Content-Type: application/json' \
  -H "x-focusa-writer-id: ${writer_id}" \
  -H 'x-focusa-permissions: admin:*' \
  --data @"${TMP_DIR}/checkpoint-payload.json" || true)"
if [[ "$checkpoint_code" != "200" && "$checkpoint_code" != "202" ]]; then
  echo "✗ FAIL: LowMem Workpoint checkpoint timed out or failed code=${checkpoint_code}" >&2
  cat "${TMP_DIR}/workpoint-checkpoint.json" >&2 || true
  exit 1
fi
workpoint_id="$(jq -r '.workpoint_id // empty' "${TMP_DIR}/workpoint-checkpoint.json")"
if [[ -z "$workpoint_id" ]]; then
  echo "✗ FAIL: LowMem Workpoint checkpoint response missing workpoint_id" >&2
  cat "${TMP_DIR}/workpoint-checkpoint.json" >&2
  exit 1
fi
for _ in 1 2 3 4 5 6 7 8 9 10; do
  curl -fsS --max-time 5 "${BASE_URL}/v1/workpoint/current?project_root=${ROOT_DIR}&continuity_id=spec96-lowmem-surgical" >"${TMP_DIR}/workpoint-current.json" || true
  if jq -e --arg id "$workpoint_id" '.workpoint_id == $id' "${TMP_DIR}/workpoint-current.json" >/dev/null 2>&1; then
    break
  fi
  sleep 0.2
done
jq -n \
  --arg id "$workpoint_id" \
  '{workpoint_id:$id, target_ref:"tests/spec96_lowmem_surgical_agent_stress_test.sh", result:"LowMem evidence link stayed bounded under Pi timeout", evidence_ref:"spec96-lowmem-evidence-link-runtime"}' \
  >"${TMP_DIR}/evidence-link-payload.json"
evidence_code="$(curl -sS -o "${TMP_DIR}/evidence-link.json" -w '%{http_code}' --max-time 5 -X POST "${BASE_URL}/v1/workpoint/evidence/link" \
  -H 'Content-Type: application/json' \
  -H "x-focusa-writer-id: ${writer_id}" \
  -H 'x-focusa-permissions: admin:*' \
  --data @"${TMP_DIR}/evidence-link-payload.json" || true)"
if [[ "$evidence_code" != "200" && "$evidence_code" != "202" ]]; then
  echo "✗ FAIL: LowMem Workpoint evidence link timed out or failed code=${evidence_code}" >&2
  cat "${TMP_DIR}/evidence-link.json" >&2 || true
  exit 1
fi
if ! jq -e '(.status == "accepted") or (.status == "pending" and .failure_class == "read_model_lag" and .retry_posture == "safe_retry")' "${TMP_DIR}/evidence-link.json" >/dev/null; then
  echo "✗ FAIL: LowMem Workpoint evidence link missing accepted/read_model_lag envelope" >&2
  cat "${TMP_DIR}/evidence-link.json" >&2
  exit 1
fi

echo "✓ PASS: Workpoint checkpoint/evidence link stay bounded under LowMem"

curl -fsS --max-time 10 -X POST "${BASE_URL}/v1/ontology/context" \
  -H 'Content-Type: application/json' \
  --data '{"current_ask":"Fresh agent: identify the active identity axes and next relevant slice using summaries only","budget_tokens":240,"view_profile":"pi_operator_view","slice_type":"active_mission"}' \
  >"${TMP_DIR}/surgical-context.json"
if ! jq -e '.context_posture == "surgical_summary_only" and .identity_axes.authority_gate == "project_root_plus_continuity_id" and (.working_set.traversal_metadata.summary_only == true) and (.rehydrate.routes | index("/v1/ontology/working-set"))' "${TMP_DIR}/surgical-context.json" >/dev/null; then
  echo "✗ FAIL: fresh-agent surgical ontology task did not complete with summaries + rehydrate refs" >&2
  cat "${TMP_DIR}/surgical-context.json" >&2
  exit 1
fi

curl -fsS --max-time 10 -X POST "${BASE_URL}/v1/traverse" \
  -H 'Content-Type: application/json' \
  --data '{"surface":"evidence","selector":"recent","limit":3,"fields":["id","label","summary"]}' \
  >"${TMP_DIR}/traverse-evidence.json"
if ! jq -e '.status == "completed" and .surface == "evidence" and .traversal.limit <= 3 and (.traversal.fields.requested | type == "array") and (.traversal.metadata.summary_only == true)' "${TMP_DIR}/traverse-evidence.json" >/dev/null; then
  echo "✗ FAIL: fresh-agent focusa_traverse evidence slice failed under LowMem" >&2
  cat "${TMP_DIR}/traverse-evidence.json" >&2
  exit 1
fi

echo "✓ PASS: fresh-agent surgical task completed using summaries + focusa_traverse"
echo "SPEC96 LowMem surgical-agent stress test: PASS"
