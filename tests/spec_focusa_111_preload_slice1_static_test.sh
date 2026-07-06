#!/usr/bin/env bash
# Spec 111 §19.1 Slice 1 — schema + static contracts for /v1/preload/*
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

PRE="$ROOT_DIR/crates/focusa-api/src/routes/preload.rs"
MOD="$ROOT_DIR/crates/focusa-api/src/routes/mod.rs"
SERVER="$ROOT_DIR/crates/focusa-api/src/server.rs"
[[ -f "$PRE" ]] || fail "preload.rs missing"

for needle in \
  'focusa.preload.v1' \
  'AgentBootstrapPacket' \
  'AgentBootstrapProfile' \
  'AgentBootstrapReceipt' \
  'FOCUSA_PRELOAD_FAIL' \
  'bootstrap_delivery' \
  '/v1/preload/profiles' \
  '/v1/preload/build' \
  '/v1/preload/render' \
  '/v1/preload/verify' \
  '/v1/preload/doctor' \
  '/v1/preload/receipt-preview' \
  '/v1/preload/receipt-commit' \
  'PROFILE_RULES_ONLY' \
  'PROFILE_RULES_AND_CONTEXT' \
  'PROFILE_BUDGET_LIGHT' \
  'PROFILE_BUDGET_DEEP'; do
  grep -qF -- "$needle" "$PRE" || fail "preload route missing: $needle"
done
pass "preload routes cover spec-required endpoints, profiles, schemas, and fail code"

grep -qF 'pub mod preload;' "$MOD" || fail "routes mod missing preload export"
grep -qF 'routes::preload::router()' "$SERVER" || fail "server does not merge preload router"
pass "preload router wired into daemon router"
echo "focusa-111 preload slice1 static test: PASS"
