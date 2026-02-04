//! Constitution CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum ConstitutionCmd {
    /// Show active constitution.
    Active,
    /// List all versions.
    Versions,
}

pub async fn run(cmd: ConstitutionCmd, json: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    match cmd {
        ConstitutionCmd::Active => {
            let resp = api.get("/v1/constitution/active").await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else if let Some(err) = resp.get("error") {
                println!("No active constitution: {}", err);
            } else {
                println!("Constitution v{}", resp["version"].as_str().unwrap_or("?"));
                println!("  Agent: {}", resp["agent_id"].as_str().unwrap_or("?"));
                if let Some(principles) = resp["principles"].as_array() {
                    println!("  Principles ({}):", principles.len());
                    for p in principles {
                        println!("    [{}] {}", p["id"].as_str().unwrap_or("?"), p["text"].as_str().unwrap_or("?"));
                    }
                }
            }
        }
        ConstitutionCmd::Versions => {
            let resp = api.get("/v1/constitution/versions").await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                let active = resp["active"].as_str().unwrap_or("none");
                println!("Active: {}", active);
                if let Some(versions) = resp["versions"].as_array() {
                    println!("Versions:");
                    for v in versions {
                        let marker = if v.as_str() == Some(active) { "►" } else { " " };
                        println!("  {} {}", marker, v.as_str().unwrap_or("?"));
                    }
                }
            }
        }
    }
    Ok(())
}
