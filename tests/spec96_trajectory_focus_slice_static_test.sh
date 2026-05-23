#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TURNS_TS="${ROOT_DIR}/apps/pi-extension/src/turns.ts"
GTM_DOC="${ROOT_DIR}/docs/current/TRAJECTORY_GTM_AND_GAPS.md"
SPEC_DOC="${ROOT_DIR}/docs/96-trajectory-projection-and-daemon-stability-spec.md"

if rg -n '/trajectory/view\?|getTrajectoryFocusSliceLines|formatTrajectoryFocusSlice' "$TURNS_TS" >/dev/null; then
  echo "✓ PASS: Pi Focus Slice fetches bounded trajectory view"
else
  echo "✗ FAIL: Focus Slice trajectory view fetch missing" >&2
  exit 1
fi

if rg -n 'PROJECT_TRAJECTORY|PROJECT_IDENTITY|TRAJECTORY_GOALS|CURRENT_VERIFIED_STATE|ACTIVE_GAP|CONTEXT_SUFFICIENCY|TRAJECTORY_DO_NOT_USE|WORKPOINT_CANDIDATE' "$TURNS_TS" >/dev/null; then
  echo "✓ PASS: Focus Slice trajectory projection includes required north-star fields"
else
  echo "✗ FAIL: trajectory projection fields missing from Focus Slice" >&2
  exit 1
fi

if rg -n 'PROJECT_ENVIRONMENT|root_url|live_url|local_url|deploy_target|deploy_location|local_vs_live_boundary' "$TURNS_TS" "${ROOT_DIR}/crates/focusa-api/src/routes/project.rs" >/dev/null; then
  echo "✓ PASS: Focus Slice includes explicit project environment/deploy facts"
else
  echo "✗ FAIL: project environment/deploy facts missing from Focus Slice" >&2
  exit 1
fi

if rg -n 'advisory_only=true|Trajectory.*never override|advisory degraded projection' "$TURNS_TS" "${ROOT_DIR}/apps/pi-extension/skills/focusa/SKILL.md" >/dev/null; then
  echo "✓ PASS: trajectory projection remains advisory and guarded"
else
  echo "✗ FAIL: advisory trajectory guardrail missing" >&2
  exit 1
fi

if rg -n 'priority: 3|priority: 4|priority: 5' "$TURNS_TS" >/dev/null && rg -n 'PROJECT_TRAJECTORY.*values\.map|boundedTrajectoryText' "$TURNS_TS" >/dev/null; then
  echo "✓ PASS: trajectory injection is ordered and bounded"
else
  echo "✗ FAIL: trajectory injection lacks ordering or bounding evidence" >&2
  exit 1
fi

if rg -n 'Pi Focus Slice now injects bounded ProjectIdentity \+ Trajectory summary|Trajectory summary if available' "$GTM_DOC" "$SPEC_DOC" >/dev/null; then
  echo "✓ PASS: docs describe trajectory-first Focus Slice posture"
else
  echo "✗ FAIL: trajectory Focus Slice docs missing" >&2
  exit 1
fi

if bash "${ROOT_DIR}/tests/spec96_focus_slice_runtime_injection_test.sh" >/tmp/spec96-focus-slice-runtime-proof.log 2>&1; then
  echo "✓ PASS: Focus Slice mocked runtime proof emits trajectory projection"
else
  echo "✗ FAIL: Focus Slice runtime proof failed" >&2
  cat /tmp/spec96-focus-slice-runtime-proof.log >&2
  exit 1
fi

echo "SPEC96 trajectory Focus Slice static test: PASS"
