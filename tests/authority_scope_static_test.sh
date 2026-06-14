#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AUTH="$ROOT_DIR/docs/current/AUTHORITY_MODEL.md"
SPEC="$ROOT_DIR/docs/106-focusa-vision-tightening-spec.md"
CTX="$ROOT_DIR/crates/focusa-api/src/routes/context_cognition.rs"
WORKPOINT="$ROOT_DIR/crates/focusa-api/src/routes/workpoint.rs"
TRAJECTORY="$ROOT_DIR/crates/focusa-api/src/routes/trajectory.rs"
PROJECT="$ROOT_DIR/crates/focusa-api/src/routes/project.rs"
WORK_LOOP="$ROOT_DIR/crates/focusa-api/src/routes/work_loop.rs"
PI_STATE="$ROOT_DIR/apps/pi-extension/src/state.ts"
PI_SESSION="$ROOT_DIR/apps/pi-extension/src/session.ts"

fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

rg -n -F 'project_root + continuity_id = authority boundary' "$AUTH" >/dev/null || fail "Authority Model missing exact scope invariant"
rg -n -F 'No canonical read/write without verified project_root + continuity_id' "$SPEC" >/dev/null || fail "Spec106 missing exact scope invariant"
pass "authority docs declare exact project_root + continuity_id invariant"

for file in "$CTX" "$WORKPOINT" "$TRAJECTORY" "$PROJECT"; do
  rg -n 'project_root' "$file" >/dev/null || fail "$file missing project_root handling"
  rg -n 'continuity_id' "$file" >/dev/null || fail "$file missing continuity_id handling"
done
rg -n 'writer_claim_key_from_state|project:.*workstream:.*work_item:|work_loop_scope_root' "$WORK_LOOP" >/dev/null \
  || fail "$WORK_LOOP missing scoped writer partition"
pass "canonical routes expose project_root+continuity_id; Work-loop exposes scoped writer partition"

for needle in \
  'missing_continuity_id' \
  'canonical Workpoint/Trajectory selection requires verified project_root + continuity_id' \
  'r.continuity_id.as_deref() == continuity_id.as_deref()' \
  'record.continuity_id.as_deref() == continuity_id.as_deref()'; do
  rg -n -F "$needle" "$CTX" >/dev/null || fail "Context Cognition exact-scope guard missing $needle"
done
pass "Context Cognition exact-scope guards present"

rg -n 'isWorkpointPacketScopedToCurrentSession|currentContinuityId|packetContinuityId|projectRootAuthorityFailure|project_root.*continuity_id|continuity_id.*project_root' "$PI_STATE" "$PI_SESSION" >/dev/null \
  || fail "Pi extension missing project_root/continuity scoped packet guard"
pass "Pi extension has scoped packet guard references"

echo "authority scope static test: PASS"
