#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKFLOW="$ROOT_DIR/crates/focusa-cli/src/commands/workflow.rs"
MAIN="$ROOT_DIR/crates/focusa-cli/src/main.rs"
MOD="$ROOT_DIR/crates/focusa-cli/src/commands/mod.rs"
DOC="$ROOT_DIR/docs/current/WORKFLOW_COMMAND.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$WORKFLOW" ] || fail "workflow.rs missing"
for needle in \
  'pub enum WorkflowCmd' \
  'List' \
  'Show' \
  'Apply' \
  'WorkflowTemplate' \
  'when_to_use' \
  'expected_outcome' \
  'commands' \
  'recovery_hint' \
  'long-refactor' \
  'multi-session-resume' \
  'incident-response' \
  'agent-handoff' \
  'feature-add' \
  'doc-update' \
  'assert_eq!(templates().len(), 6)'; do
  rg -n -F "$needle" "$WORKFLOW" >/dev/null || fail "workflow.rs missing marker: $needle"
done
pass "workflow command defines six canonical templates with outcome/commands/recovery"

rg -n -F 'pub mod workflow;' "$MOD" >/dev/null || fail "commands/mod.rs missing workflow module"
rg -n -F 'Workflow(commands::workflow::WorkflowCmd)' "$MAIN" >/dev/null || fail "main.rs missing Workflow command variant"
rg -n -F 'commands::workflow::run(cmd, cli.json).await' "$MAIN" >/dev/null || fail "main.rs missing Workflow dispatch"
pass "workflow command is wired into CLI"

[ -f "$DOC" ] || fail "WORKFLOW_COMMAND.md missing"
for needle in \
  'focusa workflow list --json' \
  'long-refactor' \
  'multi-session-resume' \
  'incident-response' \
  'agent-handoff' \
  'feature-add' \
  'doc-update' \
  'focusa-workflow-cmd'; do
  rg -n -F "$needle" "$DOC" >/dev/null || fail "workflow doc missing marker: $needle"
done
pass "workflow docs describe evaluator acceptance and usage"

echo "workflow command static test: PASS"
