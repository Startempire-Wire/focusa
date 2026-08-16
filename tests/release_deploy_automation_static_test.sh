#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "=== release deploy automation static test ==="

WORKFLOW_VALIDATION_OUT="$(mktemp /tmp/focusa-workflow-graph-validation.XXXXXX)"
trap 'rm -f "$WORKFLOW_VALIDATION_OUT"' EXIT
python3 scripts/validate-github-workflows.py .github/workflows/release.yml .github/workflows/auto-retry-deploy.yml .github/workflows/release-pipeline-watchdog.yml >"$WORKFLOW_VALIDATION_OUT" || {
  cat "$WORKFLOW_VALIDATION_OUT" >&2
  exit 1
}

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
assert_grep 'Auto Heal Release Pipeline' .github/workflows/auto-retry-deploy.yml 'auto-heal workflow name missing'

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
assert_grep 'timeout-minutes: 12' .github/workflows/release.yml 'Release macOS/Tauri job timeout must stay bounded'
assert_grep 'timeout-minutes: 15' .github/workflows/release.yml 'Release Windows/cross-target job timeout must be enough but bounded'
assert_grep 'Release workflow validation' .github/workflows/release.yml 'release workflow needs unconditional validation step to avoid No jobs were run'
assert_grep 'on: [push]' .github/workflows/release.yml 'release workflow must use canonical unfiltered push trigger to avoid no-job validation failures'
assert_grep 'release_branch_validation=ok' .github/workflows/release.yml 'release branch validation step missing'
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
assert_grep 'render-self-heal-decision-summary.sh' .github/workflows/auto-retry-deploy.yml 'auto heal must call summary renderer'
assert_grep 'render-self-heal-decision-summary.sh' .github/workflows/release-pipeline-watchdog.yml 'watchdog must call summary renderer'
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
python3 scripts/self-heal-decision-drill.py --fixture all --json > "$drill_json"
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
printf '%s
' "$summary_out" | grep -q 'ci_clippy_failure: 1' || { echo "✗ audit summary missing class count" >&2; exit 1; }
printf '%s
' "$summary_out" | grep -q 'hard_failure_no_rerun: 1' || { echo "✗ audit summary missing retry count" >&2; exit 1; }
printf '%s
' "$summary_out" | grep -q 'crates/focusa-api/src/routes/project.rs:1650:13' || { echo "✗ audit summary missing source ref" >&2; exit 1; }
transient_out="$(python3 scripts/audit-failure-summary.py --audit "$summary_ledger" --class transient_github_or_network_failure --limit 1)"
printf '%s
' "$transient_out" | grep -q '.github/workflows/release.yml:1:1' || { echo "✗ audit summary missing string source ref" >&2; exit 1; }
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

# DRY release-path failure classifier
[[ -f scripts/classify-ci-failure.py ]] || { echo "✗ missing DRY classifier"; exit 1; }
assert_grep 'focusa.release_failure_classification.v1' scripts/classify-ci-failure.py 'classifier schema missing'
assert_grep 'rust_compile_api_drift' scripts/classify-ci-failure.py 'classifier missing API drift class'
assert_grep 'source_refs' scripts/classify-ci-failure.py 'classifier must emit source refs'
assert_grep 'scripts/classify-ci-failure.py' .github/scripts/classify-release-failure.sh 'release classifier wrapper must delegate to DRY classifier'
assert_grep 'classify-release-failure.sh' .github/workflows/auto-retry-deploy.yml 'auto retry must call shared classifier wrapper'
assert_grep 'classify-release-failure.sh' .github/workflows/release-pipeline-watchdog.yml 'watchdog must call shared classifier wrapper'
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

for script in scripts/audit-distribution-parity.mjs scripts/release-readiness-evaluation.mjs; do
  [ -f "$script" ] || { echo "FAIL: missing $script" >&2; exit 1; }
done
echo "distribution parity + release readiness scripts present: PASS"

echo "Release deploy automation static test: PASS"
