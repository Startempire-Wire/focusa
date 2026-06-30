#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "=== release deploy automation static test ==="

[[ -f .github/workflows/deploy-live-daemon.yml ]] || { echo "✗ missing deploy-live-daemon workflow"; exit 1; }
[[ -f scripts/install-daemon.sh ]] || { echo "✗ missing install-daemon.sh"; exit 1; }
[[ -f scripts/verify-version-surfaces.py ]] || { echo "✗ missing verify-version-surfaces.py"; exit 1; }
[[ -f scripts/safe-disk-cleanup.sh ]] || { echo "✗ missing safe-disk-cleanup.sh"; exit 1; }
[[ -f scripts/install-self-hosted-runner.sh ]] || { echo "✗ missing install-self-hosted-runner.sh"; exit 1; }
[[ -f scripts/deploy-smoke-check.sh ]] || { echo "✗ missing deploy-smoke-check.sh"; exit 1; }

assert_grep() {
  local needle="$1"
  local file="$2"
  local label="$3"
  if ! grep -Fq -- "$needle" "$file"; then
    echo "✗ $label"
    exit 1
  fi
}

# Workflow file assertions
assert_grep 'name: Deploy Live Daemon' .github/workflows/deploy-live-daemon.yml 'workflow name missing'
assert_grep 'types: [published]' .github/workflows/deploy-live-daemon.yml 'release trigger missing'
assert_grep 'workflow_dispatch:' .github/workflows/deploy-live-daemon.yml 'workflow_dispatch trigger missing'
assert_grep 'gh release download' .github/workflows/deploy-live-daemon.yml 'release artifact download missing'
assert_grep 'install-daemon.sh' .github/workflows/deploy-live-daemon.yml 'installer invocation missing'
assert_grep 'safe-disk-cleanup.sh' .github/workflows/deploy-live-daemon.yml 'safe disk cleanup preflight missing'
assert_grep 'Require successful GitHub CI for target commit' .github/workflows/deploy-live-daemon.yml 'CI gate missing'
assert_grep 'runs-on: [self-hosted, linux, x64, focusa-deploy]' .github/workflows/deploy-live-daemon.yml 'self-hosted runner binding missing'
assert_grep 'Cleanup release artifact temp dir' .github/workflows/deploy-live-daemon.yml 'temp artifact cleanup missing'
assert_grep 'Self-healing smoke check' .github/workflows/deploy-live-daemon.yml 'post-deploy smoke check missing'
assert_grep 'concurrency:' .github/workflows/deploy-live-daemon.yml 'deploy concurrency guard missing'

# install-daemon.sh assertions
assert_grep 'flock -n 9' scripts/install-daemon.sh 'deploy lock missing'
assert_grep 'backup saved to' scripts/install-daemon.sh 'backup path log missing'
assert_grep 'rollback' scripts/install-daemon.sh 'rollback path missing'
assert_grep 'pgrep -x' scripts/install-daemon.sh 'duplicate-daemon guard missing'
assert_grep 'health version mismatch' scripts/install-daemon.sh 'version verification rollback missing'
assert_grep 'FOCUSA_DEPLOY_AUDIT_LOG' scripts/install-daemon.sh 'deploy audit log support missing'
assert_grep 'ExecStart mismatch' scripts/install-daemon.sh 'service ExecStart validation missing'
assert_grep 'binary_checksum' scripts/install-daemon.sh 'checksum capture missing'

# safe-disk-cleanup.sh assertions
assert_grep 'target' scripts/safe-disk-cleanup.sh 'target cleanup missing'
assert_grep '/tmp/focusa-release-' scripts/safe-disk-cleanup.sh 'temp cleanup missing'
assert_grep 'MIN_FREE_GB' scripts/safe-disk-cleanup.sh 'disk threshold guard missing'

# install-self-hosted-runner.sh assertions
assert_grep 'actions.runner' scripts/install-self-hosted-runner.sh 'runner service setup missing'
assert_grep 'focusa-deploy,production' scripts/install-self-hosted-runner.sh 'runner labels missing'

# deploy-smoke-check.sh assertions
assert_grep 'audit_event "smoke_check"' scripts/deploy-smoke-check.sh 'smoke check audit emission missing'

# release workflow version verification
assert_grep 'verify-version-surfaces.py' .github/workflows/release.yml 'release workflow does not verify stamped versions'
assert_grep 'Deploy Live Daemon' scripts/create-dev-release-tag.sh 'create-dev-release-tag does not wait for deploy workflow'

echo "Release deploy automation static test: PASS"