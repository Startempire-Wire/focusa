//! Focus stack CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand)]
pub enum FocusCmd {
    /// Push a new focus frame.
    Push {
        /// Frame title.
        title: String,
        /// Frame goal.
        #[arg(long)]
        goal: String,
        /// Beads issue ID.
        #[arg(long)]
        beads_issue_id: String,
        /// Constraints (comma-separated).
        #[arg(long)]
        constraints: Option<String>,
        /// Tags (comma-separated).
        #[arg(long)]
        tags: Option<String>,
    },
    /// Pop (complete) the active frame.
    Pop {
        /// Completion reason.
        #[arg(long, default_value = "goal_achieved")]
        reason: String,
    },
    /// Complete the active frame (alias for pop with goal_achieved).
    Complete,
    /// Set active frame by ID.
    Set {
        /// Frame ID.
        frame_id: String,
    },
}

pub async fn run(cmd: FocusCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();

    match cmd {
        FocusCmd::Push {
            title,
            goal,
            beads_issue_id,
            constraints,
            tags,
        } => {
            let constraints: Vec<String> = constraints
                .map(|s| s.split(',').map(|c| c.trim().to_string()).collect())
                .unwrap_or_default();
            let tags: Vec<String> = tags
                .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
                .unwrap_or_default();

            let resp = api
                .post(
                    "/v1/focus/push",
                    &json!({
                        "title": title,
                        "goal": goal,
                        "beads_issue_id": beads_issue_id,
                        "constraints": constraints,
                        "tags": tags,
                    }),
                )
                .await?;

            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("✓ Frame pushed: {}", title);
            }
        }
        FocusCmd::Pop { reason } => {
            let resp = api
                .post("/v1/focus/pop", &json!({"completion_reason": reason}))
                .await?;

            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("✓ Frame popped ({})", reason);
            }
        }
        FocusCmd::Complete => {
            let resp = api
                .post(
                    "/v1/focus/pop",
                    &json!({"completion_reason": "goal_achieved"}),
                )
                .await?;

            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("✓ Frame completed");
            }
        }
        FocusCmd::Set { frame_id } => {
            let resp = api
                .post("/v1/focus/set-active", &json!({"frame_id": frame_id}))
                .await?;

            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("✓ Active frame set: {}", frame_id);
            }
        }
    }
    Ok(())
}
