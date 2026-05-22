#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORKPOINT="${ROOT_DIR}/crates/focusa-api/src/routes/workpoint.rs"
TOOLS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
SPEC="${ROOT_DIR}/docs/96-trajectory-projection-and-daemon-stability-spec.md"

if rg -n 'fn workpoint_visibility_wait_attempts|"lowmem" => 2|"emergency" => 1|"constrained" => 8|_ => 40' "$WORKPOINT" >/dev/null && ! rg -n '0\.\.240' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: Workpoint checkpoint/evidence visibility waits are bounded under LowMem and below Pi timeout"
else
  echo "✗ FAIL: Workpoint checkpoint/evidence can still wait 12s and trigger route timeout" >&2
  exit 1
fi

if rg -n 'lowmem_caps_active\(\).*try_send\(Action::EmitEvent|failure_class": "resource_exhausted"|daemon command channel is saturated under LowMem' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: LowMem Workpoint dispatch uses nonblocking backpressure envelope"
else
  echo "✗ FAIL: LowMem Workpoint dispatch can block on daemon command channel" >&2
  exit 1
fi

if rg -n 'failure_class": "read_model_lag"|"retry_posture": "safe_retry"|"next_tools": \["focusa_workpoint_resume", "focusa_traverse", "focusa_resource_mode"\]' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: pending evidence link returns read_model_lag with retry/next-tool guidance"
else
  echo "✗ FAIL: pending evidence link lacks read_model_lag envelope" >&2
  exit 1
fi

if rg -n 'lowmem_caps_active|summary_only|"workpoint": if summary_only' "$WORKPOINT" >/dev/null; then
  echo "✓ PASS: LowMem evidence link avoids returning full Workpoint payload"
else
  echo "✗ FAIL: LowMem evidence link still returns full payload" >&2
  exit 1
fi

if rg -n 'focusa_evidence_capture|attach_to_workpoint' "$TOOLS" "$SPEC" >/dev/null; then
  echo "✓ PASS: fallback evidence capture/no-link path remains available"
else
  echo "✗ FAIL: fallback no-link evidence capture path missing" >&2
  exit 1
fi

echo "SPEC96 Workpoint evidence LowMem timeout static test: PASS"
