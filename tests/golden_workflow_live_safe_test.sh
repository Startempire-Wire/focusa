#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
PROJECT_ROOT="${FOCUSA_PROJECT_ROOT:-$ROOT_DIR}"
CONTINUITY_ID="${FOCUSA_CONTINUITY_ID:-golden-workflow-live-safe}"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }
urlenc(){ python3 -c 'import sys,urllib.parse; print(urllib.parse.quote(sys.argv[1]))' "$1"; }

if ! curl -fsS --max-time 3 "$BASE_URL/v1/health" >/dev/null 2>&1; then
  echo "SKIP: Focusa daemon unavailable at $BASE_URL"
  exit 0
fi
pass "daemon health reachable"

project_q="$(urlenc "$PROJECT_ROOT")"
cont_q="$(urlenc "$CONTINUITY_ID")"

curl -fsS --max-time 8 "$BASE_URL/v1/project/identity?project_root=$project_q" >/tmp/golden-project.json \
  || fail "project identity request failed"
jq -e '.status == "verified" or .project_identity.status == "verified" or .project_root != null' /tmp/golden-project.json >/dev/null \
  || fail "project identity response lacks verified/project scope"
pass "ProjectIdentity route returns scoped project data"

curl -fsS --max-time 8 "$BASE_URL/v1/trajectory/view?mode=summary&project_root=$project_q&continuity_id=$cont_q" >/tmp/golden-trajectory.json \
  || fail "trajectory view request failed"
jq -e '.canonical != null or .degraded != null or .status != null' /tmp/golden-trajectory.json >/dev/null \
  || fail "trajectory view response lacks status posture"
pass "Trajectory view returns status/posture"

echo "golden workflow live-safe test: PASS"
