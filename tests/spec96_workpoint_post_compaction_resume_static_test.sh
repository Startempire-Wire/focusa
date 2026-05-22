#!/bin/bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
WORKPOINT_RS="${ROOT_DIR}/crates/focusa-api/src/routes/workpoint.rs"
FOCUS_RS="${ROOT_DIR}/crates/focusa-api/src/routes/focus.rs"
REDUCER_RS="${ROOT_DIR}/crates/focusa-core/src/reducer.rs"
STATE_TS="${ROOT_DIR}/apps/pi-extension/src/state.ts"
TOOLS_TS="${ROOT_DIR}/apps/pi-extension/src/tools.ts"
COMPACTION_TS="${ROOT_DIR}/apps/pi-extension/src/compaction.ts"
SESSION_TS="${ROOT_DIR}/apps/pi-extension/src/session.ts"
AWARENESS_TS="${ROOT_DIR}/apps/pi-extension/src/awareness.ts"

if rg -n 'session_id_change_preserves_canonical_when_project_root_matches|project_root_and_continuity_id_preserve_post_compaction_continuity' "$WORKPOINT_RS" >/dev/null; then
  echo "✓ PASS: post-compaction session changes preserve Workpoint continuity only after hard gates"
else
  echo "✗ FAIL: session-id temporal continuity rule missing" >&2
  exit 1
fi

if rg -n 'continuity_id_mismatch_rejects_inside_same_project_root|rejected_continuity_mismatch|workpoint continuity_id does not match current logical session' "$WORKPOINT_RS" >/dev/null; then
  echo "✓ PASS: same-root continuity mismatch is a hard rejection"
else
  echo "✗ FAIL: continuity_id hard rejection missing" >&2
  exit 1
fi

if rg -n 'same_project_distinct_continuity_frames_remain_active_without_cross_pause|At most one active Focus Frame exists per logical scope' "$REDUCER_RS" >/dev/null; then
  echo "✓ PASS: Focus Stack supports active same-root sessions separated by continuity_id"
else
  echo "✗ FAIL: scoped active-frame invariant missing" >&2
  exit 1
fi

if rg -n 'continuity_id|matched_by.*continuity_id' "$FOCUS_RS" >/dev/null; then
  echo "✓ PASS: Focus frame reads can scope by continuity_id"
else
  echo "✗ FAIL: focus frame continuity_id scoping missing" >&2
  exit 1
fi

if rg -n 'identity_confidence_percent|trajectory_id.*corroborating_only|hard_gates_required_before_corroborating_signals_count' "$WORKPOINT_RS" >/dev/null; then
  echo "✓ PASS: identity confidence is audit metadata after hard gates"
else
  echo "✗ FAIL: identity confidence audit metadata missing" >&2
  exit 1
fi

if rg -n 'workpoint session_id does not match current Pi session|safe_recovery.*current session|project/session mismatch' "$WORKPOINT_RS" "$TOOLS_TS" >/dev/null; then
  echo "✗ FAIL: session-id mismatch is still treated as hard resume rejection" >&2
  exit 1
else
  echo "✓ PASS: temporal session-id hard rejection removed"
fi

python3 - <<'PY' "$COMPACTION_TS" "$SESSION_TS" "$STATE_TS" "$TOOLS_TS" "$AWARENESS_TS"
from pathlib import Path
import sys
for raw in sys.argv[1:]:
    text = Path(raw).read_text()
    if raw.endswith(('compaction.ts','session.ts','tools.ts')) and '/workpoint/' in text and 'continuity_id' not in text:
        raise SystemExit(f'missing continuity_id propagation in {raw}')
    if raw.endswith('state.ts') and ('ensureContinuityId' not in text or 'continuity_id:' not in text):
        raise SystemExit('Pi state does not create/tag continuity_id before Workpoint')
    if raw.endswith('awareness.ts') and 'Continuity:' not in text:
        raise SystemExit('utility card does not surface continuity_id')
PY
echo "✓ PASS: Pi propagates continuity_id before Workpoint and in operator utility card"

echo "SPEC96 Workpoint post-compaction resume static test: PASS"
