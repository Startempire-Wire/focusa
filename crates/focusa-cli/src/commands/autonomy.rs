//! Autonomy CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum AutonomyCmd {
    /// Show autonomy state.
    Status,
    /// Show autonomy history.
    History,
}

pub async fn run(cmd: AutonomyCmd, json: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    match cmd {
        AutonomyCmd::Status => {
            let resp = api.get("/v1/autonomy").await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("Autonomy Level: {:?}", resp["level"]);
                println!(
                    "  ARI score:    {:.1}",
                    resp["ari_score"].as_f64().unwrap_or(0.0)
                );
                println!("  Samples:      {}", resp["sample_count"]);
                if let Some(rec) = resp.get("recommendation")
                    && !rec.is_null()
                {
                    println!("  Recommendation: promote to {:?}", rec);
                }
            }
        }
        AutonomyCmd::History => {
            let resp = api.get("/v1/autonomy/history").await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else if let Some(events) = resp["history"].as_array() {
                println!("Autonomy History ({} events):", events.len());
                for e in events {
                    println!(
                        "  {} → {} ({})",
                        e["from_level"],
                        e["to_level"],
                        e["reason"].as_str().unwrap_or("?")
                    );
                }
            }
        }
    }
    Ok(())
}
