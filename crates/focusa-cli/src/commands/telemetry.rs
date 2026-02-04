//! Telemetry CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum TelemetryCmd {
    /// Show token usage.
    Tokens,
    /// Show cost estimate.
    Cost,
}

pub async fn run(cmd: TelemetryCmd, json: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    match cmd {
        TelemetryCmd::Tokens => {
            let resp = api.get("/v1/telemetry/tokens").await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("Token Usage:");
                println!("  prompt:     {}", resp["total_prompt_tokens"]);
                println!("  completion: {}", resp["total_completion_tokens"]);
                println!("  events:     {}", resp["total_events"]);
            }
        }
        TelemetryCmd::Cost => {
            let resp = api.get("/v1/telemetry/cost").await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("Cost Estimate: ${:.4}", resp["estimated_cost_usd"].as_f64().unwrap_or(0.0));
                println!("  prompt tokens:     {}", resp["prompt_tokens"]);
                println!("  completion tokens: {}", resp["completion_tokens"]);
            }
        }
    }
    Ok(())
}
