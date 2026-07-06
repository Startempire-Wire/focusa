#!/usr/bin/env bash
# focusa-claude-code-hook.sh
# Spec 101 §5.12.11 — Claude Code recent-turns adapter (Adapter 2)
#
# Claude Code hooks receive JSON via stdin and may emit text via stdout which
# is appended to the model context. This script implements the Focusa adapter
# contract by:
#   1. Reading the hook event JSON from stdin
#   2. Detecting the hook type (UserPromptSubmit, SessionStart, Stop, etc.)
#   3. Calling the Focusa daemon for capture/inject/recall-trigger as appropriate
#   4. Failing soft (exit 0) when the daemon is unreachable
#
# Required tools: bash, curl, jq
# Configurable via env:
#   FOCUSA_DAEMON_URL  (default http://127.0.0.1:8787)
#   FOCUSA_CONTINUITY_ID  (default: auto-derived from CWD or session_id)
#   FOCUSA_N_DEFAULT  (default 4)
#   FOCUSA_RECALL_INTENT_WORDS_PATH  (optional; otherwise built-in default)

set -uo pipefail

DAEMON_URL="${FOCUSA_DAEMON_URL:-http://127.0.0.1:8787}"
N_DEFAULT="${FOCUSA_N_DEFAULT:-4}"
CONTINUITY_ID="${FOCUSA_CONTINUITY_ID:-}"

# Built-in recall-intent categories (mirrors spec §5.12.10).
# Pattern matched against normalized user text.
RECALL_PATTERNS='recall|remember|remind me|bring me back|catch up|orient me|refocus|rewind|earlier|last time|previously|where were we|as we discussed|we talked about|you mentioned|you said|i asked|i said|i meant|didn.t we|already covered|already done|already filed|duplicate|going in circles|^wait$|^hold on$|^back up$|^scratch that$|on track|where (were|are) we going|what.s the state|context'

log() { printf '[focusa-hook] %s\n' "$*" 1>&2; }

# Read stdin JSON. Bail silently if not parseable.
INPUT=""
if [[ ! -t 0 ]]; then
  INPUT="$(cat || true)"
fi

EVENT=""
PROMPT=""
SESSION_ID=""
CWD=""
TRANSCRIPT_PATH=""
if [[ -n "$INPUT" ]] && command -v jq >/dev/null 2>&1; then
  EVENT="$(printf '%s' "$INPUT" | jq -r '.hook_event_name // ""' 2>/dev/null || echo "")"
  PROMPT="$(printf '%s' "$INPUT" | jq -r '.prompt // .user_prompt // ""' 2>/dev/null || echo "")"
  SESSION_ID="$(printf '%s' "$INPUT" | jq -r '.session_id // ""' 2>/dev/null || echo "")"
  CWD="$(printf '%s' "$INPUT" | jq -r '.cwd // ""' 2>/dev/null || echo "")"
  TRANSCRIPT_PATH="$(printf '%s' "$INPUT" | jq -r '.transcript_path // ""' 2>/dev/null || echo "")"
fi

# Derive continuity_id if not provided
if [[ -z "$CONTINUITY_ID" ]]; then
  if [[ -n "$SESSION_ID" ]]; then
    CONTINUITY_ID="claude-code:$SESSION_ID"
  elif [[ -n "$CWD" ]]; then
    CONTINUITY_ID="claude-code:$(basename "$CWD")"
  else
    CONTINUITY_ID="claude-code:default"
  fi
fi

daemon_get_recent() {
  local n="$1" cid="$2"
  local url="${DAEMON_URL}/v1/turns/recent?n=${n}&continuity_id=$(printf '%s' "$cid" | sed 's/ /%20/g')"
  curl -sS --max-time 3 "$url" 2>/dev/null || true
}

daemon_append_turn() {
  local payload="$1"
  curl -sS --max-time 3 -X POST -H "Content-Type: application/json" \
    -d "$payload" "${DAEMON_URL}/v1/turns/recent" 2>/dev/null || true
}

daemon_recall_trigger() {
  local payload="$1"
  curl -sS --max-time 3 -X POST -H "Content-Type: application/json" \
    -d "$payload" "${DAEMON_URL}/v1/events/recall-trigger" 2>/dev/null || true
}

detect_recall_intent() {
  local text="${1:-}"
  if [[ -z "$text" ]] || [[ ${#text} -gt 240 ]]; then
    return 1
  fi
  local normalized
  normalized="$(printf '%s' "$text" | tr '[:upper:]' '[:lower:]' | tr -d '[:punct:]')"
  if printf '%s' "$normalized" | grep -qE "$RECALL_PATTERNS"; then
    printf 'matched'
    return 0
  fi
  return 1
}

format_recent_turns() {
  local json="$1" n="$2"
  if [[ -z "$json" ]] || ! command -v jq >/dev/null 2>&1; then
    return 0
  fi
  # Validate shape
  if ! printf '%s' "$json" | jq -e '.schema' >/dev/null 2>&1; then
    return 0
  fi
  local count
  count="$(printf '%s' "$json" | jq -r '.count // 0' 2>/dev/null || echo 0)"
  if [[ "$count" == "0" || -z "$count" ]]; then
    return 0
  fi
  printf '\n## Recent turns (last %s, Focusa daemon)\n' "$count"
  # Format each turn on its own line via jq -r with a safe filter.
  while IFS= read -r line; do
    [[ -n "$line" ]] && printf '%s\n' "$line"
  done < <(printf '%s' "$json" | jq -r '.turns[]? |
    "- T[" + .turn_id + "] mission=\"" + .mission_at_turn + "\" outcome=" + .outcome + " tools=" + (.tool_call_count|tostring) +
    (if (.evidence_refs|length) > 0 then " ev=" + (.evidence_refs|join(",")) else "" end)')
}

capture_from_transcript() {
  # Best-effort: extract the latest assistant text from the Claude Code
  # transcript JSONL. Skip if no transcript_path or jq missing.
  if [[ -z "$TRANSCRIPT_PATH" ]] || [[ ! -r "$TRANSCRIPT_PATH" ]] || ! command -v jq >/dev/null 2>&1; then
    return 0
  fi
  local last_assistant last_user outcome mission
  last_assistant="$(jq -r 'select(.type=="assistant") | .message.content[]? | select(.type=="text") | .text' "$TRANSCRIPT_PATH" 2>/dev/null | tail -1 || true)"
  last_user="$(jq -r 'select(.type=="user") | .message.content[]? | select(.type=="text") | .text' "$TRANSCRIPT_PATH" 2>/dev/null | tail -1 || true)"
  if [[ -z "$last_assistant" && -z "$last_user" ]]; then
    return 0
  fi
  # Skip status-only turns
  if printf '%s' "$last_user" | grep -qiE '^(test|cont|ack|ok|got it|continue|next|k|yes|no|y|n)\s*$'; then
    return 0
  fi
  outcome="tooled"
  mission="$(printf '%s' "$last_user" | head -c 120)"
  if [[ -z "$mission" ]]; then
    outcome="observed"
    mission="$(printf '%s' "$last_assistant" | head -c 120)"
  fi
  local payload
  payload=$(jq -nc \
    --arg turn_id "cc-${SESSION_ID:-default}-$(date +%s)" \
    --arg cid "$CONTINUITY_ID" \
    --arg mission "$mission" \
    --arg outcome "$outcome" \
    --argjson tool_calls 0 \
    '{turn_id: $turn_id, continuity_id: $cid, mission_at_turn: $mission, outcome: $outcome, evidence_refs: [], tool_call_count: $tool_calls, emitted_at: (now | floor)}' 2>/dev/null || true)
  if [[ -n "$payload" ]]; then
    daemon_append_turn "$payload"
  fi
}

case "$EVENT" in
  UserPromptSubmit)
    # Detect recall intent
    if detect_recall_intent "$PROMPT"; then
      daemon_recall_trigger "$(jq -nc \
        --arg cid "$CONTINUITY_ID" \
        --arg phrase '' \
        --argjson ring_size 0 \
        '{matched_category:"direct_recall", matched_phrase: $phrase, slice_size: 0, ring_size: $ring_size, forced_re_emit: true, alternative_tools_surfaced: ["focusa_lineage_tree","focusa_awareness_packet"], continuity_id: $cid, agent_kind: "claude-code"}' 2>/dev/null || true)"
    fi
    # Inject recent turns slice
    recent="$(daemon_get_recent "$N_DEFAULT" "$CONTINUITY_ID")"
    formatted="$(format_recent_turns "$recent" "$N_DEFAULT")"
    if [[ -n "$formatted" ]]; then
      printf '%s\n' "$formatted"
    fi
    # Always exit 0 (Claude Code treats non-zero as soft-block)
    exit 0
    ;;
  SessionStart)
    recent="$(daemon_get_recent "$N_DEFAULT" "$CONTINUITY_ID")"
    formatted="$(format_recent_turns "$recent" "$N_DEFAULT")"
    if [[ -n "$formatted" ]]; then
      printf '%s\n' "$formatted"
    fi
    exit 0
    ;;
  Stop|StopHook|PostToolUse|SubagentStop|Notification)
    # Capture turn
    capture_from_transcript
    exit 0
    ;;
  PreCompact)
    # Capture before compaction so we don't lose context
    capture_from_transcript
    exit 0
    ;;
  *)
    # Unknown event: noop
    exit 0
    ;;
esac
