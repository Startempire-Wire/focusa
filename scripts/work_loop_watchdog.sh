#!/usr/bin/env bash
set -euo pipefail

BASE="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
SLEEP_SECS="${WATCHDOG_SLEEP_SECS:-2}"

pick_parent_epic() {
  (cd /home/wirebot/focusa && bd list --type epic --status open --status in_progress 2>/dev/null | awk '{print $2}' | head -n1) || true
}

while true; do
  status_json=$(curl -sS "$BASE/v1/work-loop/status" || true)
  if [[ -z "${status_json}" ]]; then
    sleep "$SLEEP_SECS"
    continue
  fi

  enabled=$(jq -r '.enabled // false' <<<"$status_json")
  writer=$(jq -r '.active_writer // empty' <<<"$status_json")
  status=$(jq -r '.status // empty' <<<"$status_json")
  current_task=$(jq -r '.current_task.work_item_id // empty' <<<"$status_json")
  session_id=$(jq -r '.transport.daemon_supervised_session.session_id // empty' <<<"$status_json")

  if [[ "$enabled" != "true" || -z "$writer" ]]; then
    sleep "$SLEEP_SECS"
    continue
  fi

  if [[ -z "$session_id" ]]; then
    curl -sS -X POST "$BASE/v1/work-loop/driver/start" \
      -H 'Content-Type: application/json' \
      -H "x-focusa-writer-id: $writer" \
      -d '{"cwd":"/home/wirebot/focusa"}' >/dev/null || true
  fi

  if [[ -z "$current_task" && ( "$status" == "awaiting_harness_turn" || "$status" == "idle" || "$status" == "selecting_ready_work" ) ]]; then
    parent=$(pick_parent_epic)
    if [[ -n "$parent" ]]; then
      curl -sS -X POST "$BASE/v1/work-loop/select-next" \
        -H 'Content-Type: application/json' \
        -H "x-focusa-writer-id: $writer" \
        -d "{\"parent_work_item_id\":\"$parent\"}" >/dev/null || true
    fi
  fi

  sleep "$SLEEP_SECS"
done
