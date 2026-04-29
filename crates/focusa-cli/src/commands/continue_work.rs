//! Continue command — Spec92 §9 agent-first command center.

use crate::api_client::ApiClient;
use clap::Args;
use serde_json::{Value, json};

#[derive(Args, Debug)]
pub struct ContinueArgs {
    /// Reason recorded with the work-loop resume action.
    #[arg(long, default_value = "agent requested focusa continue")]
    pub reason: String,

    /// Select the next ready subtask under this Beads/root work item before resuming.
    #[arg(long)]
    pub parent_work_item_id: Option<String>,

    /// Enable continuous work first when the loop is stopped/not enabled.
    #[arg(long)]
    pub enable: bool,

    /// Work-loop preset when --enable is used.
    #[arg(long, default_value = "balanced")]
    pub preset: String,

    /// Writer id for safe single-writer work-loop governance.
    #[arg(long, default_value = "focusa-cli")]
    pub writer_id: String,
}

fn envelope(status: &str, summary: String, next_action: &str, details: Value) -> Value {
    json!({
        "status": status,
        "summary": summary,
        "next_action": next_action,
        "why": "Spec92 continue resumes bounded, governed work-loop execution without relying on transcript tail.",
        "commands": ["focusa continue", "focusa work-loop status", "focusa_workpoint_resume"],
        "recovery": ["focusa doctor", "focusa continue --enable --parent-work-item-id <id>", "journalctl -u focusa-daemon -n 80 --no-pager"],
        "evidence_refs": ["/v1/work-loop/status", "/v1/workpoint/current"],
        "docs": ["docs/92-agent-first-polish-hooks-efficiency-spec.md", "docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md"],
        "warnings": [],
        "details": details,
    })
}

pub async fn run(args: ContinueArgs, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let headers = [("x-focusa-writer-id", args.writer_id.as_str())];
    let before_status = api
        .get("/v1/work-loop/status")
        .await
        .unwrap_or_else(|err| json!({"status":"blocked","error":err.to_string()}));
    let workpoint = api
        .get("/v1/workpoint/current")
        .await
        .unwrap_or_else(|err| json!({"status":"blocked","error":err.to_string()}));

    let mut actions = Vec::new();
    if args.enable {
        let enable_headers = [
            ("x-focusa-writer-id", args.writer_id.as_str()),
            ("x-focusa-approval", "approved"),
        ];
        let resp = api
            .post_with_headers(
                "/v1/work-loop/enable",
                &json!({
                    "preset": args.preset,
                    "root_work_item_id": args.parent_work_item_id,
                }),
                &enable_headers,
            )
            .await?;
        actions.push(json!({"action":"enable", "response": resp}));
    }

    if let Some(parent) = args.parent_work_item_id.as_ref() {
        let resp = api
            .post_with_headers(
                "/v1/work-loop/select-next",
                &json!({"parent_work_item_id": parent}),
                &headers,
            )
            .await?;
        actions
            .push(json!({"action":"select_next", "parent_work_item_id": parent, "response": resp}));
    }

    let resume = api
        .post_with_headers(
            "/v1/work-loop/resume",
            &json!({"reason": args.reason}),
            &headers,
        )
        .await?;
    actions.push(json!({"action":"resume", "response": resume}));

    let after_status = api
        .get("/v1/work-loop/status")
        .await
        .unwrap_or_else(|err| json!({"status":"blocked","error":err.to_string()}));
    let response = envelope(
        "completed",
        "Work-loop continue request accepted and current state refreshed".to_string(),
        "Watch the next Pi turn or run focusa work-loop status to confirm follow-on dispatch",
        json!({
            "before_work_loop": before_status,
            "current_workpoint": workpoint,
            "actions": actions,
            "after_work_loop": after_status,
        }),
    );

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        println!(
            "Status: {}",
            response["status"].as_str().unwrap_or("completed")
        );
        println!(
            "Summary: {}",
            response["summary"].as_str().unwrap_or("continue accepted")
        );
        println!(
            "Next action: {}",
            response["next_action"]
                .as_str()
                .unwrap_or("focusa work-loop status")
        );
        println!(
            "Why: {}",
            response["why"]
                .as_str()
                .unwrap_or("Spec92 governed continuation")
        );
        println!("Command: focusa work-loop status");
        println!("Recovery: focusa doctor && focusa continue --enable --parent-work-item-id <id>");
        println!("Evidence: /v1/work-loop/status, /v1/workpoint/current");
        println!("Docs: docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md");
    }
    Ok(())
}
