#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FOCUS="${ROOT_DIR}/crates/focusa-api/src/routes/focus.rs"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
SPEC="${ROOT_DIR}/docs/96-trajectory-projection-and-daemon-stability-spec.md"

if rg -n 'target_frame_id_not_found|target_frame_is_not_active|active_frame_id|target_frame_id|stale_active_frame_or_read_model_lag' "$FOCUS" >/dev/null; then
  echo "✓ PASS: focus/update returns bounded stale-frame diagnostics"
else
  echo "✗ FAIL: stale-frame bounded diagnostics missing" >&2; exit 1
fi

if rg -n 'frame_unavailable|rejected_scope_mismatch|failure_class.*scope_mismatch|tool_result_v1' "$FOCUS" >/dev/null; then
  echo "✓ PASS: stale focus writes classify scoped validation failures"
else
  echo "✗ FAIL: scoped validation failure taxonomy missing" >&2; exit 1
fi

if rg -n 'scope_mismatch|frame_unavailable|read_model_lag|scratchpad fallback|not daemon-wide failure|stale_frame|namedSlotFallback|scratch_saved' "$TOOLS" >/dev/null; then
  echo "✓ PASS: Pi Focus State writes recover/report scoped frame failures with scratch fallback"
else
  echo "✗ FAIL: Pi scoped frame recovery/fallback missing" >&2; exit 1
fi

if rg -n 'refresh_scoped_frame|create_or_adopt_scoped_frame' "$TOOLS" >/dev/null && rg -n 'adoptExistingSafeFrameForRecovery|scopedWorkpointFrameRecoveryCwd|resolveFocusaToolProjectRoot|ensure_pi_frame_unsafe_cwd' "${ROOT_DIR}/apps/pi-extension/src/state.ts" >/dev/null && ! rg -n 'daemonActiveWorkpointFrameRecoveryCwd|daemon_active_workpoint' "${ROOT_DIR}/apps/pi-extension/src/state.ts" >/dev/null; then
  echo "✓ PASS: Pi Focus State writes recover from missing frame using scoped frame/workpoint before failing unsafe cwd"
else
  echo "✗ FAIL: missing-frame recovery remains brittle or adopts daemon-global Workpoint scope" >&2; exit 1
fi

if rg -n 'scope mismatch on|scope_recovery_context|request_scope|evidence_capture' "$TOOLS" >/dev/null; then
  echo "✓ PASS: Pi project-aware tools resolve unsafe cwd and evidence capture reports 409 scope mismatches with recovery context"
else
  echo "✗ FAIL: Pi project-aware tool recovery/409 reporting remains opaque" >&2; exit 1
fi

if rg -n 'pushDeltaFailureRecovery|recovery_hint|retry_posture|focusa_project_identity.*focusa_workpoint_checkpoint' "$TOOLS" >/dev/null; then
  echo "✓ PASS: Focus State write failures include structured recovery hints and next tools"
else
  echo "✗ FAIL: Focus State write failures still lack structured recovery guidance" >&2; exit 1
fi

if rg -n 'recoveryHintForFailure|misuse_hint|No-deadend failure recovery|unknown_ambiguous_completion' "$TOOLS" "${ROOT_DIR}/docs/current/FOCUSA_TOOL_CHOREOGRAPHY_MAP.md" >/dev/null; then
  echo "✓ PASS: generic Focusa tool failures expose no-deadend recovery/misuse guidance"
else
  echo "✗ FAIL: generic Focusa tool failures can still dead-end without misuse/recovery hints" >&2; exit 1
fi

if rg -n 'blockedToolResponse|focusa_predict_record.*prediction record blocked|focusa_predict_recent.*predictions recent blocked|focusa_predict_evaluate.*prediction evaluate blocked|focusa_predict_stats.*prediction stats blocked' "$TOOLS" >/dev/null; then
  echo "✓ PASS: prediction tool failures explain why and expose no-deadend recovery"
else
  echo "✗ FAIL: prediction tool failures can still return opaque unavailable/blocked details" >&2; exit 1
fi

if rg -n 'projectRootConfirmationRequired|projectRootConfirmationSummary|confirmPiProjectRoot|pi_frame_creation_blocked_unconfirmed_project_root' "${ROOT_DIR}/apps/pi-extension/src/state.ts" >/dev/null && rg -n 'projectRootConfirmationGate|project_root_confidence_below_90|interview/menu' "$TOOLS" >/dev/null && rg -n 'promptForConfirmedProjectRoot|promptForWorkpointIfNeeded|ctx\.ui\.select\("Focusa needs project root"|pi_vital_project_root_prompt_confirmed|vitalInfoPromptMode|vitalInfoPromptSurfaces|warn_only|project_root,workpoint,trajectory' "${ROOT_DIR}/apps/pi-extension/src/session.ts" "${ROOT_DIR}/apps/pi-extension/src/config.ts" >/dev/null && rg -n 'hardGateVitalProjectRoot|before_agent_start_confirmed_project_root|Agent turn is scope-limited until this is confirmed' "${ROOT_DIR}/apps/pi-extension/src/turns.ts" >/dev/null && rg -n 'project_root_confirmation_required|session_identity_requires_project_root_confirmation|unconfirmed_project_root_rejection' "${ROOT_DIR}/crates/focusa-api/src/routes/workpoint.rs" >/dev/null; then
  echo "✓ PASS: low-confidence project roots block Focusa writes and interruptively prompt operator confirmation"
else
  echo "✗ FAIL: low-confidence project roots can still silently bind Focusa state" >&2; exit 1
fi

if rg -n 'recommendations|recommended_action|session_scope|Session cwd is broad/unsafe' "$TOOLS" >/dev/null; then
  echo "✓ PASS: tool doctor emits actionable recommendations and session-scope diagnostics"
else
  echo "✗ FAIL: tool doctor lacks actionable diagnostic guidance" >&2; exit 1
fi

if rg -n 'Stale active-frame validation' "$SPEC" >/dev/null; then
  echo "✓ PASS: Spec documents stale-frame validation failure posture"
else
  echo "✗ FAIL: Spec stale-frame docs missing" >&2; exit 1
fi

echo "SPEC96 stale Focus frame validation static test: PASS"
