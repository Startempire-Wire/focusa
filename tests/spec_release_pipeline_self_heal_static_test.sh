#!/usr/bin/env bash
# spec_release_pipeline_self_heal_static_test.sh
# Guard that live release self-heal covers all stages: CI, Release, Deploy.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HEAL="$ROOT_DIR/.github/workflows/auto-retry-deploy.yml"
AUDIT="$ROOT_DIR/.github/workflows/audit-recorder.yml"
fail(){ echo "✗ FAIL: $*" >&2; exit 1; }
pass(){ echo "✓ PASS: $*"; }

# Auto-heal must listen to all release-path stages.
for wf in '"CI"' '"Release"' '"Deploy Live Daemon"'; do
  grep -q "$wf" "$HEAL" || fail "auto-heal workflow missing trigger/workflow: $wf"
done
pass "auto-heal listens to CI, Release, and Deploy Live Daemon"

# CI/Release self-heal reruns failed jobs only, once; does not skip gates.
grep -q 'maybe-rerun-ci-release' "$HEAL" || fail "missing CI/Release rerun job"
grep -q 'gh run rerun "$RUN_ID" --failed' "$HEAL" || fail "CI/Release self-heal must rerun failed jobs"
grep -q 'RUN_ATTEMPT' "$HEAL" || fail "self-heal must cap attempts via run_attempt"
grep -q 'already attempted' "$HEAL" || fail "self-heal missing attempt cap message"
pass "CI/Release self-heal reruns failed jobs once"

# Deploy self-heal remains deploy-specific redispatch with release tag and asset suffix.
grep -q 'maybe-retry-deploy' "$HEAL" || fail "missing deploy retry job"
grep -q "gh workflow run 'Deploy Live Daemon'" "$HEAL" || fail "deploy retry must redispatch Deploy Live Daemon"
grep -q 'release_tag="$RELEASE_TAG"' "$HEAL" || fail "deploy retry must preserve release_tag"
grep -q 'asset_suffix="$ASSET_SUFFIX"' "$HEAL" || fail "deploy retry must preserve asset_suffix"
pass "deploy self-heal redispatches deploy with tag + asset suffix"

# Durable audit recorder must cover same stages.
grep -q 'workflows: \["CI", "Release", "Deploy Live Daemon"\]' "$AUDIT" \
  || fail "audit recorder must record CI, Release, and Deploy failures"
grep -q 'scripts/record-workflow-failure.py' "$AUDIT" \
  || fail "audit recorder missing failure recorder"
grep -q 'scripts/auto-heal-audit.py' "$AUDIT" \
  || fail "audit recorder missing audit self-heal script"
pass "audit recorder covers all stages and self-heals audit trail"

# No bypass language/commands: self-heal retries only; it must not mark success or skip clippy/tests.
if grep -qE 'continue-on-error: true|cargo clippy.*\|\| true|cargo test.*\|\| true' "$HEAL" "$AUDIT"; then
  fail "self-heal/audit workflow appears to bypass gates"
fi
pass "self-heal retries without bypassing CI/release/deploy gates"

echo "✓ All release pipeline self-heal static checks passed"