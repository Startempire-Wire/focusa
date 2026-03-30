//! Token CLI — docs/25-capability-permissions.md §5

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand)]
pub enum TokensCmd {
    /// Create a new API token.
    Create {
        /// Token type: owner, agent, integration.
        #[arg(long)]
        token_type: String,
        /// Scopes (domain:action), comma-separated.
        #[arg(long, default_value = "read:*")]
        scopes: String,
        /// TTL in seconds (optional).
        #[arg(long)]
        ttl: Option<u64>,
    },
    /// Revoke a token.
    Revoke {
        /// Token ID to revoke.
        token_id: String,
    },
    /// List active tokens.
    List,
}

pub async fn run(cmd: TokensCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        TokensCmd::Create {
            token_type,
            scopes,
            ttl,
        } => {
            let scope_list: Vec<serde_json::Value> = scopes
                .split(',')
                .filter_map(|s| {
                    let parts: Vec<&str> = s.trim().splitn(2, ':').collect();
                    if parts.len() == 2 {
                        Some(json!({"domain": parts[0], "action": parts[1]}))
                    } else {
                        None
                    }
                })
                .collect();
            let mut body = json!({
                "token_type": token_type,
                "scopes": scope_list,
            });
            if let Some(t) = ttl {
                body["ttl_secs"] = json!(t);
            }
            let resp = api.post("/v1/tokens/create", &body).await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!(
                    "✓ Created {} token: {}",
                    token_type,
                    resp["token_id"].as_str().unwrap_or("?")
                );
            }
        }
        TokensCmd::Revoke { token_id } => {
            let resp = api
                .post("/v1/tokens/revoke", &json!({"token_id": token_id}))
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("✓ Revoked {}", token_id);
            }
        }
        TokensCmd::List => {
            let resp = api.get("/v1/tokens/list").await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                if let Some(tokens) = resp["tokens"].as_array() {
                    if tokens.is_empty() {
                        println!("No active tokens");
                    } else {
                        for t in tokens {
                            println!(
                                "  {} [{}] created={} expires={}",
                                t["token_id"].as_str().unwrap_or("?"),
                                t["token_type"].as_str().unwrap_or("?"),
                                t["created_at"].as_str().unwrap_or("?"),
                                t["expires_at"].as_str().unwrap_or("never"),
                            );
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
