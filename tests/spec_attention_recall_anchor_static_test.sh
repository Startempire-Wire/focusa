#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE_TS="$ROOT_DIR/apps/pi-extension/src/state.ts"
TURNS_TS="$ROOT_DIR/apps/pi-extension/src/turns.ts"
COMPACTION_TS="$ROOT_DIR/apps/pi-extension/src/compaction.ts"

fail() { echo "✗ FAIL: $1" >&2; exit 1; }
pass() { echo "✓ PASS: $1"; }

rg -n 'interface PiAttentionRecallVerdict|focusa\.attention_recall_verdict\.v1|visible_recap_required|latest_report_summary_ref|must_not_forget' "$STATE_TS" >/dev/null \
  || fail "AttentionRecallVerdict schema/fields missing from Pi state helpers"
pass "AttentionRecallVerdict schema fields exist"

rg -n 'MEMORY_ANCHOR|ATTENTION_RECALL_VERDICT|END_ATTENTION_RECALL|formatAttentionRecallFocusSliceLines' "$STATE_TS" "$TURNS_TS" >/dev/null \
  || fail "Memory anchor formatting is not wired into Focus Slice path"
pass "Memory anchor formatting is wired"

rg -n 'protectedPrefixCount|END_ATTENTION_RECALL|Truncate from bottom while preserving the non-droppable attention/recall prefix' "$TURNS_TS" >/dev/null \
  || fail "Focus Slice truncation does not preserve attention/recall prefix"
pass "Focus Slice truncation preserves attention/recall prefix"

python3 - "$TURNS_TS" <<'PY'
from pathlib import Path
import sys
text = Path(sys.argv[1]).read_text()
if text.find('...attentionLines') == -1 or text.find('...attentionLines') > text.find('...scopedEntries.map'):
    raise SystemExit('attention lines are not before scoped Focus Slice entries')
PY
pass "Attention anchor precedes verbose Focus Slice entries"

rg -n '# Attention Recall Anchor|## AttentionRecallVerdict|formatAttentionRecallFocusSliceLines\(buildAttentionRecallVerdict' "$COMPACTION_TS" >/dev/null \
  || fail "Compaction output does not include attention/recall anchor before Workpoint packet"
pass "Compaction output includes attention/recall anchor"

echo "Attention/recall anchor static test: PASS"
