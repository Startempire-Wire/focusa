#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/NON_PI_AGENT_ADAPTER_EXAMPLES.md"
CONTRACT="$ROOT_DIR/docs/current/AGENT_ADAPTER_CONTRACT.md"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$DOC" ] || fail "NON_PI_AGENT_ADAPTER_EXAMPLES.md missing"
[ -f "$CONTRACT" ] || fail "AGENT_ADAPTER_CONTRACT.md missing"
for agent in 'Codex CLI' 'Claude Code' 'OpenCode' 'OpenClaw / Wirebot' 'Generic shell agent' 'MCP-compatible agents'; do
  rg -n -F "$agent" "$DOC" >/dev/null || fail "adapter examples missing $agent"
done
pass "target non-Pi adapters documented"

for command in \
  'focusa awareness card --json' \
  'focusa project verify' \
  'focusa workpoint resume' \
  'focusa action preflight' \
  'focusa evidence capture' \
  'focusa context-cognition render' \
  'curl -fsS http://127.0.0.1:8787/v1/awareness/card' \
  'focusa_workpoint_checkpoint'; do
  rg -n -F "$command" "$DOC" >/dev/null || fail "adapter examples missing command $command"
done
pass "adapter examples include CLI/HTTP/MCP command paths"

for marker in \
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
  rg -n -F "$marker" "$DOC" >/dev/null || fail "adapter checklist missing $marker"
  rg -n -F "$marker" "$CONTRACT" >/dev/null || fail "contract missing $marker"
done
pass "adapter examples mirror minimal contract checklist"

for marker in 'Focusa daemon/core remains cognitive authority' 'project_root + continuity_id' 'transcript tail' 'canonical' 'advisory' 'degraded' 'tool_result_v1'; do
  rg -n -F "$marker" "$DOC" >/dev/null || fail "adapter examples missing authority marker $marker"
done
pass "adapter examples preserve authority semantics"

for marker in 'NON_PI_AGENT_ADAPTER_EXAMPLES.md' 'non_pi_agent_adapter_examples_static_test.sh'; do
  rg -n -F "$marker" "$SPEC" >/dev/null || fail "Spec106 missing adapter proof marker $marker"
done
pass "Spec106 references adapter example proof"

echo "non-Pi agent adapter examples static test: PASS"
