//! Env export CLI.

use crate::api_client::ApiClient;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum EnvCmd {
    /// Print env exports for shell integration.
    Shell,
}

pub async fn run(cmd: EnvCmd, json: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let resp = api.get("/v1/env").await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&resp)?);
        return Ok(());
    }

    match cmd {
        EnvCmd::Shell => {
            let exports = [
                ("ANTHROPIC_BASE_URL", resp["anthropic_base_url"].as_str()),
                ("KIMI_BASE_URL", resp["kimi_base_url"].as_str()),
                (
                    "KIMI_ANTHROPIC_BASE_URL",
                    resp["kimi_anthropic_base_url"].as_str(),
                ),
                ("OPENAI_BASE_URL", resp["openai_base_url"].as_str()),
            ];
            for (key, val) in exports {
                if let Some(v) = val {
                    println!("export {}=\"{}\"", key, v);
                }
            }
        }
    }

    Ok(())
}
