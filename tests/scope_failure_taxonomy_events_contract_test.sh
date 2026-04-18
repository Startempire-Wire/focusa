#!/bin/bash
# SPEC-79 / Doc 69 slice A contract: scope-failure taxonomy + event surfaces.

set -euo pipefail

FAILED=0
PASSED=0

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
STATE_TS="$ROOT/apps/pi-extension/src/state.ts"
TURNS_TS="$ROOT/apps/pi-extension/src/turns.ts"
CORE_TYPES_RS="$ROOT/crates/focusa-core/src/types.rs"
TELEMETRY_RS="$ROOT/crates/focusa-api/src/routes/telemetry.rs"

log_pass() { echo "✓ PASS: $1"; PASSED=$((PASSED+1)); }
log_fail() { echo "✗ FAIL: $1"; FAILED=$((FAILED+1)); }

require_rg() {
  local pattern="$1"
  local file="$2"
  local label="$3"
  if rg -n "$pattern" "$file" >/dev/null 2>&1; then
    log_pass "$label"
  else
    log_fail "$label"
  fi
}

echo "=== Scope-failure taxonomy/events contract ==="

require_rg 'type ScopeFailureKind' "$STATE_TS" 'Pi bridge defines scope failure taxonomy type'
require_rg 'scope_contamination' "$STATE_TS" 'Taxonomy includes scope_contamination'
require_rg 'adjacent_thread_leakage' "$STATE_TS" 'Taxonomy includes adjacent_thread_leakage'
require_rg 'answer_broadening' "$STATE_TS" 'Taxonomy includes answer_broadening'
require_rg 'wrong_question_answered' "$STATE_TS" 'Taxonomy includes wrong_question_answered'
require_rg 'context_overcarry' "$STATE_TS" 'Taxonomy includes context_overcarry'
require_rg 'detectScopeFailureSignals' "$STATE_TS" 'Pi bridge exposes scope failure detector'

require_rg 'scope_verified' "$TURNS_TS" 'turn_end emits scope_verified trace'
require_rg 'scope_contamination_detected' "$TURNS_TS" 'turn_end emits scope_contamination_detected trace'
require_rg 'wrong_question_detected' "$TURNS_TS" 'turn_end emits wrong_question_detected trace'
require_rg 'answer_broadening_detected' "$TURNS_TS" 'turn_end emits answer_broadening_detected trace'
require_rg 'scope_failure_recorded' "$TURNS_TS" 'turn_end emits scope_failure_recorded trace'

require_rg 'CurrentAskDetermined' "$CORE_TYPES_RS" 'Telemetry enum includes CurrentAskDetermined'
require_rg 'QueryScopeBuilt' "$CORE_TYPES_RS" 'Telemetry enum includes QueryScopeBuilt'
require_rg 'RelevantContextSelected' "$CORE_TYPES_RS" 'Telemetry enum includes RelevantContextSelected'
require_rg 'IrrelevantContextExcluded' "$CORE_TYPES_RS" 'Telemetry enum includes IrrelevantContextExcluded'
require_rg 'ScopeVerified' "$CORE_TYPES_RS" 'Telemetry enum includes ScopeVerified'
require_rg 'ScopeContaminationDetected' "$CORE_TYPES_RS" 'Telemetry enum includes ScopeContaminationDetected'
require_rg 'WrongQuestionDetected' "$CORE_TYPES_RS" 'Telemetry enum includes WrongQuestionDetected'
require_rg 'AnswerBroadeningDetected' "$CORE_TYPES_RS" 'Telemetry enum includes AnswerBroadeningDetected'
require_rg 'ScopeFailureRecorded' "$CORE_TYPES_RS" 'Telemetry enum includes ScopeFailureRecorded'

require_rg '"current_ask_determined" => TelemetryEventType::CurrentAskDetermined' "$TELEMETRY_RS" 'telemetry route maps current_ask_determined'
require_rg '"query_scope_built" => TelemetryEventType::QueryScopeBuilt' "$TELEMETRY_RS" 'telemetry route maps query_scope_built'
require_rg '"relevant_context_selected" => TelemetryEventType::RelevantContextSelected' "$TELEMETRY_RS" 'telemetry route maps relevant_context_selected'
require_rg '"irrelevant_context_excluded" => TelemetryEventType::IrrelevantContextExcluded' "$TELEMETRY_RS" 'telemetry route maps irrelevant_context_excluded'
require_rg '"scope_verified" => TelemetryEventType::ScopeVerified' "$TELEMETRY_RS" 'telemetry route maps scope_verified'
require_rg '"scope_contamination_detected" => TelemetryEventType::ScopeContaminationDetected' "$TELEMETRY_RS" 'telemetry route maps scope_contamination_detected'
require_rg '"wrong_question_detected" => TelemetryEventType::WrongQuestionDetected' "$TELEMETRY_RS" 'telemetry route maps wrong_question_detected'
require_rg '"answer_broadening_detected" => TelemetryEventType::AnswerBroadeningDetected' "$TELEMETRY_RS" 'telemetry route maps answer_broadening_detected'
require_rg '"scope_failure_recorded" => TelemetryEventType::ScopeFailureRecorded' "$TELEMETRY_RS" 'telemetry route maps scope_failure_recorded'

echo ""
echo "=== RESULTS: $PASSED passed, $FAILED failed ==="
[ "$FAILED" -eq 0 ]
