#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AUDIT="$ROOT_DIR/docs/current/PI_EXTENSION_FINAL_TOOLSET_AUDIT.md"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
AWARE="$ROOT_DIR/apps/pi-extension/src/awareness.ts"
SESSION="$ROOT_DIR/apps/pi-extension/src/session.ts"
COMPACT="$ROOT_DIR/apps/pi-extension/src/compaction.ts"
STATE="$ROOT_DIR/apps/pi-extension/src/state.ts"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"
CONTRACTS="$ROOT_DIR/docs/current/focusa-tool-contracts.json"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for path in "$AUDIT" "$SPEC" "$AWARE" "$SESSION" "$COMPACT" "$STATE" "$TOOLS" "$CONTRACTS"; do
  [ -f "$path" ] || fail "required final audit ref missing $path"
done
for section in 'Surfaces reviewed' 'Redundant/noisy calls removed or justified' 'Authority compliance' 'Exact-scope rejection evidence' 'Final proof commands' 'Completion rule'; do
  rg -n -F "$section" "$AUDIT" >/dev/null || fail "final audit doc missing section $section"
done
pass "final audit doc sections present"

for surface in 'Pi tool registry/contracts' 'Utility/agent cards' 'Post-compaction cards' 'Auto-bootstrap/session' 'Skills/agent guidance' 'Docs/current surfaces'; do
  rg -n -F "$surface" "$AUDIT" >/dev/null || fail "final audit missing reviewed surface $surface"
done
pass "all required Pi extension/card/bootstrap surfaces reviewed"

rg -n -F 'Golden route: Orient project/Trajectory/Workpoint; Execute active object + checkpoint; Prove with evidence; Learn via prediction/metacog; Recover with tool_doctor.' "$AWARE" >/dev/null || fail "Utility Card missing tightened Golden route"
! rg -n -F 'Ontology = focusa_traverse(surface=ontology)' "$AWARE" >/dev/null || fail "Utility Card still contains old overlong route hint"
for marker in 'HLT' 'MLG' 'STG' 'Waypoints' 'Workpoint' 'authority/advisory'; do
  rg -n -F "$marker" "$AUDIT" >/dev/null || fail "final audit missing vocabulary marker $marker"
done
pass "utility/agent card tightening preserves canonical vocabulary"

for marker in \
  'export function isWorkpointPacketScopedToCurrentSession' \
  'currentProjectRoot !== packetProjectRoot' \
  'currentContinuityId !== packetContinuityId' \
  'packet.canonical === false' \
  'status === "rejected_scope_mismatch"' \
  'export function getScopedWorkpointPacket'; do
  rg -n -F "$marker" "$STATE" >/dev/null || fail "exact-scope helper missing $marker"
done
pass "exact-scope stale/mismatched Workpoint rejection is implemented"

for path in "$AWARE" "$COMPACT"; do
  rg -n -F 'getScopedWorkpointPacket' "$path" >/dev/null || fail "$(basename "$path") missing scoped packet use"
done
for path in "$SESSION" "$COMPACT"; do
  rg -n -F 'isWorkpointPacketScopedToCurrentSession' "$path" >/dev/null || fail "$(basename "$path") missing exact-scope check"
  rg -n -F 'normalizeWorkpointResumePacketEnvelope' "$path" >/dev/null || fail "$(basename "$path") missing packet envelope normalization"
done
pass "bootstrap/post-compaction flows use scoped packet checks"

for marker in 'canonical' 'advisory' 'degraded' 'tool_result_v1' 'failure_class' 'next_tools' 'evidence_refs'; do
  rg -n -F "$marker" "$TOOLS" "$CONTRACTS" >/dev/null || fail "tool envelope marker missing $marker"
done
for marker in 'json_assert_degraded_envelope' 'daemon_unavailable' 'resource_exhausted'; do
  rg -n -F "$marker" "$ROOT_DIR/tests/pi_extension_contract_test.sh" "$AUDIT" >/dev/null || fail "strict contract degraded-envelope marker missing $marker"
done
pass "tool_result_v1 degraded envelope markers preserved"

for marker in 'PI_EXTENSION_FINAL_TOOLSET_AUDIT.md' 'pi_extension_final_toolset_audit_static_test.sh'; do
  rg -n -F "$marker" "$SPEC" >/dev/null || fail "Spec106 missing final gate marker $marker"
done
pass "Spec106 references final Pi extension audit proof"

echo "pi extension final toolset audit static test: PASS"
