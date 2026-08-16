#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source "$ROOT/tests/focusa_portable_bin.sh"
BIN="$(focusa_resolve_test_cli_binary "$ROOT")"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

if ! command -v rg >/dev/null 2>&1; then
  rg() { grep -E "$@"; }
fi

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

bash "$ROOT/tests/spec128_update_status_static_test.sh"
bash "$ROOT/tests/spec128_installer_preflight_static_test.sh"

preflight="$($BIN --json install --preflight --no-animation --quiet)"
jq -e '.schema=="focusa.install_preflight.v1" and .read_only==true and .mutations_performed==false and .dependency_install_offer.auto_install_performed==false and .dependency_install_offer.requires_explicit_consent==true' <<<"$preflight" >/dev/null || fail "installer preflight did not prove safe read-only dependency prompt posture"

mkdir -p "$TMP/bin"
cat > "$TMP/bin/focusa" <<'FAKE'
#!/usr/bin/env bash
echo 'focusa 0.9.74-dev'
FAKE
chmod +x "$TMP/bin/focusa"

status="$(FOCUSA_FOCUSA_PATH="$TMP/bin/focusa" "$BIN" --json update status --latest-version 0.9.80-dev)"
jq -e '.schema=="focusa.update_inventory.v1" and .read_only==true and (.parts[] | select(.part=="cli" and .version=="0.9.74-dev" and .stale==true))' <<<"$status" >/dev/null || fail "stale CLI detection failed"

policy_dev="$(FOCUSA_UPDATE_POLICY="$TMP/no-policy.json" FOCUSA_DEV_MODE=1 "$BIN" --json update policy show)"
jq -e '.schema=="focusa.update_policy_status.v1" and .policy.channel=="dev" and .policy.mode=="automatic" and .auto_apply_allowed==true and ([.policy.parts[]] | all)' <<<"$policy_dev" >/dev/null || fail "dev_mode policy did not authorize automatic all-surface updates"

policy_eval="$(FOCUSA_UPDATE_POLICY="$TMP/no-policy.json" FOCUSA_DEV_MODE=0 "$BIN" --json update policy show)"
jq -e '.schema=="focusa.update_policy_status.v1" and .policy.mode!="automatic" and .auto_apply_allowed==false' <<<"$policy_eval" >/dev/null || fail "eval/unattended auto-apply denial failed"

plan=""
for attempt in 1 2 3; do
  plan="$(FOCUSA_FOCUSA_PATH="$TMP/bin/focusa" "$BIN" --json update plan)"
  jq -e '.schema=="focusa.update_plan.v1" and .latest.trust.release_resolved==true' <<<"$plan" >/dev/null && break
  [[ "$attempt" == 3 ]] || sleep 2
done
# Before deployment, the newest release cannot possess deploy-success proof yet;
# both fully trusted and explicitly fail-closed states are valid CI outcomes.
jq -e '.schema=="focusa.update_plan.v1" and .latest.trust.release_resolved==true and .latest.trust.key_revoked==false and (.safety.staging.verify_before_promote | index("asset_sha256")) and (.safety.staging.verify_before_promote | index("release_manifest_signature")) and (.safety.no_half_written_executable_rule | test("never write directly")) and (((.latest.trust.signature_verified==true) and (.latest.trust.manifest_signature_verified==true) and (.latest.trust.provenance_verified==true) and (.latest.trust.deploy_proof_verified==true) and ((.latest.trust.blockers|length)==0)) or ((.latest.trust.signature_verified==false) and (.apply_blocked_until|index("release_signature_not_verified"))))' <<<"$plan" >/dev/null || fail "plan is neither fully trusted nor explicitly fail-closed after bounded retries"

apply_same="$(FOCUSA_FOCUSA_PATH="$TMP/bin/focusa" "$BIN" --json update apply --latest-version 0.9.80-dev)"
jq -e '.schema=="focusa.update_apply.v1" and .status=="blocked_read_only" and .apply_executed==false and .daemon_restart.allowed==false and .data_safety.overwrite_data==false and .data_safety.overwrite_env==false and .data_safety.overwrite_license==false' <<<"$apply_same" >/dev/null || fail "guarded apply failed no-mutation/data-safety assertions"

apply_daemon_changed="$($BIN --json update apply --latest-version 9.9.9-dev)"
jq -e '.schema=="focusa.update_apply.v1" and .apply_executed==false and .daemon_restart.allowed==false and .daemon_restart.health_proof=="GET /v1/health version and API contract must match target release"' <<<"$apply_daemon_changed" >/dev/null || fail "daemon restart proof/allowance guard missing"

rollback="$($BIN --json update rollback --part all)"
jq -e '.schema=="focusa.update_rollback.v1" and .rollback_executed==false and (.restore_order | index("health_contract_check")) and (.proof_required | index("history_event_written")) and .data_safety.overwrite_license==false' <<<"$rollback" >/dev/null || fail "rollback/history health proof guard missing"

history="$($BIN --json update history)"
jq -e '.schema=="focusa.update_history.v1" and .read_only==true and .retention.keep_last_successful_bundles==3 and (.observability.counters | index("update_apply_blocked_total"))' <<<"$history" >/dev/null || fail "history/observability envelope missing"

admin="$($BIN --json update admin --pause --force-check --skip-version 0.9.80-dev)"
jq -e '.schema=="focusa.update_admin_control.v1" and .read_only==true and .mutations_performed==false and (.requested_controls | index("pause")) and (.requested_controls | index("force_check")) and (.requested_controls | index("skip_version:0.9.80-dev"))' <<<"$admin" >/dev/null || fail "admin control preview missing requested controls"

admin_state="$TMP/update-admin.json"
admin_applied="$(FOCUSA_UPDATE_ADMIN_STATE="$admin_state" "$BIN" --json update admin --pause --force-check --pin-version 0.9.80-dev --skip-version 0.9.81-dev --dry-run=false --yes)"
jq -e '.status=="applied" and .read_only==false and .mutations_performed==true and .effective_state.paused==true and .effective_state.pinned_version=="0.9.80-dev" and (.effective_state.skipped_versions|index("0.9.81-dev"))' <<<"$admin_applied" >/dev/null || fail "admin controls did not persist"
admin_resumed="$(FOCUSA_UPDATE_ADMIN_STATE="$admin_state" "$BIN" --json update admin --resume --unpin --unskip-version 0.9.81-dev --dry-run=false --yes)"
jq -e '.status=="applied" and .effective_state.paused==false and .effective_state.pinned_version==null and (.effective_state.skipped_versions|length)==0' <<<"$admin_resumed" >/dev/null || fail "admin resume/unpin/unskip did not persist"

scheduler="$($BIN --json update scheduler)"
jq -e '.schema=="focusa.update_scheduler.v1" and (.scheduler_installed == .background_worker_started) and (.automatic_apply.allowed == .scheduler_installed) and .interval.jitter_percent==20 and .offline.skip_when_offline==true' <<<"$scheduler" >/dev/null || fail "scheduler/background updater policy missing"

notifications="$(FOCUSA_FOCUSA_PATH="$TMP/bin/focusa" "$BIN" --json update notifications --latest-version 0.9.80-dev)"
jq -e '.schema=="focusa.update_notifications.v1" and .read_only==true and .severity=="warning" and (.stale_parts | index("cli")) and .surfaces.cli==true and .surfaces.api==true and .surfaces.pi_doctor==true' <<<"$notifications" >/dev/null || fail "stale-surface notification proof missing"

# Static manifest/release safety strings cover the release-side block classes until network resolver is wired.
rg -q 'checksum|signature|revoked|yanked' "$ROOT/crates/focusa-core/src/update.rs" "$ROOT/docs/128-focusa-over-the-air-auto-update-and-dev-mode-license-spec.md" || fail "release checksum/signature/revoked/yanked block references missing"

pass "Spec128 installer/update runtime suite complete"
