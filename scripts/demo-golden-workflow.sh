#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
PROJECT_ROOT="${1:-${FOCUSA_PROJECT_ROOT:-$(pwd)}}"
CONTINUITY_ID="${FOCUSA_CONTINUITY_ID:-golden-workflow-demo}"

json_get(){
  # Entitlement-gated endpoints may answer 4xx with a bounded JSON body; the
  # demo must render that blocked/degraded result instead of aborting (#302).
  curl -sS --max-time 5 "$1"
}

printf 'Golden Workflow demo (safe/read-mostly)\n'
printf 'project_root=%s\ncontinuity_id=%s\nbase_url=%s\n\n' "$PROJECT_ROOT" "$CONTINUITY_ID" "$BASE_URL"

printf '1. Verify ProjectIdentity\n'
json_get "$BASE_URL/v1/project/identity?project_root=$(python3 -c 'import sys,urllib.parse; print(urllib.parse.quote(sys.argv[1]))' "$PROJECT_ROOT")" | jq '{status, project_root: .project_identity.project_root, canonical_name: .project_identity.canonical_name, project_id: .project_identity.project_id}'

printf '\n2. Load Trajectory Hierarchy\n'
json_get "$BASE_URL/v1/trajectory/view?mode=summary&project_root=$(python3 -c 'import sys,urllib.parse; print(urllib.parse.quote(sys.argv[1]))' "$PROJECT_ROOT")&continuity_id=$(python3 -c 'import sys,urllib.parse; print(urllib.parse.quote(sys.argv[1]))' "$CONTINUITY_ID")" | jq '{status, canonical, degraded, trajectory: {long_term_goal: .trajectory.long_term_goal, mid_level_goal: .trajectory.mid_level_goal, short_term_goal: .trajectory.short_term_goal}}'

printf '\n3. Render Context Cognition compact packet (advisory)\n'
json_get "$BASE_URL/v1/context-cognition/render?project_root=$(python3 -c 'import sys,urllib.parse; print(urllib.parse.quote(sys.argv[1]))' "$PROJECT_ROOT")&continuity_id=$(python3 -c 'import sys,urllib.parse; print(urllib.parse.quote(sys.argv[1]))' "$CONTINUITY_ID")" | jq '{status, canonical, degraded, workpoint_id, trajectory_id, rehydrate_id}'

printf '\nNext manual steps: create/resume Workpoint, create Call Stack Design, implement, capture/link Evidence Refs, evaluate prediction/metacog, save session transfer, final proof report.\n'
