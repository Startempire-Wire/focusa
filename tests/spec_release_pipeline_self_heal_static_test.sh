#!/usr/bin/env bash
# spec_release_pipeline_self_heal_static_test.sh
# Guard that live release self-heal covers all stages: CI, Release, Deploy.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HEAL="$ROOT_DIR/.github/workflows/auto-retry-deploy.yml"
CLASSIFIER="$ROOT_DIR/.github/scripts/classify-release-failure.sh"
RELEASE="$ROOT_DIR/.github/workflows/release.yml"
AUDIT="$ROOT_DIR/.github/workflows/audit-recorder.yml"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

# Auto-heal must listen to all release-path stages.
for wf in '"CI"' '"Release"' '"Deploy Live Daemon"' '"Release Pipeline Watchdog"'; do
  grep -q "$wf" "$HEAL" || fail "auto-heal workflow missing trigger/workflow: $wf"
done
pass "auto-heal listens to CI, Release, Deploy Live Daemon, and Watchdog"

# CI/Release self-heal reruns failed jobs only, once; does not skip gates.
grep -q 'maybe-rerun-ci-release' "$HEAL" || fail "missing CI/Release rerun job"
grep -q 'gh run rerun "$RUN_ID" --failed --repo "$REPO"' "$HEAL" || fail "CI/Release self-heal must rerun failed jobs with explicit repo"
grep -q 'RUN_ATTEMPT' "$HEAL" || fail "self-heal must cap attempts via run_attempt"
grep -q 'classify-release-failure.sh' "$HEAL" || fail "self-heal must use shared DRY failure classifier"
grep -q 'hard_failure_no_rerun' "$HEAL" || fail "self-heal must avoid rerun loops for deterministic failures"
grep -q 'self-heal_stop: deterministic failure' "$HEAL" || fail "self-heal must emit clear deterministic stop reason"
grep -q 'already attempted' "$HEAL" || fail "self-heal missing attempt cap message"
pass "CI/Release self-heal reruns failed jobs once"


# Shared classifier must encode learned hard/transient release failure classes.
[ -f "$CLASSIFIER" ] || fail "missing shared release failure classifier"
for class in ci_clippy_failure ci_test_failure release_cross_target_compile_failure release_static_proof_failure deploy_health_failure runner_resource_failure auto_heal_process_error transient_github_or_network_failure unknown_process_failure; do
  grep -q "$class" "$CLASSIFIER" || fail "classifier missing failure class: $class"
done
grep -q 'hard_failure_no_rerun' "$CLASSIFIER" || fail "classifier must distinguish deterministic hard failures"
grep -q 'plain_language_error' "$CLASSIFIER" || fail "classifier must emit plain_language_error"
pass "shared classifier distinguishes learned hard/transient release failures"

# Release workflow must avoid two-hour loops and dispatch deploy after one green Release.
grep -q 'fail-fast: true' "$RELEASE" || fail "Release matrices must fail fast"
grep -q 'timeout-minutes:' "$RELEASE" || fail "Release jobs must have timeouts"
grep -q 'dispatch-deploy-live-daemon' "$RELEASE" || fail "Release must explicitly dispatch deploy after assets/checksums"
grep -q "gh workflow run 'Deploy Live Daemon'" "$RELEASE" || fail "Release dispatch must use gh workflow run Deploy Live Daemon"
grep -q 'needs: checksums' "$RELEASE" || fail "Deploy dispatch must wait for the actual checksums job id"
grep -q 'process-health-check.py' "$RELEASE" || fail "Release should include process health wrapper for rust gates"
pass "Release is bounded and dispatches deploy after green artifacts"

if ! grep -q 'process-health-check.py' "$ROOT_DIR/.github/workflows/ci.yml"; then
  fail "CI should include process health checkpoints for Rust gates"
fi

# Deploy self-heal remains deploy-specific redispatch with release tag and asset suffix.
grep -q 'maybe-retry-deploy' "$HEAL" || fail "missing deploy retry job"
grep -q "gh workflow run 'Deploy Live Daemon'" "$HEAL" || fail "deploy retry must redispatch Deploy Live Daemon"
grep -q -- '--repo "$REPO"' "$HEAL" || fail "deploy retry must use explicit repo"
grep -q 'release_tag="$RELEASE_TAG"' "$HEAL" || fail "deploy retry must preserve release_tag"
grep -q 'asset_suffix="$ASSET_SUFFIX"' "$HEAL" || fail "deploy retry must preserve asset_suffix"
pass "deploy self-heal redispatches deploy with tag + asset suffix"

# Durable audit recorder must cover release stages and Auto Heal's own process errors.
grep -q 'workflows: \["CI", "Release", "Deploy Live Daemon", "Auto Heal Release Pipeline", "Release Pipeline Watchdog"\]' "$AUDIT" \
  || fail "audit recorder must record CI, Release, Deploy, Auto Heal, and Watchdog failures"
grep -q 'scripts/record-workflow-failure.py' "$AUDIT" \
  || fail "audit recorder missing failure recorder"
grep -q 'scripts/auto-heal-audit.py' "$AUDIT" \
  || fail "audit recorder missing audit self-heal script"
pass "audit recorder covers all stages plus Auto Heal process errors"

# Auto Heal process errors must be captured visibly for the audit recorder.
grep -q 'process_error=gh_run_rerun_failed' "$HEAL" || fail "CI/Release process errors must be surfaced"
grep -q 'process_error=gh_workflow_run_failed' "$HEAL" || fail "Deploy process errors must be surfaced"
pass "Auto Heal captures its own process errors"

# No bypass language/commands: self-heal retries only; it must not mark success or skip clippy/tests.
if grep -qE 'continue-on-error: true|cargo clippy.*\|\| true|cargo test.*\|\| true' "$HEAL" "$AUDIT"; then
  fail "self-heal/audit workflow appears to bypass gates"
fi
pass "self-heal retries without bypassing CI/release/deploy gates"

echo "✓ All release pipeline self-heal static checks passed"