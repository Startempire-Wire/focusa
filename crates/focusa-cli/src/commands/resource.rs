//! Spec96 ResourceMode CLI parity commands.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::{Value, json};

#[derive(Subcommand)]
pub enum ResourceCmd {
    /// Read current ResourceMode/LowMem status.
    Status,
    /// Activate LowMem mode override.
    ActivateLowmem {
        #[arg(long)]
        reason: Option<String>,
        #[arg(long)]
        preflight: bool,
    },
    /// Clear LowMem override back to auto.
    DeactivateLowmem {
        #[arg(long)]
        reason: Option<String>,
        #[arg(long)]
        preflight: bool,
    },
    /// Set an explicit resource mode override.
    SetMode {
        #[arg(long)]
        mode: String,
        #[arg(long)]
        reason: Option<String>,
        #[arg(long)]
        preflight: bool,
    },
}

fn print_summary(resp: &Value) {
    let status = resp
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let mode = resp
        .pointer("/resource_mode/mode")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let reason = resp
        .pointer("/resource_mode/reason")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    println!("resource mode: status={status} mode={mode} reason={reason}");
    if let Some(summary) = resp.get("summary").and_then(Value::as_str) {
        println!("  summary: {summary}");
    }
}

pub async fn run(cmd: ResourceCmd, json_output: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let resp = match cmd {
        ResourceCmd::Status => api.get("/v1/resource/mode").await?,
        ResourceCmd::ActivateLowmem { reason, preflight } => api.post("/v1/resource/mode", &json!({ "action": "activate_lowmem", "reason": reason, "preflight": preflight })).await?,
        ResourceCmd::DeactivateLowmem { reason, preflight } => api.post("/v1/resource/mode", &json!({ "action": "deactivate_lowmem", "reason": reason, "preflight": preflight })).await?,
        ResourceCmd::SetMode { mode, reason, preflight } => api.post("/v1/resource/mode", &json!({ "action": "set_mode", "mode": mode, "reason": reason, "preflight": preflight })).await?,
    };
    if json_output {
        println!("{}", serde_json::to_string_pretty(&resp)?);
    } else {
        print_summary(&resp);
    }
    Ok(())
}
