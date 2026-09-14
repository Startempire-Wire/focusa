#!/usr/bin/env bash
# Spec125-17: Required read-only runtime/eval tests (§15.2).
#
# This script deliberately requires a caller-provided isolated daemon fixture.
# It must never turn an unavailable daemon, failed request, malformed response,
# or absent semantic field into passing evidence.
set -euo pipefail

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

API="${FOCUSA_SPEC125_API:-}"
PROJECT_ROOT="${FOCUSA_SPEC125_PROJECT_ROOT:-}"
CONTINUITY_ID="${FOCUSA_SPEC125_CONTINUITY_ID:-spec125-runtime-eval}"

[[ -n "$API" ]] || fail "FOCUSA_SPEC125_API must identify an isolated exact-version daemon fixture"
[[ -n "$PROJECT_ROOT" ]] || fail "FOCUSA_SPEC125_PROJECT_ROOT must identify the isolated fixture project"
case "$PROJECT_ROOT" in
  /tmp/focusa-spec125-*) ;;
  *) fail "FOCUSA_SPEC125_PROJECT_ROOT must be an isolated /tmp/focusa-spec125-* path" ;;
esac

request_json() {
  local label="$1"
  shift
  local response
  if ! response="$(curl --fail --show-error --silent --max-time 5 "$@")"; then
    fail "$label request failed"
  fi
  [[ -n "$response" ]] || fail "$label returned an empty response"
  if ! printf '%s' "$response" | jq -e . >/dev/null 2>&1; then
    fail "$label returned malformed JSON"
  fi
  printf '%s' "$response"
}

require_marker() {
  local label="$1"
  local response="$2"
  local marker="$3"
  if printf '%s' "$response" | grep -Eq "$marker"; then
    pass "$label"
  else
    fail "$label missing required semantic marker: $marker"
  fi
}

health="$(request_json "daemon health" "$API/health")"
require_marker "Daemon health reports a status" "$health" '"(status|ok|healthy)"'

echo "=== Spec125-15.2 Read-only Runtime/Eval Tests ==="

trajectory="$(request_json "Trajectory view" -X POST "$API/trajectory/view" -H "Content-Type: application/json" \
  -d "{\"project_root\":\"$PROJECT_ROOT\",\"continuity_id\":\"$CONTINUITY_ID\"}")"
require_marker "Trajectory view includes HLT state" "$trajectory" 'hlt_status|loud_warning|canonical'

history="$(request_json "HLT history" "$API/trajectory/hlt-history?project_root=$PROJECT_ROOT")"
require_marker "HLT history returns a typed projection" "$history" 'entries|history|items|status'

workpoint="$(request_json "Workpoint resume" -X POST "$API/workpoint/resume" -H "Content-Type: application/json" \
  -d "{\"project_root\":\"$PROJECT_ROOT\",\"continuity_id\":\"$CONTINUITY_ID\"}")"
require_marker "Workpoint resume returns a typed projection" "$workpoint" 'workpoint|status|next_action|canonical'

receipt="$(request_json "Preload receipt preview" "$API/preload/receipt-preview?profile=rules_and_context")"
require_marker "Preload receipt preview returns a typed projection" "$receipt" 'receipt|profile|status|canonical'

utility="$(request_json "Utility card" "$API/utility/card")"
require_marker "Utility card returns a typed projection" "$utility" 'utility|status|content|summary'

context="$(request_json "Context cognition" -X POST "$API/context-cognition" -H "Content-Type: application/json" \
  -d "{\"project_root\":\"$PROJECT_ROOT\"}")"
require_marker "Context cognition returns a typed projection" "$context" 'context|status|project|canonical'

cat <<'EOF'
✓ PASS: Read-only runtime cases completed against an isolated daemon fixture.
NOTE: trajectory define-goal is intentionally excluded here because it mutates state;
it requires a separate isolated mutation fixture with explicit cleanup evidence.
EOF
