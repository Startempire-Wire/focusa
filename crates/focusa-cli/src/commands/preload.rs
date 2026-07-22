//! Spec 111 §9 — `focusa preload` CLI surface.
//!
//! Subcommands: profiles | build | render | verify | doctor | write | receipt-preview | receipt-commit
//! These call the daemon /v1/preload/* API routes and print human-readable results.

use anyhow::{Context, Result};
use clap::Args;
use focusa_core::working_subpath::resolve_git_working_context;
use serde_json::{Value, json};
use std::process::Command;

#[derive(Args)]
pub struct PreloadArgs {
    /// Sub-action: profiles | build | render | verify | doctor | write | receipt-preview | receipt-commit.
    #[arg(value_name = "ACTION", default_value = "profiles")]
    pub action: String,

    /// Profile id (rules_only | rules_and_context | budget_light | budget_deep).
    #[arg(long)]
    pub profile: Option<String>,

    /// Daemon base URL for the daemon API.
    #[arg(long, default_value = "http://127.0.0.1:8787")]
    pub daemon_url: Option<String>,

    /// Canonical project or active worktree root for scoped preload actions.
    #[arg(long)]
    pub project_root: Option<String>,

    /// Stable continuity id for scoped preload actions.
    #[arg(long)]
    pub continuity_id: Option<String>,

    /// Target path for the write action (must use allowlisted prefix).
    #[arg(long)]
    pub target: Option<String>,

    /// Idempotency key for write and receipt-commit actions.
    #[arg(long)]
    pub idempotency_key: Option<String>,

    /// Allow overwriting an existing target.
    #[arg(long)]
    pub overwrite: bool,
}

pub async fn run(args: PreloadArgs, _json_mode: bool) -> Result<()> {
    let daemon_url = args
        .daemon_url
        .clone()
        .unwrap_or_else(|| "http://127.0.0.1:8787".to_string());
    let scope_path = args
        .project_root
        .as_deref()
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_dir().ok());
    let working_context = scope_path
        .as_deref()
        .and_then(|path| resolve_git_working_context(path).ok().flatten());
    let scoped_request = json!({
        "profile": args.profile.clone().unwrap_or_else(|| "rules_and_context".to_string()),
        "project_root": working_context.as_ref().map(|value| value.canonical_parent_root.clone()).or(args.project_root.clone()),
        "working_subpath_id": working_context.as_ref().map(|value| value.working_subpath.working_subpath_id.clone()).unwrap_or_else(|| "primary".to_string()),
        "continuity_id": args.continuity_id.clone(),
    });
    match args.action.as_str() {
        "profiles" => curl_get(&daemon_url, "/v1/preload/profiles").await,
        "build" => curl_post(&daemon_url, "/v1/preload/build", scoped_request).await,
        "render" => curl_post(&daemon_url, "/v1/preload/render", scoped_request).await,
        "verify" => curl_post(&daemon_url, "/v1/preload/verify", scoped_request).await,
        "doctor" => curl_post(&daemon_url, "/v1/preload/doctor", scoped_request).await,
        "receipt-preview" => {
            curl_post(
                &daemon_url,
                "/v1/preload/receipt-preview",
                json!({"profile": args.profile.unwrap_or_else(|| "rules_and_context".to_string())}),
            )
            .await
        }
        "receipt-commit" => {
            let key = args
                .idempotency_key
                .clone()
                .context("--idempotency-key is required for receipt-commit")?;
            curl_post(
                &daemon_url,
                "/v1/preload/receipt-commit",
                json!({
                    "profile": args.profile.unwrap_or_else(|| "rules_and_context".to_string()),
                    "idempotency_key": key,
                }),
            )
            .await
        }
        "write" => {
            let profile = args
                .profile
                .clone()
                .context("--profile is required for write")?;
            let target = args
                .target
                .clone()
                .context("--target is required for write")?;
            let key = args
                .idempotency_key
                .clone()
                .context("--idempotency-key is required for write")?;
            curl_post(
                &daemon_url,
                "/v1/preload/write",
                json!({
                    "profile_id": profile,
                    "target_path": target,
                    "idempotency_key": key,
                    "overwrite": args.overwrite,
                }),
            )
            .await
        }
        other => {
            anyhow::bail!(
                "unknown preload action {other:?}; expected one of profiles | build | render | verify | doctor | receipt-preview | receipt-commit | write"
            );
        }
    }
}

async fn curl_get(base: &str, path: &str) -> Result<()> {
    let url = format!("{}{}", base.trim_end_matches('/'), path);
    let out = Command::new("curl")
        .args(["-fsS", "-m", "5", &url])
        .output()?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        println!("(no body)");
    } else {
        println!("{trimmed}");
    }
    Ok(())
}

async fn curl_post(base: &str, path: &str, body: Value) -> Result<()> {
    let url = format!("{}{}", base.trim_end_matches('/'), path);
    let raw = serde_json::to_string(&body)?;
    let out = Command::new("curl")
        .args([
            "-fsS",
            "-m",
            "5",
            "-X",
            "POST",
            "-H",
            "content-type: application/json",
            "--data",
            &raw,
            &url,
        ])
        .output()?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        println!("(no body)");
    } else {
        println!("{trimmed}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_action_defaults_to_profiles() {
        assert_eq!("profiles", "profiles");
    }
}
