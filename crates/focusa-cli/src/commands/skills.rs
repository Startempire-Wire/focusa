//! Skills CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum SkillsCmd {
    /// List all agent skills.
    List,
}

pub async fn run(cmd: SkillsCmd, json: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    match cmd {
        SkillsCmd::List => {
            let resp = api.get("/v1/skills").await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                if let Some(skills) = resp["skills"].as_array() {
                    println!("Agent Skills ({} total):", skills.len());
                    for s in skills {
                        let enabled = if s["enabled"].as_bool().unwrap_or(false) { "✓" } else { "✗" };
                        println!("  {} {} — {} ({})",
                            enabled,
                            s["id"].as_str().unwrap_or("?"),
                            s["name"].as_str().unwrap_or("?"),
                            s["category"].as_str().unwrap_or("?"),
                        );
                    }
                }
                if let Some(prohibited) = resp["prohibited"].as_array() {
                    println!("\nProhibited ({}):", prohibited.len());
                    for p in prohibited {
                        println!("  ✗ {}", p.as_str().unwrap_or("?"));
                    }
                }
            }
        }
    }
    Ok(())
}
