//! Spec 128 read-only update inventory/status/check API routes.
//!
//! These routes intentionally do not mutate local state. They inventory local
//! Focusa surfaces and expose stale-part information for CLI/Pi/TUI/menubar
//! consumers before update planning/apply exists.

use axum::extract::Query;
use axum::{
    Json, Router,
    routing::{get, post},
};
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
        .route("/v1/update/check", post(update_check))
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

async fn update_check(Json(body): Json<UpdateQuery>) -> Json<Value> {
    Json(build_update_inventory("check", body).await)
}

async fn update_plan(Json(body): Json<UpdateQuery>) -> Json<Value> {
    let inventory = build_update_inventory("plan", body).await;
    Json(build_update_plan(inventory))
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
    use super::normalize_version;

    #[test]
    fn normalizes_common_version_outputs() {
        assert_eq!(normalize_version("focusa 0.9.74-dev"), "0.9.74-dev");
        assert_eq!(normalize_version("v0.9.80-dev"), "0.9.80-dev");
    }
}
