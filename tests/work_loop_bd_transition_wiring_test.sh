#!/bin/bash
# SPEC-79/BD: continuous loop should record concrete BD transitions beyond claim-in-progress.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DAEMON_FILE="${ROOT_DIR}/crates/focusa-core/src/runtime/daemon.rs"
ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }
if rg -n 'record_bd_blocked_transition_if_possible|--append-notes' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "daemon has blocked transition note writer for BD"; else log_fail "daemon missing blocked transition BD writer"; fi
if rg -n 'record_bd_completion_transition_if_possible|args\(\["close", work_item_id, "--reason"' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "daemon has completion close transition writer for BD"; else log_fail "daemon missing completion close BD writer"; fi
if rg -n 'record_bd_blocked_transition_if_possible\(' "$DAEMON_FILE" >/dev/null 2>&1 && rg -n 'required verification not yet satisfied|implementation remains non-conformant with linked spec|linked spec implementation evidence not yet satisfied' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "blocked outcome paths invoke BD transition writer"; else log_fail "blocked outcomes missing BD transition write wiring"; fi
if rg -n 'linked_spec_implementation_evidenced\(|linked spec implementation evidence not yet satisfied' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "completion path requires linked-spec implementation evidence before BD close"; else log_fail "completion path missing linked-spec implementation evidence gate"; fi
if rg -n 'record_bd_completion_transition_if_possible\(|verified completion; continuous loop advanced outcome' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "completion outcome path invokes BD close transition writer"; else log_fail "completion outcome missing BD close transition wiring"; fi
if rg -n 'linked_spec_implementation_evidence_accepts_spec_and_file_anchored_completion|linked_spec_implementation_evidence_rejects_non_implementation_reports|linked_spec_implementation_evidence_requires_spec_or_artifact_anchor' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "linked-spec evidence gate has explicit verification tests"; else log_fail "missing linked-spec evidence verification tests"; fi
if rg -n 'run_secondary_adversarial_closure_audit|secondary adversarial closure verifier|evaluate_secondary_closure_audit_payload' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "completion path includes secondary adversarial closure verifier gate"; else log_fail "missing secondary adversarial closure verifier gate"; fi
if rg -n 'secondary_closure_audit_payload_accepts_supported_sufficient_evidence|secondary_closure_audit_payload_rejects_critical_objections|secondary_closure_audit_payload_rejects_insufficient_evidence' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "secondary closure verifier has explicit payload validation tests"; else log_fail "missing secondary closure verifier payload validation tests"; fi
if rg -n 'build_bd_closure_certificate|record_bd_closure_certificate_if_possible|secondary_loop_closure_replay_evidence|closure_certificate=|replay_comparative=' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "completion path emits closure certificate evidence before BD close"; else log_fail "missing closure certificate evidence emission"; fi
if rg -n 'bd_closure_certificate_contains_policy_and_spec_refs|minimax_json_payload_parser_handles_wrapped_json|secondary_loop_closure_replay_evidence_reads_persisted_pairs' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "closure certificate + parser helpers have unit tests"; else log_fail "missing closure certificate/parser unit tests"; fi
if rg -n 'record_secondary_closure_trace|secondary_closure_trace_payloads|append_trace_event' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "closure path emits spec-56 trace dimensions for verifier decisions"; else log_fail "missing closure trace dimension emission"; fi
if rg -n 'secondary_closure_trace_payloads_include_required_dimensions|secondary_closure_trace_payloads_mark_blocked_transition_for_rejections' "$DAEMON_FILE" >/dev/null 2>&1; then log_pass "closure trace emission has explicit unit tests"; else log_fail "missing closure trace emission unit tests"; fi
if rg -n 'defer_work_item_for_alternate_switch|--defer", "\+1d"' "$ROUTE_FILE" >/dev/null 2>&1 && rg -n 'if blocked \{' "$ROUTE_FILE" >/dev/null 2>&1; then log_pass "alternate-ready switch path records explicit BD defer transition"; else log_fail "alternate-ready switch missing explicit BD defer transition"; fi
echo "=== WORK-LOOP BD TRANSITION WIRING RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
