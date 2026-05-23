//! Focus stack and Focus State CLI commands.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::{Map, Value, json};
use std::time::{SystemTime, UNIX_EPOCH};

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
    /// Update bounded Focus State slots via /v1/focus/update.
    Update {
        /// Idempotency/source turn id; generated when omitted.
        #[arg(long)]
        turn_id: Option<String>,
        /// Crystallized architectural decision. Repeatable.
        #[arg(long = "decision")]
        decisions: Vec<String>,
        /// Discovered hard requirement. Repeatable.
        #[arg(long = "constraint")]
        constraints: Vec<String>,
        /// Specific failure and diagnosis. Repeatable.
        #[arg(long = "failure")]
        failures: Vec<String>,
        /// Current intent summary.
        #[arg(long)]
        intent: Option<String>,
        /// Current focus summary.
        #[arg(long = "current-focus")]
        current_focus: Option<String>,
        /// Next bounded step. Repeatable.
        #[arg(long = "next-step")]
        next_steps: Vec<String>,
        /// Open question. Repeatable.
        #[arg(long = "open-question")]
        open_questions: Vec<String>,
        /// Recent verified result. Repeatable.
        #[arg(long = "recent-result")]
        recent_results: Vec<String>,
        /// Short note. Repeatable.
        #[arg(long = "note")]
        notes: Vec<String>,
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

fn generated_turn_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("cli-{millis}")
}

fn put_array(delta: &mut Map<String, Value>, key: &str, values: Vec<String>) {
    if !values.is_empty() {
        delta.insert(key.to_string(), json!(values));
    }
}

fn put_string(delta: &mut Map<String, Value>, key: &str, value: Option<String>) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        delta.insert(key.to_string(), json!(value));
    }
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
        FocusCmd::Update {
            turn_id,
            decisions,
            constraints,
            failures,
            intent,
            current_focus,
            next_steps,
            open_questions,
            recent_results,
            notes,
        } => {
            let mut delta = Map::new();
            put_array(&mut delta, "decisions", decisions);
            put_array(&mut delta, "constraints", constraints);
            put_array(&mut delta, "failures", failures);
            put_string(&mut delta, "intent", intent);
            put_string(&mut delta, "current_focus", current_focus);
            put_array(&mut delta, "next_steps", next_steps);
            put_array(&mut delta, "open_questions", open_questions);
            put_array(&mut delta, "recent_results", recent_results);
            put_array(&mut delta, "notes", notes);
            if delta.is_empty() {
                anyhow::bail!("[CLI_INPUT_ERROR] focus update requires at least one slot flag");
            }
            let resp = api
                .post(
                    "/v1/focus/update",
                    &json!({
                        "turn_id": turn_id.unwrap_or_else(generated_turn_id),
                        "delta": Value::Object(delta),
                    }),
                )
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                let status = resp.get("status").and_then(Value::as_str).unwrap_or("unknown");
                println!("focus update: status={status}");
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
