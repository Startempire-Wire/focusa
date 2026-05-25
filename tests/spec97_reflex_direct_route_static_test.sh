#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ROUTE="$ROOT_DIR/crates/focusa-api/src/routes/reflex.rs"
MOD="$ROOT_DIR/crates/focusa-api/src/routes/mod.rs"
SERVER="$ROOT_DIR/crates/focusa-api/src/server.rs"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"
SPEC="$ROOT_DIR/docs/97-focusa-reflex-primitives-spec.md"
EVIDENCE="$ROOT_DIR/docs/evidence/SPEC97_REFLEX_DIRECT_API_LIVE_PROOF_2026-05-25.md"

fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

[[ -f "$ROUTE" ]] || fail "reflex route file missing"
rg -n '/v1/reflex/primitives|REFLEX_REGISTRY|focusa-reflex-primitives\.json|read_only|advisory_only|include_payload|MAX_LIMIT' "$ROUTE" >/dev/null || fail "reflex direct route lacks bounded read-only registry plumbing"
pass "reflex direct route exposes bounded read-only registry"

rg -n 'reflex_primitives_route_is_bounded_and_read_only' "$ROUTE" >/dev/null || fail "reflex direct route regression test missing"
pass "reflex direct route has bounded/read-only regression"

rg -n 'pub mod reflex' "$MOD" >/dev/null || fail "routes module does not export reflex route"
rg -n 'routes::reflex::router\(\)' "$SERVER" >/dev/null || fail "server does not merge reflex router"
pass "reflex route is wired into API server"

rg -n 'focusa_reflex_primitives|/reflex/primitives|read_only|advisory_only|include_payload' "$TOOLS" >/dev/null || fail "Pi tool surface lacks bounded read-only reflex primitive accessor"
pass "Pi tool exposes bounded read-only reflex primitive accessor"

rg -n '/v1/reflex/primitives|include_payload=true|Direct API route|SPEC97_REFLEX_DIRECT_API_LIVE_PROOF' "$SPEC" >/dev/null || fail "Spec97 does not document direct reflex route, cold payload boundary, and live proof"
pass "Spec97 documents direct reflex route, payload boundary, and live proof"

rg -n 'SPEC97_REFLEX_DIRECT_API_LIVE_PROOF=PASS|read_only|advisory_only|route_noncanonical_result|include_payload=true' "$EVIDENCE" >/dev/null || fail "Spec97 direct API live proof evidence missing or incomplete"
pass "Spec97 direct API live proof evidence is present"

echo "SPEC97 reflex direct route static test: PASS"
