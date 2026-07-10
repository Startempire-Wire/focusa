//! CLT CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::{Value, json};

#[derive(Subcommand)]
pub enum CltCmd {
    /// Show CLT path from head to root.
    Path,
    /// Show CLT statistics.
    Stats,
    /// List all CLT nodes.
    Nodes,
}

fn advisory_json(data: Value) -> Value {
    json!({
        "status": "completed",
        "authority": "daemon_global_advisory",
        "canonical": false,
        "next_step_hint": "This surface needs Spec104 scoped API work before it can be treated as project-canonical.",
        "data": data,
    })
}

pub async fn run(cmd: CltCmd, json: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    if !json {
        println!("Authority: daemon_global_advisory canonical=false");
    }
    match cmd {
        CltCmd::Path => {
            let resp = api.get("/v1/clt/path").await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&advisory_json(resp))?);
            } else {
                let depth = resp["depth"].as_u64().unwrap_or(0);
                println!("Lineage depth: {}", depth);
                if let Some(path) = resp["path"].as_array() {
                    for (i, id) in path.iter().enumerate() {
                        let prefix = if i == 0 { "HEAD →" } else { "     →" };
                        println!("  {} {}", prefix, id.as_str().unwrap_or("?"));
                    }
                }
            }
        }
        CltCmd::Stats => {
            let resp = api.get("/v1/clt/stats").await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&advisory_json(resp))?);
            } else {
                println!("CLT Statistics:");
                println!("  interactions:   {}", resp["interactions"]);
                println!("  summaries:      {}", resp["summaries"]);
                println!("  branch markers: {}", resp["branch_markers"]);
                println!("  total:          {}", resp["total"]);
            }
        }
        CltCmd::Nodes => {
            let resp = api.get("/v1/clt/nodes").await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&advisory_json(resp))?);
            } else {
                let total = resp["total"].as_u64().unwrap_or(0);
                println!("CLT Nodes ({} total):", total);
                if let Some(nodes) = resp["nodes"].as_array() {
                    for n in nodes {
                        println!(
                            "  {} [{}] parent={}",
                            n["node_id"].as_str().unwrap_or("?"),
                            n["node_type"].as_str().unwrap_or("?"),
                            n["parent_id"].as_str().unwrap_or("root"),
                        );
                    }
                }
            }
        }
    }
    Ok(())
}
