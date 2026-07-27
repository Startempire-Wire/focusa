#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
QUARANTINED="$ROOT_DIR/.github/workflows/auto-retry-deploy.yml"
WATCHDOG="$ROOT_DIR/.github/workflows/release-pipeline-watchdog.yml"
AUDIT="$ROOT_DIR/.github/workflows/audit-recorder.yml"
GOVERNOR="$ROOT_DIR/scripts/self_heal_governor.py"

fail() { echo "✗ FAIL: $*"; exit 1; }
pass() { echo "✓ PASS: $*"; }

for file in "$QUARANTINED" "$WATCHDOG" "$AUDIT" "$GOVERNOR"; do
  [[ -f "$file" ]] || fail "missing $file"
done

! grep -q 'workflow_run:' "$QUARANTINED" || fail "legacy auto-retry still has automatic trigger"
grep -q 'status=quarantined' "$QUARANTINED" || fail "legacy boundary is not explicit"
grep -q 'self_heal_governor.py claim' "$WATCHDOG" || fail "watchdog bypasses governor claim"
grep -q 'self_heal_governor.py settle' "$WATCHDOG" || fail "watchdog lacks settlement"
grep -q 'attempt >= 2' "$WATCHDOG" || fail "watchdog lacks per-run attempt bound"
grep -q 'same_tag_runs > 1' "$WATCHDOG" || fail "deploy redispatch lacks duplicate bound"
grep -q 'mutation_budget' "$WATCHDOG" || fail "watchdog lacks mutation budget"
grep -q 'proposal_fingerprint' "$AUDIT" || fail "audit proposal lacks fingerprint identity"
grep -q 'open_count >= 3' "$AUDIT" || fail "audit proposal lacks open budget"
! grep -q -- '--force' "$AUDIT" || fail "fingerprint claim may overwrite remote state"

python3 -m py_compile "$GOVERNOR" "$ROOT_DIR/scripts/propose-system-fix.py"
pass "one governed retry actor with bounded proposal authority"
