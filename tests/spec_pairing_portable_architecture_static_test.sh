#!/usr/bin/env bash
# Pairing architecture guard: protocol must be host-neutral and transport-verified.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

SPEC="$ROOT_DIR/docs/53-focusa-device-pairing-spec.md"
PLAN="$ROOT_DIR/docs/54-focusa-pairing-room-plan.md"
PAIR_RS="$ROOT_DIR/crates/focusa-cli/src/commands/pair.rs"

rg -n 'Host-neutral|transport-agnostic|future hosted Focusa|verified reachable transport' "$SPEC" "$PLAN" >/dev/null \
  || fail "pairing docs missing host-neutral portable architecture language"
rg -n 'localhost.*same-machine|never a portable cross-device URL|resolver is the pairing authority boundary|/connect.*v1/connect/room' "$PLAN" >/dev/null \
  || fail "phone bridge plan missing transport authority boundary"
rg -n 'localhost_not_phone_scannable|checked_candidates|connect_route_reachable|bridge_api_reachable|just_works_recovery' "$PAIR_RS" >/dev/null \
  || fail "focusa pair missing verified-transport diagnostics/recovery fields"

if rg -n 'verious\.net|host\.philoveracity\.com|connect\.focusa\.dev' "$SPEC" "$PLAN" "$PAIR_RS" >/dev/null; then
  fail "pairing architecture contains environment-specific domain assumptions"
fi

pass "pairing architecture is host-neutral and transport-verified"
