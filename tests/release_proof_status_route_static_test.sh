#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ROUTE="${ROOT_DIR}/crates/focusa-api/src/routes/release.rs"
SERVER="${ROOT_DIR}/crates/focusa-api/src/server.rs"
MOD="${ROOT_DIR}/crates/focusa-api/src/routes/mod.rs"
MENUBAR="${ROOT_DIR}/apps/menubar/src/routes/+page.svelte"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

rg -n '/v1/release/proof/status|manual_proof_required|focusa release prove --tag <tag> --github|tool_result_v1' "$ROUTE" >/dev/null \
  || fail "release proof status route missing manual-gate envelope"
pass "release proof status route exposes manual-gate envelope"

rg -n 'pub mod release' "$MOD" >/dev/null && rg -n 'routes::release::router\(\)' "$SERVER" >/dev/null \
  || fail "release proof route not wired into API router"
pass "release proof route wired into API router"

rg -n '/v1/release/proof/status' "$MENUBAR" >/dev/null \
  || fail "menubar release card does not fetch API proof posture"
pass "menubar fetches release proof status API"

if rg -n "releaseProof: \{\s*status: 'ready'|releaseProof: \{ status: 'ready'" "$MENUBAR"; then
  fail "menubar still hardcodes ready release proof"
fi
pass "menubar avoids hardcoded ready release proof"

echo "Release proof status route static test: PASS"
