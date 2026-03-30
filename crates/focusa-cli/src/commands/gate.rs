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
            duration,
        } => {
            // Map duration to scope (API uses scope internally).
            let scope = match duration.as_str() {
                "permanent" | "forever" => "permanent",
                "session" => "session",
                _ => &duration, // Pass through duration strings like "10m", "1h"
            };
            let resp = api
                .post(
                    "/v1/focus-gate/suppress",
                    &json!({"candidate_id": candidate_id, "scope": scope}),
                )
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("✓ Suppressed {} for {}", candidate_id, duration);
            }
        }
        GateCmd::Pin { candidate_id } => {
            let resp = api
                .post("/v1/focus-gate/pin", &json!({"candidate_id": candidate_id}))
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("✓ Pinned {}", candidate_id);
            }
        }
        GateCmd::Resolve { candidate_id } => {
            // Resolve = suppress permanently (frame scope).
            let resp = api
                .post(
                    "/v1/focus-gate/suppress",
                    &json!({"candidate_id": candidate_id, "scope": "permanent"}),
                )
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("✓ Resolved {}", candidate_id);
            }
        }
        GateCmd::Promote {
            candidate_id,
            beads_issue_id,
        } => {
            // Get candidate info first.
            let candidates_resp = api.get("/v1/focus-gate/candidates").await?;
            let candidate = candidates_resp["candidates"]
                .as_array()
                .and_then(|arr| arr.iter().find(|c| c["id"].as_str() == Some(&candidate_id)))
                .ok_or_else(|| anyhow::anyhow!("Candidate not found"))?;

            let label = candidate["label"].as_str().unwrap_or("Promoted task");
            let description = candidate["description"].as_str().unwrap_or(label);

            // Push a new focus frame.
            let resp = api
                .post(
                    "/v1/focus/push",
                    &json!({
                        "title": label,
                        "goal": description,
                        "beads_issue_id": beads_issue_id,
                        "constraints": [],
                        "tags": ["promoted"],
                    }),
                )
                .await?;

            // Resolve the candidate.
            let _ = api
                .post(
                    "/v1/focus-gate/suppress",
                    &json!({"candidate_id": candidate_id, "scope": "permanent"}),
                )
                .await;

            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("✓ Promoted {} → new focus frame", candidate_id);
            }
        }
    }
    Ok(())
}
