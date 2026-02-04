//! Memory CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand)]
pub enum MemoryCmd {
    /// List semantic memory.
    List,
    /// Set a semantic key=value.
    Set {
        /// Format: key=value
        key_value: String,
    },
    /// Show procedural rules.
    Rules,
    /// Reinforce a procedural rule.
    Reinforce {
        /// Rule ID.
        rule_id: String,
    },
}

pub async fn run(cmd: MemoryCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        MemoryCmd::List => {
            let resp = api.get("/v1/memory/semantic").await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                let records = resp["semantic"].as_array();
                match records {
                    Some(r) if r.is_empty() => println!("No semantic memory"),
                    Some(r) => {
                        for rec in r {
                            println!(
                                "  {} = {}",
                                rec["key"].as_str().unwrap_or("?"),
                                rec["value"].as_str().unwrap_or("?"),
                            );
                        }
                    }
                    None => println!("No semantic memory"),
                }
            }
        }
        MemoryCmd::Set { key_value } => {
            let (key, value) = key_value
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("Format: key=value"))?;

            let resp = api
                .post(
                    "/v1/memory/semantic/upsert",
                    &json!({"key": key, "value": value}),
                )
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("✓ Set {} = {}", key, value);
            }
        }
        MemoryCmd::Rules => {
            let resp = api.get("/v1/memory/procedural").await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                let rules = resp["procedural"].as_array();
                match rules {
                    Some(r) if r.is_empty() => println!("No procedural rules"),
                    Some(r) => {
                        for rule in r {
                            println!(
                                "  [w={:.2}] {}",
                                rule["weight"].as_f64().unwrap_or(0.0),
                                rule["rule"].as_str().unwrap_or("?"),
                            );
                        }
                    }
                    None => println!("No procedural rules"),
                }
            }
        }
        MemoryCmd::Reinforce { rule_id } => {
            let resp = api
                .post(
                    "/v1/memory/procedural/reinforce",
                    &json!({"rule_id": rule_id}),
                )
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("✓ Reinforced {}", rule_id);
            }
        }
    }
    Ok(())
}
