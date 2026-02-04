//! ECS (reference store) CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand)]
pub enum EcsCmd {
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
        EcsCmd::Resolve { handle_id } => {
            let resp = api
                .get(&format!("/v1/ecs/resolve/{}", handle_id))
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else if let Some(handle) = resp.get("handle") {
                println!(
                    "  id:   {}",
                    handle["id"].as_str().unwrap_or("?")
                );
                println!(
                    "  kind: {}",
                    handle["kind"].as_str().unwrap_or("?")
                );
                println!(
                    "  label: {}",
                    handle["label"].as_str().unwrap_or("?")
                );
                println!(
                    "  size: {} bytes",
                    handle["size"].as_u64().unwrap_or(0)
                );
            } else {
                println!("Handle not found");
            }
        }
    }
    Ok(())
}
