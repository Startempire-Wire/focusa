//! Thread management commands.

use crate::api_client::ApiClient;
use anyhow::{Context, Result};
use clap::Subcommand;
use serde_json::Value;

/// Thread management commands (docs/38).
#[derive(Subcommand)]
pub enum ThreadCmd {
    /// List all threads
    List,

    /// Create a new thread
    Create {
        /// Thread name
        #[arg(short, long)]
        name: String,

        /// Primary intent/thesis
        #[arg(short, long)]
        intent: String,

        /// Owner machine ID (optional)
        #[arg(short, long)]
        owner: Option<String>,
    },

    /// Get thread details
    Get {
        /// Thread ID
        thread_id: String,
    },

    /// Transfer thread ownership
    Transfer {
        /// Thread ID
        thread_id: String,

        /// New owner machine ID
        #[arg(short, long)]
        to: String,

        /// Reason for transfer (optional)
        #[arg(short, long)]
        reason: Option<String>,
    },
}

pub async fn run(cmd: ThreadCmd, _json: bool, client: &ApiClient) -> Result<()> {
    match cmd {
        ThreadCmd::List => {
            let response: Value = client
                .get("/v1/threads")
                .await
                .context("Failed to fetch threads")?;

            let threads = response["threads"].as_array().cloned().unwrap_or_default();

            if threads.is_empty() {
                println!("No threads found.");
                return Ok(());
            }

            println!("Threads:");
            println!("{:>36} {:<20} {:<12} Owner", "ID", "Name", "Status");
            println!("{}", "-".repeat(80));
            for thread in threads {
                let id = thread["id"].as_str().unwrap_or("?");
                let name = thread["name"].as_str().unwrap_or("?");
                let status = thread["status"].as_str().unwrap_or("?");
                let owner = thread["owner_machine_id"].as_str().unwrap_or("unowned");
                println!("{} {:<20} {:<12} {}", &id[..8], name, status, owner);
            }
        }

        ThreadCmd::Create {
            name,
            intent,
            owner,
        } => {
            let payload = serde_json::json!({
                "name": name,
                "primary_intent": intent,
                "owner_machine_id": owner
            });

            let response: Value = client
                .post("/v1/threads", &payload)
                .await
                .context("Failed to create thread")?;

            let thread = &response["thread"];
            println!("Created thread:");
            println!("  ID:   {}", thread["id"].as_str().unwrap_or("?"));
            println!("  Name: {}", thread["name"].as_str().unwrap_or("?"));
            if let Some(owner) = thread["owner_machine_id"].as_str() {
                println!("  Owner: {}", owner);
            }
        }

        ThreadCmd::Get { thread_id } => {
            let response: Value = client
                .get(&format!("/v1/threads/{}", thread_id))
                .await
                .context("Failed to fetch thread")?;

            let thread = &response["thread"];
            println!("Thread: {}", thread["name"].as_str().unwrap_or("?"));
            println!("  ID:           {}", thread["id"].as_str().unwrap_or("?"));
            println!(
                "  Status:       {}",
                thread["status"].as_str().unwrap_or("?")
            );
            println!(
                "  Created:      {}",
                thread["created_at"].as_str().unwrap_or("?")
            );
            if let Some(owner) = thread["owner_machine_id"].as_str() {
                println!("  Owner:        {}", owner);
            }
        }

        ThreadCmd::Transfer {
            thread_id,
            to,
            reason,
        } => {
            let payload = serde_json::json!({
                "to_machine_id": to,
                "reason": reason
            });

            let response: Value = client
                .post(&format!("/v1/threads/{}/transfer", thread_id), &payload)
                .await
                .context("Failed to transfer thread ownership")?;

            println!("Transferred ownership:");
            println!(
                "  Thread ID:    {}",
                response["thread_id"].as_str().unwrap_or("?")
            );
            println!(
                "  Previous:     {}",
                response["previous_owner"].as_str().unwrap_or("none")
            );
            println!(
                "  New Owner:    {}",
                response["new_owner"].as_str().unwrap_or("?")
            );
            if let Some(reason) = response["reason"].as_str() {
                println!("  Reason:       {}", reason);
            }
        }
    }

    Ok(())
}
