#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AWARENESS="$ROOT_DIR/docs/current/FOCUSA_MODEL_VISIBLE_AWARENESS.md"
WORKPOINT="$ROOT_DIR/docs/current/WORKPOINT_SESSION_SCOPE_GUARD.md"
SPEC="$ROOT_DIR/docs/current/PROJECT_SCOPE_OVERRIDE_INCIDENT_AND_GUARD_SPEC_2026-05-31.md"

fail() { echo "✗ FAIL: $1" >&2; exit 1; }
pass() { echo "✓ PASS: $1"; }

rg -n 'Stored memory|Retrieved memory|Attended memory|Action authority|MEMORY_ANCHOR|ATTENTION_RECALL_VERDICT|CURRENT_ASK_SCOPE_VERDICT|scope_conflict_detected' "$AWARENESS" >/dev/null \
  || fail "model-visible awareness doc lacks stored/retrieved/attended/action-authority explanation"
pass "Awareness doc explains memory layers and action authority"

rg -n 'canonical_for_saved_scope|action_authority_for_current_ask|CURRENT_ASK_SCOPE_VERDICT|detect_semantic_project_scope_conflict|CurrentScopeVerdict|scope_conflict_detected' "$WORKPOINT" >/dev/null \
  || fail "Workpoint scope guard doc lacks current-ask action authority/runtime fields"
pass "Workpoint doc explains saved canonicality vs current action authority"

rg -n 'implementation-backed guard spec|Implementation status snapshot|tests/spec_attention_recall_anchor_static_test.sh|tests/scope_routing_regression_eval.sh|tests/spec97_semantic_scope_conflict_primitive_static_test.sh' "$SPEC" >/dev/null \
  || fail "incident spec lacks implementation snapshot and proof handles"
pass "Incident spec cites implementation proof handles"

rg -n 'rejected hypotheses|Agreement with an operator correction is not proof|Focus Slice/compaction block|telemetry trace|project-switch ledger' "$AWARENESS" "$SPEC" >/dev/null \
  || fail "docs do not require evidence surfaces and rejected hypotheses for failure reports"
pass "Docs require evidence-backed failure reports"

echo "Attention/action-authority docs static test: PASS"
