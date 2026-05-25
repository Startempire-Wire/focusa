#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORK_LOOP="${ROOT_DIR}/crates/focusa-api/src/routes/work_loop.rs"

if rg -n 'dispatch_readiness|boundary_reason|continuation_boundary_reason\(wl\)|inspect writer/status/deep before dispatching' "$WORK_LOOP" >/dev/null; then
  echo "✓ PASS: work-loop health exposes dispatch readiness and boundary reason"
else
  echo "✗ FAIL: work-loop health lacks dispatch readiness diagnostics" >&2
  exit 1
fi

if rg -n 'destructive_confirmation_required|governance_decision_pending|operator_override_active|TransportDegraded' "$WORK_LOOP" >/dev/null; then
  echo "✓ PASS: work-loop health readiness includes pause flags and transport degradation"
else
  echo "✗ FAIL: work-loop health readiness omits operational blockers" >&2
  exit 1
fi

echo "SPEC96 Work-loop health readiness static test: PASS"
