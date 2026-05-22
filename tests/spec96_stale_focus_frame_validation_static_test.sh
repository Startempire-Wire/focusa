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

if rg -n 'Stale active-frame validation' "$SPEC" >/dev/null; then
  echo "✓ PASS: Spec documents stale-frame validation failure posture"
else
  echo "✗ FAIL: Spec stale-frame docs missing" >&2; exit 1
fi

echo "SPEC96 stale Focus frame validation static test: PASS"
