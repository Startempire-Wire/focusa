#!/usr/bin/env bash
# Bounded extended soak: repeated hot routes + scoped Workpoint checkpoint/resume + post-run daemon profile.
set -euo pipefail
BASE="${FOCUSA_API_BASE_URL:-http://127.0.0.1:8787}"
PROJECT_ROOT="${FOCUSA_SOAK_PROJECT_ROOT:-$(pwd -P)}"
DURATION_SECS="${FOCUSA_SOAK_DURATION_SECS:-45}"
INTERVAL_SECS="${FOCUSA_SOAK_INTERVAL_SECS:-2}"
TMP_DIR="${TMPDIR:-/tmp}/focusa-soak-$$"
mkdir -p "$TMP_DIR"
urlencode(){ python3 -c 'import urllib.parse,sys; print(urllib.parse.quote(sys.argv[1], safe=""))' "$1"; }
ROOT_Q="$(urlencode "$PROJECT_ROOT")"
START_JSON="$TMP_DIR/status-start.json"
END_JSON="$TMP_DIR/status-end.json"
FAILURES=0
ITER=0
MAX_RSS=0
curl_json(){
  local name="$1"
  local path="$2"
  local out="$TMP_DIR/${name}-${ITER}.json"
  if ! curl -fsS --max-time 10 "$BASE$path" >"$out" 2>"$out.err"; then
    echo "✗ FAIL: $name :: $(tail -c 240 "$out.err" 2>/dev/null)" >&2
    FAILURES=$((FAILURES+1)); return 1
  fi
  jq empty "$out" >/dev/null || { echo "✗ FAIL: $name invalid json" >&2; FAILURES=$((FAILURES+1)); return 1; }
}
profile(){
  local path="$1"
  curl -fsS --max-time 10 "$BASE/v1/status?summary_only=true" >"$path"
  local rss mode
  rss="$(jq -r '.resource_mode.rss_kb // .rss_kb // 0' "$path")"
  mode="$(jq -r '.resource_mode.mode // .mode // "unknown"' "$path")"
  [[ "$rss" =~ ^[0-9]+$ ]] && (( rss > MAX_RSS )) && MAX_RSS="$rss"
  if [[ "$mode" == "emergency" ]]; then
    echo "✗ FAIL: daemon entered emergency mode during soak" >&2
    FAILURES=$((FAILURES+1))
  fi
}
profile "$START_JSON"
START_TS="$(date +%s)"
END_TS=$((START_TS + DURATION_SECS))
while (( $(date +%s) < END_TS )); do
  ITER=$((ITER+1))
  CONT="soak-${START_TS}-${ITER}"
  CONT_Q="$(urlencode "$CONT")"
  curl_json health /v1/health || true
  curl_json contracts /v1/ontology/tool-contracts || true
  curl_json choreography /v1/ontology/tool-choreography || true
  curl_json project "/v1/project/identity?project_root=$ROOT_Q" || true
  curl_json trajectory "/v1/trajectory/view?project_root=$ROOT_Q&continuity_id=$CONT_Q&mode=summary" || true
  if (( ITER % 5 == 0 )); then
    BODY="$(jq -nc --arg root "$PROJECT_ROOT" --arg cont "$CONT" --arg key "soak-${START_TS}-${ITER}" '{mission:"Focusa soak",next_slice:"Continue soak",project_root:$root,continuity_id:$cont,checkpoint_reason:"manual",confidence:"high",canonical:true,promote:true,idempotency_key:$key,action_intent:{action_type:"soak_verify",target_ref:"FocusaSoak",verification_hooks:["health","trajectory","choreography"],status:"ready"}}')"
    WP_OUT="$TMP_DIR/workpoint-${ITER}.json"
    if curl -fsS --max-time 15 -X POST "$BASE/v1/workpoint/checkpoint" -H 'content-type: application/json' -d "$BODY" >"$WP_OUT" 2>"$WP_OUT.err"; then
      sleep 0.5
      curl_json "workpoint-current" "/v1/workpoint/current?project_root=$ROOT_Q&continuity_id=$CONT_Q" || true
      RESUME_BODY="$(jq -nc --arg root "$PROJECT_ROOT" --arg cont "$CONT" '{project_root:$root,continuity_id:$cont,mode:"compact_prompt"}')"
      RESUME_OUT="$TMP_DIR/resume-${ITER}.json"
      curl -fsS --max-time 15 -X POST "$BASE/v1/workpoint/resume" -H 'content-type: application/json' -d "$RESUME_BODY" >"$RESUME_OUT" 2>"$RESUME_OUT.err" || { echo "✗ FAIL: workpoint resume soak" >&2; FAILURES=$((FAILURES+1)); }
    else
      echo "✗ FAIL: workpoint checkpoint soak :: $(tail -c 240 "$WP_OUT.err" 2>/dev/null)" >&2
      FAILURES=$((FAILURES+1))
    fi
  fi
  profile "$TMP_DIR/status-${ITER}.json"
  sleep "$INTERVAL_SECS"
done
profile "$END_JSON"
START_RSS="$(jq -r '.resource_mode.rss_kb // .rss_kb // 0' "$START_JSON")"
END_RSS="$(jq -r '.resource_mode.rss_kb // .rss_kb // 0' "$END_JSON")"
END_MODE="$(jq -r '.resource_mode.mode // .mode // "unknown"' "$END_JSON")"
if (( FAILURES > 0 )); then
  echo "=== FOCUSA EXTENDED SOAK FAILED ==="
  echo "iterations=$ITER failures=$FAILURES start_rss_kb=$START_RSS end_rss_kb=$END_RSS max_rss_kb=$MAX_RSS mode=$END_MODE artifacts=$TMP_DIR"
  exit 1
fi
echo "✓ PASS: extended soak iterations=$ITER start_rss_kb=$START_RSS end_rss_kb=$END_RSS max_rss_kb=$MAX_RSS mode=$END_MODE artifacts=$TMP_DIR"
