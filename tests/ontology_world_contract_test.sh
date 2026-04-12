#!/bin/bash
# SPEC 45-48: Ontology world contract
# Verifies bounded runtime ontology projection exists beyond the Pi hot path.

set -euo pipefail

BASE_URL="${FOCUSA_BASE_URL:-http://127.0.0.1:8787}"
FAILED=0
PASSED=0

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_pass() { echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail() { echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
log_info() { echo -e "${YELLOW}INFO${NC}: $1"; }

http_code() {
  curl -sS -o /tmp/focusa-ontology-world-body.json -w "%{http_code}" "$@"
}

json_assert() {
  local expr="$1"
  local desc="$2"
  if jq -e "$expr" /tmp/focusa-ontology-world-body.json >/dev/null 2>&1; then
    log_pass "$desc"
  else
    log_fail "$desc :: $(cat /tmp/focusa-ontology-world-body.json)"
  fi
}

echo "=== SPEC 45-48: Ontology world contract ==="
echo "Base URL: ${BASE_URL}"
echo ""

log_info "Seed bounded working world"
WORKSPACE_ROOT="/tmp/focusa-ontology-workspace-$(date +%s%N)"
mkdir -p "${WORKSPACE_ROOT}/.git" "${WORKSPACE_ROOT}/src/routes" "${WORKSPACE_ROOT}/tests" "${WORKSPACE_ROOT}/migrations"
cat > "${WORKSPACE_ROOT}/Cargo.toml" <<'EOF'
[package]
name = "ontology-fixture"
version = "0.1.0"

[dependencies]
serde = "1"
EOF
cat > "${WORKSPACE_ROOT}/src/lib.rs" <<'EOF'
pub struct FixtureState;
pub fn build_fixture() {}
EOF
cat > "${WORKSPACE_ROOT}/src/routes/api.rs" <<'EOF'
use axum::{routing::get, Router};
pub fn router() -> Router {
    Router::new().route("/fixture", get(handler))
}
async fn handler() {}
EOF
cat > "${WORKSPACE_ROOT}/tests/fixture_test.rs" <<'EOF'
#[test]
fn fixture_works() { assert!(true); }
EOF
cat > "${WORKSPACE_ROOT}/migrations/001_init.sql" <<'EOF'
create table widgets(id integer primary key);
EOF
FRAME_TITLE="ontology-world-$(date +%s%N)"
FRAME_GOAL="verify broader ontology projection"
curl -sS -X POST "${BASE_URL}/v1/session/close" -H "Content-Type: application/json" -d '{"reason":"ontology-world-reset"}' >/dev/null || true
curl -sS -X POST "${BASE_URL}/v1/session/start" -H "Content-Type: application/json" -d "{\"workspace_id\":\"${WORKSPACE_ROOT}\"}" >/dev/null
workspace_ready=0
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30; do
  if curl -sS "${BASE_URL}/v1/status" | jq -e --arg root "$WORKSPACE_ROOT" '.session.workspace_id == $root' >/dev/null 2>&1; then
    workspace_ready=1
    break
  fi
  sleep 0.2
done
if [ "$workspace_ready" = "1" ]; then
  log_pass "Workspace session materialized"
else
  log_info "Workspace session metadata not yet visible; waiting on world projection instead"
fi
curl -sS -X POST "${BASE_URL}/v1/focus/push" -H "Content-Type: application/json" -d "{\"title\":\"${FRAME_TITLE}\",\"goal\":\"${FRAME_GOAL}\",\"beads_issue_id\":\"ontology-001\"}" >/dev/null
frame_id=""
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15; do
  frame_id=$(curl -sS "${BASE_URL}/v1/focus/stack" | jq -r --arg title "$FRAME_TITLE" '.stack.frames | map(select(.title == $title)) | last | .id // empty')
  if [ -n "$frame_id" ] && [ "$frame_id" != "null" ]; then
    break
  fi
  sleep 0.2
done
curl -sS -X POST "${BASE_URL}/v1/ascc/update-delta" -H "Content-Type: application/json" \
  -d "{\"frame_id\":\"${frame_id}\",\"delta\":{\"decisions\":[\"Use bounded ontology world projection\"],\"constraints\":[\"No unbounded ontology blob\"],\"failures\":[\"Software world gap under test\"],\"recent_results\":[\"Projection route added\"]}}" >/dev/null
curl -sS -X POST "${BASE_URL}/v1/ecs/store" -H "Content-Type: application/json" \
  -d '{"kind":"text","label":"ontology-artifact","content":"artifact for ontology world contract","surface":"test"}' >/dev/null
seeded=0
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49 50 51 52 53 54 55 56 57 58 59 60; do
  if curl -sS "${BASE_URL}/v1/ascc/frame/${frame_id}" | jq -e '.focus_state.decisions | index("Use bounded ontology world projection")' >/dev/null 2>&1; then
    seeded=1
    break
  fi
  sleep 0.2
done
if [ "$seeded" = "1" ]; then
  log_pass "Ontology seed state materialized"
else
  log_fail "Ontology seed state did not materialize"
  echo ""
  echo "=== ONTOLOGY WORLD CONTRACT RESULTS ==="
  echo "Tests passed: ${PASSED}"
  echo "Tests failed: ${FAILED}"
  echo ""
  exit 1
fi

log_info "Primitive catalog"
code=$(http_code "${BASE_URL}/v1/ontology/primitives")
if [ "$code" = "200" ]; then
  json_assert '.object_types | length > 10' "ObjectType catalog exposed"
  json_assert '.object_types | any(.type_name == "decision") and any(.type_name == "artifact") and any(.type_name == "goal")' "Core object families present"
  json_assert '.link_types | any(.name == "depends_on") and any(.name == "verifies") and any(.name == "blocks")' "Required link types present"
  json_assert '.action_types | any(.name == "refactor_module") and any(.name == "add_test") and any(.name == "rollback_change")' "Required action types present"
  json_assert '.status_vocabulary | index("canonical") and index("verified") and index("blocked")' "Status vocabulary present"
  json_assert '.provenance_classes | index("tool_derived") and index("reducer_promoted") and index("verification_confirmed")' "Provenance classes present"
else
  log_fail "Ontology primitives endpoint failed with HTTP ${code}"
fi

log_info "Runtime ontology world projection"
world_ready=0
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30; do
  curl -sS "${BASE_URL}/v1/ontology/world?frame_id=${frame_id}" > /tmp/focusa-ontology-world-body.json
  if jq -e '.objects | any(.object_type == "repo")' /tmp/focusa-ontology-world-body.json >/dev/null 2>&1; then
    world_ready=1
    break
  fi
  sleep 0.2
done
if [ "$world_ready" = "1" ]; then
  log_pass "Workspace world projection materialized"
fi
code=$(http_code "${BASE_URL}/v1/ontology/world?frame_id=${frame_id}")
if [ "$code" = "200" ]; then
  json_assert '.working_sets.active_mission_set.count >= 1' "Active mission working set exposed"
  json_assert '.working_sets.debugging_set.count >= 1 and .working_sets.refactor_set.count >= 1 and .working_sets.architecture_set.count >= 1' "Multiple bounded working-set types exposed"
  if jq -e --arg title "$FRAME_TITLE" --arg goal "$FRAME_GOAL" '.objects | any(.object_type == "active_focus" and .title == $title) and any(.object_type == "goal" and .objective == $goal)' /tmp/focusa-ontology-world-body.json >/dev/null 2>&1; then
    log_pass "Mission world objects projected"
  else
    log_fail "Mission world objects projected :: $(cat /tmp/focusa-ontology-world-body.json)"
  fi
  json_assert '.objects | any(.object_type == "decision" and .statement == "Use bounded ontology world projection")' "Decision object projected canonically"
  json_assert '.objects | any(.object_type == "constraint" and .rule_text == "No unbounded ontology blob")' "Constraint object projected canonically"
  json_assert '.objects | any(.object_type == "failure" and .summary == "Software world gap under test")' "Failure object projected canonically"
  json_assert '.objects | any(.object_type == "verification" and .result == "Projection route added")' "Verification object projected canonically"
  json_assert '.objects | any(.object_type == "artifact")' "Artifact object projected canonically"
  json_assert '.objects | any(.object_type == "repo") and any(.object_type == "package") and any(.object_type == "module") and any(.object_type == "file") and any(.object_type == "symbol") and any(.object_type == "route") and any(.object_type == "endpoint") and any(.object_type == "schema") and any(.object_type == "migration") and any(.object_type == "dependency") and any(.object_type == "test") and any(.object_type == "environment")' "Code-world object families projected from workspace"
  json_assert '.links | any(.type == "belongs_to_goal") and any(.type == "blocks") and any(.type == "verifies") and any(.type == "depends_on") and any(.type == "tested_by") and any(.type == "implements") and any(.type == "persists_to") and any(.type == "configured_by") and any(.type == "declared_in")' "Typed ontology links projected"
  json_assert '.action_catalog | any(.name == "refactor_module" and .reducer_visible == true)' "Action catalog projected"
else
  log_fail "Ontology world endpoint failed with HTTP ${code}"
fi

log_info "Slice endpoint surfaces"
code=$(http_code "${BASE_URL}/v1/ontology/slices?frame_id=${frame_id}&slice_type=active_mission")
if [ "$code" = "200" ]; then
  json_assert '.slice_type == "active_mission" and .count >= 1 and (.members | length >= 1)' "Active-mission slice endpoint bounded members"
else
  log_fail "Ontology slices endpoint failed with HTTP ${code}"
fi
code=$(http_code "${BASE_URL}/v1/ontology/slices?frame_id=${frame_id}&slice_type=architecture")
if [ "$code" = "200" ]; then
  json_assert '.slice_type == "architecture" and (.members | any(.object_type == "package" or .object_type == "module" or .object_type == "endpoint"))' "Architecture slice returns code-world members"
else
  log_fail "Architecture slice endpoint failed with HTTP ${code}"
fi
code=$(http_code "${BASE_URL}/v1/ontology/slices?frame_id=${frame_id}&slice_type=debugging")
if [ "$code" = "200" ]; then
  json_assert '.slice_type == "debugging" and (.members | any(.object_type == "failure" or .object_type == "verification"))' "Debugging slice returns runtime failure/verification members"
else
  log_fail "Debugging slice endpoint failed with HTTP ${code}"
fi

echo ""
echo "=== ONTOLOGY WORLD CONTRACT RESULTS ==="
echo "Tests passed: ${PASSED}"
echo "Tests failed: ${FAILED}"
echo ""

if [ $FAILED -eq 0 ]; then
  echo -e "${GREEN}Ontology world contract verified${NC}"
  exit 0
else
  echo -e "${RED}Ontology world contract failed${NC}"
  exit 1
fi
