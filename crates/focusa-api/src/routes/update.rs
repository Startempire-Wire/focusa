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
}

#[derive(Debug, Deserialize, Default)]
struct UpdateQuery {
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    latest_version: Option<String>,
}

async fn update_status(Query(query): Query<UpdateQuery>) -> Json<Value> {
    Json(build_update_inventory("status", query).await)
}

async fn update_check(Json(body): Json<UpdateQuery>) -> Json<Value> {
    Json(build_update_inventory("check", body).await)
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
        "policy": {
            "path": "/usr/local/lib/focusa/update-policy.json",
            "exists": Path::new("/usr/local/lib/focusa/update-policy.json").exists(),
            "enabled": false,
            "mode": "manual",
            "note": "Spec128 policy read/write is not implemented yet; auto-apply remains disabled"
        },
        "license": {
            "level": "unknown",
            "dev_mode": false,
            "source": "not_wired_in_update_status_yet",
            "note": "Spec128 license/dev_mode policy integration is next; this route does not grant auto-update authority"
        },
        "parts": parts,
        "stale_parts": stale_parts,
        "warnings": [
            "read-only inventory only; no update apply, download, binary replacement, or daemon restart was attempted",
            "release manifest eligibility/signature/provenance is required before trusted auto-apply"
        ],
        "next_tools": ["focusa update status --json", "focusa update check --channel dev --json"]
    })
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
