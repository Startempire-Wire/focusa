//! Memory CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::{Value, json};

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

fn advisory_envelope(operation: &str, data: Value) -> Value {
    json!({
        "status": "completed",
        "authority": "daemon_global_advisory",
        "canonical": false,
        "operation": operation,
        "next_step_hint": "This surface needs Spec104 scoped API work before it can be treated as project-canonical.",
        "data": data,
    })
}

fn block_global_memory_mutation(json_mode: bool, operation: &str) -> anyhow::Result<()> {
    let envelope = json!({
        "status": "blocked",
        "failure_class": "project_scope_required",
        "authority": "daemon_global_advisory",
        "canonical": false,
        "operation": operation,
        "next_step_hint": "This memory mutation needs Spec104 scoped API work before it can run safely.",
    });
    if json_mode {
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        println!("Status: blocked");
        println!("Authority: daemon_global_advisory canonical=false");
        println!(
            "Next: This memory mutation needs Spec104 scoped API work before it can run safely."
        );
    }
    Ok(())
}

pub async fn run(cmd: MemoryCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        MemoryCmd::List => {
            let resp = api.get("/v1/memory/semantic").await?;
            let envelope = advisory_envelope("memory list", resp.clone());
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            } else {
                println!("Authority: daemon_global_advisory canonical=false");
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
                println!(
                    "Next: This surface needs Spec104 scoped API work before it can be treated as project-canonical."
                );
            }
        }
        MemoryCmd::Set { key_value } => {
            key_value
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("Format: key=value"))?;
            block_global_memory_mutation(json_mode, "memory set")?;
        }
        MemoryCmd::Rules => {
            let resp = api.get("/v1/memory/procedural").await?;
            let envelope = advisory_envelope("memory rules", resp.clone());
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&envelope)?);
            } else {
                println!("Authority: daemon_global_advisory canonical=false");
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
                println!(
                    "Next: This surface needs Spec104 scoped API work before it can be treated as project-canonical."
                );
            }
        }
        MemoryCmd::Reinforce { rule_id: _ } => {
            block_global_memory_mutation(json_mode, "memory reinforce")?;
        }
    }
    Ok(())
}
