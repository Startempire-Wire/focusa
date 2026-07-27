#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WATCH="$ROOT_DIR/.github/workflows/release-pipeline-watchdog.yml"
CLASSIFIER="$ROOT_DIR/scripts/classify-ci-failure.py"
GOVERNOR="$ROOT_DIR/scripts/self_heal_governor.py"

fail() { echo "✗ FAIL: $*"; exit 1; }
pass() { echo "✓ PASS: $*"; }

[[ -f "$WATCH" ]] || fail "watchdog workflow missing"
grep -q 'cron: "\*/10 \* \* \* \*"' "$WATCH" || fail "schedule missing"
grep -q 'workflow_dispatch:' "$WATCH" || fail "manual dispatch missing"
grep -q "MAX_HEALS:.*'1'" "$WATCH" || fail "default mutation budget is not one"
grep -q 'CI|Release|Deploy' "$WATCH" || fail "release-path allowlist missing"
grep -q 'classify-ci-failure.py failed.log --format json' "$WATCH" || fail "typed classification missing"
grep -q 'focusa.self_heal.failure.v1' "$WATCH" || fail "typed failure envelope missing"
grep -q 'policy:release-pipeline-watchdog' "$WATCH" || fail "policy approval missing"
grep -q 'rerun_failed_jobs' "$WATCH" || fail "bounded retry action missing"
grep -q 'redispatch_deploy' "$WATCH" || fail "immutable deploy recovery missing"
grep -q -- '--status operator_review' "$WATCH" || fail "failed action settlement missing"
! grep -q 'Auto Heal Release Pipeline' "$WATCH" || fail "quarantined actor remains in watchdog loop"
python3 -m py_compile "$CLASSIFIER" "$GOVERNOR"
pass "watchdog uses typed claim, bounded mutation, and settlement"
