use crate::api_client::ApiClient;
use anyhow::Context;
use clap::Subcommand;
use serde_json::{Value, json};

#[derive(Subcommand, Debug)]
pub enum DaemonRoutingCmd {
    /// Resolve one explicit project/worktree/continuity/session against a registry snapshot.
    Status {
        #[arg(long)]
        registry: std::path::PathBuf,
        #[arg(long)]
        project_root: String,
        #[arg(long)]
        continuity_id: String,
        #[arg(long)]
        working_subpath_id: String,
        #[arg(long)]
        native_session_id: String,
    },
}

pub async fn run(command: DaemonRoutingCmd, output_json: bool) -> anyhow::Result<()> {
    let DaemonRoutingCmd::Status {
        registry,
        project_root,
        continuity_id,
        working_subpath_id,
        native_session_id,
    } = command;
    let registry: Value = serde_json::from_slice(
        &std::fs::read(&registry)
            .with_context(|| format!("read daemon registry {}", registry.display()))?,
    )
    .context("parse daemon registry JSON")?;
    let response = ApiClient::new()
        .post(
            "/v1/daemon-routing/resolve",
            &json!({
                "schema": "focusa.daemon_routing_resolve.v1",
                "registry": registry,
                "route": {
                    "project_root": project_root,
                    "continuity_id": continuity_id,
                    "working_subpath_id": working_subpath_id
                },
                "native_session_id": native_session_id
            }),
        )
        .await?;
    if output_json {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        println!(
            "Focusa daemon routing: status={} daemon={} health={} recovery_required={} failure={}",
            response["status"].as_str().unwrap_or("unresolved"),
            response["selected_daemon_id"].as_str().unwrap_or("none"),
            response["health"].as_str().unwrap_or("unknown"),
            response["recovery_required"].as_bool().unwrap_or(true),
            response["failure_class"].as_str().unwrap_or("none")
        );
        println!(
            "scope: root={} continuity={} subpath={} session={}",
            response
                .pointer("/route/project_root")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            response
                .pointer("/route/continuity_id")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            response
                .pointer("/route/working_subpath_id")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            response["native_session_id"].as_str().unwrap_or("unknown")
        );
    }
    Ok(())
}
