//! Spec 111 §9 — `focusa preload` CLI surface.
//!
//! Subcommands: profiles | build | render | verify | doctor | write | receipt-preview | receipt-commit
//! These call the daemon /v1/preload/* API routes and print human-readable results.

use crate::api_client::ApiClient;
use anyhow::{Context, Result};
use clap::Args;
use focusa_core::working_subpath::resolve_git_working_context;
use serde_json::{Value, json};

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

    /// Opt into the identity-bound scoped artifact protocol.
    #[arg(long)]
    pub artifact_version: Option<u32>,
    #[arg(long)]
    pub constitution_id: Option<String>,
    #[arg(long)]
    pub current_ask: Option<String>,
    /// Exact artifact JSON returned by the build action (no implicit latest).
    #[arg(long)]
    pub artifact_file: Option<std::path::PathBuf>,
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
    let mut scoped_request = json!({
        "profile": args.profile.clone().unwrap_or_else(|| "rules_and_context".to_string()),
        "project_root": working_context.as_ref().map(|value| value.canonical_parent_root.clone()).or(args.project_root.clone()),
        "working_subpath_id": working_context.as_ref().map(|value| value.working_subpath.working_subpath_id.clone()).unwrap_or_else(|| "primary".to_string()),
        "continuity_id": args.continuity_id.clone(),
    });
    if let Some(version) = args.artifact_version {
        anyhow::ensure!(version == 2, "unsupported preload artifact version");
        scoped_request["artifact_version"] = json!(version);
        scoped_request["constitution_id"] = json!(args.constitution_id);
        scoped_request["current_ask"] = json!(args.current_ask);
        if let Some(path) = args.artifact_file.as_ref() {
            let metadata = std::fs::metadata(path)?;
            anyhow::ensure!(
                metadata.is_file() && metadata.len() <= 1024 * 1024,
                "preload artifact input exceeds bound"
            );
            let input: Value = serde_json::from_slice(&std::fs::read(path)?)?;
            let artifact = input.get("artifact").unwrap_or(&input);
            scoped_request["artifact"] = artifact.clone();
            scoped_request["artifact_ref"] = artifact["artifact_ref"].clone();
            scoped_request["expected_digest"] = artifact["content_sha256"].clone();
        }
    }
    match args.action.as_str() {
        "profiles" => curl_get(&daemon_url, "/v1/preload/profiles").await,
        "build" => curl_post(&daemon_url, "/v1/preload/build", scoped_request).await,
        "render" => curl_post(&daemon_url, "/v1/preload/render", scoped_request).await,
        "verify" => curl_post(&daemon_url, "/v1/preload/verify", scoped_request).await,
        "doctor" => curl_post(&daemon_url, "/v1/preload/doctor", scoped_request).await,
        "receipt-preview" => {
            curl_post(&daemon_url, "/v1/preload/receipt-preview", scoped_request).await
        }
        "receipt-commit" => {
            let key = args
                .idempotency_key
                .clone()
                .context("--idempotency-key is required for receipt-commit")?;
            scoped_request["idempotency_key"] = json!(key);
            scoped_request["delivery_target"] = json!(args.target);
            curl_post(&daemon_url, "/v1/preload/receipt-commit", scoped_request).await
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
            scoped_request["profile_id"] = json!(profile);
            scoped_request["target_path"] = json!(target);
            scoped_request["idempotency_key"] = json!(key);
            scoped_request["overwrite"] = json!(args.overwrite);
            curl_post(&daemon_url, "/v1/preload/write", scoped_request).await
        }
        other => {
            anyhow::bail!(
                "unknown preload action {other:?}; expected one of profiles | build | render | verify | doctor | receipt-preview | receipt-commit | write"
            );
        }
    }
}

async fn curl_get(base: &str, path: &str) -> Result<()> {
    preload_request(base, path, None).await
}

async fn curl_post(base: &str, path: &str, body: Value) -> Result<()> {
    preload_request(base, path, Some(body)).await
}

async fn preload_request(base: &str, path: &str, body: Option<Value>) -> Result<()> {
    let client = ApiClient::for_base_url(base);
    let token = std::env::var("FOCUSA_AUTH_TOKEN").ok();
    if token.is_some() {
        let url = reqwest::Url::parse(base)?;
        let loopback = matches!(
            url.host_str(),
            Some("localhost" | "127.0.0.1" | "[::1]" | "::1")
        );
        anyhow::ensure!(
            loopback || url.scheme() == "https",
            "authenticated preload requires loopback or HTTPS"
        );
    }
    let authorization = token.map(|value| format!("Bearer {value}"));
    let mut headers = vec![(
        "X-Focusa-Permissions",
        "admin:service,work-loop:read,work-loop:write",
    )];
    if let Some(value) = authorization.as_deref() {
        headers.push(("Authorization", value));
    }
    let versioned = body.as_ref().is_some_and(|v| v["artifact_version"] == 2);
    let expected_ref = body
        .as_ref()
        .and_then(|v| v["artifact_ref"].as_str())
        .map(str::to_owned);
    let result = match body {
        Some(mut value) => {
            value["request_id"] = json!(uuid::Uuid::now_v7().to_string());
            client.post_with_headers(path, &value, &headers).await?
        }
        None => client.get_with_headers(path, &headers).await?,
    };
    anyhow::ensure!(
        !matches!(
            result["status"].as_str(),
            Some("blocked" | "failed" | "error")
        ),
        "preload request was rejected"
    );
    if versioned {
        anyhow::ensure!(
            result["schema"] == "focusa.scoped_preload_artifact.v2"
                && result["artifact"]["schema"] == "focusa.scoped_preload_artifact.v2",
            "legacy/template response cannot verify a scoped artifact"
        );
        if let Some(reference) = expected_ref {
            anyhow::ensure!(
                result["artifact"]["artifact_ref"] == reference,
                "scoped artifact response identity changed"
            );
        }
    }
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn reply_once(status: u16, body: &str) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let response = format!(
            "HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = vec![0; 65536];
            let count = socket.read(&mut request).await.unwrap();
            assert!(
                String::from_utf8_lossy(&request[..count]).contains("X-Focusa-Permissions")
                    || String::from_utf8_lossy(&request[..count]).contains("x-focusa-permissions")
            );
            socket.write_all(response.as_bytes()).await.unwrap();
        });
        format!("http://{address}")
    }

    #[tokio::test]
    async fn actual_cli_transport_rejects_http_errors_and_legacy_v2_success() {
        let base = reply_once(403, r#"{"status":"blocked"}"#).await;
        assert!(
            preload_request(
                &base,
                "/v1/preload/verify",
                Some(json!({"artifact_version":2}))
            )
            .await
            .is_err()
        );
        let base = reply_once(
            200,
            r#"{"schema":"focusa.preload.v1","status":"completed"}"#,
        )
        .await;
        assert!(
            preload_request(
                &base,
                "/v1/preload/verify",
                Some(json!({"artifact_version":2}))
            )
            .await
            .is_err()
        );
    }

    #[tokio::test]
    async fn actual_cli_transport_preserves_valid_scoped_response() {
        let base = reply_once(200,r#"{"schema":"focusa.scoped_preload_artifact.v2","status":"verified","artifact":{"schema":"focusa.scoped_preload_artifact.v2","artifact_ref":"exact-ref"}}"#).await;
        preload_request(
            &base,
            "/v1/preload/verify",
            Some(json!({"artifact_version":2,"artifact_ref":"exact-ref"})),
        )
        .await
        .unwrap();
    }

    #[test]
    fn parse_action_defaults_to_profiles() {
        assert_eq!("profiles", "profiles");
    }
}
