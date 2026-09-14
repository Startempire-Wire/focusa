#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "=== release deploy automation static test ==="
python3 tests/release_learning_guard_command_test.py

WORKFLOW_VALIDATION_OUT="$(mktemp /tmp/focusa-workflow-graph-validation.XXXXXX)"
trap 'rm -f "$WORKFLOW_VALIDATION_OUT"' EXIT
python3 scripts/validate-github-workflows.py .github/workflows/release.yml .github/workflows/auto-retry-deploy.yml .github/workflows/release-pipeline-watchdog.yml >"$WORKFLOW_VALIDATION_OUT" || {
  cat "$WORKFLOW_VALIDATION_OUT" >&2
  exit 1
}

tests/release_version_asset_test.sh
tests/system_daemon_lifecycle_static_test.sh

# GH5 remote marker onboarding guard.
tests/spec_focusa_gh5_remote_marker_static_test.sh

# L5 TUI usage evidence guard.
tests/spec_focusa_yixp_tui_usage_static_test.sh

# GH7 Pi unbound project nag guard.
tests/spec_focusa_gh7_pi_unbound_nag_static_test.sh

[[ -f .github/workflows/deploy-live-daemon.yml ]] || { echo "✗ missing deploy-live-daemon workflow"; exit 1; }
[[ -f scripts/install-daemon.sh ]] || { echo "✗ missing install-daemon.sh"; exit 1; }
[[ -f scripts/verify-version-surfaces.py ]] || { echo "✗ missing verify-version-surfaces.py"; exit 1; }
[[ -f scripts/release-gate.py ]] || { echo "✗ missing release-gate.py"; exit 1; }
[[ -f scripts/validate-github-workflows.py ]] || { echo "✗ missing validate-github-workflows.py"; exit 1; }
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

assert_not_grep() {
  local needle="$1"
  local file="$2"
  local label="$3"
  if grep -Fq -e "$needle" "$file"; then
    echo "✗ $label"
    exit 1
  fi
}

# Workflow file assertions
assert_grep 'name: Deploy Live Daemon' .github/workflows/deploy-live-daemon.yml 'workflow name missing'
assert_grep 'types: [published]' .github/workflows/deploy-live-daemon.yml 'release trigger missing'
assert_grep "github.event.release.prerelease == false" .github/workflows/deploy-live-daemon.yml 'candidate publication can dispatch a duplicate production deploy'
assert_grep 'workflow_dispatch:' .github/workflows/deploy-live-daemon.yml 'workflow_dispatch trigger missing'
assert_grep "*) CHANNEL='stable'" .github/workflows/deploy-live-daemon.yml 'stable tags must select the stable updater channel'
assert_grep '--channel "$CHANNEL"' .github/workflows/deploy-live-daemon.yml 'OTA trust gate must use the tag-derived channel'
assert_grep 'gh release download' .github/workflows/deploy-live-daemon.yml 'release artifact download missing'
assert_grep --clobber .github/workflows/deploy-live-daemon.yml 'release artifact clobber flag missing'
assert_grep 'install-daemon.sh' .github/workflows/deploy-live-daemon.yml 'installer invocation missing'
assert_grep 'safe-disk-cleanup.sh' .github/workflows/deploy-live-daemon.yml 'safe disk cleanup preflight missing'
assert_grep 'Require successful GitHub CI for target commit' .github/workflows/deploy-live-daemon.yml 'CI gate missing'
assert_grep 'runs-on: [self-hosted, linux, x64, focusa-deploy]' .github/workflows/deploy-live-daemon.yml 'self-hosted runner binding missing'
assert_grep 'Cleanup release artifact temp dir' .github/workflows/deploy-live-daemon.yml 'temp artifact cleanup missing'
assert_grep 'Self-healing smoke check' .github/workflows/deploy-live-daemon.yml 'post-deploy smoke check missing'
assert_grep 'Verify deployed full release locally' .github/workflows/deploy-live-daemon.yml 'full installed release parity gate missing'
assert_grep 'focusa focusa-daemon focusa-tui focusa-session-runner' .github/workflows/deploy-live-daemon.yml 'all canonical binaries must be verified after deploy'
assert_grep 'WorkingDirectory --value' .github/workflows/deploy-live-daemon.yml 'canonical state-root verification missing'
assert_grep 'concurrency:' .github/workflows/deploy-live-daemon.yml 'deploy concurrency guard missing'

# Per-run strict-spec daemon and cleanup propagation guards (#387).
assert_grep 'export DAEMON_BIN="${DAEMON_BIN:-$CARGO_TARGET_DIR/release/focusa-daemon}"' scripts/ci/run-spec-gates.sh 'strict spec child gates must inherit the isolated daemon path'
assert_grep 'spec-gates daemon missing after successful build: $DAEMON_BIN' scripts/ci/run-spec-gates.sh 'strict spec gate must fail immediately when its daemon artifact is absent'
assert_grep 's.bind(("127.0.0.1", 0))' scripts/ci/run-spec-gates.sh 'strict spec gate must allocate an isolated loopback port by default'
assert_grep 'http://127.0.0.1:${GATE_PORT}' scripts/ci/run-spec-gates.sh 'strict spec base URL must match its isolated daemon port'
assert_grep 'kill -0 "$DAEMON_PID"' scripts/ci/run-spec-gates.sh 'strict spec health poll must verify its own daemon remains alive'
assert_grep 'spec-gates daemon exited before health' scripts/ci/run-spec-gates.sh 'strict spec bind failures must stop immediately'
assert_grep 'TEST_BEADS_FIXTURE="$ROOT_DIR/.beads/issues.jsonl"' scripts/ci/run-spec-gates.sh 'isolated spec gates must provision only a synthetic Beads fixture when history is absent'
assert_grep 'rm -f "$TEST_BEADS_FIXTURE"' scripts/ci/run-spec-gates.sh 'synthetic Beads fixture must be removed on exit'
assert_grep 'mkdir "$ROOT_DIR/.git"' scripts/ci/run-spec-gates.sh 'isolated spec gates must claim only absent Git metadata'
assert_grep 'TEST_GIT_DIR="$ROOT_DIR/.git"' scripts/ci/run-spec-gates.sh 'disposable Git metadata must preserve the canonical workspace root'
assert_grep 'cleanup_test_git' scripts/ci/run-spec-gates.sh 'owned Git metadata must be cleaned on exit'
assert_grep 'git init -q "$ROOT_DIR"' scripts/ci/run-spec-gates.sh 'disposable Git metadata must initialize the exact gate worktree'
assert_grep 'FOCUSA_HISTORYLESS_GATE' scripts/ci/run-spec-gates.sh 'isolated historyless gate mode must be explicit'
assert_grep 'historyless isolated source sync' tests/bead_closure_evidence_gate.py 'history-only gate must report bounded historyless mode'
assert_grep 'import os' tests/bead_closure_evidence_gate.py 'historyless gate must import its environment authority'
assert_grep 'focusa-$(id -u)-${VERSION}-learning-guards.json' scripts/create-dev-release-tag.sh 'release guard artifact must use a per-user path'
assert_grep 'FOCUSA_LEARNING_GUARDS_ARTIFACT' scripts/run-release-learning-guards.py 'release guard runner must consume the canonical artifact path'
assert_grep 'os.getuid()' scripts/run-release-learning-guards.py 'release guard fallback must isolate users'
assert_grep 'FOCUSA_LEARNING_GUARDS_ARTIFACT' scripts/canonical-release-journal.py 'release journal must consume the producing user artifact path'
assert_grep 'rm -rf -- "$TEST_GIT_DIR"' scripts/ci/run-spec-gates.sh 'disposable Git metadata must be removed on exit'
assert_grep 'FOCUSA_ROUTE_DRY_RUN=1 cargo --version' scripts/local-release-preflight.sh 'strict preflight must query the canonical cargo routing authority'
assert_grep '[[ "$CARGO_ROUTE" == route=ovh* ]]' scripts/local-release-preflight.sh 'strict preflight must detect the canonical OVH cargo route'
assert_grep 'FOCUSA_SOURCE_ROOT="$ROOT" /usr/local/bin/focusa-ovh-build' scripts/local-release-preflight.sh 'routed OVH spec gate must bind the exact release checkout'
assert_grep 'env -u CARGO_TARGET_DIR -u FOCUSA_CARGO_TARGET_DIR -u DAEMON_BIN' scripts/local-release-preflight.sh 'routed OVH spec gate must preserve ephemeral target isolation'
assert_grep 'FOCUSA_HISTORYLESS_GATE=1 bash scripts/ci/run-spec-gates.sh' scripts/local-release-preflight.sh 'routed OVH spec gate must explicitly classify its historyless source sync'
assert_grep 'bash scripts/ci/run-spec-gates.sh' scripts/local-release-preflight.sh 'strict preflight must preserve native runner execution'
cleanup_block="$(awk '/^cleanup\(\) \{/{capture=1} capture{print} capture && /^}/{exit}' scripts/ci/run-spec-gates.sh)"
grep -Fq 'cleanup_ephemeral_builds' <<<"$cleanup_block" || { echo '✗ strict spec combined EXIT cleanup missing'; exit 1; }

# Self-hosted AlmaLinux cannot provision actions/setup-python 3.13 (#388).
docs_workflow=.github/workflows/spec152-documentation-consistency.yml
assert_not_grep 'actions/setup-python' "$docs_workflow" 'Spec 152 docs workflow must use the installed self-hosted Python runtime'
assert_grep "sys.version_info >= (3, 12)" "$docs_workflow" 'Spec 152 docs workflow must enforce its Python runtime floor'
assert_grep 'python3 tests/spec152_documentation_consistency_gate.py' "$docs_workflow" 'Spec 152 documentation gate command missing'

# install-daemon.sh is compatibility-only; Rust owns lifecycle mutation.
assert_grep 'exec "$BOOTSTRAP"' scripts/install-daemon.sh 'compatibility adapter must delegate exactly once'
assert_grep --system-install scripts/install-daemon.sh 'compatibility adapter must request full system install'
assert_grep 'scripts/install-focusa.sh' scripts/install-daemon.sh 'verified Rust bootstrap delegation missing'
assert_grep 'FOCUSA_RELEASE_TAG="$TAG"' scripts/install-daemon.sh 'exact immutable release binding missing'
assert_grep '--no-verify is unsupported' scripts/install-daemon.sh 'verification bypass must fail closed'
assert_not_grep 'pgrep -x' scripts/install-daemon.sh 'compatibility adapter must not inspect by process name'
assert_not_grep 'systemctl ' scripts/install-daemon.sh 'compatibility adapter must not mutate systemd'
assert_not_grep 'kill -TERM' scripts/install-daemon.sh 'compatibility adapter must not signal daemon processes'
assert_grep 'prepare_system_service' crates/focusa-cli/src/commands/install.rs 'Rust installer must own system service transaction'
assert_grep 'acquire_system_deploy_lock' crates/focusa-cli/src/commands/install.rs 'Rust installer deploy lock missing'
assert_grep 'RefuseManualStart=yes' crates/focusa-cli/src/commands/system_service_process.rs 'operator halt must fail closed'
assert_grep 'unmanaged focusa-daemon process' crates/focusa-cli/src/commands/system_service_process.rs 'unmanaged process rejection missing'
assert_grep 'canonical system service must own exactly one' crates/focusa-cli/src/commands/system_service_process.rs 'exact process invariant missing'
assert_grep 'Environment=FOCUSA_HOME={}' crates/focusa-cli/src/commands/system_service.rs 'canonical state-root binding missing'
assert_grep 'Environment=FOCUSA_DATA_DIR={}' crates/focusa-cli/src/commands/system_service.rs 'canonical data-root binding missing'
assert_grep 'automatic system service rollback failed' crates/focusa-cli/src/commands/system_service.rs 'service rollback evidence missing'
assert_grep 'canonical daemon health verification failed' crates/focusa-cli/src/commands/system_service.rs 'health/version rollback gate missing'
assert_grep 'prepare_distribution_manifest' crates/focusa-cli/src/commands/install.rs 'installed distribution manifest is outside the Rust rollback boundary'
assert_grep 'distribution-manifest.json' crates/focusa-cli/src/commands/system_service_manifest.rs 'canonical system state omits distribution manifest parity'
assert_grep 'dist/distribution-manifest.json' .github/workflows/release.yml 'release omits the signed distribution manifest asset'
assert_grep 'distribution-manifest.json' scripts/verify-canonical-release-assets.py 'canonical release matrix does not require the distribution manifest'
assert_grep '--candidate' .github/workflows/release.yml 'initial release manifest must remain candidate-only before deployment'
assert_grep '--latest=false' .github/workflows/release.yml 'candidate publication must not change GitHub Latest'
assert_grep 'Promote accepted stable release to Latest' .github/workflows/deploy-live-daemon.yml 'stable/Latest promotion is not downstream of installed and OTA proof'
assert_grep '--distribution-parity' scripts/release-deploy-proof.py 'settled release manifest lacks installed parity binding'
assert_grep 'execute_manifest_bound_apply' crates/focusa-cli/src/commands/update.rs 'v0.9.188+ OTA does not reuse the canonical install lifecycle'
assert_grep 'crate::commands::install::run(args).await' crates/focusa-cli/src/commands/update.rs 'OTA retains a parallel promotion implementation for manifest-bound releases'
assert_grep '("session_runner", "focusa-session-runner")' crates/focusa-cli/src/commands/update.rs 'OTA release inventory omits the fourth canonical binary'
assert_grep 'exact_release_reinstall' crates/focusa-cli/src/commands/update.rs 'manifest-bound OTA does not retain an exact full-release rollback route'
assert_grep 'manifest-bound updates roll back as one full release' crates/focusa-cli/src/commands/update.rs 'part-only rollback can recreate a mixed installed runtime'
proof_line="$(grep -n -m1 'name: Publish signed deploy-success proof' .github/workflows/deploy-live-daemon.yml | cut -d: -f1)"
ota_line="$(grep -n -m1 'name: Gate OTA installability against signed deployed release' .github/workflows/deploy-live-daemon.yml | cut -d: -f1)"
settlement_line="$(grep -n -m1 'name: Settle signed release manifest after OTA acceptance' .github/workflows/deploy-live-daemon.yml | cut -d: -f1)"
promotion_line="$(grep -n -m1 'name: Promote accepted stable release to Latest' .github/workflows/deploy-live-daemon.yml | cut -d: -f1)"
if ! (( proof_line < ota_line && ota_line < settlement_line && settlement_line < promotion_line )); then
  echo '✗ stable promotion must follow signed installed parity and OTA acceptance'
  exit 1
fi

# safe-disk-cleanup.sh assertions
assert_grep 'target' scripts/safe-disk-cleanup.sh 'target cleanup missing'
assert_grep '/tmp/focusa-release-' scripts/safe-disk-cleanup.sh 'temp cleanup missing'
assert_grep 'MIN_FREE_GB' scripts/safe-disk-cleanup.sh 'disk threshold guard missing'
assert_grep 'BACKUP_KEEP' scripts/safe-disk-cleanup.sh 'backup keep bound missing'
assert_grep 'backup_keep=${{ steps.cfg.outputs.backup_keep }}' .github/workflows/deploy-live-daemon.yml 'workflow backup_keep wiring missing'

# Canonical Rust lifecycle and deployment topology.
assert_grep 'ExecStart={}' crates/focusa-cli/src/commands/system_service.rs 'Rust unit renderer must own ExecStart'
assert_grep 'MemoryHigh=2G' crates/focusa-cli/src/commands/system_service.rs 'outer memory pressure boundary missing'
assert_grep 'MemoryMax=3G' crates/focusa-cli/src/commands/system_service.rs 'outer memory kill boundary missing'
assert_grep 'x86_64-unknown-linux-musl' .github/workflows/deploy-live-daemon.yml 'musl default suffix missing (AlmaLinux 8 glibc)'
assert_not_grep '/usr/bin/kill' scripts/install-self-hosted-runner.sh 'runner must not receive direct process-kill authority'
assert_not_grep '/usr/bin/sed' scripts/install-self-hosted-runner.sh 'runner must not patch canonical service files outside Rust'

# Retry automation remains quarantined; lifecycle readiness is bounded in Rust.
assert_grep 'for _ in 0..40' crates/focusa-cli/src/commands/system_service.rs 'bounded daemon readiness gate missing'
assert_not_grep '  workflow_run:' .github/workflows/auto-retry-deploy.yml 'quarantined auto-retry must not retain automatic workflow_run authority'
assert_grep 'status=quarantined' .github/workflows/auto-retry-deploy.yml 'auto-retry quarantine boundary missing'

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
assert_grep 'python3 scripts/release-gate.py' scripts/create-dev-release-tag.sh 'release helper must enforce significant-delta ReleaseGate'
assert_grep 'CI_TIMEOUT_SECS=1200' scripts/create-dev-release-tag.sh 'release helper wait cap must be 20 minutes, not 1 hour+'
assert_grep 'headBranch' scripts/create-dev-release-tag.sh 'release helper must track tag Release run, not main branch validation run'
assert_grep 'wait_for_workflow "Release" "$HEAD_SHA" "${TAG}"' scripts/create-dev-release-tag.sh 'release helper must wait for tag-specific Release run'
assert_grep 'apps/pi-extension/package.json apps/pi-extension/package-lock.json' scripts/create-dev-release-tag.sh 'release helper must commit and dry-run-revert stamped Pi extension versions'
[[ "$(grep -o 'apps/pi-extension/package.json apps/pi-extension/package-lock.json' scripts/create-dev-release-tag.sh | wc -l)" -ge 2 ]] \
  || fail 'Pi extension version surfaces must appear in both commit and dry-run rollback sets'
manifest_surface='docs/contracts/spec141/generated-capability-v2/distribution-manifest.json'
[[ "$(grep -o "$manifest_surface" scripts/create-dev-release-tag.sh | wc -l)" -eq 3 ]] \
  || fail 'Distribution manifest must appear in retry allowlist, dry-run rollback, and release commit sets'
assert_grep 'timeout-minutes: 150' .github/workflows/release.yml 'External Menubar receipt gate timeout must cover the bounded 145-minute Codemagic/AppVeyor wait'
assert_grep 'timeout-minutes: 30' .github/workflows/release.yml 'Release Windows/cross-target job timeout must be enough but bounded'
rust_check_block="$(awk '/^  rust-check:/{job=1} /^  tag-ci-proof:/{job=0} job{print}' .github/workflows/release.yml)"
grep -q 'timeout-minutes: 25' <<<"$rust_check_block" || {
  echo '✗ Release Contract Check timeout must cover its bounded 20-minute candidate-CI polling window' >&2
  exit 1
}
final_gap_block="$(awk '/^  final-release-gap-gate:/{job=1} /^  version-policy:/{job=0} job{print}' .github/workflows/release.yml)"
grep -q 'unset NODE_OPTIONS' <<<"$final_gap_block" || {
  echo '✗ Final release gap gate must sanitize incompatible ambient Node options (GH#350)' >&2
  exit 1
}
for target in x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-unknown-linux-musl; do
  cache="/home/wirebot/.cache/focusa-release-target/${target}"
  assert_grep "$cache" .github/workflows/warmup.yml "warmup target cache is not writable and ABI-partitioned: ${target}"
  assert_grep "target_dir: ${cache}" .github/workflows/release.yml "release target cache does not reuse the ABI partition: ${target}"
done
assert_grep 'Release workflow validation' .github/workflows/release.yml 'release workflow needs unconditional validation step to avoid No jobs were run'
assert_grep "- 'v*'" .github/workflows/release.yml 'release workflow must trigger for immutable stable and preview tags'
assert_grep 'scripts/verify-release-tag-trigger.py' scripts/create-dev-release-tag.sh 'release helper must verify trigger compatibility before immutable tagging'
tests/release_candidate_main_push_retry_test.sh
assert_grep 'release_tag_validation=ok' .github/workflows/release.yml 'release tag validation step missing'
assert_grep 'needs: checksums' .github/workflows/release.yml 'deploy dispatch must depend on the actual checksums job id'
if grep -A3 '^  rust-check:' .github/workflows/release.yml | grep -q 'if:'; then
  echo '✗ rust-check job must be unconditional to avoid No jobs were run' >&2
  exit 1
fi
assert_grep "startsWith(github.ref, 'refs/tags/')" .github/workflows/release.yml 'release expensive jobs must use canonical tag gate expression'
assert_grep 'Deploy Live Daemon' scripts/create-dev-release-tag.sh 'create-dev-release-tag does not wait for deploy workflow'


# Beads ownership guard: bd sync/daemon must not rewrite project JSONL as root.
[[ -f tests/bd_sync_ownership_policy_test.sh ]] || { echo "✗ missing bd sync ownership policy test"; exit 1; }
tests/bd_sync_ownership_policy_test.sh

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


# Audit Recorder must persist DRY classifier output, not only raw workflow metadata.
assert_grep 'FAILED_LOG_PATH' .github/workflows/audit-recorder.yml 'audit-recorder must capture failed logs for classification'
assert_grep 'gh run view "$RUN_ID"' .github/workflows/audit-recorder.yml 'audit-recorder must download failed workflow logs'
assert_grep 'load_failure_classification' scripts/record-workflow-failure.py 'workflow failure recorder must load classifier output'
assert_grep 'failure_class' scripts/record-workflow-failure.py 'workflow failure recorder must persist failure_class'
assert_grep 'source_refs' scripts/record-workflow-failure.py 'workflow failure recorder must persist source refs'
assert_grep 'remediation_template' scripts/record-workflow-failure.py 'workflow failure recorder must persist remediation template'

# changelog generator
assert_grep 'changelog-gen.py' scripts/changelog-gen.py 'changelog gen must self-reference'
assert_grep 'CATEGORIES_BY_LAYER' scripts/changelog-gen.py 'changelog gen missing layer grouping'
assert_grep 'Layer 1 — Runner' scripts/changelog-gen.py 'changelog gen missing runner layer'


# Changelog/report views must expose classifier summaries from audit rows.
assert_grep 'failure_class = row.get("failure_class"' scripts/changelog-gen.py 'changelog must read classifier failure class'
assert_grep 'source refs:' scripts/changelog-gen.py 'changelog must render classifier source refs'
assert_grep 'Failure classes' scripts/changelog-gen.py 'changelog must render failure-class counts'
changelog_ledger="$(mktemp)"
changelog_out="$(mktemp)"
cat > "$changelog_ledger" <<'JSONL'
{"id":"fail-sample","ts":"2026-07-05T13:20:36Z","event":"failure","subsystem":"ci","scope":"CI","category":"ci_workflow_failure","symptom":"Blocked: clippy found a deterministic code issue.","root_cause":"clippy lint failure","fix":"Patch lint violations or narrow code changes.","guard":"scripts/classify-ci-failure.py","test":"scripts/classify-ci-failure.py","linked_run":"123","failure_class":"ci_clippy_failure","retry_policy":"hard_failure_no_rerun","source_refs":["crates/focusa-api/src/routes/project.rs:1650:13"],"remediation_template":"Patch lint violations or narrow code changes; do not rerun unchanged CI."}
JSONL
python3 scripts/changelog-gen.py --ledger "$changelog_ledger" --out "$changelog_out" --tag test >/dev/null
assert_grep 'classifier: `ci_clippy_failure`; retry: `hard_failure_no_rerun`' "$changelog_out" 'changelog output missing classifier summary'
assert_grep 'source refs: `crates/focusa-api/src/routes/project.rs:1650:13`' "$changelog_out" 'changelog output missing classifier source refs'
assert_grep 'ci_clippy_failure: 1' "$changelog_out" 'changelog output missing failure-class count'
rm -f "$changelog_ledger" "$changelog_out"



# Fixture suite locks classifier behavior across core self-heal failure classes.
[[ -f tests/self_heal_classifier_fixture_test.py ]] || { echo "✗ missing classifier fixture test"; exit 1; }
[[ -d tests/fixtures/self-heal-classifier ]] || { echo "✗ missing classifier fixtures"; exit 1; }
python3 tests/self_heal_classifier_fixture_test.py




# Uniform self-heal decision summary blocks for workflow operator visibility.
[[ -f scripts/render-self-heal-decision-summary.sh ]] || { echo "✗ missing self-heal decision summary renderer"; exit 1; }
assert_grep 'Self-heal decision' scripts/render-self-heal-decision-summary.sh 'summary renderer must emit decision heading'
assert_grep 'repair_required_no_rerun' scripts/render-self-heal-decision-summary.sh 'summary renderer must emit deterministic no-rerun decision'
assert_grep 'rerun_once_allowed' scripts/render-self-heal-decision-summary.sh 'summary renderer must emit transient rerun decision'
assert_grep 'status=quarantined' .github/workflows/auto-retry-deploy.yml 'quarantined auto heal must explain its non-mutation boundary'
assert_grep 'render-self-heal-decision-summary.sh' .github/workflows/release-pipeline-watchdog.yml 'governed watchdog must call summary renderer'
summary_tmp="$(mktemp)"
GITHUB_STEP_SUMMARY="$summary_tmp" SELF_HEAL_SURFACE="test" SELF_HEAL_WORKFLOW="CI" SELF_HEAL_RUN_ID="123" SELF_HEAL_HEAD_SHA="abc" failure_class="ci_clippy_failure" retry_policy="hard_failure_no_rerun" deterministic="true" safe_to_rerun_unchanged="false" plain_language_error="blocked" likely_root_cause="lint" remediation_template="patch it" source_refs="crates/example.rs:1:1" signals="clippy" bash scripts/render-self-heal-decision-summary.sh
assert_grep 'decision | `repair_required_no_rerun`' "$summary_tmp" 'summary renderer output missing deterministic decision'
GITHUB_STEP_SUMMARY="$summary_tmp" retry_policy="rerun_once" bash scripts/render-self-heal-decision-summary.sh
assert_grep 'decision | `rerun_once_allowed`' "$summary_tmp" 'summary renderer output missing transient decision'
rm -f "$summary_tmp"

# Deploy self-heal proof drill: non-mutating deploy-health retry vs deterministic stop proof.
[[ -f scripts/deploy-self-heal-proof-drill.py ]] || { echo "✗ missing deploy self-heal proof drill"; exit 1; }
[[ -f .github/workflows/deploy-self-heal-proof-drill.yml ]] || { echo "✗ missing deploy self-heal proof workflow"; exit 1; }
assert_grep 'workflow_dispatch:' .github/workflows/deploy-self-heal-proof-drill.yml 'deploy self-heal proof must be manual only'
assert_grep 'deploy_health_failure' scripts/deploy-self-heal-proof-drill.py 'deploy drill must exercise deploy health failure'
assert_grep 'auto_heal_process_error' scripts/deploy-self-heal-proof-drill.py 'deploy drill must exercise deterministic stop class'
deploy_drill_json="$(mktemp)"
python3 scripts/deploy-self-heal-proof-drill.py --health-url skip --json > "$deploy_drill_json"
python3 - "$deploy_drill_json" <<'PY'
import json, sys
payload = json.load(open(sys.argv[1]))
assert payload["schema"] == "focusa.deploy_self_heal_proof_drill.v1", payload
assert payload["failure_rows"] == 2, payload
assert 0 <= payload["self_heal_rows"] <= payload["failure_rows"], payload
assert payload["deploy_health_decision"]["decision"] == "rerun_once_allowed", payload
assert payload["deterministic_decision"]["decision"] == "repair_required_no_rerun", payload
assert payload["health"]["checked"] is False, payload
PY
rm -f "$deploy_drill_json"

# Safe self-heal failure-injection drill: proves stop-vs-rerun decisions without production mutation.
[[ -f scripts/self-heal-decision-drill.py ]] || { echo "✗ missing self-heal decision drill"; exit 1; }
[[ -f .github/workflows/self-heal-failure-injection.yml ]] || { echo "✗ missing self-heal failure injection workflow"; exit 1; }
assert_grep 'workflow_dispatch:' .github/workflows/self-heal-failure-injection.yml 'failure injection drill must be manual only'
assert_grep 'scripts/self-heal-decision-drill.py' .github/workflows/self-heal-failure-injection.yml 'failure injection workflow must call dry-run drill'
assert_grep 'repair_required_no_rerun' scripts/self-heal-decision-drill.py 'drill must prove deterministic no-rerun decision'
assert_grep 'rerun_once_allowed' scripts/self-heal-decision-drill.py 'drill must prove transient rerun-once decision'
drill_json="$(mktemp)"
tracked_self_heal_result=release-proof/audit/self-heal-result.json
tracked_self_heal_before="$(sha256sum "$tracked_self_heal_result" | awk '{print $1}')"
python3 scripts/self-heal-decision-drill.py --fixture all --json > "$drill_json"
tracked_self_heal_after="$(sha256sum "$tracked_self_heal_result" | awk '{print $1}')"
[[ "$tracked_self_heal_before" == "$tracked_self_heal_after" ]] \
  || fail 'Self-heal failure injection drill must not rewrite tracked release proof'
python3 - "$drill_json" <<'PY'
import json, sys
payload = json.load(open(sys.argv[1]))
assert payload["schema"] == "focusa.self_heal_failure_injection_drill.v1", payload
assert payload["case_count"] >= 9, payload
assert payload["failure_rows"] == payload["case_count"], payload
assert 0 <= payload["self_heal_rows"] <= payload["failure_rows"], payload
decisions = {case["decision"]["decision"] for case in payload["cases"]}
assert "repair_required_no_rerun" in decisions, payload
assert "rerun_once_allowed" in decisions, payload
PY
rm -f "$drill_json"


# Self-heal telemetry: repeated classes, repair-needed, stale unhealed failures.
[[ -f scripts/self-heal-telemetry.py ]] || { echo "✗ missing self-heal telemetry script"; exit 1; }
assert_grep 'repeated_classes' scripts/self-heal-telemetry.py 'telemetry must report repeated classes'
assert_grep 'open_repair_needed' scripts/self-heal-telemetry.py 'telemetry must report repair-needed failures'
assert_grep 'stale_unhealed_failures' scripts/self-heal-telemetry.py 'telemetry must report stale unhealed failures'
telemetry_ledger="$(mktemp)"
old_ts="2000-01-01T00:00:00Z"
cat > "$telemetry_ledger" <<JSONL
{"id":"fail-a","ts":"$old_ts","event":"failure","subsystem":"ci","scope":"CI","category":"ci_workflow_failure","symptom":"clippy deterministic","root_cause":"clippy","fix":"patch","guard":"scripts/classify-ci-failure.py","test":"scripts/classify-ci-failure.py","linked_run":"1","failure_class":"ci_clippy_failure","retry_policy":"hard_failure_no_rerun","deterministic":true}
{"id":"fail-b","ts":"$old_ts","event":"failure","subsystem":"ci","scope":"CI","category":"ci_workflow_failure","symptom":"clippy deterministic","root_cause":"clippy","fix":"patch","guard":"scripts/classify-ci-failure.py","test":"scripts/classify-ci-failure.py","linked_run":"2","failure_class":"ci_clippy_failure","retry_policy":"hard_failure_no_rerun","deterministic":true}
{"id":"fail-c","ts":"$old_ts","event":"failure","subsystem":"release","scope":"Release","category":"ci_workflow_failure","symptom":"network","root_cause":"network","fix":"rerun","guard":"scripts/classify-ci-failure.py","test":"scripts/classify-ci-failure.py","linked_run":"3","failure_class":"transient_github_or_network_failure","retry_policy":"rerun_once","deterministic":false}
{"id":"fail-d","ts":"$old_ts","event":"failure","subsystem":"release","scope":"Release","category":"ci_workflow_failure","symptom":"historical unknown","root_cause":"see logs","fix":"rerun","guard":"scripts/classify-ci-failure.py","test":"scripts/classify-ci-failure.py","linked_run":"4"}
{"id":"add-backfill-classifier-fail-d","ts":"2026-07-05T00:00:00Z","event":"addition","subsystem":"audit","scope":"release-proof/audit/audit.jsonl","category":"self_heal","change":"Backfilled classifier fields for historical failure fail-d","derived_from":"fail-d","classifier_schema":"focusa.release_failure_classifier.v1","failure_class":"unknown_process_failure","retry_policy":"rerun_once","deterministic":false,"safe_to_rerun_unchanged":true,"source_refs":[],"remediation_template":"Retry once.","classifier_signals":["historical_failure_row"]}
{"ts":"2026-07-05T00:00:00Z","event":"self_heal","subsystem":"ops","scope":"CI","category":"ci_workflow_failure","derived_from":"fail-a","symptom":"clippy deterministic","root_cause":"clippy","fix":"patch","guard":"scripts/classify-ci-failure.py","test":"scripts/classify-ci-failure.py","linked_run":"1","auto_generated":true}
JSONL
telemetry_json="$(mktemp)"
python3 scripts/self-heal-telemetry.py --audit "$telemetry_ledger" --stale-hours 1 --json > "$telemetry_json"
python3 - "$telemetry_json" <<'PY'
import json, sys
payload = json.load(open(sys.argv[1]))
assert payload["schema"] == "focusa.self_heal_telemetry.v1", payload
assert payload["class_counts"]["ci_clippy_failure"] == 2, payload
assert payload["retry_policy_counts"]["hard_failure_no_rerun"] == 2, payload
assert payload["class_counts"]["unknown_process_failure"] == 1, payload
assert payload["repeated_classes"]["ci_clippy_failure"] == 2, payload
assert set(payload["open_repair_needed"]) == {"fail-a", "fail-b"}, payload
assert set(payload["stale_unhealed_failures"]) == {"fail-b", "fail-c", "fail-d"}, payload
assert payload["latest_heal_ts"] == "2026-07-05T00:00:00Z", payload
PY
rm -f "$telemetry_ledger" "$telemetry_json"

# Audit failure-class triage summary
[[ -f scripts/audit-failure-summary.py ]] || { echo "✗ missing audit failure summary script"; exit 1; }
assert_grep 'failure_classes' scripts/audit-failure-summary.py 'audit summary must count failure classes'
assert_grep 'retry_policies' scripts/audit-failure-summary.py 'audit summary must count retry policies'
assert_grep 'source_refs' scripts/audit-failure-summary.py 'audit summary must display source refs'
assert_grep 'audit-failure-summary.py --limit 10' docs/deploy-runbook.md 'deploy runbook must document audit triage limit command'
assert_grep 'audit-failure-summary.py --class ci_clippy_failure --limit 5 --json' docs/deploy-runbook.md 'deploy runbook must document audit triage JSON command'
assert_grep 'Audit failure triage CLI' docs/self-heal-chain.md 'self-heal docs must expose audit triage CLI'
summary_ledger="$(mktemp)"
cat > "$summary_ledger" <<'JSONL'
{"id":"fail-a","ts":"2026-07-05T13:20:36Z","event":"failure","subsystem":"ci","scope":"CI","category":"ci_workflow_failure","symptom":"clippy deterministic","root_cause":"clippy lint failure","fix":"Patch lint","guard":"scripts/classify-ci-failure.py","test":"scripts/classify-ci-failure.py","linked_run":"123","failure_class":"ci_clippy_failure","retry_policy":"hard_failure_no_rerun","source_refs":["crates/focusa-api/src/routes/project.rs:1650:13"],"remediation_template":"Patch lint violations; do not rerun unchanged CI.","log_url":"https://github.com/example/actions/runs/123"}
{"id":"fail-b","ts":"2026-07-05T13:10:36Z","event":"failure","subsystem":"ci","scope":"CI","category":"ci_workflow_failure","symptom":"transient upload","root_cause":"network","fix":"rerun once","guard":"scripts/classify-ci-failure.py","test":"scripts/classify-ci-failure.py","linked_run":"122","failure_class":"transient_github_or_network_failure","retry_policy":"rerun_once","source_refs":".github/workflows/release.yml:1:1,.github/workflows/ci.yml:1:1","remediation_template":"Rerun once."}
JSONL
summary_out="$(python3 scripts/audit-failure-summary.py --audit "$summary_ledger" --class ci_clippy_failure --limit 5)"
grep -q 'ci_clippy_failure: 1' <<<"$summary_out" || { echo "✗ audit summary missing class count" >&2; exit 1; }
grep -q 'hard_failure_no_rerun: 1' <<<"$summary_out" || { echo "✗ audit summary missing retry count" >&2; exit 1; }
grep -q 'crates/focusa-api/src/routes/project.rs:1650:13' <<<"$summary_out" || { echo "✗ audit summary missing source ref" >&2; exit 1; }
transient_out="$(python3 scripts/audit-failure-summary.py --audit "$summary_ledger" --class transient_github_or_network_failure --limit 1)"
grep -q '.github/workflows/release.yml:1:1' <<<"$transient_out" || { echo "✗ audit summary missing string source ref" >&2; exit 1; }
python3 scripts/audit-failure-summary.py --audit "$summary_ledger" --json >/tmp/focusa-audit-summary.json
python3 - /tmp/focusa-audit-summary.json <<'PY'
import json, sys
payload = json.load(open(sys.argv[1]))
assert payload["failure_classes"]["ci_clippy_failure"] == 1, payload
assert payload["failure_classes"]["transient_github_or_network_failure"] == 1, payload
assert payload["retry_policies"]["hard_failure_no_rerun"] == 1, payload
PY
rm -f "$summary_ledger" /tmp/focusa-audit-summary.json


# Append-only classifier backfill for historical audit failures.
[[ -f scripts/backfill-audit-classifier-fields.py ]] || { echo "✗ missing classifier backfill script"; exit 1; }
assert_grep 'add-backfill-classifier' scripts/backfill-audit-classifier-fields.py 'backfill must append explicit addition rows'
assert_grep '--dry-run' scripts/backfill-audit-classifier-fields.py 'backfill must support dry-run'
assert_grep '--apply' scripts/backfill-audit-classifier-fields.py 'backfill must support apply'
assert_grep 'Historical classifier backfill' docs/self-heal-chain.md 'self-heal docs must document backfill'
backfill_ledger="$(mktemp)"
cat > "$backfill_ledger" <<'JSONL'
{"id":"fail-hist-a","ts":"2026-07-05T13:20:36Z","event":"failure","subsystem":"ci","scope":"CI","category":"ci_workflow_failure","symptom":"clippy deterministic","root_cause":"clippy lint failure","fix":"Patch lint","guard":"scripts/classify-ci-failure.py","test":"scripts/classify-ci-failure.py","linked_run":"123"}
{"id":"fail-hist-b","ts":"2026-07-05T13:21:36Z","event":"failure","subsystem":"ci","scope":"CI","category":"ci_workflow_failure","symptom":"CI run concluded failure","root_cause":"see workflow logs","fix":"open","guard":"scripts/classify-ci-failure.py","test":"scripts/classify-ci-failure.py","linked_run":"124"}
JSONL
before_hash="$(sha256sum "$backfill_ledger" | awk '{print $1}')"
python3 scripts/backfill-audit-classifier-fields.py --audit "$backfill_ledger" --dry-run --json >/tmp/focusa-backfill-dry.json
after_hash="$(sha256sum "$backfill_ledger" | awk '{print $1}')"
[[ "$before_hash" == "$after_hash" ]] || { echo "✗ backfill dry-run mutated ledger" >&2; exit 1; }
python3 - /tmp/focusa-backfill-dry.json <<'PY'
import json, sys
payload = json.load(open(sys.argv[1]))
assert payload["schema"] == "focusa.audit_classifier_backfill.v1", payload
assert payload["candidate_count"] == 2, payload
assert payload["append_count"] == 0, payload
assert payload["failure_classes"]["ci_clippy_failure"] == 1, payload
PY
python3 scripts/backfill-audit-classifier-fields.py --audit "$backfill_ledger" --apply --json >/tmp/focusa-backfill-apply.json
python3 scripts/audit-schema.py validate "$backfill_ledger" >/tmp/focusa-backfill-schema.out
python3 scripts/audit-failure-summary.py --audit "$backfill_ledger" --class ci_clippy_failure --limit 5 --json >/tmp/focusa-backfill-summary.json
python3 - /tmp/focusa-backfill-apply.json /tmp/focusa-backfill-summary.json <<'PY'
import json, sys
applied = json.load(open(sys.argv[1]))
summary = json.load(open(sys.argv[2]))
assert applied["append_count"] == 2, applied
assert summary["count"] == 1, summary
row = summary["failures"][0]
assert row["classifier_schema"] == "focusa.release_failure_classifier.v1", row
assert row["failure_class"] == "ci_clippy_failure", row
assert row["retry_policy"] == "hard_failure_no_rerun", row
assert row["deterministic"] is True, row
assert row["safe_to_rerun_unchanged"] is False, row
assert row["remediation_template"], row
PY
rm -f "$backfill_ledger" /tmp/focusa-backfill-dry.json /tmp/focusa-backfill-apply.json /tmp/focusa-backfill-summary.json /tmp/focusa-backfill-schema.out


# install-daemon contract spec
assert_grep 'sole owner' docs/install-daemon-contract.md 'contract missing canonical Rust owner'
assert_grep 'RefuseManualStart=yes' docs/install-daemon-contract.md 'contract missing operator halt boundary'
assert_grep 'exactly one systemd-owned daemon' docs/install-daemon-contract.md 'contract missing exact process invariant'
assert_grep 'restores the prior binaries and unit' docs/install-daemon-contract.md 'contract missing transaction rollback'

# operator runbook
assert_grep 'GitHub Actions is down' docs/deploy-runbook.md 'runbook must cover GitHub outage'
assert_grep 'Runner token is expired' docs/deploy-runbook.md 'runbook must cover token expiry'
assert_grep 'Audit trail fails to validate' docs/deploy-runbook.md 'runbook must cover audit validation'

# cross-links
assert_grep 'self-heal-chain.md' docs/production-deployment-guide.md 'prod guide missing self-heal link'
assert_grep 'deploy-runbook.md' docs/production-deployment-guide.md 'prod guide missing runbook link'

# DRY release-path failure classifier
[[ -f scripts/classify-ci-failure.py ]] || { echo "✗ missing DRY classifier"; exit 1; }
assert_grep 'focusa.release_failure_classification.v1' scripts/classify-ci-failure.py 'classifier schema missing'
assert_grep 'rust_compile_api_drift' scripts/classify-ci-failure.py 'classifier missing API drift class'
assert_grep 'source_refs' scripts/classify-ci-failure.py 'classifier must emit source refs'
assert_grep 'scripts/classify-ci-failure.py' .github/scripts/classify-release-failure.sh 'release classifier wrapper must delegate to DRY classifier'
assert_grep 'status=quarantined' .github/workflows/auto-retry-deploy.yml 'quarantined auto retry must not classify or mutate'
assert_grep 'scripts/classify-ci-failure.py failed.log --format json' .github/workflows/release-pipeline-watchdog.yml 'governed watchdog must call the canonical structured classifier'
classifier_sample="$(mktemp)"
cat > "$classifier_sample" <<'LOG'
Rust	UNKNOWN STEP	2026-07-04T21:27:55Z error: 14 positional arguments in format string, but there are 13 arguments
Rust	UNKNOWN STEP	2026-07-04T21:27:55Z     --> crates/focusa-api/src/routes/project.rs:1998:14
Rust	UNKNOWN STEP	2026-07-04T21:27:28Z error[E0061]: this function takes 6 arguments but 4 arguments were supplied
Rust	UNKNOWN STEP	2026-07-04T21:27:28Z     --> crates/focusa-api/src/routes/project.rs:3444:17
LOG
classifier_json="$(python3 scripts/classify-ci-failure.py "$classifier_sample")"
python3 - "$classifier_json" <<'PY'
import json, sys
payload = json.loads(sys.argv[1])
assert payload["schema"] == "focusa.release_failure_classification.v1", payload
assert payload["failure_class"] == "rust_compile_api_drift", payload
assert payload["retry_policy"] == "hard_failure_no_rerun", payload
assert payload["deterministic"] is True, payload
assert "crates/focusa-api/src/routes/project.rs:1998:14" in payload["source_refs"], payload
PY
rm -f "$classifier_sample"

echo "Release deploy automation static test: PASS"
