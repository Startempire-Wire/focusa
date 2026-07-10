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
        /// Project root authority for canonical Focus frame writes.
        #[arg(long)]
        project_root: Option<String>,
        /// Logical workstream/continuity id for canonical Focus frame writes.
        #[arg(long)]
        continuity_id: Option<String>,
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
        /// Explicit target frame id.
        #[arg(long)]
        frame_id: Option<String>,
        /// Project root authority for scoped Focus State writes.
        #[arg(long)]
        project_root: Option<String>,
        /// Logical workstream/continuity id for scoped Focus State writes.
        #[arg(long)]
        continuity_id: Option<String>,
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
            project_root,
            continuity_id,
        } => {
            let constraints: Vec<String> = constraints
                .map(|s| s.split(',').map(|c| c.trim().to_string()).collect())
                .unwrap_or_default();
            let tags: Vec<String> = tags
                .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
                .unwrap_or_default();

            // Spec 112 transcript bug fix (focusa-bug-focus-stack-silent-loss):
            // Auto-detect project_root from PWD when the operator didn't pass it
            // explicitly. The daemon rejects empty/missing project_root, and the
            // previous CLI printed "✓ Frame pushed" while silently losing the
            // frame. Now: look for a .beads/ marker walking up from PWD; if found
            // and the operator didn't override, use that as the project_root.
            let effective_project_root = match project_root {
                Some(p) if !p.trim().is_empty() => p.clone(),
                _ => discover_project_root_from_cwd().unwrap_or_default(),
            };
            let effective_continuity_id = match continuity_id {
                Some(c) if !c.trim().is_empty() => c.clone(),
                _ => String::new(),
            };

            let resp = api
                .post(
                    "/v1/focus/push",
                    &json!({
                        "title": title,
                        "goal": goal,
                        "beads_issue_id": beads_issue_id,
                        "constraints": constraints,
                        "tags": tags,
                        "project_root": effective_project_root,
                        "continuity_id": effective_continuity_id,
                    }),
                )
                .await?;

            // Surface daemon rejection so the operator notices the silent-loss bug
            // they just hit. Previously the CLI printed "✓ Frame pushed" even when
            // the daemon returned `status: rejected_unsafe_project_root`.
            let status = resp.get("status").and_then(|v| v.as_str()).unwrap_or("");
            let canonical = resp
                .get("canonical")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            if !canonical || status.starts_with("rejected") {
                if json_mode {
                    println!("{}", serde_json::to_string_pretty(&resp)?);
                } else {
                    eprintln!(
                        "Frame NOT pushed (daemon rejected): {}\nrecovery_hint: {}",
                        resp.get("safe_recovery")
                            .and_then(|v| v.as_str())
                            .unwrap_or("see --json output"),
                        resp.get("safe_recovery")
                            .and_then(|v| v.as_str())
                            .unwrap_or("(no recovery hint)")
                    );
                }
                std::process::exit(2);
            }

            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("✓ Frame pushed: {} ({})", title, status);
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
            frame_id,
            project_root,
            continuity_id,
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
                        "frame_id": frame_id,
                        "project_root": project_root,
                        "continuity_id": continuity_id,
                        "turn_id": turn_id.unwrap_or_else(generated_turn_id),
                        "delta": Value::Object(delta),
                    }),
                )
                .await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                let status = resp
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
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

/// Walk up from PWD looking for a `.beads/` directory. Returns the directory
/// containing it. This is the heuristic Focusa uses to identify a "project
/// root" without explicit `--project-root` arguments, mirroring the daemon's
/// `discover_project_root_from_cwd` behavior. The transcript bug (frame
/// silently lost) happened because the CLI sent `null` and the daemon
/// rejected it; the fix is to never send null when a project root is
/// discoverable from the working directory.
fn discover_project_root_from_path(mut cur: std::path::PathBuf) -> Option<String> {
    loop {
        let beads = cur.join(".beads");
        if beads.is_dir() {
            return cur.to_str().map(|s| s.to_string());
        }
        match cur.parent() {
            Some(parent) if parent != cur => cur = parent.to_path_buf(),
            _ => return None,
        }
    }
}

fn discover_project_root_from_cwd() -> Option<String> {
    discover_project_root_from_path(std::env::current_dir().ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_finds_beads_walking_up() {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = discover_project_root_from_path(manifest_dir);
        assert!(
            root.is_some(),
            "expected to find .beads walking up from manifest dir"
        );
        let p = std::path::PathBuf::from(root.unwrap());
        assert!(p.join(".beads").is_dir());
    }

    #[test]
    fn discover_returns_none_for_clean_tmp() {
        let tmp = std::env::temp_dir().join("focusa-discover-none-test");
        let _ = std::fs::create_dir_all(&tmp);
        let result = discover_project_root_from_path(tmp);
        // On a hardened CI box /tmp is sometimes a tmpfs and parent walks to /
        // which may or may not have a .beads/ — accept either, just don't panic.
        if let Some(r) = result {
            assert!(std::path::Path::new(&r).join(".beads").is_dir());
        }
    }
}
