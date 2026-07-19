#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MISSION_CANVAS="$ROOT_DIR/apps/menubar/src/lib/components/MissionCanvasView.svelte"
DOC="$ROOT_DIR/docs/current/MAC_APP_MISSION_CONTROL.md"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

for needle in \
  'Mission-centered Focusa status' \
  'mission-brief' \
  'MISSION' \
  'ProjectIdentity' \
  'Continuity ID' \
  'HLT' \
  'MLG' \
  'STG' \
  'Current Workpoint' \
  'Next action' \
  'Evidence count' \
  'Scope status' \
  'Context Authority status' \
  'Daemon/CLI version status' \
  'Pairing status' \
  'Warnings' \
  'Resume/copy' \
  'copyResumeCommand'; do
  rg -n -F "$needle" "$MISSION_CANVAS" >/dev/null || fail "MissionCanvasView missing $needle"
done
pass "MissionCanvasView exposes mission-centered required fields and copy action"

for needle in \
  'Mission-centered main panel' \
  'ProjectIdentity' \
  'Continuity ID' \
  'HLT' \
  'MLG' \
  'STG' \
  'Warnings' \
  'Resume/copy button'; do
  rg -n -F "$needle" "$DOC" >/dev/null || fail "MAC_APP_MISSION_CONTROL missing $needle"
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec106 missing $needle"
done
pass "menubar docs/spec preserve required mission fields"

(cd "$ROOT_DIR/apps/menubar" && bun run check) >/tmp/focusa-menubar-check.out
rg -n -F 'svelte-check found 0 errors' /tmp/focusa-menubar-check.out >/dev/null || fail "menubar check did not report 0 errors"
pass "menubar svelte-check passes"

echo "menubar mission-centered static test: PASS"
