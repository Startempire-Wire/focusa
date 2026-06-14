#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/TEAM_MULTI_AGENT_FEDERATION_PLAN.md"
SCOPE="$ROOT_DIR/docs/current/MULTI_AGENT_SCOPE_MODEL.md"
CONTRACT="$ROOT_DIR/docs/current/AGENT_ADAPTER_CONTRACT.md"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$DOC" ] || fail "TEAM_MULTI_AGENT_FEDERATION_PLAN.md missing"
[ -f "$SCOPE" ] || fail "MULTI_AGENT_SCOPE_MODEL.md missing"
[ -f "$CONTRACT" ] || fail "AGENT_ADAPTER_CONTRACT.md missing"
for section in 'Federation model' 'Roles' 'Handoff protocol' 'Conflict resolution' 'Evidence sharing' 'Non-goals' 'Proof'; do
  rg -n -F "$section" "$DOC" >/dev/null || fail "federation plan missing section $section"
done
pass "federation plan sections present"

for marker in 'Project root identifies the project authority boundary' 'Continuity id identifies the logical workstream authority boundary' 'Session id is temporal metadata only' 'Workpoint is immediate continuation authority' 'Trajectory is north-star route context'; do
  rg -n -F "$marker" "$DOC" >/dev/null || fail "federation model missing $marker"
done
pass "federation authority model present"

for role in 'Operator' 'Primary agent' 'Reviewer agent' 'Background agent' 'Adapter'; do
  rg -n -F "$role" "$DOC" >/dev/null || fail "federation role missing $role"
done
pass "federation roles present"

for marker in 'Verify project identity' 'Resume or checkpoint Workpoint' 'Link evidence refs' 'do-not-drift boundaries' 'transcript tail is never authority'; do
  rg -n -F "$marker" "$DOC" >/dev/null || fail "handoff marker missing $marker"
done
pass "federation handoff protocol present"

for marker in 'Operator steering supersedes' 'Same project root does not imply same Workpoint' 'Similar mission text does not merge continuity ids' 'Writer conflicts pause/resume through work-loop writer status/preflight'; do
  rg -n -F "$marker" "$DOC" >/dev/null || fail "conflict marker missing $marker"
done
pass "federation conflict rules present"

rg -n -F 'Same project root does not imply same Workpoint' "$SCOPE" >/dev/null || fail "scope model missing same-project rule"
rg -n -F 'Adapters stay thin' "$CONTRACT" >/dev/null || fail "adapter contract missing thin adapter rule"
for marker in 'TEAM_MULTI_AGENT_FEDERATION_PLAN.md' 'team_multi_agent_federation_static_test.sh'; do
  rg -n -F "$marker" "$SPEC" >/dev/null || fail "Spec106 missing federation proof marker $marker"
done
pass "Spec106 references federation proof"

echo "team multi-agent federation static test: PASS"
