//! RFM CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum RfmCmd {
    /// Show RFM status.
    Status,
}

pub async fn run(cmd: RfmCmd, json: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    match cmd {
        RfmCmd::Status => {
            let resp = api.get("/v1/rfm").await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("RFM Level:  {:?}", resp["level"]);
                println!(
                    "  AIS:      {:.2}",
                    resp["ais_score"].as_f64().unwrap_or(1.0)
                );
                println!("  regen:    {}", resp["needs_regeneration"]);
                println!("  ensemble: {}", resp["needs_ensemble"]);
            }
        }
    }
    Ok(())
}
