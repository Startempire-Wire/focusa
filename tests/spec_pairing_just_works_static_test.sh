#!/usr/bin/env bash
# Pairing Just Works static guard: no localhost trap, clear fallback UX.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

PAIR_RS="$ROOT_DIR/crates/focusa-cli/src/commands/pair.rs"
FIRST_RUN="$ROOT_DIR/apps/menubar/src/lib/components/FirstRunConnect.svelte"

rg -n 'Localhost-only pairing room selected|NOT phone-scannable|focusa pair --url https://YOUR-FOCUSA-HOST|FOCUSA_PAIRING_URL|manual completion payload fallback|localhost_not_phone_scannable|just_works_recovery' "$PAIR_RS" >/dev/null \
  || fail "focusa pair missing localhost cross-device recovery guidance"
pass "focusa pair warns/recovers on localhost-only cross-device rooms"

rg -n 'phone camera shows raw JSON|Focusa Connect Page scanner|Paste completion payload fallback|Apply completion payload|mac_completion_payload' "$FIRST_RUN" >/dev/null \
  || fail "Mac first-run missing raw-JSON guidance or manual completion fallback"
pass "Mac first-run explains raw JSON and exposes manual paste fallback"
