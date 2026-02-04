//! Proposal CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum ProposalCmd {
    /// List pending proposals.
    List,
    /// Submit a proposal.
    Submit {
        /// Proposal kind: focus_change, thesis_update, autonomy_adjustment, constitution_revision, memory_write.
        #[arg(short, long, default_value = "focus_change")]
        kind: String,
        /// Source identifier.
        #[arg(short, long, default_value = "cli")]
        source: String,
    },
}

pub async fn run(cmd: ProposalCmd, json: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    match cmd {
        ProposalCmd::List => {
            let resp = api.get("/v1/proposals").await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                let pending = resp["pending"].as_u64().unwrap_or(0);
                println!("Pending proposals: {}", pending);
                if let Some(proposals) = resp["proposals"].as_array() {
                    for p in proposals {
                        println!("  {} [{}] score={:.2} status={}",
                            p["id"].as_str().unwrap_or("?"),
                            p["kind"].as_str().unwrap_or("?"),
                            p["score"].as_f64().unwrap_or(0.0),
                            p["status"].as_str().unwrap_or("?"),
                        );
                    }
                }
            }
        }
        ProposalCmd::Submit { kind, source } => {
            let body = serde_json::json!({
                "kind": kind,
                "source": source,
                "payload": {},
                "deadline_ms": 5000,
            });
            let resp = api.post("/v1/proposals", &body).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("Proposal submitted (async — check 'focusa proposals list')");
            }
        }
    }
    Ok(())
}
