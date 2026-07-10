//! Focus Gate CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::{Value, json};

#[derive(Subcommand)]
pub enum GateCmd {
    /// List candidates.
    List,
    /// Suppress a candidate.
    Suppress {
        candidate_id: String,
        /// Duration to suppress (e.g., "10m", "1h", "session", "permanent").
        #[arg(long = "for", default_value = "session")]
        duration: String,
    },
    /// Pin a candidate.
    Pin { candidate_id: String },
    /// Resolve a candidate (mark as addressed).
    Resolve { candidate_id: String },
    /// Promote a candidate to a focus frame.
    Promote {
        candidate_id: String,
        /// Beads issue ID for the new frame.
        #[arg(long)]
        beads_issue_id: String,
    },
}

fn advisory_envelope(data: Value) -> Value {
    json!({
        "status": "completed",
        "authority": "daemon_global_advisory",
        "canonical": false,
        "next_step_hint": "This surface needs Spec104 scoped API work before it can be treated as project-canonical.",
        "data": data,
    })
}

fn block_global_gate_mutation(
    json_mode: bool,
    operation: &str,
    candidate_id: &str,
) -> anyhow::Result<()> {
    let envelope = json!({
        "status": "blocked",
        "failure_class": "project_scope_required",
        "authority": "daemon_global_advisory",
        "canonical": false,
        "operation": operation,
        "candidate_id": candidate_id,
        "next_step_hint": "This Focus Gate mutation needs Spec104 scoped API work before it can run safely.",
    });
    if json_mode {
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        println!("Status: blocked");
        println!("Authority: daemon_global_advisory canonical=false");
        println!(
            "Next: This Focus Gate mutation needs Spec104 scoped API work before it can run safely."
        );
    }
    Ok(())
}

pub async fn run(cmd: GateCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        GateCmd::List => {
            let resp = api.get("/v1/focus-gate/candidates").await?;
            if json_mode {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&advisory_envelope(resp.clone()))?
                );
            } else {
                println!("Authority: daemon_global_advisory canonical=false");
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
            duration: _,
        } => {
            block_global_gate_mutation(json_mode, "gate suppress", &candidate_id)?;
        }
        GateCmd::Pin { candidate_id } => {
            block_global_gate_mutation(json_mode, "gate pin", &candidate_id)?;
        }
        GateCmd::Resolve { candidate_id } => {
            block_global_gate_mutation(json_mode, "gate resolve", &candidate_id)?;
        }
        GateCmd::Promote {
            candidate_id,
            beads_issue_id: _,
        } => {
            block_global_gate_mutation(json_mode, "gate promote", &candidate_id)?;
        }
    }
    Ok(())
}
