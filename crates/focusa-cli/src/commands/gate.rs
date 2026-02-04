//! Focus Gate CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand)]
pub enum GateCmd {
    /// List candidates.
    List,
    /// Suppress a candidate.
    Suppress {
        candidate_id: String,
        #[arg(long, default_value = "session")]
        scope: String,
    },
    /// Pin a candidate.
    Pin { candidate_id: String },
}

pub async fn run(cmd: GateCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        GateCmd::List => {
            let resp = api.get("/v1/focus-gate/candidates").await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                let candidates = resp["candidates"].as_array();
                match candidates {
                    Some(c) if c.is_empty() => println!("No candidates"),
                    Some(c) => {
                        for candidate in c {
                            println!(
                                "  {} [p={:.1}] {}",
                                candidate["id"].as_str().unwrap_or("?"),
                                candidate["pressure"].as_f64().unwrap_or(0.0),
                                candidate["label"].as_str().unwrap_or("?"),
                            );
                        }
                    }
                    None => println!("No candidates"),
                }
            }
        }
        GateCmd::Suppress {
            candidate_id,
            scope,
        } => {
            let resp = api
                .post(
                    "/v1/focus-gate/suppress",
                    &json!({"candidate_id": candidate_id, "scope": scope}),
                )
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("✓ Suppressed {}", candidate_id);
            }
        }
        GateCmd::Pin { candidate_id } => {
            let resp = api
                .post(
                    "/v1/focus-gate/pin",
                    &json!({"candidate_id": candidate_id}),
                )
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("✓ Pinned {}", candidate_id);
            }
        }
    }
    Ok(())
}
