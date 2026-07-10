#!/usr/bin/env bash
# AX GAP v3: workpoint checkpoint response/text must show what was checkpointed.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKPOINT="$ROOT_DIR/crates/focusa-api/src/routes/workpoint.rs"
TOOLS="$ROOT_DIR/apps/pi-extension/src/tools.ts"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[ -f "$WORKPOINT" ] || fail "workpoint route missing"
[ -f "$TOOLS" ] || fail "Pi tools missing"

for token in \
  'fn checkpoint_summary' \
  '"checkpoint_summary"' \
  '"rendered_summary"' \
  'checkpointed mission=' \
  'action_type' \
  'target_ref' \
  'next_slice' \
  'checkpointed typed mission/action/next_slice'; do
  grep -q "$token" "$WORKPOINT" || fail "API checkpoint summary missing: $token"
done

for token in \
  'checkpointSummary' \
  'checkpoint_summary?.one_line' \
  'workpoint checkpoint →' \
  'rendered_summary'; do
  grep -q "$token" "$TOOLS" || fail "Pi checkpoint text missing: $token"
done

pass "workpoint checkpoint summary appears in API and Pi text"
