//! Cache CLI — docs/19-intentional-cache-busting.md §3
//!
//! focusa cache status/bust/policy

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand)]
pub enum CacheCmd {
    /// Show cache status.
    Status,
    /// Bust cache with reason.
    Bust {
        /// Bust reason (category A-F or description).
        #[arg(long)]
        reason: String,
    },
    /// Show cache policy.
    Policy,
}

pub async fn run(cmd: CacheCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        CacheCmd::Status => {
            let resp = api.get("/v1/cache/status").await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("Cache Status:");
                println!("  Entries: {}", resp["entry_count"].as_u64().unwrap_or(0));
                println!(
                    "  Hit rate: {:.1}%",
                    resp["hit_rate"].as_f64().unwrap_or(0.0) * 100.0
                );
            }
        }
        CacheCmd::Bust { reason } => {
            // Map reason to bust category for the API.
            let category = match reason.to_lowercase().as_str() {
                "a" | "fresh_evidence" | "fresh" => "FreshEvidence",
                "b" | "authority_change" | "authority" => "AuthorityChange",
                "c" | "compaction" => "Compaction",
                "d" | "staleness" | "stale" => "Staleness",
                "e" | "salience_collapse" | "salience" => "SalienceCollapse",
                "f" | "provider_mismatch" | "provider" => "ProviderMismatch",
                _ => "FreshEvidence", // Default to category A.
            };
            // Use commands API to submit a cache bust.
            let resp = api
                .post(
                    "/v1/commands/submit",
                    &json!({
                        "command": "cache.bust",
                        "args": { "category": category, "reason": reason },
                    }),
                )
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!(
                    "✓ Cache busted (category: {}, reason: {})",
                    category, reason
                );
            }
        }
        CacheCmd::Policy => {
            let resp = api.get("/v1/cache/policy").await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("Cache Policy:");
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
        }
    }
    Ok(())
}
