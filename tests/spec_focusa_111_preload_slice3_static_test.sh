#!/usr/bin/env bash
# Spec 111 Slice 3 — API routes /v1/preload/{profiles,build,render,verify,doctor} static guard.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

PRE="$ROOT_DIR/crates/focusa-api/src/routes/preload.rs"
[[ -f "$PRE" ]] || fail "preload.rs missing"
for needle in \
  'AGENT_BOOTSTRAP_PROFILES' \
  'pub fn build_packet_for_profile' \
  'rendered' \
  'Focusa Agent Bootstrap' \
  'profile_by_id'; do
  grep -qF -- "$needle" "$PRE" || fail "preload slice 3 missing: $needle"
done
pass "preload slice 3 dispatches read-mostly routes to slice 2 logic"
echo "focusa-111 preload slice3 static test: PASS"
