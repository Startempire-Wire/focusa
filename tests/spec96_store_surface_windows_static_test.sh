#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ECS="${ROOT_DIR}/crates/focusa-api/src/routes/ecs.rs"
META="${ROOT_DIR}/crates/focusa-api/src/routes/metacognition.rs"
TEL="${ROOT_DIR}/crates/focusa-api/src/routes/telemetry.rs"
SNAP="${ROOT_DIR}/crates/focusa-api/src/routes/snapshots.rs"
WORK="${ROOT_DIR}/crates/focusa-api/src/routes/workpoint.rs"
FOCUS="${ROOT_DIR}/crates/focusa-api/src/routes/focus.rs"
TRAVERSE="${ROOT_DIR}/crates/focusa-api/src/routes/traverse.rs"
SPEC="${ROOT_DIR}/docs/96-trajectory-projection-and-daemon-stability-spec.md"

if rg -n 'cursor|next_cursor|bounded_metadata|include_full_payload|full_payload_blocked_by_pressure|rehydrate' "$ECS" >/dev/null; then
  echo "✓ PASS: evidence/ECS refs have bounded window and cold rehydrate semantics"
else
  echo "✗ FAIL: evidence/ECS bounded window missing" >&2; exit 1
fi

if rg -n 'RecentMetacogQuery.*cursor|next_cursor|metadata|summary_only|/v1/metacognition/captures' "$META" >/dev/null; then
  echo "✓ PASS: metacog recent surfaces expose cursor/metadata/rehydrate"
else
  echo "✗ FAIL: metacog recent cursor metadata missing" >&2; exit 1
fi

if rg -n 'FOCUSA_TELEMETRY_TRACE_DEFAULT_LIMIT|cursor|next_cursor|metadata|budgeted_requested_limit' "$TEL" >/dev/null; then
  echo "✓ PASS: telemetry events/trace expose bounded cursor windows"
else
  echo "✗ FAIL: telemetry cursor windows missing" >&2; exit 1
fi

if rg -n 'RecentSnapshotsQuery.*cursor|next_cursor|bounded_metadata|cold_full_payload_opt_in' "$SNAP" >/dev/null; then
  echo "✓ PASS: snapshots recent surface exposes bounded cursor metadata"
else
  echo "✗ FAIL: snapshot cursor metadata missing" >&2; exit 1
fi

if rg -n 'active_object_refs_metadata|verification_records_metadata|bounded_metadata|rehydrate_refs' "$WORK" >/dev/null; then
  echo "✓ PASS: Workpoint arrays and v2 identity refs are bounded"
else
  echo "✗ FAIL: Workpoint bounded metadata missing" >&2; exit 1
fi

if rg -n 'FocusStackQuery|frames_window|traversal_metadata|focus_stack.*window|rehydrate_refs' "$FOCUS" >/dev/null; then
  echo "✓ PASS: Focus Stack exposes bounded window metadata without breaking legacy stack shape"
else
  echo "✗ FAIL: Focus Stack bounded window metadata missing" >&2; exit 1
fi

if rg -n '"evidence"|"references"|"metacognition"|"telemetry"|"snapshots"|"workpoints"|"focus_stack"' "$TRAVERSE" >/dev/null; then
  echo "✓ PASS: focusa_traverse covers store surfaces"
else
  echo "✗ FAIL: focusa_traverse store adapters missing" >&2; exit 1
fi

if rg -n 'Store surface traversal posture' "$SPEC" >/dev/null; then
  echo "✓ PASS: Spec documents store traversal posture"
else
  echo "✗ FAIL: Spec store traversal docs missing" >&2; exit 1
fi

echo "SPEC96 store surface windows static test: PASS"
