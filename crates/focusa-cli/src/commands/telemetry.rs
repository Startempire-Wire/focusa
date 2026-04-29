//! Telemetry CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum TelemetryCmd {
    /// Show token usage.
    Tokens,
    /// Show Spec92 token budget telemetry.
    TokenBudget {
        /// Number of recent records to return.
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
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
        TelemetryCmd::TokenBudget { limit } => {
            let resp = api
                .get(&format!("/v1/telemetry/token-budget/status?limit={limit}"))
                .await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!(
                    "Token Budget: {}",
                    resp["status"].as_str().unwrap_or("unknown")
                );
                println!(
                    "  summary: {}",
                    resp["summary"].as_str().unwrap_or("unknown")
                );
                println!("  records: {}", resp["record_count"].as_u64().unwrap_or(0));
                println!(
                    "  next: {}",
                    resp["next_action"].as_str().unwrap_or("monitor")
                );
            }
        }
        TelemetryCmd::Cost => {
            let resp = api.get("/v1/telemetry/cost").await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!(
                    "Cost Estimate: ${:.4}",
                    resp["estimated_cost_usd"].as_f64().unwrap_or(0.0)
                );
                println!("  prompt tokens:     {}", resp["prompt_tokens"]);
                println!("  completion tokens: {}", resp["completion_tokens"]);
            }
        }
    }
    Ok(())
}
