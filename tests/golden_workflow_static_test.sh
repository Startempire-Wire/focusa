#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/GOLDEN_WORKFLOW.md"
SCRIPT="$ROOT_DIR/scripts/demo-golden-workflow.sh"
CHOREO="$ROOT_DIR/docs/current/focusa-tool-choreography.json"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[[ -f "$DOC" ]] || fail "missing GOLDEN_WORKFLOW.md"
[[ -x "$SCRIPT" ]] || fail "missing executable demo-golden-workflow.sh"

for step in \
  "Verify ProjectIdentity" \
  "Load or define HLT / Trajectory Hierarchy" \
  "Create or resume Workpoint" \
  "Generate Context Cognition packet" \
  "Create Call Stack Design" \
  "Run implementation" \
  "Capture Evidence Refs" \
  "Link evidence to Workpoint" \
  "Evaluate prediction/metacog outcomes" \
  "Save session transfer" \
  "Resume after compaction/model switch" \
  "Reconcile, advance or settle with proof"; do
  rg -n "$step" "$DOC" >/dev/null || fail "Golden Workflow missing step: $step"
done
pass "Golden Workflow preserves its 12 capability/proof obligations"
rg -n 'Conditional capability crosswalk' "$DOC" >/dev/null || fail "capabilities must be conditional, not a mandatory sequence"
rg -n 'Bootstrap/Genesis when needed' "$DOC" >/dev/null || fail "workflow must connect conditional project initialization"
rg -n 'does not stop unfinished admitted work' "$DOC" >/dev/null || fail "reporting must not terminate unfinished work"
pass "Golden Workflow connects initialization and verified autonomous advancement"

for tool in focusa_project_identity focusa_trajectory_view focusa_workpoint_resume focusa_context_cognition focusa_call_stack_design focusa_evidence_capture focusa_workpoint_link_evidence focusa_session_transfer; do
  rg -n "$tool" "$DOC" >/dev/null || fail "Golden Workflow doc missing tool $tool"
done
pass "Golden Workflow names required tool route"

rg -n 'AUTHORITY_MODEL\.md' "$DOC" >/dev/null || fail "Golden Workflow must reference Authority Model"
rg -n 'canonical/advisory/degraded/blocked/stale' "$DOC" >/dev/null || fail "Golden Workflow must show posture labels"
pass "Golden Workflow preserves authority/posture boundary"

[[ -f "$CHOREO" ]] || fail "missing tool choreography registry"
pass "tool choreography registry present for follow-up alignment"

echo "golden workflow static test: PASS"
