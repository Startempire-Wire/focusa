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
  if [[ "$needle" == --* ]]; then
    # Caller explicitly passed the needle as --needle; grep needs it as --end-of-options sentinel.
    needle="${needle#--}"
  fi
  if ! grep -Fq -e "$needle" "$file"; then
    echo "✗ $label"
    exit 1
  fi
}

# Workflow file assertions
assert_grep 'name: Deploy Live Daemon' .github/workflows/deploy-live-daemon.yml 'workflow name missing'
assert_grep 'types: [published]' .github/workflows/deploy-live-daemon.yml 'release trigger missing'
assert_grep 'workflow_dispatch:' .github/workflows/deploy-live-daemon.yml 'workflow_dispatch trigger missing'
assert_grep 'gh release download' .github/workflows/deploy-live-daemon.yml 'release artifact download missing'
assert_grep --clobber .github/workflows/deploy-live-daemon.yml 'release artifact clobber flag missing'
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
assert_grep 'set +e' scripts/install-daemon.sh 'strict-mode kill guard missing'
assert_grep 'health version mismatch' scripts/install-daemon.sh 'version verification rollback missing'
assert_grep 'FOCUSA_DEPLOY_AUDIT_LOG' scripts/install-daemon.sh 'deploy audit log support missing'
assert_grep 'ExecStart mismatch' scripts/install-daemon.sh 'service ExecStart validation missing'
assert_grep 'binary_checksum' scripts/install-daemon.sh 'checksum capture missing'

# safe-disk-cleanup.sh assertions
assert_grep 'target' scripts/safe-disk-cleanup.sh 'target cleanup missing'
assert_grep '/tmp/focusa-release-' scripts/safe-disk-cleanup.sh 'temp cleanup missing'
assert_grep 'MIN_FREE_GB' scripts/safe-disk-cleanup.sh 'disk threshold guard missing'
assert_grep 'BACKUP_KEEP' scripts/safe-disk-cleanup.sh 'backup keep bound missing'
assert_grep 'backup_keep=${{ steps.cfg.outputs.backup_keep }}' .github/workflows/deploy-live-daemon.yml 'workflow backup_keep wiring missing'

# install-daemon.sh unit-patch branch guards (auto-heal stale ExecStart)
assert_grep 'patch_service_unit_execstart' scripts/install-daemon.sh 'unit auto-patch branch missing'
assert_grep 'ExecStart=${INSTALL_PATH}' scripts/install-daemon.sh 'unit ExecStart rewrite pattern missing'
assert_grep 'x86_64-unknown-linux-musl' .github/workflows/deploy-live-daemon.yml 'musl default suffix missing (AlmaLinux 8 glibc)'
assert_grep '/usr/bin/sed' scripts/install-self-hosted-runner.sh 'runner sudoers sed allowlist missing'

# Self-healing safety net (wall clock + RSS) and auto-retry workflow
assert_grep 'WALL_CLOCK_SEC' scripts/install-daemon.sh 'wall clock guard missing'
assert_grep 'RSS_LIMIT_MB' scripts/install-daemon.sh 'RSS memory guard missing'
assert_grep 'deploy_oom_killed' scripts/install-daemon.sh 'OOM audit event missing'
assert_grep 'deploy_health' scripts/install-daemon.sh 'health-timeout audit event missing'
assert_grep 'watchdog_check' scripts/install-daemon.sh 'watchdog wiring missing'
assert_grep 'watchdog_loop' scripts/install-daemon.sh 'background watchdog loop missing'
assert_grep 'timeout 3' scripts/install-daemon.sh 'binary_version must use timeout fallback'
assert_grep 'workflow_run' .github/workflows/auto-retry-deploy.yml 'auto-retry must be self-triggered via workflow_run'
assert_grep 'Auto Retry Deploy' .github/workflows/auto-retry-deploy.yml 'auto-retry workflow name missing'

# Self-hosted runner must self-heal from kernel OOM kills
assert_grep 'MemoryMax=' scripts/install-self-hosted-runner.sh 'runner MemoryMax override missing'
assert_grep 'Restart=always' scripts/install-self-hosted-runner.sh 'runner Restart=always override missing'

# install-self-hosted-runner.sh assertions
assert_grep 'actions.runner' scripts/install-self-hosted-runner.sh 'runner service setup missing'
assert_grep 'focusa-deploy,production' scripts/install-self-hosted-runner.sh 'runner labels missing'

# deploy-smoke-check.sh assertions
assert_grep 'audit_event "smoke_check"' scripts/deploy-smoke-check.sh 'smoke check audit emission missing'

# release workflow version verification
assert_grep 'verify-version-surfaces.py' .github/workflows/release.yml 'release workflow does not verify stamped versions'
assert_grep 'Deploy Live Daemon' scripts/create-dev-release-tag.sh 'create-dev-release-tag does not wait for deploy workflow'

# audit schema validation (single canonical shape)
assert_grep 'audit-schema.py' scripts/audit-schema.py 'audit schema script must self-reference'
assert_grep 'REQUIRED_FAILURE' scripts/audit-schema.py 'audit schema missing required failure fields'
assert_grep 'REQUIRED_ADDITION' scripts/audit-schema.py 'audit schema missing required addition fields'
assert_grep 'REQUIRED_SELF_HEAL' scripts/audit-schema.py 'audit schema missing required self_heal fields'
assert_grep 'VALID_CATEGORIES' scripts/audit-schema.py 'audit schema missing category enum'
assert_grep 'VALID_SUBSYSTEMS' scripts/audit-schema.py 'audit schema missing subsystem enum'
assert_grep 'ci_workflow_failure' scripts/audit-schema.py 'audit schema must include ci_workflow_failure category used by audit-recorder.yml'
if ! python3 scripts/audit-schema.py validate release-proof/audit/audit.jsonl >/dev/null; then
  echo "✗ audit schema validation failed"
  python3 scripts/audit-schema.py validate release-proof/audit/audit.jsonl
  exit 1
fi

# changelog generator
assert_grep 'changelog-gen.py' scripts/changelog-gen.py 'changelog gen must self-reference'
assert_grep 'CATEGORIES_BY_LAYER' scripts/changelog-gen.py 'changelog gen missing layer grouping'
assert_grep 'Layer 1 — Runner' scripts/changelog-gen.py 'changelog gen missing runner layer'

# install-daemon contract spec
assert_grep 'binary_version' docs/install-daemon-contract.md 'contract missing binary_version'
assert_grep 'patch_service_unit_execstart' docs/install-daemon-contract.md 'contract missing execstart patch'
assert_grep 'watchdog_check' docs/install-daemon-contract.md 'contract missing watchdog'
assert_grep 'wait_for_health' docs/install-daemon-contract.md 'contract missing wait_for_health'

# operator runbook
assert_grep 'GitHub Actions is down' docs/deploy-runbook.md 'runbook must cover GitHub outage'
assert_grep 'Runner token is expired' docs/deploy-runbook.md 'runbook must cover token expiry'
assert_grep 'Audit trail fails to validate' docs/deploy-runbook.md 'runbook must cover audit validation'

# cross-links
assert_grep 'self-heal-chain.md' docs/production-deployment-guide.md 'prod guide missing self-heal link'
assert_grep 'deploy-runbook.md' docs/production-deployment-guide.md 'prod guide missing runbook link'

echo "Release deploy automation static test: PASS"