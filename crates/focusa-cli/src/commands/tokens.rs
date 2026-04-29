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
    /// Show Spec92 token-budget doctor.
    Doctor {
        /// Number of recent budget records to inspect.
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
    /// Show recommended compaction/ECS-handle plan for token bloat.
    CompactPlan,
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
        TokensCmd::Doctor { limit } => {
            let resp = api
                .get(&format!("/v1/telemetry/token-budget/status?limit={limit}"))
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("Status: {}", resp["status"].as_str().unwrap_or("watch"));
                println!(
                    "Summary: {}",
                    resp["summary"]
                        .as_str()
                        .unwrap_or("Token telemetry unavailable")
                );
                println!(
                    "Next action: {}",
                    resp["next_action"]
                        .as_str()
                        .unwrap_or("Run a provider turn, then re-run focusa tokens doctor")
                );
                println!("Why: token budget health controls context bloat before provider calls");
                println!("Command: focusa tokens compact-plan");
                println!("Recovery: use ECS handles for large tool outputs, then compact");
                println!("Evidence: docs/current/EFFICIENCY_GUIDE.md");
                println!("Docs: docs/92-agent-first-polish-hooks-efficiency-spec.md");
            }
        }
        TokensCmd::CompactPlan => {
            let resp = api
                .get("/v1/telemetry/token-budget/status?limit=20")
                .await?;
            if json_mode {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "status": resp["status"],
                        "summary": "Token compaction plan",
                        "next_action": "Store large evidence as ECS handles, summarize tool outputs, then compact before the next high-risk provider call.",
                        "why": "Spec92 prefers bounded context and evidence refs over raw transcript blobs.",
                        "commands": ["focusa ecs", "focusa tokens doctor", "focusa_workpoint_checkpoint mission=\"...\" next_action=\"...\""],
                        "recovery": ["focusa doctor", "focusa cache doctor"],
                        "evidence_refs": ["docs/current/EFFICIENCY_GUIDE.md"],
                        "docs": ["docs/92-agent-first-polish-hooks-efficiency-spec.md"],
                        "warnings": [],
                        "details": resp,
                    }))?
                );
            } else {
                println!("Status: {}", resp["status"].as_str().unwrap_or("watch"));
                println!("Summary: Token compaction plan");
                println!(
                    "Next action: Store large evidence as ECS handles, summarize tool outputs, then compact before the next high-risk provider call."
                );
                println!("Why: bounded context and evidence refs beat raw transcript blobs");
                println!(
                    "Command: focusa_workpoint_checkpoint mission=\"...\" next_action=\"...\""
                );
                println!("Recovery: focusa doctor && focusa tokens doctor");
                println!("Evidence: docs/current/EFFICIENCY_GUIDE.md");
                println!("Docs: docs/92-agent-first-polish-hooks-efficiency-spec.md");
            }
        }
    }
    Ok(())
}
