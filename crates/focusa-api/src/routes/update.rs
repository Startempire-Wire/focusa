//! Spec 128 update inventory, mutable policy, scheduler status, and guarded apply API routes.
//!
//! Inventory and planning remain read-only; policy mutation is explicit and
//! automatic apply authority is derived from mode, enabled parts, and license.

use axum::extract::Query;
use axum::{
    Json, Router,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::license::load_license_status;
use focusa_core::update::{
    ReleaseChannel, UPDATE_POLICY_SCHEMA_V1, UpdateMode, UpdatePolicy, UpdatePolicyParts,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

use super::update_authority::{
    ApplyRequest as AuthorityApplyRequest, CliUpdateAuthority,
    RollbackRequest as AuthorityRollbackRequest, UpdateRequest as AuthorityUpdateRequest,
};

pub fn router() -> Router<Arc<crate::server::AppState>> {
    Router::new()
        .route("/v1/update/status", get(update_status))
        .route("/v1/update/check", get(update_check_get).post(update_check))
        .route("/v1/update/plan", get(update_plan_get).post(update_plan))
        .route("/v1/update/apply", post(update_apply))
        .route("/v1/update/history", get(update_history))
        .route("/v1/update/rollback", post(update_rollback))
        .route("/v1/update/admin", post(update_admin))
        .route(
            "/v1/update/scheduler",
            get(update_scheduler).post(update_scheduler_set),
        )
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
    #[serde(default)]
    include_hashes: bool,
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
struct UpdateSchedulerSetBody {
    enabled: bool,
    #[serde(default = "default_dev_channel")]
    channel: String,
}

fn default_dev_channel() -> String {
    "dev".to_string()
}

#[derive(Debug, Deserialize, Default)]
struct UpdatePolicySetBody {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    dev_mode: Option<bool>,
    #[serde(default)]
    parts: Option<UpdatePolicyParts>,
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
    Json(
        CliUpdateAuthority::installed()
            .plan(authority_update_request(query))
            .await,
    )
}

async fn update_plan(Json(body): Json<UpdateQuery>) -> Json<Value> {
    update_plan_get(Query(body)).await
}

async fn update_apply(Json(body): Json<UpdateApplyBody>) -> Json<Value> {
    Json(
        CliUpdateAuthority::installed()
            .apply(AuthorityApplyRequest {
                update: authority_update_request(body.query),
                dry_run: body.dry_run,
                yes: body.yes,
                allow_apply: body.allow_apply,
            })
            .await,
    )
}

async fn update_history(Query(query): Query<UpdateHistoryQuery>) -> Json<Value> {
    Json(build_history_envelope(query.limit))
}

async fn update_rollback(Json(body): Json<UpdateRollbackBody>) -> Json<Value> {
    Json(
        CliUpdateAuthority::installed()
            .rollback(AuthorityRollbackRequest {
                part: body.part,
                dry_run: body.dry_run,
                yes: body.yes,
            })
            .await,
    )
}

fn authority_update_request(query: UpdateQuery) -> AuthorityUpdateRequest {
    AuthorityUpdateRequest {
        channel: query.channel,
        latest_version: query.latest_version,
    }
}

async fn update_admin(Json(body): Json<UpdateAdminBody>) -> Json<Value> {
    Json(build_admin_envelope(body))
}

async fn update_scheduler_set(Json(body): Json<UpdateSchedulerSetBody>) -> Json<Value> {
    let cli = std::env::var_os("FOCUSA_CLI_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/usr/local/bin/focusa"));
    let action = if body.enabled {
        "--install"
    } else {
        "--uninstall"
    };
    match Command::new(&cli)
        .args([
            "update",
            "scheduler",
            action,
            "--channel",
            &body.channel,
            "--json",
        ])
        .output()
        .await
    {
        Ok(output) if output.status.success() => Json(json!({
            "schema": "focusa.update_scheduler_mutation.v1",
            "status": "completed",
            "enabled": body.enabled,
            "channel": body.channel,
            "mutations_performed": true,
            "scheduler": serde_json::from_slice::<Value>(&output.stdout).unwrap_or(Value::Null)
        })),
        Ok(output) => Json(json!({
            "schema": "focusa.update_scheduler_mutation.v1",
            "status": "blocked",
            "failure_class": "scheduler_command_failed",
            "enabled": body.enabled,
            "error": String::from_utf8_lossy(&output.stderr).chars().take(512).collect::<String>(),
            "mutations_performed": false
        })),
        Err(error) => Json(json!({
            "schema": "focusa.update_scheduler_mutation.v1",
            "status": "blocked",
            "failure_class": "scheduler_cli_unavailable",
            "error": error.to_string(),
            "mutations_performed": false
        })),
    }
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

fn refresh_update_policy_authority(policy: &mut UpdatePolicy) {
    let dev_override = policy.dev_mode_override
        || std::env::var("FOCUSA_DEV_MODE")
            .map(|value| matches!(value.as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);
    match load_license_status() {
        Ok(status) => {
            policy.license_level = if dev_override {
                "dev_mode".into()
            } else {
                status.tier
            };
            policy.refresh_auto_apply_authority(&status.features, dev_override);
        }
        Err(_) => {
            policy.license_level = if dev_override {
                "dev_mode".into()
            } else {
                "evaluation".into()
            };
            policy.refresh_auto_apply_authority(&[], dev_override);
        }
    }
}

async fn update_policy() -> Json<Value> {
    let path = update_policy_path();
    let exists = path.exists();
    let mut policy = read_update_policy().unwrap_or_else(|_| default_policy_from_license());
    refresh_update_policy_authority(&mut policy);
    Json(json!({
        "schema": "focusa.update_policy_status.v1",
        "status": "completed",
        "path": path,
        "exists": exists,
        "policy": policy,
        "mutations_performed": false,
        "auto_apply_allowed": policy.auto_apply_allowed,
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
    if let Some(dev_mode) = body.dev_mode {
        policy.dev_mode_override = dev_mode;
        if dev_mode {
            policy.channel = ReleaseChannel::Dev;
            policy.mode = UpdateMode::Automatic;
            policy.parts = UpdatePolicyParts::all_surfaces(true);
            policy.maintenance_window = "always".into();
        }
    }
    if let Some(parts) = body.parts {
        policy.parts = parts;
    }
    refresh_update_policy_authority(&mut policy);
    match write_update_policy(&policy) {
        Ok(path) => Json(json!({
            "schema": "focusa.update_policy_write.v1",
            "status": "completed",
            "path": path,
            "policy": policy,
            "mutations_performed": true,
            "mutation_scope": "update_policy_file_only",
            "auto_apply_allowed": policy.auto_apply_allowed,
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

fn env_path(name: &str, fallback: PathBuf) -> PathBuf {
    std::env::var_os(name)
        .map(PathBuf::from)
        .unwrap_or(fallback)
}

fn platform_config_home() -> PathBuf {
    if let Some(path) = std::env::var_os("FOCUSA_CONFIG_DIR") {
        return PathBuf::from(path);
    }
    #[cfg(target_os = "windows")]
    {
        env_path("APPDATA", PathBuf::from(".")).join("Focusa")
    }
    #[cfg(target_os = "macos")]
    {
        env_path("HOME", PathBuf::from(".")).join("Library/Application Support/Focusa")
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        env_path(
            "XDG_CONFIG_HOME",
            env_path("HOME", PathBuf::from(".")).join(".config"),
        )
        .join("focusa")
    }
}

fn first_existing(candidates: impl IntoIterator<Item = PathBuf>) -> Option<PathBuf> {
    candidates.into_iter().find(|path| path.exists())
}

fn discover_source_root() -> PathBuf {
    if let Some(path) = std::env::var_os("FOCUSA_SOURCE_ROOT") {
        return PathBuf::from(path);
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    cwd.ancestors()
        .find(|path| {
            path.join(".focusa-project.json").is_file() || path.join("Cargo.toml").is_file()
        })
        .map(Path::to_path_buf)
        .unwrap_or(cwd)
}

fn running_executable() -> PathBuf {
    std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from(std::env::args_os().next().unwrap_or_default()))
}

fn platform_install_prefix(running_exe: &Path) -> PathBuf {
    if let Some(path) = std::env::var_os("FOCUSA_INSTALL_PREFIX") {
        return PathBuf::from(path);
    }
    let executable_dir = running_exe.parent().unwrap_or_else(|| Path::new("."));
    if executable_dir.file_name().and_then(|name| name.to_str()) == Some("bin") {
        executable_dir
            .parent()
            .unwrap_or(executable_dir)
            .to_path_buf()
    } else {
        executable_dir.to_path_buf()
    }
}

fn resolved_binary_path(env_name: &str, binary_name: &str, running_exe: &Path) -> PathBuf {
    if let Some(path) = std::env::var_os(env_name) {
        return PathBuf::from(path);
    }
    if binary_name == "focusa-daemon" {
        return running_exe.to_path_buf();
    }
    let suffix = if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    };
    running_exe
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{binary_name}{suffix}"))
}

fn platform_service_paths() -> (Option<PathBuf>, Option<PathBuf>, &'static str) {
    let explicit_definition = std::env::var_os("FOCUSA_SERVICE_DEFINITION").map(PathBuf::from);
    let explicit_overrides = std::env::var_os("FOCUSA_SERVICE_OVERRIDES").map(PathBuf::from);
    if explicit_definition.is_some() || explicit_overrides.is_some() {
        return (explicit_definition, explicit_overrides, "configured");
    }
    #[cfg(target_os = "linux")]
    {
        let user_units = env_path(
            "XDG_CONFIG_HOME",
            env_path("HOME", PathBuf::from(".")).join(".config"),
        )
        .join("systemd/user");
        let definition = first_existing([
            user_units.join("focusa-daemon.service"),
            PathBuf::from("/etc/systemd/system/focusa-daemon.service"),
            PathBuf::from("/usr/lib/systemd/system/focusa-daemon.service"),
        ]);
        let overrides = definition.as_ref().and_then(|path| {
            let candidate = PathBuf::from(format!("{}.d", path.display()));
            candidate.exists().then_some(candidate)
        });
        (definition, overrides, "systemd")
    }
    #[cfg(target_os = "macos")]
    {
        let candidate = env_path("HOME", PathBuf::from("."))
            .join("Library/LaunchAgents/com.focusa.daemon.plist");
        (candidate.exists().then_some(candidate), None, "launchd")
    }
    #[cfg(target_os = "windows")]
    {
        (None, None, "windows_service")
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        (None, None, "unsupported")
    }
}

fn discover_desktop_app() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("FOCUSA_DESKTOP_APP_PATH") {
        return Some(PathBuf::from(path));
    }
    #[cfg(target_os = "macos")]
    {
        first_existing([
            env_path("HOME", PathBuf::from(".")).join("Applications/Focusa.app"),
            PathBuf::from("/Applications/Focusa.app"),
        ])
    }
    #[cfg(target_os = "windows")]
    {
        first_existing([
            env_path("LOCALAPPDATA", PathBuf::from(".")).join("Focusa/Focusa.exe"),
            env_path("PROGRAMFILES", PathBuf::from(".")).join("Focusa/Focusa.exe"),
        ])
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        first_existing([
            env_path(
                "XDG_DATA_HOME",
                env_path("HOME", PathBuf::from(".")).join(".local/share"),
            )
            .join("applications/focusa.desktop"),
            PathBuf::from("/usr/share/applications/focusa.desktop"),
        ])
    }
}

async fn build_update_inventory(command: &'static str, query: UpdateQuery) -> Value {
    let latest = resolve_latest(query.latest_version.as_deref());
    let running_exe = running_executable();
    let prefix = platform_install_prefix(&running_exe);
    let cli_path = resolved_binary_path("FOCUSA_CLI_PATH", "focusa", &running_exe);
    let daemon_path = resolved_binary_path("FOCUSA_DAEMON_PATH", "focusa-daemon", &running_exe);
    let tui_path = resolved_binary_path("FOCUSA_TUI_PATH", "focusa-tui", &running_exe);
    let config_home = platform_config_home();
    let source_root = discover_source_root();
    let data_home = env_path("FOCUSA_DATA_DIR", config_home.join("data"));
    let env_file = std::env::var_os("FOCUSA_ENV_FILE")
        .map(PathBuf::from)
        .or_else(|| first_existing([source_root.join(".env"), config_home.join("focusa.env")]))
        .unwrap_or_else(|| config_home.join("focusa.env"));
    let agent_extension = std::env::var_os("FOCUSA_AGENT_EXTENSION_PATH")
        .map(PathBuf::from)
        .or_else(|| {
            let candidate = source_root.join("apps/pi-extension");
            candidate.exists().then_some(candidate)
        });
    let desktop_app = discover_desktop_app();
    let (service_definition, service_overrides, service_manager) = platform_service_paths();
    let mut parts = vec![
        inspect_binary(
            "cli",
            &cli_path.to_string_lossy(),
            &latest,
            query.include_hashes,
        )
        .await,
        inspect_daemon(
            &latest,
            &daemon_path.to_string_lossy(),
            query.include_hashes,
        ),
        inspect_binary(
            "tui",
            &tui_path.to_string_lossy(),
            &latest,
            query.include_hashes,
        )
        .await,
    ];
    parts.extend([
        inspect_optional_path(
            "service_definition",
            service_definition.as_deref(),
            "service_contract_only",
            service_manager,
        ),
        inspect_optional_path(
            "service_overrides",
            service_overrides.as_deref(),
            "preserve_local_overrides",
            service_manager,
        ),
        inspect_protected_path(
            "runtime_home",
            &data_home.to_string_lossy(),
            "never_wholesale_replace",
        ),
        inspect_protected_path("env", &env_file.to_string_lossy(), "never_auto_overwrite"),
        inspect_protected_path(
            "license_files",
            &config_home.to_string_lossy(),
            "validate_never_downgrade",
        ),
        inspect_protected_path(
            "source_checkout",
            &source_root.to_string_lossy(),
            "git_managed_source",
        ),
        inspect_external_part(
            "release_assets",
            "signed release manifest assets",
            "accepted_release_only",
        ),
        inspect_optional_path(
            "desktop_app",
            desktop_app.as_deref(),
            "client_update_channel",
            std::env::consts::OS,
        ),
        inspect_optional_path(
            "agent_extension",
            agent_extension.as_deref(),
            "package_contract_channel",
            "agent_extension",
        ),
        inspect_external_part(
            "public_installer",
            "configured installer release channel",
            "installer_release_only",
        ),
    ]);
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
            "eligibility_status": "unresolved_fail_closed"
        },
        "policy": policy_summary_json(),
        "license": license_summary_json(),
        "inventory_resolution": {
            "platform": std::env::consts::OS,
            "architecture": std::env::consts::ARCH,
            "running_executable": running_exe,
            "install_prefix": prefix,
            "config_home": config_home,
            "data_home": data_home,
            "source_root": source_root,
            "hashes_included": query.include_hashes,
            "hash_opt_in": "?include_hashes=true",
            "environment_overrides": {
                "install_prefix": std::env::var_os("FOCUSA_INSTALL_PREFIX").is_some(),
                "config_dir": std::env::var_os("FOCUSA_CONFIG_DIR").is_some(),
                "data_dir": std::env::var_os("FOCUSA_DATA_DIR").is_some(),
                "source_root": std::env::var_os("FOCUSA_SOURCE_ROOT").is_some(),
                "cli_path": std::env::var_os("FOCUSA_CLI_PATH").is_some(),
                "daemon_path": std::env::var_os("FOCUSA_DAEMON_PATH").is_some(),
                "tui_path": std::env::var_os("FOCUSA_TUI_PATH").is_some(),
                "service_definition": std::env::var_os("FOCUSA_SERVICE_DEFINITION").is_some(),
                "service_overrides": std::env::var_os("FOCUSA_SERVICE_OVERRIDES").is_some(),
                "desktop_app": std::env::var_os("FOCUSA_DESKTOP_APP_PATH").is_some(),
                "agent_extension": std::env::var_os("FOCUSA_AGENT_EXTENSION_PATH").is_some()
            }
        },
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
            "reason": "daemon API is planning authority only; trusted apply executes through the transactional focusa update CLI",
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
    let state_root = update_state_root();
    let pi_restart = [
        "pi-extension-silent-restart-required.json",
        "pi-extension-restart-required.json",
    ]
    .into_iter()
    .find_map(|name| {
        std::fs::read_to_string(state_root.join(name))
            .ok()
            .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
    });
    let severity = if !stale_names.is_empty() || pi_restart.is_some() {
        "warning"
    } else {
        "none"
    };
    let body = if let Some(restart) = &pi_restart {
        format!(
            "Focusa Pi extension {} was updated and will activate silently at the next safe lifecycle boundary.",
            restart
                .get("version")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
        )
    } else if stale_names.is_empty() {
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
        "pi_extension_restart_required": pi_restart,
        "surfaces": notification_routes_json(),
        "messages": [
            {"surface": "cli", "title": "Focusa update status", "body": body, "action": "focusa update plan"},
            {"surface": "api", "title": "Focusa update status", "body": body, "action": "POST /v1/update/plan"},
            {"surface": "pi_doctor", "title": "Focusa update status", "body": body, "action": "focusa update status --json"},
            {"surface": "tui", "title": "Focusa update status", "body": body, "action": "open TUI footer update indicator"},
            {"surface": "menubar", "title": "Focusa update status", "body": body, "action": "open menubar update badge"}
        ],
        "suppress_if": ["version_pinned", "version_skipped", "updates_paused", "offline_without_prior_success"]
    })
}

fn notification_routes_json() -> Value {
    json!({
        "cli": true,
        "api": true,
        "pi_doctor": true,
        "tui": "active_footer_update_indicator",
        "menubar": "active_update_badge"
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
        "blocked_reason": ["admin_mutation_requires_focusa_update_cli", "dry_run_preview_only"]
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
    let mut policy = read_update_policy().unwrap_or_else(|_| default_policy_from_license());
    refresh_update_policy_authority(&mut policy);
    json!({
        "path": path,
        "exists": exists,
        "enabled": policy.enabled,
        "channel": policy.channel.label(),
        "mode": policy.mode.label(),
        "auto_apply_allowed": policy.auto_apply_allowed,
        "auto_apply_blocked_until": policy.auto_apply_blocked_until,
        "note": if exists {
            "policy file loaded; apply still requires release trust, lock, rollback, and health gates"
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
                "note": "policy defaults are derived from license; canonical updater still enforces signed-release and transaction safety gates"
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

fn inspect_optional_path(
    part: &'static str,
    expected_path: Option<&Path>,
    update_policy: &'static str,
    platform_capability: &str,
) -> Value {
    match expected_path {
        Some(path) => {
            let mut value = inspect_protected_path(part, &path.to_string_lossy(), update_policy);
            value["platform_capability"] = json!(platform_capability);
            value
        }
        None => json!({
            "part": part,
            "expected_path": Value::Null,
            "resolved_path": Value::Null,
            "exists": Value::Null,
            "version": Value::Null,
            "version_source": "platform_capability",
            "version_probe_safe": true,
            "sha256": Value::Null,
            "stale": Value::Null,
            "stale_reason": "surface is not file-backed on this platform",
            "update_policy": update_policy,
            "auto_replace_allowed": false,
            "platform_capability": platform_capability,
        }),
    }
}

fn inspect_protected_path(
    part: &'static str,
    expected_path: &str,
    update_policy: &'static str,
) -> Value {
    let path = Path::new(expected_path);
    let exists = path.exists();
    json!({
        "part": part,
        "expected_path": expected_path,
        "resolved_path": if exists { Some(expected_path) } else { None },
        "exists": exists,
        "version": Value::Null,
        "version_source": "protected_or_contract_surface",
        "version_probe_safe": true,
        "sha256": Value::Null,
        "stale": Value::Null,
        "stale_reason": if exists { "protected surface inventoried" } else { "protected surface not present on this platform/install" },
        "update_policy": update_policy,
        "auto_replace_allowed": false,
    })
}

fn inspect_external_part(
    part: &'static str,
    location: &'static str,
    update_policy: &'static str,
) -> Value {
    json!({
        "part": part,
        "expected_path": Value::Null,
        "resolved_path": Value::Null,
        "external_location": location,
        "exists": Value::Null,
        "version": Value::Null,
        "version_source": "signed_manifest_or_external_channel",
        "version_probe_safe": true,
        "sha256": Value::Null,
        "stale": Value::Null,
        "stale_reason": "external surface requires signed channel metadata",
        "update_policy": update_policy,
        "auto_replace_allowed": false,
    })
}

async fn inspect_binary(
    part: &'static str,
    expected_path: &str,
    latest: &Latest,
    include_hash: bool,
) -> Value {
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
        "sha256": if exists && include_hash { sha256_file(path).ok() } else { None },
        "stale": stale,
        "stale_reason": stale_reason(part, version.as_deref(), stale, &latest.version, exists),
    })
}

fn inspect_daemon(latest: &Latest, expected_path: &str, include_hash: bool) -> Value {
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
        "sha256": if exists && include_hash { sha256_file(path).ok() } else { None },
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
                include_hashes: false,
            },
        )
        .await;
        assert_eq!(inventory["continuous_currency"]["enabled"], true);
        assert_eq!(inventory["inventory_resolution"]["hashes_included"], false);
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
