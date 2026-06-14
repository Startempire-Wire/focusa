#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONTRACT="$ROOT_DIR/docs/current/AGENT_ADAPTER_CONTRACT.md"
NONPI="$ROOT_DIR/docs/current/NON_PI_AGENT_FOCUSA_USAGE.md"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
AWARENESS="$ROOT_DIR/crates/focusa-api/src/routes/awareness.rs"
SKILL="$ROOT_DIR/apps/pi-extension/skills/focusa/SKILL.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$CONTRACT" ] || fail "AGENT_ADAPTER_CONTRACT.md missing"
for needle in \
  'Adapters stay thin' \
  'Focusa daemon/core remains cognitive authority' \
  'Read awareness card' \
  'Verify project identity' \
  'Resume Workpoint' \
  'Create Workpoint checkpoint' \
  'Capture evidence' \
  'Link evidence' \
  'Run Context Authority preflight' \
  'Render Context Cognition compact packet' \
  'Surface `tool_result_v1` envelopes' \
  'Respect canonical/advisory/degraded states'; do
  rg -n -F "$needle" "$CONTRACT" >/dev/null || fail "contract missing $needle"
done
pass "Agent Adapter Contract declares minimum capability list"

for adapter in 'Pi' 'Codex CLI' 'Claude Code' 'OpenCode' 'OpenClaw' 'generic shell agent' 'MCP-compatible agents'; do
  rg -n -F "$adapter" "$CONTRACT" >/dev/null || fail "contract missing adapter $adapter"
done
pass "Agent Adapter Contract lists target adapters"

for needle in \
  'project_root + continuity_id' \
  'session_id` is temporal metadata' \
  'Transcript tail is never authority' \
  'Risky mutation preflight' \
  'failure_class' \
  'preserve proof handles'; do
  rg -n -F "$needle" "$CONTRACT" >/dev/null || fail "contract missing boundary/failure marker $needle"
done
pass "Agent Adapter Contract preserves authority and recovery boundaries"

for file in "$NONPI" "$SPEC"; do
  rg -n -F 'AGENT_ADAPTER_CONTRACT.md' "$file" >/dev/null || fail "$file missing contract link"
  rg -n -F 'tool_result_v1' "$file" >/dev/null || fail "$file missing tool_result_v1 adapter requirement"
done
pass "Spec106 and Non-Pi usage reference adapter contract"

rg -n -F '/v1/awareness/card' "$AWARENESS" >/dev/null || fail "awareness card route missing"
rg -n -F 'focusa_workpoint_resume' "$SKILL" >/dev/null || fail "Pi skill missing Workpoint resume guidance"
pass "existing adapter surfaces expose awareness and Pi Workpoint guidance"

echo "agent adapter contract static test: PASS"
