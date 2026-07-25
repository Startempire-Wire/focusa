#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
PI_CARD="apps/pi-extension/src/awareness.ts"
API_CARD="crates/focusa-api/src/routes/awareness.rs"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for file in "$PI_CARD" "$API_CARD"; do
  if [[ "$file" == "$API_CARD" ]]; then
    source_window=$(awk '/^fn render_card/{on=1} /^async fn card/{on=0} on' "$file")
  else
    source_window=$(awk '/^export function buildFocusaUtilityCard/{on=1} on' "$file")
  fi
  for section in MISSION_PACKET NOW_CARD WHY_CARD HEALTH_CARD DO_CARD RECONCILIATION_ENVELOPE 'Friendly Focusa Q'; do
    if rg -F "$section" <<<"$source_window" >/dev/null; then fail "$file retained stale default card section: $section"; fi
  done
  for term in 'Status:' 'Scope:' 'Mission:' 'Next:' 'Boundary:'; do
    rg -F "$term" <<<"$source_window" >/dev/null || fail "$file missing concise card term: $term"
  done
  pass "$file exposes concise Spec108 awareness contract"
done

echo "Spec102 legacy card supersession / Spec108 concise card test: PASS"
