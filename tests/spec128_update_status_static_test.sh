#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MAIN="$ROOT/crates/focusa-cli/src/main.rs"
MOD="$ROOT/crates/focusa-cli/src/commands/mod.rs"
SRC="$ROOT/crates/focusa-cli/src/commands/update.rs"
API="$ROOT/crates/focusa-api/src/routes/update.rs"
API_MOD="$ROOT/crates/focusa-api/src/routes/mod.rs"
API_SERVER="$ROOT/crates/focusa-api/src/server.rs"
SPEC="$ROOT/docs/128-focusa-over-the-air-auto-update-and-dev-mode-license-spec.md"
INSTALLER="$ROOT/scripts/install-focusa.sh"
RELEASE_WORKFLOW="$ROOT/.github/workflows/release.yml"

if ! command -v rg >/dev/null 2>&1; then
  rg() { grep -E "$@"; }
fi

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[[ -f "$SRC" ]] || fail "missing update command module"
rg -q 'pub mod update;' "$MOD" || fail "update module not exported"
rg -q 'Update\(commands::update::UpdateCmd\)' "$MAIN" || fail "main CLI missing Update command"
rg -q 'Signed OTA status, plan, apply, rollback, policy, scheduler, notification, and history' "$MAIN" || fail "main CLI still hides completed OTA functionality"
rg -q 'Commands::Update\(cmd\) => commands::update::run\(cmd, cli\.json\)' "$MAIN" || fail "main CLI missing update dispatch"
rg -q 'enum UpdateCmd' "$SRC" || fail "missing UpdateCmd enum"
if rg -q 'latest version placeholder until the release manifest resolver is wired|apply remains disabled until Spec128 apply gates are implemented|once Spec128 planning is implemented|no mutation until all gates are wired|future apply|unless future gates are wired|Still blocked until implementation gates pass' "$SRC"; then
  fail "completed OTA surface still advertises obsolete scaffold/disabled behavior"
fi
rg -q 'println!\("apply_allowed: \{\}", plan\.apply_allowed\)' "$SRC" \
  || fail "human update plan hard-codes apply_allowed instead of reporting resolved trust"
rg -q 'Status\(UpdateStatusArgs\)' "$SRC" || fail "missing update status subcommand"
rg -q 'Check\(UpdateStatusArgs\)' "$SRC" || fail "missing update check subcommand"
rg -q 'Plan\(UpdateStatusArgs\)|focusa.update_plan.v1|build_update_plan|CompatibilityPlan|PromptPlan' "$SRC" || fail "missing CLI update plan/compatibility/prompt envelope"
rg -q 'Apply\(UpdateApplyArgs\)|focusa.update_apply.v1|build_apply_envelope|apply_executed: false|apply_executor_not_enabled_in_spec128_07_scaffold' "$SRC" || fail "missing CLI guarded apply blocked/read-only envelope"
rg -q 'History\(UpdateHistoryArgs\)|Rollback\(UpdateRollbackArgs\)|Admin\(UpdateAdminArgs\)|focusa.update_history.v1|focusa.update_rollback.v1|focusa.update_admin_control.v1' "$SRC" || fail "missing CLI history/rollback/admin envelopes"
rg -q 'Scheduler\(UpdateSchedulerArgs\)|Notifications\(UpdateStatusArgs\)|focusa.update_scheduler.v1|focusa.update_notifications.v1|background_worker_started|offline_without_prior_success' "$SRC" || fail "missing CLI scheduler/notification envelopes"
rg -q 'UpdateSafetyPlan|LockPlan|StagingPlan|AtomicInstallPlan|RecoveryPlan|update.lock|update-journal.json|no_half_written_executable_rule' "$SRC" || fail "missing CLI lock/staging/atomic/recovery safety plan"
rg -q 'read_only: true' "$SRC" || fail "update status must be read-only"
rg -q 'mutations_performed: false' "$SRC" || fail "update status must not mutate"
rg -q 'Policy\(UpdatePolicyCmd\)|UpdatePolicySetArgs|FOCUSA_UPDATE_POLICY|default_policy_from_license' "$SRC" || fail "missing CLI update policy show/set/defaults"
rg -q 'refresh_auto_apply_authority|automatic_apply_not_authorized_by_policy|auto_apply_allowed' "$SRC" || fail "CLI policy must derive automatic authority and enforce it for scheduler runs"
rg -q 'apply_allowed|apply_blocked_until' "$SRC" || fail "CLI plan must expose dynamic release/policy gates"
rg -q 'mutations_performed: false|read_only: true|dry_run_requested|explicit_yes_and_allow_apply_required' "$SRC" || fail "CLI apply must default to no mutation and explicit consent gates"
rg -q 'write_temp_fsync_rename_then_smoke_test|preserve_permissions_owner_xattrs_capabilities_when_supported|daemon binary is promoted last' "$SRC" || fail "atomic install safety rules missing"
rg -q 'rollback_executor_not_enabled_in_spec128_08_scaffold|update-history.jsonl|update_apply_blocked_total|trusted_dev_force_latest|focusa.update_admin_state.v1|mutation_requires_dry_run_false_and_yes|updates_paused_by_admin' "$SRC" || fail "rollback/history/admin control safety rules missing"
rg -q 'jitter_percent|max_silent_failures_before_notice|maintenance-window.json|policy_allows_automatic_apply|planned_when_tui_update_banner_available|planned_when_menubar_update_badge_available' "$SRC" || fail "scheduler/background notification rules missing"
rg -q 'focusa-daemon --version starts the server|--version as startup input' "$SRC" || fail "daemon version probe safety note missing"
rg -q 'release manifest eligibility/signature/provenance is required|automatic_apply_not_authorized_by_policy' "$SRC" || fail "missing automatic apply trust/policy guard"
rg -q 'cli|daemon|tui' "$SRC" || fail "inventory must include CLI/daemon/TUI"
[[ -f "$API" ]] || fail "missing update API route module"
rg -q 'pub mod update;' "$API_MOD" || fail "API routes module does not export update"
rg -q 'routes::update::router\(\)' "$API_SERVER" || fail "API server does not merge update router"
rg -q '/v1/update/status|/v1/update/check' "$API" || fail "missing update status/check API routes"
rg -q '/v1/update/plan|focusa.update_plan.v1|build_update_plan' "$API" || fail "missing update plan API route/envelope"
rg -q '/v1/update/apply|focusa.update_apply.v1|build_apply_envelope|apply_executor_not_enabled_in_spec128_07_scaffold' "$API" || fail "missing update apply API blocked/read-only envelope"
rg -q '/v1/update/history|/v1/update/rollback|/v1/update/admin|focusa.update_history.v1|focusa.update_rollback.v1|focusa.update_admin_control.v1' "$API" || fail "missing update history/rollback/admin API envelopes"
rg -q '/v1/update/scheduler|/v1/update/notifications|focusa.update_scheduler.v1|focusa.update_notifications.v1|background_worker_started' "$API" || fail "missing update scheduler/notification API envelopes"
rg -q 'update_scheduler_set|focusa.update_scheduler_mutation.v1|scheduler_cli_unavailable' "$API" || fail "missing scheduler enable/disable mutation route"
rg -q 'com.startempire.focusa-update|StartInterval|ThrottleInterval|focusa-update.timer|RandomizedDelaySec|updates_paused_by_admin' "$SRC" || fail "missing cross-platform scheduler cadence/backoff/single-policy markers"
rg -q 'FOCUSA_UPDATE_FAULT_AFTER_PROMOTE|rollback_promoted_parts|rollback_after_apply_failure|state":"rolled_back' "$SRC" || fail "missing interrupted-update rollback fault matrix hooks"
rg -q 'FOCUSA_UPDATE_INVENTORY_INTERVAL_SECONDS|fleet_truth_status|notification_required|blind_latest_allowed' "$API" || fail "missing continuous currency/drift policy envelope"
rg -q 'build_safety_plan_json|update.lock|update-journal.json|no_half_written_executable_rule|atomic_install' "$API" || fail "missing API lock/staging/atomic/recovery safety plan"
rg -q '/v1/update/policy|update_policy_set|default_policy_from_license' "$API" || fail "missing update policy API route/defaults"
rg -q 'read_only.*true|mutations_performed.*false' "$API" || fail "API inventory/plan routes must remain read-only"
rg -q 'refresh_auto_apply_authority|dev_mode|UpdatePolicyParts' "$API" || fail "API policy mutation must expose dev/all-surface automatic authority"
rg -q 'cli|daemon|tui' "$API" || fail "API inventory must include CLI/daemon/TUI"
rg -q 'focusa update status --json|focusa update check --channel dev --json|focusa update plan --tag latest --json|focusa update status' "$SPEC" || fail "Spec128 update CLI/plan surface missing"
rg -q 'dev_mode.*automatic|evaluation notify-only|update policy' "$SPEC" || fail "Spec128 policy/dev_mode requirements missing"
rg -q 'FOCUSA_INSTALLER_VERSION|--version' "$INSTALLER" || fail "installer lacks safe version surface"
rg -q 'focusa-installer-.*\.sh' "$RELEASE_WORKFLOW" || fail "release workflow lacks signed installer asset"
rg -q 'part: "installer"|release_asset_unavailable' "$SRC" || fail "updater lacks installer asset inventory/gate"

pass "Spec128 update inventory, guarded apply, mutable policy, and automatic authority surfaces present"
