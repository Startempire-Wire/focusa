//! Contribute CLI — docs/22-data-contribution.md §9
//!
//! focusa contribute status/enable/pause/review/purge

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand)]
pub enum ContributeCmd {
    /// Show contribution status (queue size, last upload, policy).
    Status,
    /// Enable contribution (explicit opt-in per docs/22 §3.1).
    Enable,
    /// Pause contribution.
    Pause,
    /// Review contribution queue.
    Review,
    /// Approve a contribution item.
    Approve {
        /// Item ID to approve.
        item_id: String,
    },
    /// Submit approved items.
    Submit,
}

pub async fn run(cmd: ContributeCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        ContributeCmd::Status => {
            let resp = api.get("/v1/training/status").await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("Contribution Status:");
                println!(
                    "  Enabled: {}",
                    resp["contribution_enabled"].as_bool().unwrap_or(false)
                );
                println!(
                    "  Queue: {} items",
                    resp["queue_size"].as_u64().unwrap_or(0)
                );
                println!(
                    "  Total contributed: {}",
                    resp["total_contributed"].as_u64().unwrap_or(0)
                );
            }
        }
        ContributeCmd::Enable => {
            let resp = api.post("/v1/contribute/enable", &json!({})).await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("✓ Contribution enabled");
            }
        }
        ContributeCmd::Pause => {
            let resp = api.post("/v1/contribute/pause", &json!({})).await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("✓ Contribution paused");
            }
        }
        ContributeCmd::Review => {
            let resp = api.get("/v1/contribute/queue").await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                let count = resp["count"].as_u64().unwrap_or(0);
                println!("Contribution Queue ({} items):", count);
                if let Some(items) = resp["queue"].as_array() {
                    for item in items {
                        println!(
                            "  {} [{}] {} ({})",
                            item["id"].as_str().unwrap_or("?"),
                            item["status"].as_str().unwrap_or("?"),
                            item["dataset_family"].as_str().unwrap_or("?"),
                            item["created_at"].as_str().unwrap_or("?"),
                        );
                    }
                }
            }
        }
        ContributeCmd::Approve { item_id } => {
            let resp = api
                .post("/v1/contribute/approve", &json!({"item_id": item_id}))
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("✓ Approved {}", item_id);
            }
        }
        ContributeCmd::Submit => {
            let resp = api.post("/v1/contribute/submit", &json!({})).await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!(
                    "✓ Submitted {} items",
                    resp["submitted"].as_u64().unwrap_or(0)
                );
            }
        }
    }
    Ok(())
}
