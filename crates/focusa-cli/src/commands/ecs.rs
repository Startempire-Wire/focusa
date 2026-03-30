//! ECS (reference store) CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand)]
pub enum EcsCmd {
    /// List all handles.
    List,
    /// Store an artifact from a file.
    Store {
        /// File path to store.
        path: String,
        /// Label for the artifact.
        #[arg(long)]
        label: String,
        /// Handle kind.
        #[arg(long, default_value = "text")]
        kind: String,
    },
    /// Show handle metadata.
    Resolve {
        /// Handle ID.
        handle_id: String,
    },
    /// Show handle metadata (alias for resolve).
    Meta {
        /// Handle ID.
        handle_id: String,
    },
    /// Show artifact content.
    Cat {
        /// Handle ID.
        handle_id: String,
    },
    /// Rehydrate artifact with token limit.
    Rehydrate {
        /// Handle ID.
        handle_id: String,
        /// Maximum tokens to return.
        #[arg(long, default_value = "300")]
        max_tokens: u32,
    },
}

pub async fn run(cmd: EcsCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        EcsCmd::Store { path, label, kind } => {
            use base64::Engine;
            let content = std::fs::read(&path)?;
            let content_b64 = base64::engine::general_purpose::STANDARD.encode(&content);

            let resp = api
                .post(
                    "/v1/ecs/store",
                    &json!({
                        "kind": kind,
                        "label": label,
                        "content_b64": content_b64,
                    }),
                )
                .await?;

            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("✓ Stored {} ({} bytes)", label, content.len());
            }
        }
        EcsCmd::Resolve { handle_id } | EcsCmd::Meta { handle_id } => {
            let resp = api.get(&format!("/v1/ecs/resolve/{}", handle_id)).await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else if let Some(handle) = resp.get("handle") {
                println!("  id:    {}", handle["id"].as_str().unwrap_or("?"));
                println!("  kind:  {}", handle["kind"].as_str().unwrap_or("?"));
                println!("  label: {}", handle["label"].as_str().unwrap_or("?"));
                println!("  size:  {} bytes", handle["size"].as_u64().unwrap_or(0));
            } else {
                println!("Handle not found");
            }
        }
        EcsCmd::List => {
            let resp = api.get("/v1/ecs/handles").await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                let handles = resp["handles"].as_array();
                match handles {
                    Some(h) if h.is_empty() => println!("No handles"),
                    Some(handles) => {
                        for handle in handles {
                            let id = handle["id"].as_str().unwrap_or("?");
                            let short_id = if id.len() >= 8 { &id[..8] } else { id };
                            println!(
                                "  {} [{}] {}",
                                short_id,
                                handle["kind"].as_str().unwrap_or("?"),
                                handle["label"].as_str().unwrap_or("?"),
                            );
                        }
                    }
                    None => println!("No handles"),
                }
            }
        }
        EcsCmd::Cat { handle_id } => {
            let resp = api.get(&format!("/v1/ecs/content/{}", handle_id)).await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else if let Some(content_b64) = resp["content_b64"].as_str() {
                use base64::Engine;
                let content = base64::engine::general_purpose::STANDARD
                    .decode(content_b64)
                    .unwrap_or_default();
                print!("{}", String::from_utf8_lossy(&content));
            } else {
                eprintln!("Error: {}", resp["error"].as_str().unwrap_or("unknown"));
            }
        }
        EcsCmd::Rehydrate {
            handle_id,
            max_tokens,
        } => {
            let resp = api
                .post(
                    &format!("/v1/ecs/rehydrate/{}?max_tokens={}", handle_id, max_tokens),
                    &json!({}),
                )
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                let content = resp["content"].as_str().unwrap_or("");
                let truncated = resp["truncated"].as_bool().unwrap_or(false);
                println!("{}", content);
                if truncated {
                    eprintln!("--- truncated to {} tokens ---", max_tokens);
                }
            }
        }
    }
    Ok(())
}
