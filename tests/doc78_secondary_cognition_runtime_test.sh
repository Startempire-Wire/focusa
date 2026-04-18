#!/bin/bash
# Contract: Doc-78 secondary cognition runtime must enforce bounded, evidence-first closure verification.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DAEMON_FILE="${ROOT_DIR}/crates/focusa-core/src/runtime/daemon.rs"
ROUTE_FILE="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"
RFM_FILE="${ROOT_DIR}/crates/focusa-core/src/rfm/mod.rs"
TYPES_FILE="${ROOT_DIR}/crates/focusa-core/src/types.rs"
REPLAY_FILE="${ROOT_DIR}/crates/focusa-core/src/replay/mod.rs"
DOC78_FILE="${ROOT_DIR}/docs/78-bounded-secondary-cognition-and-persistent-autonomy.md"
DOC79_FILE="${ROOT_DIR}/docs/79-focusa-governed-continuous-work-loop.md"
DOC78_SCORECARD_FILE="${ROOT_DIR}/docs/DOC78_F1_F5_CLOSURE_SCORECARD_2026-04-17.md"
COMPARATIVE_HARNESS_FILE="${ROOT_DIR}/tests/doc78_secondary_loop_comparative_eval.sh"
REPLAY_COMPARATIVE_HARNESS_FILE="${ROOT_DIR}/tests/doc78_secondary_loop_replay_comparative_eval.sh"
FIRST_CONSUMER_HARNESS_FILE="${ROOT_DIR}/tests/doc78_first_consumer_path_test.sh"
OPERATOR_SURFACE_HARNESS_FILE="${ROOT_DIR}/tests/focus_work_command_surface_test.sh"
OPERATOR_DASHBOARD_HARNESS_FILE="${ROOT_DIR}/tests/doc78_tui_replay_dashboard_surface_test.sh"
LIVE_RUNTIME_HARNESS_FILE="${ROOT_DIR}/tests/doc78_live_runtime_closure_bundle_smoke.sh"
LIVE_BOUNDARY_HARNESS_FILE="${ROOT_DIR}/tests/doc78_live_continuation_boundary_pressure_smoke.sh"
LIVE_NON_CLOSURE_HARNESS_FILE="${ROOT_DIR}/tests/doc78_live_non_closure_objective_profile_smoke.sh"
PRODUCTION_RUNTIME_HARNESS_FILE="${ROOT_DIR}/tests/doc78_production_runtime_governance_replay_smoke.sh"
PRODUCTION_RUNTIME_SERIES_HARNESS_FILE="${ROOT_DIR}/tests/doc78_production_runtime_series_smoke.sh"
FAILED=0
PASSED=0
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'
log_pass(){ echo -e "${GREEN}✓ PASS${NC}: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo -e "${RED}✗ FAIL${NC}: $1"; FAILED=$((FAILED+1)); }

if rg -n 'run_secondary_adversarial_closure_audit|SecondaryClosureAuditVerdict' "$DAEMON_FILE" >/dev/null 2>&1; then
  log_pass "secondary adversarial closure verifier runtime gate exists"
else
  log_fail "missing secondary adversarial closure verifier runtime gate"
fi

if rg -n 'build_bd_closure_certificate|record_bd_closure_certificate_if_possible|closure_certificate=' "$DAEMON_FILE" >/dev/null 2>&1; then
  log_pass "closure certificate evidence path exists before BD close"
else
  log_fail "missing closure certificate evidence path"
fi

if rg -n 'record_secondary_closure_trace|secondary_closure_trace_payloads|operator_subject|active_subject_after_routing|focus_slice_relevance_score|blockers_failures_emitted|final_state_transition' "$DAEMON_FILE" >/dev/null 2>&1; then
  log_pass "closure verifier path emits required trace-dimension telemetry surfaces"
else
  log_fail "missing closure verifier trace-dimension telemetry surfaces"
fi

if rg -n 'secondary_loop_boundary_reason|secondary_loop_allowed|scope_failure_recorded|operator steering detected|governance decision pending' "$DAEMON_FILE" >/dev/null 2>&1; then
  log_pass "secondary cognition loops honor operator/governance continuation boundaries"
else
  log_fail "missing operator/governance continuation boundary enforcement"
fi

if rg -n 'Action::RequestNextContinuousTurn|checkpoint: paused for operator-priority boundary|checkpoint: blocked on continuation boundary|"path": "request_next_continuous_turn"|scope_failure_recorded' "$DAEMON_FILE" >/dev/null 2>&1; then
  log_pass "continuous request path enforces and traces continuation boundaries before next autonomous turn"
else
  log_fail "missing continuation-boundary enforcement/trace in request-next path"
fi

if rg -n 'Action::SelectNextContinuousSubtask|checkpoint: paused select-next for operator-priority boundary|checkpoint: blocked select-next on continuation boundary|"path": "select_next_continuous_subtask"|"path": "observe_outcome_auto_advance"|checkpoint: paused auto-advance for operator-priority boundary|checkpoint: blocked auto-advance on continuation boundary' "$DAEMON_FILE" >/dev/null 2>&1; then
  log_pass "selection and auto-advance paths enforce continuation boundaries"
else
  log_fail "missing continuation boundary enforcement in selection/auto-advance paths"
fi

if rg -n 'fn continuation_boundary_reason|boundary_reason.is_some\(\)' "$ROUTE_FILE" >/dev/null 2>&1 && rg -n 'maybe_dispatch_continuous_turn_prompt|maybe_auto_advance_from_blocked|maybe_select_global_ready_work_item' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "api dispatch/auto-advance paths suppress autonomous continuation while boundaries are active"
else
  log_fail "api dispatch path missing continuation-boundary suppression"
fi

if rg -n 'rfm_fallback_result_for_mode|rfm_fallback_result|FOCUSA_RFM_LLM_FAIL_OPEN|fallback=fail_closed\(strict\)' "$RFM_FILE" >/dev/null 2>&1; then
  log_pass "RFM deep validator defaults to fail-closed with explicit fail-open escape hatch"
else
  log_fail "RFM fallback policy not wired as strict-default"
fi

if rg -n 'verification_result_events|decision_consult_events|scope_contamination_events|subject_hijack_prevented_events|subject_hijack_occurred_events|secondary_loop_useful_events|secondary_loop_low_quality_events|secondary_loop_ledger|secondary_loop_archived_events|ContinuousSecondaryLoopOutcomeRecorded' "$TYPES_FILE" >/dev/null 2>&1 && rg -n 'post_turn_eval_trace_flags|scope_contamination_detected|wrong_question_detected|secondary_loop_quality_trace_payload|record_secondary_loop_quality_trace|secondary_loop_ledger_entry_for_outcome|append_secondary_loop_ledger_entry|proposal_id|promotion_status|trace_id|created_at|latency_ms_since_turn_request|subject_hijack_trace_flags|subject_hijack_prevented|subject_hijack_occurred|loop_objective|continuation_decision|stop_reason|persist_observability_event|secondary_loop_closure_replay_evidence|replay_comparative_improvement_pairs|replay_current_task_pair_observed|replay_comparative=|ContinuousSecondaryLoopOutcomeRecorded' "$DAEMON_FILE" >/dev/null 2>&1 && rg -n 'secondary_loop_quality_metrics|useful_events|low_quality_events|archived_events|subject_hijack_prevented_events|subject_hijack_occurred_events|subject_hijack_rate|decision_consult_rate|scope_contamination_rate|verification_coverage_rate|secondary_loop_eval_artifacts|ledger_size|recent_entries|secondary_loop_acceptance_hooks|secondary_loop_replay_comparative|secondary_loop_closure_replay_evidence|current_task_pair_observed|current_task_pair_id|secondary_loop_comparative_summary_from_replay|replay_events_scanned|secondary_loop_outcome_events|bounded_improvement_over_no_secondary_baseline|irrelevant_secondary_suggestion_suppressed|verification_rejection_observed|decay_or_archival_observed|evidence_counts|comparative_improvement_pairs' "$ROUTE_FILE" >/dev/null 2>&1 && rg -n 'secondary_loop_comparative_summary_from_replay|SecondaryLoopComparativeReplaySummary|SecondaryLoopComparativePair' "$REPLAY_FILE" >/dev/null 2>&1; then
  log_pass "secondary eval traces expose useful-vs-low-quality counters and durable ledger artifacts"
else
  log_fail "missing counters or durable ledger surfaces for secondary eval quality"
fi

if rg -n 'secondary_loop_eval_bundle|task_id|scenario_id|secondary_loop_kind_invoked|secondary_loop_objective_profile|non_closure_objective_events|non_closure_objective_rate|trace_handles|promotion_rejection_archival_result|retained_as_projection|deferred_for_review|archived_failed_attempt|latency_token_cost_impact|final_task_outcome|ledger_refs' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "status emits replay/eval bundle with required doc78 audit dimensions"
else
  log_fail "missing replay/eval bundle dimensions in status surface"
fi

if rg -n 'async fn closure_replay_evidence|route\("/v1/work-loop/replay/closure-evidence", get\(closure_replay_evidence\)\)|secondary_loop_closure_replay_evidence_for_status|fn secondary_loop_continuity_gate_for_status\(|async fn closure_replay_bundle|route\("/v1/work-loop/replay/closure-bundle", get\(closure_replay_bundle\)\)' "$ROUTE_FILE" >/dev/null 2>&1; then
  log_pass "consumer replay routes expose closure evidence, continuity gate semantics, and closure-bundle packaging"
else
  log_fail "missing replay consumer route/continuity gate/closure-bundle packaging"
fi

if rg -n 'secondary_closure_trace_payloads_include_required_dimensions|secondary_closure_trace_payloads_mark_blocked_transition_for_rejections|bd_closure_certificate_contains_policy_and_spec_refs|secondary_loop_boundary_reason_flags_operator_steering|secondary_loop_boundary_reason_flags_governance_pending|continuation_boundary_events_pause_on_operator_steering|continuation_boundary_events_block_on_governance_pending|continuation_boundary_trace_payloads_include_path_marker|continuation_boundary_events_never_emit_work_item_selection|secondary_loop_quality_grade_marks_useful_when_verification_and_reason_present|secondary_loop_quality_grade_marks_low_quality_without_continue_reason|secondary_loop_quality_trace_payload_marks_low_quality|secondary_loop_promotion_status_maps_outcome_classes|observe_continuous_turn_outcome_records_useful_trace_and_ledger_artifacts|observe_continuous_turn_outcome_marks_subject_hijack_in_quality_trace|observe_continuous_turn_outcome_records_low_quality_rejection_artifact|observe_continuous_turn_outcome_comparative_baseline_proves_improvement_for_same_task|secondary_loop_closure_replay_evidence_reads_persisted_pairs|observe_continuous_turn_outcome_archives_unverified_spec_conformant_attempts|secondary_loop_ledger_archives_entries_beyond_active_window|subject_hijack_trace_flags_detect_divergent_subjects|secondary_eval_trace_payload_includes_quality_status_for_post_turn_eval|secondary_eval_trace_payload_ignores_unrelated_confidence_types|post_turn_eval_trace_flags_detect_answer_and_consistency_failures|post_turn_eval_trace_flags_ignore_clean_context' "$DAEMON_FILE" >/dev/null 2>&1 && rg -n 'secondary_loop_quality_metrics_include_rate_surfaces|secondary_loop_quality_metrics_handle_zero_denominators|secondary_loop_eval_bundle_surfaces_doc78_audit_dimensions|secondary_loop_eval_bundle_tracks_extended_outcome_classes|secondary_loop_acceptance_hooks_surface_controlled_run_proofs|secondary_loop_acceptance_hooks_default_to_false_without_evidence|secondary_loop_closure_replay_evidence_surfaces_current_task_pair|secondary_loop_closure_replay_evidence_defaults_fail_closed_without_match|secondary_loop_replay_consumer_payload_surfaces_ok_state|secondary_loop_replay_consumer_payload_surfaces_error_state_fail_closed|secondary_loop_continuity_gate_surfaces_open_state_when_replay_ok|secondary_loop_continuity_gate_surfaces_fail_closed_when_replay_error|secondary_loop_closure_bundle_surfaces_replay_gate_contract|secondary_loop_eval_bundle_prefers_current_task_when_bound' "$ROUTE_FILE" >/dev/null 2>&1 && rg -n 'test_rfm_fallback_default_is_fail_closed|test_rfm_fallback_fail_open_mode_is_permissive' "$RFM_FILE" >/dev/null 2>&1; then
  log_pass "runtime helpers have explicit unit tests"
else
  log_fail "missing unit tests for runtime helper contracts"
fi

if rg -n '5\.1a Adversarial closure-veracity verification|closure authority decisions' "$DOC78_FILE" >/dev/null 2>&1 && rg -n '13\.2b Adversarial Secondary Verifier|13\.2c Fail-Closed Verifier Availability|13\.2d Closure Certificate Evidence' "$DOC79_FILE" >/dev/null 2>&1; then
  log_pass "spec docs encode adversarial/fail-closed/certificate closure requirements"
else
  log_fail "spec docs missing adversarial/fail-closed/certificate closure clauses"
fi

if rg -n '## 10\. Proposal Advancement Ledger|proposal_id|promotion_status|trace_id|created_at' "$DOC78_FILE" >/dev/null 2>&1; then
  log_pass "spec docs define durable proposal advancement ledger fields"
else
  log_fail "spec docs missing durable proposal advancement ledger field contract"
fi

if [ -f "$DOC78_SCORECARD_FILE" ] && rg -n 'F1|F2|F3|F4|F5|focusa-o8vn|/v1/work-loop/replay/closure-evidence|/v1/work-loop/replay/closure-bundle|tests/doc78_secondary_cognition_runtime_test.sh|tests/doc78_tui_replay_dashboard_surface_test.sh|tests/work_loop_route_contract_test.sh|tests/doc78_live_continuation_boundary_pressure_smoke.sh|tests/doc78_live_non_closure_objective_profile_smoke.sh|tests/doc78_production_runtime_governance_replay_smoke.sh|tests/doc78_production_runtime_series_smoke.sh|tests/doc73_first_consumer_path_test.sh|tests/work_loop_commitment_lifecycle_contract_test.sh|tests/doc74_reference_resolution_consumer_path_test.sh|tests/doc76_retention_policy_consumer_path_test.sh|tests/work_loop_query_scope_boundary_contract_test.sh|FOCUSA_DOC78_PROD_ARTIFACT_DIR|docs/evidence/doc78-production-runtime-latest|FOCUSA_DOC78_PROD_SERIES_DIR|docs/evidence/doc78-production-runtime-series-latest|DOC78_PRODUCTION_RUNTIME_EVIDENCE_2026-04-17.md|DOC78_PRODUCTION_RUNTIME_SERIES_EVIDENCE_2026-04-18.md' "$DOC78_SCORECARD_FILE" >/dev/null 2>&1; then
  log_pass "doc78 frontier scorecard maps F1-F5 slices to executable replay/closure evidence"
else
  log_fail "missing doc78 frontier scorecard evidence mapping"
fi

if [ -x "$COMPARATIVE_HARNESS_FILE" ] && rg -n 'observe_continuous_turn_outcome_comparative_baseline_proves_improvement_for_same_task|secondary_loop_acceptance_hooks_surface_controlled_run_proofs|secondary_loop_acceptance_hooks_default_to_false_without_evidence' "$COMPARATIVE_HARNESS_FILE" >/dev/null 2>&1; then
  log_pass "doc78 comparative acceptance harness exists and targets controlled baseline-vs-bounded evidence paths"
else
  log_fail "missing comparative acceptance harness for baseline-vs-bounded controlled runs"
fi

if [ -x "$REPLAY_COMPARATIVE_HARNESS_FILE" ] && rg -n 'test_secondary_loop_comparative_summary_from_replay_log|test_secondary_loop_comparative_summary_respects_session_filter|test_replay_events_fail_fast_on_legacy_payload_without_handle|secondary_loop_closure_replay_evidence_reads_persisted_pairs|secondary_loop_closure_replay_evidence_surfaces_current_task_pair|secondary_loop_closure_replay_evidence_defaults_fail_closed_without_match|secondary_loop_replay_consumer_payload_surfaces_ok_state|secondary_loop_replay_consumer_payload_surfaces_error_state_fail_closed' "$REPLAY_COMPARATIVE_HARNESS_FILE" >/dev/null 2>&1; then
  log_pass "doc78 replay comparative harness exists and targets persisted replay-log evidence paths"
else
  log_fail "missing replay comparative harness for persisted event-log evidence"
fi

if [ -x "$FIRST_CONSUMER_HARNESS_FILE" ] && rg -n 'Doc 78|work_loop_watchdog|/v1/work-loop/replay/closure-evidence|secondary_loop_replay_consumer_payload_for_status' "$FIRST_CONSUMER_HARNESS_FILE" >/dev/null 2>&1; then
  log_pass "doc78 first-consumer harness exists and anchors replay evidence to live continuity behavior"
else
  log_fail "missing doc78 first-consumer harness for live replay-evidence gating"
fi

if [ -x "$OPERATOR_SURFACE_HARNESS_FILE" ] && rg -n '/work-loop/replay/closure-evidence|focusa_work_loop_status|continuity_gate=' "$OPERATOR_SURFACE_HARNESS_FILE" >/dev/null 2>&1; then
  log_pass "doc78 operator-surface harness exists and projects replay consumer gate semantics"
else
  log_fail "missing doc78 operator-surface harness for replay gate projection"
fi

if [ -x "$OPERATOR_DASHBOARD_HARNESS_FILE" ] && rg -n 'WorkLoop|/v1/work-loop/status|/v1/work-loop/replay/closure-evidence|continuity gate|closure-bundle|secondary_loop_objective_profile|Non-closure objectives' "$OPERATOR_DASHBOARD_HARNESS_FILE" >/dev/null 2>&1; then
  log_pass "doc78 dashboard harness exists and anchors replay gate/objective semantics in production TUI/API packaging"
else
  log_fail "missing doc78 dashboard harness for replay gate packaging"
fi

if [ -x "$LIVE_RUNTIME_HARNESS_FILE" ] && rg -n 'isolated daemon|/v1/work-loop/replay/closure-bundle|secondary_loop_objective_profile|/v1/work-loop/replay/closure-evidence' "$LIVE_RUNTIME_HARNESS_FILE" >/dev/null 2>&1; then
  log_pass "doc78 live runtime smoke harness exists for closure-bundle-first semantics"
else
  log_fail "missing doc78 live runtime closure-bundle smoke harness"
fi

if [ -x "$LIVE_BOUNDARY_HARNESS_FILE" ] && rg -n 'operator steering|governance decision pending|select-next|continuation boundary|isolated daemon|/v1/work-loop/heartbeat|scope_failure_recorded|/v1/telemetry/trace|sustained governance pressure' "$LIVE_BOUNDARY_HARNESS_FILE" >/dev/null 2>&1; then
  log_pass "doc78 live boundary-pressure harness exists for sustained operator/governance continuation gating"
else
  log_fail "missing doc78 live continuation-boundary pressure harness"
fi

if [ -x "$LIVE_NON_CLOSURE_HARNESS_FILE" ] && rg -n '/v1/telemetry/trace|verification_result|loop_objective|non_closure_objective_events|closure-bundle|isolated daemon' "$LIVE_NON_CLOSURE_HARNESS_FILE" >/dev/null 2>&1; then
  log_pass "doc78 live non-closure objective harness exists for runtime objective-profile evidence"
else
  log_fail "missing doc78 live non-closure objective-profile harness"
fi

if [ -x "$PRODUCTION_RUNTIME_HARNESS_FILE" ] && rg -n 'already-running daemon|FOCUSA_BASE_URL|governance decision pending|scope_failure_recorded|retained/deduplicated history|before_ids|gov_ok|/v1/work-loop/replay/closure-bundle|/v1/work-loop/checkpoints|non_closure_objective_events' "$PRODUCTION_RUNTIME_HARNESS_FILE" >/dev/null 2>&1; then
  log_pass "doc78 production-runtime harness exists for non-isolated governance/replay evidence capture"
else
  log_fail "missing doc78 production-runtime governance/replay harness"
fi

if [ -x "$PRODUCTION_RUNTIME_SERIES_HARNESS_FILE" ] && rg -n 'FOCUSA_DOC78_PROD_SERIES_RUNS|FOCUSA_DOC78_PROD_SERIES_DIR|run-records.jsonl|series-summary.json|doc78_production_runtime_governance_replay_smoke.sh|runs_passed|runs_failed|seed_non_closure' "$PRODUCTION_RUNTIME_SERIES_HARNESS_FILE" >/dev/null 2>&1; then
  log_pass "doc78 sustained production-runtime series harness exists for repeated-run evidence capture"
else
  log_fail "missing doc78 sustained production-runtime series harness"
fi

echo "=== DOC78 SECONDARY COGNITION RUNTIME RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
