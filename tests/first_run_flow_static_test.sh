#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOC="$ROOT_DIR/docs/current/FIRST_RUN_FLOW.md"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
FIRST="$ROOT_DIR/apps/menubar/src/lib/components/FirstRunConnect.svelte"
PAGE="$ROOT_DIR/apps/menubar/src/routes/+page.svelte"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$DOC" ] || fail "FIRST_RUN_FLOW.md missing"
for needle in 'Primary path' 'Required UX states' 'Safety boundaries' 'Mac Completion Payload fallback' 'five-minute TTL' 'Focusa Connect'; do
  rg -n -F "$needle" "$DOC" >/dev/null || fail "first-run doc missing $needle"
done
pass "first-run doc defines path, UX, and safety boundaries"

for needle in \
  'OFFER_TTL_MS = 5 * 60_000' \
  'QRCode' \
  'mac_completion_payload' \
  'showAdvanced' \
  'callbackStatus' \
  'completionPayload' \
  'copy' \
  'Settings'; do
  rg -n -F "$needle" "$FIRST" >/dev/null || fail "FirstRunConnect missing $needle"
done
pass "FirstRunConnect has QR, TTL, callback, fallback, advanced, copy/recovery markers"

for needle in 'FirstRunConnect' 'hasEverConnected' 'focusa-connection-saved'; do
  rg -n -F "$needle" "$PAGE" >/dev/null || fail "+page missing first-run gating marker $needle"
done
pass "menubar page gates first-run before polling"

for needle in 'FIRST_RUN_FLOW.md' 'FirstRunConnect.svelte' 'first_run_flow_static_test.sh'; do
  rg -n -F "$needle" "$SPEC" >/dev/null || fail "Spec106 missing first-run proof marker $needle"
done
pass "Spec106 references first-run proof artifacts"

(cd "$ROOT_DIR/apps/menubar" && bun run check) >/tmp/focusa-first-run-menubar-check.out
rg -n -F 'svelte-check found 0 errors' /tmp/focusa-first-run-menubar-check.out >/dev/null || fail "menubar check did not report 0 errors"
pass "menubar svelte-check passes"

echo "first-run flow static test: PASS"
