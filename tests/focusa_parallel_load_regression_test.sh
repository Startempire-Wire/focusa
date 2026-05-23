#!/usr/bin/env bash
# Bounded concurrent hot-route regression: proves Focusa navigation endpoints survive audit bursts.
set -euo pipefail
BASE="${FOCUSA_API_BASE_URL:-http://127.0.0.1:8787}"
PROJECT_ROOT="${FOCUSA_PARALLEL_PROJECT_ROOT:-$(pwd -P)}"
CONTINUITY_ID="${FOCUSA_PARALLEL_CONTINUITY_ID:-parallel-smoke-$(date +%s)-$$}"
TMP_DIR="${TMPDIR:-/tmp}/focusa-parallel-load-$$"
mkdir -p "$TMP_DIR"
urlencode(){ python3 -c 'import urllib.parse,sys; print(urllib.parse.quote(sys.argv[1], safe=""))' "$1"; }
ROOT_Q="$(urlencode "$PROJECT_ROOT")"
CONT_Q="$(urlencode "$CONTINUITY_ID")"
ENDPOINT_FILE="$TMP_DIR/endpoints.txt"
{
  echo "/v1/health"
  echo "/v1/ontology/tool-contracts"
  echo "/v1/ontology/tool-choreography"
  echo "/v1/project/identity?project_root=$ROOT_Q"
  echo "/v1/trajectory/view?project_root=$ROOT_Q&continuity_id=$CONT_Q&mode=summary"
  echo "/v1/workpoint/current?project_root=$ROOT_Q&continuity_id=$CONT_Q"
  echo "/v1/resource/mode"
} > "$ENDPOINT_FILE"
export BASE TMP_DIR
awk '{for(i=0;i<4;i++) print $0}' "$ENDPOINT_FILE" | nl -ba | xargs -P 7 -I{} bash -c '
  line="$1"
  idx="${line%%$'"'"'\t'"'"'*}"
  ep="${line#*$'"'"'\t'"'"'}"
  out="$TMP_DIR/${idx}.json"
  err="$out.err"
  if ! curl -fsS --max-time 8 "$BASE$ep" >"$out" 2>"$err"; then
    echo "FAIL $ep :: $(tail -c 300 "$err" 2>/dev/null)" >&2
    exit 1
  fi
  jq -e ". != null" "$out" >/dev/null
' _ {}
STATUS_JSON="$TMP_DIR/status.json"
curl -fsS --max-time 10 "$BASE/v1/status?summary_only=true" >"$STATUS_JSON"
MODE="$(jq -r '.resource_mode.mode // .mode // "unknown"' "$STATUS_JSON")"
RSS="$(jq -r '.resource_mode.rss_kb // .rss_kb // 0' "$STATUS_JSON")"
if [[ "$MODE" == "emergency" ]]; then
  echo "✗ FAIL: daemon entered emergency mode after bounded parallel hot-route burst (rss=${RSS}KB)" >&2
  cat "$STATUS_JSON" >&2
  exit 1
fi
echo "✓ PASS: bounded parallel hot-route burst completed mode=$MODE rss_kb=$RSS artifacts=$TMP_DIR"
