//! Spec 128 read-only update inventory/status/check/plan/apply guard API routes.
//!
//! These routes intentionally do not mutate local state. They inventory local
//! Focusa surfaces and expose stale-part information for CLI/Pi/TUI/menubar
//! consumers before update planning/apply exists.

use axum::extract::Query;
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::license::load_license_status;
use focusa_core::update::{ReleaseChannel, UPDATE_POLICY_SCHEMA_V1, UpdateMode, UpdatePolicy};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

pub fn router() -> Router<Arc<crate::server::AppState>> {
    Router::new()
        .route("/v1/update/status", get(update_status))
        .route("/v1/update/check", get(update_check_get).post(update_check))
        .route("/v1/update/plan", get(update_plan_get).post(update_plan))
        .route("/v1/update/apply", post(update_apply))
        .route("/v1/update/history", get(update_history))
        .route("/v1/update/rollback", post(update_rollback))
        .route("/v1/update/admin", post(update_admin))
        .route("/v1/update/scheduler", get(update_scheduler))
        .route(
            "/v1/update/notifications",
            get(update_notifications_get).post(update_notifications),
        )
        .route(
            "/v1/update/policy",
            get(update_policy).post(update_policy_set),
        )
}

#[derive(Debug, Deserialize, Default)]
struct UpdateQuery {
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    latest_version: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct UpdateHistoryQuery {
    #[serde(default = "default_history_limit")]
    limit: usize,
}

fn default_history_limit() -> usize {
    20
}

#[derive(Debug, Deserialize, Default)]
struct UpdateRollbackBody {
    #[serde(default = "default_rollback_part")]
    part: String,
    #[serde(default = "default_true")]
    dry_run: bool,
    #[serde(default)]
    yes: bool,
}

fn default_rollback_part() -> String {
    "all".into()
}

#[derive(Debug, Deserialize, Default)]
struct UpdateAdminBody {
    #[serde(default)]
    pin_version: Option<String>,
    #[serde(default)]
    unpin: bool,
    #[serde(default)]
    skip_version: Option<String>,
    #[serde(default)]
    pause: bool,
    #[serde(default)]
    resume: bool,
    #[serde(default)]
    force_check: bool,
    #[serde(default)]
    trusted_dev_force_latest: bool,
    #[serde(default = "default_true")]
    dry_run: bool,
    #[serde(default)]
    yes: bool,
}

#[derive(Debug, Deserialize, Default)]
struct UpdateApplyBody {
    #[serde(flatten)]
    query: UpdateQuery,
    #[serde(default = "default_true")]
    dry_run: bool,
    #[serde(default)]
    yes: bool,
    #[serde(default)]
    allow_apply: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Default)]
struct UpdatePolicySetBody {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    mode: Option<String>,
}

async fn update_status(Query(query): Query<UpdateQuery>) -> Json<Value> {
    Json(build_update_inventory("status", query).await)
}

async fn update_check_get(Query(query): Query<UpdateQuery>) -> Json<Value> {
    Json(build_update_inventory("check", query).await)
}

async fn update_check(Json(body): Json<UpdateQuery>) -> Json<Value> {
    update_check_get(Query(body)).await
}

async fn update_plan_get(Query(query): Query<UpdateQuery>) -> Json<Value> {
    let inventory = build_update_inventory("plan", query).await;
    Json(build_update_plan(inventory))
}

async fn update_plan(Json(body): Json<UpdateQuery>) -> Json<Value> {
    update_plan_get(Query(body)).await
}

async fn update_apply(Json(body): Json<UpdateApplyBody>) -> Json<Value> {
    let inventory = build_update_inventory("apply", body.query).await;
    let plan = build_update_plan(inventory);
    Json(build_apply_envelope(
        plan,
        body.dry_run,
        body.yes,
        body.allow_apply,
    ))
}

async fn update_history(Query(query): Query<UpdateHistoryQuery>) -> Json<Value> {
    Json(build_history_envelope(query.limit))
}

async fn update_rollback(Json(body): Json<UpdateRollbackBody>) -> Json<Value> {
    Json(build_rollback_envelope(body))
}

async fn update_admin(Json(body): Json<UpdateAdminBody>) -> Json<Value> {
    Json(build_admin_envelope(body))
}

async fn update_scheduler(Query(query): Query<UpdateQuery>) -> Json<Value> {
    Json(build_scheduler_envelope(
        query.channel.unwrap_or_else(|| "dev".into()),
    ))
}

async fn update_notifications_get(Query(query): Query<UpdateQuery>) -> Json<Value> {
    let inventory = build_update_inventory("notifications", query).await;
    Json(build_notifications_envelope(inventory))
}

async fn update_notifications(Json(body): Json<UpdateQuery>) -> Json<Value> {
    update_notifications_get(Query(body)).await
}

async fn update_policy() -> Json<Value> {
    let path = update_policy_path();
    let exists = path.exists();
    let policy = read_update_policy().unwrap_or_else(|_| default_policy_from_license());
    Json(json!({
        "schema": "focusa.update_policy_status.v1",
        "status": "completed",
        "path": path,
        "exists": exists,
        "policy": policy,
        "mutations_performed": false,
        "auto_apply_allowed": false,
    }))
}

async fn update_policy_set(Json(body): Json<UpdatePolicySetBody>) -> Json<Value> {
    let mut policy = read_update_policy().unwrap_or_else(|_| default_policy_from_license());
    if let Some(enabled) = body.enabled {
        policy.enabled = enabled;
    }
    if let Some(channel) = body.channel {
        if let Ok(parsed) = channel.parse::<ReleaseChannel>() {
            policy.channel = parsed;
        }
    }
    if let Some(mode) = body.mode {
        if let Ok(parsed) = mode.parse::<UpdateMode>() {
            policy.mode = parsed;
        }
    }
    policy.auto_apply_allowed = false;
    if policy.auto_apply_blocked_until.is_empty() {
        policy.auto_apply_blocked_until = vec![
            "update_locking".into(),
            "atomic_install".into(),
            "rollback_apply".into(),
            "health_proof".into(),
        ];
    }
    match write_update_policy(&policy) {
        Ok(path) => Json(json!({
            "schema": "focusa.update_policy_write.v1",
            "status": "completed",
            "path": path,
            "policy": policy,
            "mutations_performed": true,
            "mutation_scope": "update_policy_file_only",
            "auto_apply_allowed": false,
            "next_action": "GET /v1/update/status"
        })),
        Err(err) => Json(json!({
            "schema": "focusa.update_policy_write.v1",
            "status": "blocked",
            "failure_class": "policy_write_failed",
            "error": err.to_string(),
            "mutations_performed": false,
            "auto_apply_allowed": false
        })),
    }
}

async fn build_update_inventory(command: &'static str, query: UpdateQuery) -> Value {
    let latest = resolve_latest(query.latest_version.as_deref());
    let parts = vec![
        inspect_binary("cli", "/usr/local/bin/focusa", &latest).await,
        inspect_daemon(&latest),
        inspect_binary("tui", "/usr/local/bin/focusa-tui", &latest).await,
    ];
    let stale_parts = parts
        .iter()
        .filter(|part| part.get("stale") == Some(&Value::Bool(true)))
        .filter_map(|part| part.get("part").and_then(Value::as_str))
        .collect::<Vec<_>>();
    let stale_count = stale_parts.len();
    let inventory_interval_seconds = std::env::var("FOCUSA_UPDATE_INVENTORY_INTERVAL_SECONDS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(21_600)
        .clamp(300, 604_800);
    json!({
        "schema": "focusa.update_inventory.v1",
        "status": "completed",
        "command": command,
        "read_only": true,
        "mutations_performed": false,
        "channel": query.channel.unwrap_or_else(|| "dev".to_string()),
        "latest": {
            "version": latest.version,
            "source": latest.source,
            "release_manifest_required": true,
            "eligibility_status": "placeholder_until_manifest_resolver"
        },
        "policy": policy_summary_json(),
        "license": license_summary_json(),
        "parts": parts,
        "stale_parts": stale_parts,
        "stale_count": stale_count,
        "fleet_truth_status": if stale_count == 0 { "current" } else { "drift_detected" },
        "continuous_currency": {
            "enabled": true,
            "checked_at": Utc::now().to_rfc3339(),
            "interval_seconds": inventory_interval_seconds,
            "trigger_surfaces": ["update/status", "update/check", "update/notifications", "admin poll"],
            "notification_required": stale_count > 0,
            "policy_driven": true,
            "blind_latest_allowed": false,
            "pin_override_env": "FOCUSA_UPDATE_LATEST_VERSION",
            "interval_override_env": "FOCUSA_UPDATE_INVENTORY_INTERVAL_SECONDS"
        },
        "warnings": [
            "read-only inventory only; no update apply, download, binary replacement, or daemon restart was attempted",
            "release manifest eligibility/signature/provenance is required before trusted auto-apply"
        ],
        "next_tools": ["focusa update status --json", "focusa update check --channel dev --json"]
    })
}

fn build_update_plan(inventory: Value) -> Value {
    let latest_version = inventory
        .pointer("/latest/version")
        .and_then(Value::as_str)
        .unwrap_or(env!("CARGO_PKG_VERSION"));
    let policy_mode = inventory
        .pointer("/policy/mode")
        .and_then(Value::as_str)
        .unwrap_or("manual");
    let mut parts = Vec::new();
    let mut order = 1u8;
    if let Some(arr) = inventory.get("parts").and_then(Value::as_array) {
        for part in arr
            .iter()
            .filter(|p| p.get("part").and_then(Value::as_str) != Some("daemon"))
        {
            parts.push(part_plan(part, latest_version, &mut order));
        }
        for part in arr
            .iter()
            .filter(|p| p.get("part").and_then(Value::as_str) == Some("daemon"))
        {
            parts.push(part_plan(part, latest_version, &mut order));
        }
    }
    let daemon_restart = parts.iter().any(|p| {
        p.get("part").and_then(Value::as_str) == Some("daemon")
            && p.get("restart_required").and_then(Value::as_bool) == Some(true)
    });
    let blockers = vec![
        "release_manifest_signature_verification_not_wired_to_plan",
        "update_locking_not_implemented",
        "atomic_install_not_implemented",
        "rollback_apply_not_implemented",
    ];
    json!({
        "schema": "focusa.update_plan.v1",
        "status": "planned_read_only",
        "read_only": true,
        "mutations_performed": false,
        "apply_allowed": false,
        "apply_blocked_until": blockers,
        "channel": inventory.get("channel").cloned().unwrap_or_else(|| json!("dev")),
        "latest": inventory.get("latest").cloned().unwrap_or_else(|| json!({"version": latest_version})),
        "policy": inventory.get("policy").cloned().unwrap_or_else(|| json!({"mode":"manual"})),
        "license": inventory.get("license").cloned().unwrap_or_else(|| json!({"level":"unknown"})),
        "compatibility": {
            "status": "blocked_until_apply_gates",
            "daemon_api_contract": "focusa.api.v1",
            "pi_tool_contract": "focusa.pi-tools.v1",
            "data_schema": "focusa.data.v1",
            "requires_migration": false,
            "blockers": blockers,
        },
        "safety": build_safety_plan_json(),
        "prompt": {
            "mode": policy_mode,
            "update_prompt_required": policy_mode != "automatic",
            "daemon_restart_prompt_required": daemon_restart,
            "copy": [
                "Your Focusa data, projects, license, Workpoints, evidence, and .env files will not be overwritten by a valid update plan.",
                "Daemon restart is shown separately because it may interrupt active sessions.",
                "This route is read-only; it has not downloaded, installed, or restarted anything."
            ],
            "choices": ["show_details", "later", "skip_version", "disable_auto_update", "apply_when_available"]
        },
        "install_order": ["cli", "tui", "daemon_last"],
        "parts": parts,
        "warnings": inventory.get("warnings").cloned().unwrap_or_else(|| json!([])),
        "next_tools": ["focusa update status --json", "focusa update plan --json"]
    })
}

fn part_plan(part: &Value, target_version: &str, order: &mut u8) -> Value {
    let part_name = part
        .get("part")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let stale = part.get("stale").and_then(Value::as_bool);
    let action = match stale {
        Some(true) => "would_update",
        Some(false) => "no_op",
        None => "probe_required",
    };
    let value = json!({
        "part": part_name,
        "current_version": part.get("version").cloned().unwrap_or(Value::Null),
        "target_version": target_version,
        "action": action,
        "reason": part.get("stale_reason").and_then(Value::as_str).unwrap_or("unknown"),
        "restart_required": part_name == "daemon" && stale == Some(true),
        "order": *order,
    });
    *order = order.saturating_add(1);
    value
}

fn build_scheduler_envelope(channel: String) -> Value {
    json!({
        "schema": "focusa.update_scheduler.v1",
        "status": "planned_read_only",
        "read_only": true,
        "mutations_performed": false,
        "scheduler_installed": false,
        "background_worker_started": false,
        "channel": channel,
        "policy": policy_summary_json(),
        "startup_check": {"enabled": true, "delay_seconds": 45, "reason": "avoid slowing interactive daemon startup"},
        "interval": {"base_seconds": 21600, "jitter_percent": 20, "backoff": ["5m", "15m", "1h", "6h"]},
        "offline": {"skip_when_offline": true, "retry_backoff": ["network_error", "dns_error", "release_host_timeout"], "max_silent_failures_before_notice": 3},
        "maintenance": {"respected": true, "default_window": "02:00-05:00 local time", "user_override_path": update_state_root().join("maintenance-window.json").display().to_string()},
        "automatic_apply": {
            "allowed": false,
            "reason": "auto apply remains disabled until manifest/signature/lock/rollback/apply gates are implemented",
            "requires": ["trusted_release_manifest", "update_lock_acquired", "rollback_snapshot_ready", "policy_allows_automatic_apply", "daemon_restart_policy_approved"]
        },
        "notifications": notification_routes_json(),
        "next_actions": ["wire daemon startup check after runtime tests", "wire interval worker after scheduler proof", "keep apply disabled until Spec128 gates pass"]
    })
}

fn build_notifications_envelope(inventory: Value) -> Value {
    let stale_parts = inventory
        .get("stale_parts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let stale_names = stale_parts
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    let severity = if stale_names.is_empty() {
        "none"
    } else {
        "warning"
    };
    let body = if stale_names.is_empty() {
        "Focusa surfaces are current or unknown; no update warning is required.".to_string()
    } else {
        format!(
            "Focusa update available for: {}. Run focusa update plan --json before applying.",
            stale_names.join(", ")
        )
    };
    json!({
        "schema": "focusa.update_notifications.v1",
        "status": "completed",
        "read_only": true,
        "mutations_performed": false,
        "stale_parts": stale_parts,
        "severity": severity,
        "surfaces": notification_routes_json(),
        "messages": [
            {"surface": "cli", "title": "Focusa update status", "body": body, "action": "focusa update plan"},
            {"surface": "api", "title": "Focusa update status", "body": body, "action": "POST /v1/update/plan"},
            {"surface": "pi_doctor", "title": "Focusa update status", "body": body, "action": "focusa update status --json"}
        ],
        "suppress_if": ["version_pinned", "version_skipped", "updates_paused", "offline_without_prior_success"]
    })
}

fn notification_routes_json() -> Value {
    json!({
        "cli": true,
        "api": true,
        "pi_doctor": true,
        "tui": "planned_when_tui_update_banner_available",
        "menubar": "planned_when_menubar_update_badge_available"
    })
}

fn build_history_envelope(limit: usize) -> Value {
    let base = update_state_root();
    let history_path = base.join("update-history.jsonl");
    let journal_path = base.join("update-journal.json");
    let events = std::fs::read_to_string(&history_path)
        .ok()
        .map(|raw| {
            raw.lines()
                .rev()
                .take(limit)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    json!({
        "schema": "focusa.update_history.v1",
        "status": "completed",
        "read_only": true,
        "mutations_performed": false,
        "history_path": history_path.display().to_string(),
        "journal_path": journal_path.display().to_string(),
        "retention": {"keep_last_successful_bundles": 3, "keep_days": 30, "prune_requires_admin_confirmation": true},
        "observability": {
            "counters": ["update_check_total", "update_plan_total", "update_apply_blocked_total", "update_apply_success_total", "update_rollback_total"],
            "events": ["check_started", "plan_created", "apply_blocked", "stage_verified", "promote_started", "daemon_restart_prompted", "rollback_started", "rollback_completed"],
            "log_paths": [base.join("update.log").display().to_string(), history_path.display().to_string(), journal_path.display().to_string()]
        },
        "events": events,
        "next_tools": ["focusa update plan --json", "focusa update rollback --dry-run --json"]
    })
}

fn build_rollback_envelope(body: UpdateRollbackBody) -> Value {
    let restore_order = match body.part.as_str() {
        "daemon" => json!(["daemon", "restart_daemon_after_health_contract_check"]),
        "cli" => json!(["cli"]),
        "tui" => json!(["tui"]),
        _ => json!(["daemon", "tui", "cli", "health_contract_check"]),
    };
    json!({
        "schema": "focusa.update_rollback.v1",
        "status": "blocked_read_only",
        "read_only": true,
        "mutations_performed": false,
        "rollback_executed": false,
        "part": body.part,
        "dry_run": body.dry_run,
        "consent_yes": body.yes,
        "blocked_reason": ["rollback_executor_not_enabled_in_spec128_08_scaffold", "snapshot_integrity_verification_required", "admin_confirmation_required"],
        "restore_order": restore_order,
        "proof_required": ["snapshot_sha256_verified", "same_filesystem_atomic_rename_available", "post_rollback_version_matches_snapshot", "no_data_env_license_overwrite", "history_event_written"],
        "data_safety": {"overwrite_data": false, "overwrite_env": false, "overwrite_license": false, "preserve": build_safety_plan_json().pointer("/preserves").cloned().unwrap_or_else(|| json!([]))},
        "recovery_hint": "No rollback was executed. Inspect update history/journal and rerun with future rollback gates when implemented."
    })
}

fn build_admin_envelope(body: UpdateAdminBody) -> Value {
    let mut requested = Vec::new();
    if let Some(version) = &body.pin_version {
        requested.push(format!("pin_version:{version}"));
    }
    if body.unpin {
        requested.push("unpin".into());
    }
    if let Some(version) = &body.skip_version {
        requested.push(format!("skip_version:{version}"));
    }
    if body.pause {
        requested.push("pause".into());
    }
    if body.resume {
        requested.push("resume".into());
    }
    if body.force_check {
        requested.push("force_check".into());
    }
    if body.trusted_dev_force_latest {
        requested.push("trusted_dev_force_latest".into());
    }
    let dev_mode = std::env::var("FOCUSA_DEV_MODE")
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes"))
        .unwrap_or(false);
    json!({
        "schema": "focusa.update_admin_control.v1",
        "status": "preview_read_only",
        "read_only": true,
        "mutations_performed": false,
        "dry_run": body.dry_run,
        "consent_yes": body.yes,
        "requested_controls": requested,
        "policy_patch_preview": {"pin_version": body.pin_version, "unpin": body.unpin, "skip_version": body.skip_version, "pause": body.pause, "resume": body.resume, "trusted_dev_force_latest": body.trusted_dev_force_latest},
        "force_check_preview": body.force_check,
        "trusted_dev_force_latest_allowed": body.trusted_dev_force_latest && dev_mode,
        "blocked_reason": ["admin_control_write_executor_not_enabled_in_spec128_08_scaffold", "dry_run_preview_only"]
    })
}

fn build_apply_envelope(plan: Value, dry_run: bool, yes: bool, allow_apply: bool) -> Value {
    let mut blocked_reason = plan
        .get("apply_blocked_until")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    blocked_reason.push("apply_executor_not_enabled_in_spec128_07_scaffold".into());
    if dry_run {
        blocked_reason.push("dry_run_requested".into());
    }
    if !(yes && allow_apply) {
        blocked_reason.push("explicit_yes_and_allow_apply_required".into());
    }
    let daemon_required = plan
        .get("parts")
        .and_then(Value::as_array)
        .map(|parts| {
            parts.iter().any(|part| {
                part.get("part").and_then(Value::as_str) == Some("daemon")
                    && part.get("action").and_then(Value::as_str) == Some("would_update")
            })
        })
        .unwrap_or(false);
    let preserve = plan
        .pointer("/safety/preserves")
        .cloned()
        .unwrap_or_else(|| json!([]));
    json!({
        "schema": "focusa.update_apply.v1",
        "status": "blocked_read_only",
        "read_only": true,
        "mutations_performed": false,
        "apply_requested": yes || allow_apply || !dry_run,
        "apply_executed": false,
        "dry_run": dry_run,
        "consent": {
            "yes": yes,
            "allow_apply": allow_apply,
            "effective": yes && allow_apply && !dry_run,
            "note": "consent is recorded only; this scaffold does not mutate binaries"
        },
        "plan": plan,
        "execution_order": ["cli", "tui", "daemon_last", "restart_daemon_only_if_changed_and_allowed"],
        "daemon_restart": {
            "allowed": false,
            "required": daemon_required,
            "when": "after daemon binary promotion, policy approval, and health/version/contract proof",
            "health_proof": "GET /v1/health version and API contract must match target release"
        },
        "data_safety": {
            "overwrite_data": false,
            "overwrite_env": false,
            "overwrite_license": false,
            "preserve": preserve
        },
        "proof_required": [
            "release_manifest_signature_verified",
            "asset_sha256_verified",
            "cli_version_matches_target",
            "tui_version_matches_target_or_not_installed",
            "daemon_health_version_matches_target_when_daemon_changed",
            "daemon_api_contract_matches_target_when_daemon_changed",
            "no_data_env_license_overwrite",
            "rollback_journal_written"
        ],
        "blocked_reason": blocked_reason,
        "recovery_hint": "No update was applied. Use /v1/update/plan to inspect blockers; apply remains disabled until Spec128 apply gates are implemented."
    })
}

fn build_safety_plan_json() -> Value {
    let base = update_state_root();
    let staging_root = base.join("staging");
    json!({
        "lock": {
            "path": base.join("update.lock").display().to_string(),
            "mode": "exclusive_create_new_with_pid_and_started_at",
            "stale_after_seconds": 1800,
            "behavior": [
                "only one update may stage or apply on a host at a time",
                "stale locks require process liveness check before takeover",
                "lock release happens after journaled success or rollback decision"
            ]
        },
        "staging": {
            "root": staging_root.display().to_string(),
            "manifest_path": staging_root.join("release-manifest.json").display().to_string(),
            "download_dir": staging_root.join("downloads").display().to_string(),
            "verify_before_promote": [
                "release_manifest_signature",
                "asset_sha256",
                "asset_size",
                "version_eligibility",
                "platform_triple_match",
                "executable_smoke_test"
            ]
        },
        "atomic_install": {
            "strategy": "write_temp_fsync_rename_then_smoke_test",
            "sequence": [
                "snapshot_existing_binary_metadata",
                "write_new_binary_to_same_filesystem_temp_path",
                "fsync_temp_file_and_parent_directory",
                "preserve_permissions_owner_xattrs_capabilities_when_supported",
                "rename_temp_over_target_atomically",
                "fsync_parent_directory_after_rename",
                "run_post_promote_smoke_test",
                "rollback_from_snapshot_on_smoke_failure"
            ],
            "daemon_policy": "daemon binary is promoted last; restart is a separate explicit/policy-gated step"
        },
        "recovery": {
            "journal_path": base.join("update-journal.json").display().to_string(),
            "interrupted_states": [
                "lock_acquired",
                "assets_staged",
                "verified",
                "promoting_cli",
                "promoting_tui",
                "promoting_daemon",
                "smoke_testing",
                "rollback_required"
            ],
            "recovery_actions": [
                "resume_verification_for_fully_staged_assets",
                "rollback_promoted_part_from_snapshot_when_journal_marks_incomplete",
                "discard_unverified_stage_on_checksum_or_signature_mismatch",
                "preserve_user_data_license_env_projects_workpoints_evidence",
                "print_manual_recovery_commands_without_running_destructive_actions"
            ],
            "rollback_available": true
        },
        "preserves": [
            "license.json",
            ".env",
            "projects",
            "workpoints",
            "evidence",
            "logs",
            "permissions",
            "owner",
            "xattrs_when_supported",
            "capabilities_when_supported"
        ],
        "no_half_written_executable_rule": "never write directly to an executable target path; promote only by same-filesystem atomic rename after verification"
    })
}

fn update_state_root() -> std::path::PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".local/state"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("focusa")
        .join("update")
}

fn update_policy_path() -> std::path::PathBuf {
    std::env::var_os("FOCUSA_UPDATE_POLICY")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/usr/local/lib/focusa/update-policy.json"))
}

fn default_policy_from_license() -> UpdatePolicy {
    match load_license_status() {
        Ok(status) => {
            let dev_override = std::env::var("FOCUSA_DEV_MODE")
                .map(|v| matches!(v.as_str(), "1" | "true" | "yes"))
                .unwrap_or(false);
            UpdatePolicy::default_for_license(status.tier, &status.features, dev_override)
        }
        Err(_) => UpdatePolicy::default_for_license("evaluation", &[], false),
    }
}

fn read_update_policy() -> anyhow::Result<UpdatePolicy> {
    let path = update_policy_path();
    let raw = std::fs::read_to_string(&path)?;
    let policy: UpdatePolicy = serde_json::from_str(&raw)?;
    if policy.schema != UPDATE_POLICY_SCHEMA_V1 {
        anyhow::bail!(
            "unsupported update policy schema: expected {}, got {}",
            UPDATE_POLICY_SCHEMA_V1,
            policy.schema
        );
    }
    Ok(policy)
}

fn write_update_policy(policy: &UpdatePolicy) -> anyhow::Result<std::path::PathBuf> {
    let path = update_policy_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(
        &path,
        format!("{}\n", serde_json::to_string_pretty(policy)?),
    )?;
    Ok(path)
}

fn policy_summary_json() -> Value {
    let path = update_policy_path();
    let exists = path.exists();
    let policy = read_update_policy().unwrap_or_else(|_| default_policy_from_license());
    json!({
        "path": path,
        "exists": exists,
        "enabled": policy.enabled,
        "channel": policy.channel.label(),
        "mode": policy.mode.label(),
        "auto_apply_allowed": policy.auto_apply_allowed,
        "auto_apply_blocked_until": policy.auto_apply_blocked_until,
        "note": if exists {
            "policy file loaded; auto-apply still requires later locking/rollback/apply gates"
        } else {
            "license-derived default policy; no policy file exists yet"
        }
    })
}

fn license_summary_json() -> Value {
    match load_license_status() {
        Ok(status) => {
            let dev_mode = status.tier == "dev_mode"
                || (status.features.iter().any(|f| f == "developer_channel")
                    && status.features.iter().any(|f| f == "ota_auto_update"));
            json!({
                "level": if dev_mode { "dev_mode" } else { status.tier.as_str() },
                "dev_mode": dev_mode,
                "features": status.features,
                "source": "local_license_file",
                "note": "policy defaults are derived from license, but update apply remains disabled until safety gates exist"
            })
        }
        Err(_) => json!({
            "level": "evaluation",
            "dev_mode": false,
            "features": [],
            "source": "fallback_evaluation",
            "note": "license unreadable; defaulting update policy posture to evaluation notify-only"
        }),
    }
}

struct Latest {
    version: String,
    source: String,
}

fn resolve_latest(override_value: Option<&str>) -> Latest {
    if let Some(v) = override_value.filter(|s| !s.trim().is_empty()) {
        return Latest {
            version: normalize_version(v),
            source: "request.latest_version".into(),
        };
    }
    for env_key in ["FOCUSA_LATEST_VERSION", "FOCUSA_UPDATE_LATEST_TAG"] {
        if let Ok(v) = std::env::var(env_key) {
            if !v.trim().is_empty() {
                return Latest {
                    version: normalize_version(&v),
                    source: env_key.into(),
                };
            }
        }
    }
    Latest {
        version: env!("CARGO_PKG_VERSION").into(),
        source: "daemon_package_version".into(),
    }
}

async fn inspect_binary(part: &'static str, expected_path: &str, latest: &Latest) -> Value {
    let path = Path::new(expected_path);
    let exists = path.exists();
    let version = if exists {
        probe_version_command(expected_path)
            .await
            .map(|s| normalize_version(&s))
    } else {
        None
    };
    let stale = version.as_ref().map(|v| v != &latest.version);
    json!({
        "part": part,
        "expected_path": expected_path,
        "resolved_path": if exists { Some(expected_path) } else { None },
        "exists": exists,
        "version": version,
        "version_source": "binary_--version",
        "version_probe_safe": true,
        "sha256": if exists { sha256_file(path).ok() } else { None },
        "stale": stale,
        "stale_reason": stale_reason(part, version.as_deref(), stale, &latest.version, exists),
    })
}

fn inspect_daemon(latest: &Latest) -> Value {
    let expected_path = "/usr/local/bin/focusa-daemon";
    let path = Path::new(expected_path);
    let exists = path.exists();
    let version = normalize_version(env!("CARGO_PKG_VERSION"));
    let stale = version != latest.version;
    json!({
        "part": "daemon",
        "expected_path": expected_path,
        "resolved_path": if exists { Some(expected_path) } else { None },
        "exists": exists,
        "version": version,
        "version_source": "running_daemon_package_version",
        "version_probe_safe": true,
        "sha256": if exists { sha256_file(path).ok() } else { None },
        "stale": stale,
        "stale_reason": if stale {
            format!("running daemon package version differs from latest {}", latest.version)
        } else {
            format!("running daemon package version matches latest {}", latest.version)
        },
        "notes": ["binary --version intentionally not invoked; current daemon binary treats --version as startup input"]
    })
}

fn stale_reason(
    part: &str,
    version: Option<&str>,
    stale: Option<bool>,
    latest: &str,
    exists: bool,
) -> String {
    if !exists {
        return format!("{part} binary not found");
    }
    match (version, stale) {
        (Some(v), Some(true)) => {
            format!("installed {part} version {v} differs from latest {latest}")
        }
        (Some(v), Some(false)) => format!("installed {part} version {v} matches latest {latest}"),
        _ => format!("{part} version probe unavailable"),
    }
}

async fn probe_version_command(path: &str) -> Option<String> {
    let output = timeout(
        Duration::from_secs(3),
        Command::new(path).arg("--version").output(),
    )
    .await
    .ok()?
    .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        None
    } else {
        Some(stdout)
    }
}

fn sha256_file(path: &Path) -> anyhow::Result<String> {
    let bytes = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex::encode(hasher.finalize()))
}

fn normalize_version(raw: &str) -> String {
    let trimmed = raw.trim();
    let last = trimmed.split_whitespace().last().unwrap_or(trimmed);
    last.trim_start_matches('v').to_string()
}

#[cfg(test)]
mod tests {
    use super::{UpdateQuery, build_update_inventory, normalize_version};

    #[test]
    fn normalizes_common_version_outputs() {
        assert_eq!(normalize_version("focusa 0.9.74-dev"), "0.9.74-dev");
        assert_eq!(normalize_version("v0.9.80-dev"), "0.9.80-dev");
    }

    #[tokio::test]
    async fn inventory_exposes_continuous_currency_and_drift_policy() {
        let inventory = build_update_inventory(
            "status",
            UpdateQuery {
                channel: Some("dev".into()),
                latest_version: Some(env!("CARGO_PKG_VERSION").into()),
            },
        )
        .await;
        assert_eq!(inventory["continuous_currency"]["enabled"], true);
        assert_eq!(
            inventory["continuous_currency"]["blind_latest_allowed"],
            false
        );
        assert!(inventory["continuous_currency"]["interval_seconds"].is_u64());
        assert!(inventory["stale_count"].is_u64());
        assert!(matches!(
            inventory["fleet_truth_status"].as_str(),
            Some("current" | "drift_detected")
        ));
    }
}
