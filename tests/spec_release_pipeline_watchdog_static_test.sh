#!/usr/bin/env bash
# spec_release_pipeline_watchdog_static_test.sh
# Guard continuous auto-detect + auto-heal watchdog for live release pipeline.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WATCH="$ROOT_DIR/.github/workflows/release-pipeline-watchdog.yml"
CLASSIFIER="$ROOT_DIR/.github/scripts/classify-release-failure.sh"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

[ -f "$WATCH" ] || fail "missing release-pipeline-watchdog.yml"
pass "watchdog workflow exists"

grep -q 'cron: "\*/10 \* \* \* \*"' "$WATCH" || fail "watchdog must run continuously on schedule"
grep -q 'workflow_dispatch:' "$WATCH" || fail "watchdog must support manual dispatch for diagnostics"
pass "watchdog has scheduled + dispatch triggers"

for marker in 'CI|Release|"Auto Heal Release Pipeline"' '"Deploy Live Daemon"'; do
  grep -q "$marker" "$WATCH" || fail "watchdog missing healing case: $marker"
done
pass "watchdog handles CI, Release, Auto Heal, and Deploy failures"

grep -q 'gh run list --repo "$REPO"' "$WATCH" || fail "watchdog must list runs with explicit repo"
grep -q 'gh run view "$id" --repo "$REPO"' "$WATCH" || fail "watchdog must inspect attempts with explicit repo"
grep -q 'gh run rerun "$id" --failed --repo "$REPO"' "$WATCH" || fail "watchdog must rerun failed jobs with explicit repo"
grep -q "gh workflow run 'Deploy Live Daemon'" "$WATCH" || fail "watchdog must redispatch deploy"
grep -q -- '--repo "$REPO"' "$WATCH" || fail "watchdog deploy redispatch must use explicit repo"
pass "watchdog GH commands are repo-explicit"

grep -q 'MAX_HEALS' "$WATCH" || fail "watchdog must bound each invocation"
grep -q 'attempt.*-ge 2' "$WATCH" || fail "watchdog must avoid rerunning same failed run repeatedly"
grep -q 'process_error=watchdog_gh_run_rerun_failed' "$WATCH" || fail "watchdog must surface rerun process errors"
grep -q 'process_error=watchdog_deploy_redispatch_failed' "$WATCH" || fail "watchdog must surface deploy process errors"
pass "watchdog is bounded and captures process errors"

grep -q 'classify-release-failure.sh' "$WATCH" || fail "watchdog must use shared DRY failure classifier"
grep -q 'hard_failure_no_rerun' "$WATCH" || fail "watchdog must not rerun deterministic hard failures"
grep -q "inputs.max_heals || '2'" "$WATCH" || fail "watchdog default heal budget must be 2, not an unbounded loop"
for class in ci_clippy_failure ci_test_failure release_cross_target_compile_failure release_static_proof_failure auto_heal_process_error deploy_health_failure runner_resource_failure; do
  grep -q "$class" "$CLASSIFIER" || fail "classifier missing recent failure class: $class"
done
grep -q 'failure_class=${failure_class}' "$WATCH" || fail "watchdog must emit failure_class to summary"
pass "watchdog uses shared classifier and avoids deterministic rerun loops"

if grep -qE 'continue-on-error: true|cargo clippy.*\|\| true|cargo test.*\|\| true' "$WATCH"; then
  fail "watchdog appears to bypass gates"
fi
pass "watchdog heals by retry/redispatch only; gates still decide success"

echo "✓ All release pipeline watchdog static checks passed"
