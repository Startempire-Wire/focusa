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

grep -Fq 'name: Deploy Live Daemon' .github/workflows/deploy-live-daemon.yml || { echo "✗ workflow name missing"; exit 1; }
grep -Fq 'types: [published]' .github/workflows/deploy-live-daemon.yml || { echo "✗ release trigger missing"; exit 1; }
grep -Fq 'workflow_dispatch:' .github/workflows/deploy-live-daemon.yml || { echo "✗ workflow_dispatch trigger missing"; exit 1; }
rg -q 'gh release download' .github/workflows/deploy-live-daemon.yml || { echo "✗ release artifact download missing"; exit 1; }
rg -q 'install-daemon.sh' .github/workflows/deploy-live-daemon.yml || { echo "✗ installer invocation missing"; exit 1; }
rg -q 'safe-disk-cleanup.sh' .github/workflows/deploy-live-daemon.yml || { echo "✗ safe disk cleanup preflight missing"; exit 1; }
rg -q 'Require successful GitHub CI for target commit' .github/workflows/deploy-live-daemon.yml || { echo "✗ CI gate missing"; exit 1; }
rg -q 'runs-on: \[self-hosted, linux, x64, focusa-deploy\]' .github/workflows/deploy-live-daemon.yml || { echo "✗ self-hosted runner binding missing"; exit 1; }
rg -q 'Cleanup release artifact temp dir' .github/workflows/deploy-live-daemon.yml || { echo "✗ temp artifact cleanup missing"; exit 1; }
rg -q 'concurrency:' .github/workflows/deploy-live-daemon.yml || { echo "✗ deploy concurrency guard missing"; exit 1; }

rg -q 'flock -n 9' scripts/install-daemon.sh || { echo "✗ deploy lock missing"; exit 1; }
rg -q 'backup saved to' scripts/install-daemon.sh || { echo "✗ backup path log missing"; exit 1; }
rg -q 'rollback' scripts/install-daemon.sh || { echo "✗ rollback path missing"; exit 1; }
rg -q 'pgrep -x' scripts/install-daemon.sh || { echo "✗ duplicate-daemon guard missing"; exit 1; }
rg -q 'health version mismatch' scripts/install-daemon.sh || { echo "✗ version verification rollback missing"; exit 1; }
rg -q 'FOCUSA_DEPLOY_AUDIT_LOG' scripts/install-daemon.sh || { echo "✗ deploy audit log support missing"; exit 1; }
rg -q 'ExecStart mismatch' scripts/install-daemon.sh || { echo "✗ service ExecStart validation missing"; exit 1; }
rg -q 'binary_checksum' scripts/install-daemon.sh || { echo "✗ checksum capture missing"; exit 1; }

rg -q 'target' scripts/safe-disk-cleanup.sh || { echo "✗ target cleanup missing"; exit 1; }
rg -q '/tmp/focusa-release-' scripts/safe-disk-cleanup.sh || { echo "✗ temp cleanup missing"; exit 1; }
rg -q 'MIN_FREE_GB' scripts/safe-disk-cleanup.sh || { echo "✗ disk threshold guard missing"; exit 1; }

rg -q 'actions.runner' scripts/install-self-hosted-runner.sh || { echo "✗ runner service setup missing"; exit 1; }
rg -q 'focusa-deploy,production' scripts/install-self-hosted-runner.sh || { echo "✗ runner labels missing"; exit 1; }

rg -q 'verify-version-surfaces.py' .github/workflows/release.yml || { echo "✗ release workflow does not verify stamped versions"; exit 1; }
rg -q 'Deploy Live Daemon' scripts/create-dev-release-tag.sh || { echo "✗ create-dev-release-tag does not wait for deploy workflow"; exit 1; }

echo "Release deploy automation static test: PASS"
