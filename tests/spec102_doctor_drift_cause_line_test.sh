#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="${ROOT_DIR:-/home/wirebot/focusa}"
cd "$ROOT_DIR"
TOOLS="apps/pi-extension/src/tools.ts"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

# Clean path: compact doctor text must not always emit drift=no or residual drift machinery.
if rg -n 'drift=\$\{contractDrift\.drift_detected \? "yes" : "no"\}|drift=\$\{contractDrift\.drift_detected \? "yes" : "no"\}' "$TOOLS" >/dev/null; then
  fail "compact doctor still emits unconditional drift yes/no machinery"
fi
if ! rg -n 'const driftSummary = contractDrift\.drift_detected' "$TOOLS" >/dev/null; then
  fail "missing conditional drift summary"
fi
if ! rg -n ': "";' "$TOOLS" >/dev/null; then
  fail "clean drift path must omit compact drift summary"
fi
pass "drift=false path omits compact drift machinery"

# Active drift path: include bounded cause counts and source refs.
for needle in \
  'drift_causes=missing_live:' \
  'extra_live:' \
  'stale_live_contracts:' \
  'source_refs=static:apps/pi-extension/src/tools.ts,live:/v1/ontology/tool-contracts' \
  'cause_counts: driftCauseCounts'; do
  rg -F "$needle" "$TOOLS" >/dev/null || fail "missing active drift cause/source marker: $needle"
done
pass "drift=yes path includes bounded cause counts and source refs"

echo "SPEC102 doctor drift cause-line test: PASS"
