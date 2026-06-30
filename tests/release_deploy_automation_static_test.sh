#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "=== release deploy automation static test ==="

[[ -f .github/workflows/deploy-live-daemon.yml ]] || { echo "✗ missing deploy-live-daemon workflow"; exit 1; }
[[ -f scripts/install-daemon.sh ]] || { echo "✗ missing install-daemon.sh"; exit 1; }
[[ -f scripts/verify-version-surfaces.py ]] || { echo "✗ missing verify-version-surfaces.py"; exit 1; }

rg -q '^name: Deploy Live Daemon$' .github/workflows/deploy-live-daemon.yml || { echo "✗ workflow name missing"; exit 1; }
rg -q 'release:' .github/workflows/deploy-live-daemon.yml || { echo "✗ release trigger missing"; exit 1; }
rg -q 'workflow_dispatch:' .github/workflows/deploy-live-daemon.yml || { echo "✗ workflow_dispatch trigger missing"; exit 1; }
rg -q 'gh release download' .github/workflows/deploy-live-daemon.yml || { echo "✗ release artifact download missing"; exit 1; }
rg -q 'install-daemon.sh' .github/workflows/deploy-live-daemon.yml || { echo "✗ remote installer invocation missing"; exit 1; }
rg -q 'concurrency:' .github/workflows/deploy-live-daemon.yml || { echo "✗ deploy concurrency guard missing"; exit 1; }

rg -q 'flock -n 9' scripts/install-daemon.sh || { echo "✗ deploy lock missing"; exit 1; }
rg -q 'backup saved to' scripts/install-daemon.sh || { echo "✗ backup path log missing"; exit 1; }
rg -q 'rollback' scripts/install-daemon.sh || { echo "✗ rollback path missing"; exit 1; }
rg -q 'pgrep -x' scripts/install-daemon.sh || { echo "✗ duplicate-daemon guard missing"; exit 1; }
rg -q 'health version mismatch' scripts/install-daemon.sh || { echo "✗ version verification rollback missing"; exit 1; }

rg -q 'verify-version-surfaces.py' .github/workflows/release.yml || { echo "✗ release workflow does not verify stamped versions"; exit 1; }
rg -q 'Deploy Live Daemon' scripts/create-dev-release-tag.sh || { echo "✗ create-dev-release-tag does not wait for deploy workflow"; exit 1; }

echo "Release deploy automation static test: PASS"
