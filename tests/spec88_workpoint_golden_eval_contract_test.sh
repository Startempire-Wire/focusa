#!/usr/bin/env bash
# Contract: Spec88 workpoint continuity golden evals must be documented and wired into Pi/API/replay surfaces.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DOC_FILE="${ROOT_DIR}/docs/evidence/SPEC88_GOLDEN_EVAL_EVIDENCE_2026-04-28.md"
FAILED=0
PASSED=0
log_pass(){ echo "✓ PASS: $1"; PASSED=$((PASSED+1)); }
log_fail(){ echo "✗ FAIL: $1"; FAILED=$((FAILED+1)); }

assert_rg(){
  local pattern="$1" file="$2" msg="$3"
  if rg -n "$pattern" "$file" >/dev/null 2>&1; then log_pass "$msg"; else log_fail "$msg"; fi
}

assert_rg 'Spec88 Golden Eval Evidence Packet|meaning lives in the typed Workpoint' "$DOC_FILE" "evidence packet anchors Spec88 authority"
assert_rg 'G1 compaction continuity|G2 context overflow continuity|G3 model switch continuity|G4 fork continuity|G5 degraded fallback|G6 drift detection|G7 pickup desirability|G8 replay evidence' "$DOC_FILE" "evidence packet defines all golden eval gates"
assert_rg 'ASAP-style Regression Scenario|/api/audio/today|homepage audio widget|notes-only work, generic validation' "$DOC_FILE" "evidence packet captures original post-compaction drift regression"
assert_rg 'WorkpointResumePacket|checkpointBeforeCompaction|refreshWorkpointResumePacket' "${ROOT_DIR}/apps/pi-extension/src/compaction.ts" "Pi compaction injects Workpoint resume packet"
assert_rg 'triggerTurn: true|auto-resume turn submitted|lastCompactResumeKey|lastCompactResumeAt|recentlySubmitted' "${ROOT_DIR}/apps/pi-extension/src/compaction.ts" "Pi compaction submits exactly one post-compact auto-resume turn with persisted duplicate suppression"
assert_rg 'after_provider_response|context_overflow|provider_status' "${ROOT_DIR}/apps/pi-extension/src/turns.ts" "Pi provider overflow path records Workpoint boundary"
assert_rg 'model_switch|checkpointDiscontinuity' "${ROOT_DIR}/apps/pi-extension/src/turns.ts" "Pi model switch path records Workpoint boundary"
assert_rg 'session_before_fork|checkpoint_reason: "fork"|refreshSessionWorkpointPacket\("fork"\)' "${ROOT_DIR}/apps/pi-extension/src/session.ts" "Pi fork path records Workpoint boundary"
assert_rg 'focusa-workpoint-fallback|canonical: false|recordLocalWorkpointFallback' "${ROOT_DIR}/apps/pi-extension/src/compaction.ts" "Pi degraded fallback is explicit and non-canonical"
assert_rg 'Focusa Workpoint Continuity Law|ACTIVE_OBJECT_SET|ACTION_INTENT|VERIFICATION_HOOKS|DRIFT_BOUNDARIES' "${ROOT_DIR}/apps/pi-extension/src/turns.ts" "Pi before-agent/context injection carries typed Workpoint sections"
assert_rg 'ensureLowConfidenceWorkpoint|confidence: "low"|session_start_low_confidence|session_switch_low_confidence' "${ROOT_DIR}/apps/pi-extension/src/session.ts" "Pi session resume creates low-confidence checkpoint when no active workpoint exists"
assert_rg 'providerStatusSuggestsContextOverflow|textSuggestsContextOverflow|context_length_exceeded|provider_status' "${ROOT_DIR}/apps/pi-extension/src/turns.ts" "Pi overflow detection handles provider status and overflow text"
assert_rg 'drift-check|emit: true|workpoint_drift_detected|workpoint_drift_checked' "${ROOT_DIR}/apps/pi-extension/src/turns.ts" "Pi turn-end drift telemetry is wired"
assert_rg 'classify_drift|notes_only_drift|wrong_object_drift|do_not_drift_boundary|work-loop:write' "${ROOT_DIR}/crates/focusa-api/src/routes/workpoint.rs" "Workpoint drift check classifies semantic drift and permission-gates event emission"
assert_rg 'WORKPOINT_IDEMPOTENCY_CACHE|idempotency_key|idempotent_replay' "${ROOT_DIR}/crates/focusa-api/src/routes/workpoint.rs" "Workpoint checkpoint implements immediate and persisted idempotency-key replay semantics"
assert_rg 'workpoint_replay_summary' "${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs" "work-loop status exposes workpoint replay summary"
assert_rg 'WorkpointReplaySummary|drift_detected_events|degraded_fallback_events' "${ROOT_DIR}/crates/focusa-core/src/replay/mod.rs" "core replay summarizes Workpoint events"
assert_rg 'focusa_workpoint_checkpoint|focusa_workpoint_resume' "${ROOT_DIR}/apps/pi-extension/src/tools.ts" "Pi exposes first-class Workpoint pickup tools"
assert_rg 'Spec88 Workpoint Continuity Operator Contract|focusa_workpoint_checkpoint|focusa_workpoint_resume|canonical: false|Drift warnings' "${ROOT_DIR}/docs/44-pi-focusa-integration-spec.md" "operator docs explain Workpoint continuity and degraded drift semantics"
assert_rg 'focusa_workpoint_checkpoint|focusa_workpoint_resume|canonical: false|Drift warnings|Workpoint Continuity Rules' "${ROOT_DIR}/apps/pi-extension/skills/focusa/SKILL.md" "Focusa skill documents Workpoint tools and recovery rules"
assert_rg 'Spec88 Rollout Gate|Rollout Gate Commands|/root/.pi/skills/focusa/SKILL.md' "${ROOT_DIR}/docs/evidence/SPEC88_ROLLOUT_GATE_2026-04-28.md" "rollout evidence packet documents gate and installed skill sync"

echo "=== SPEC88 WORKPOINT GOLDEN EVAL CONTRACT RESULTS ==="
echo "Tests passed: $PASSED"
echo "Tests failed: $FAILED"
if [ "$FAILED" -ne 0 ]; then exit 1; fi
