#!/usr/bin/env bash
# Integration regression for focusa-bug-focus-stack-silent-loss.
#
# Acceptance: push a focus frame, query stack immediately, confirm frame is
# present. This test is live-daemon based; it exits 77 (TAP-style skip) when
# daemon is not reachable so static CI can still run without a daemon.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BASE_URL="${FOCUSA_DAEMON_URL:-http://127.0.0.1:8787}"
BEAD_ID="${FOCUSA_TEST_BEAD_ID:-focusa-bug-focus-stack-silent-loss}"
CONTINUITY_ID="focusa-focus-stack-test-$(date +%s)-$$"
TITLE="focus stack regression ${CONTINUITY_ID}"

if ! command -v curl >/dev/null 2>&1; then
  echo "SKIP: curl unavailable" >&2
  exit 77
fi
if ! curl -fsS "$BASE_URL/v1/health" >/dev/null 2>&1; then
  echo "SKIP: Focusa daemon not reachable at $BASE_URL" >&2
  exit 77
fi

session_payload=$(python3 - <<PY
import json
print(json.dumps({
  "adapter_id": "integration-test",
  "workspace_id": "focusa",
  "project_root": "$ROOT_DIR",
  "continuity_id": "$CONTINUITY_ID"
}))
PY
)
curl -fsS -X POST "$BASE_URL/v1/session/start" \
  -H 'content-type: application/json' \
  --data "$session_payload" >/dev/null

payload=$(python3 - <<PY
import json
print(json.dumps({
  "title": "$TITLE",
  "goal": "prove focus push is immediately visible in stack",
  "beads_issue_id": "$BEAD_ID",
  "constraints": ["no silent frame loss"],
  "tags": ["integration", "focus-stack-regression", "$CONTINUITY_ID"],
  "project_root": "$ROOT_DIR",
  "continuity_id": "$CONTINUITY_ID"
}))
PY
)

push_json=$(curl -fsS -X POST "$BASE_URL/v1/focus/push" \
  -H 'content-type: application/json' \
  --data "$payload")

frame_id=$(printf '%s' "$push_json" | python3 -c '
import json, sys
resp = json.load(sys.stdin)
status = resp.get("status")
if status != "accepted":
    raise SystemExit(f"FAIL: focus push not accepted: {json.dumps(resp, sort_keys=True)}")
frame_id = resp.get("frame_id")
if not frame_id:
    raise SystemExit(f"FAIL: focus push accepted without frame_id: {json.dumps(resp, sort_keys=True)}")
print(frame_id)
')

stack_json=$(curl -fsS "$BASE_URL/v1/focus/stack?limit=200")
printf '%s' "$stack_json" | python3 -c '
import json, sys
frame_id = sys.argv[1]
stack = json.load(sys.stdin)
frames = stack.get("stack", {}).get("frames", [])
window = stack.get("frames_window", [])
all_frames = frames + window
if not any(str(frame.get("id")) == frame_id for frame in all_frames):
    raise SystemExit(f"FAIL: pushed frame_id {frame_id} missing from focus stack; stack returned {len(all_frames)} frames")
print(f"PASS: pushed frame {frame_id} appears in focus stack")
' "$frame_id"

# Cleanup: complete the test frame so repeated runs do not leave active frames.
curl -fsS -X POST "$BASE_URL/v1/focus/pop" \
  -H 'content-type: application/json' \
  --data '{}' >/dev/null
