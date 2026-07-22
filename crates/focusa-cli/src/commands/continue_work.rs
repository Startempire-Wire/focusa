//! Continue command — Spec92 §9 agent-first command center.

use crate::api_client::ApiClient;
use clap::Args;
use serde_json::{Value, json};

#[derive(Args, Debug)]
pub struct ContinueArgs {
    /// Reason recorded with the work-loop resume action.
    #[arg(long, default_value = "agent requested focusa continue")]
    pub reason: String,

    /// Canonical absolute project root for Work Loop authority.
    #[arg(long)]
    pub project_root: String,

    /// Canonical continuity/workstream id for Work Loop authority.
    #[arg(long)]
    pub continuity_id: String,

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
        "commands": ["focusa continue --project-root <abs> --continuity-id <id>", "focusa work-loop status", "focusa_workpoint_resume"],
        "recovery": ["focusa doctor", "focusa start", "focusa continue --project-root <abs> --continuity-id <id> --enable --parent-work-item-id <id>", "journalctl -u focusa-daemon -n 80 --no-pager (Linux service installs)"],
        "evidence_refs": ["/v1/work-loop/status?summary_only=true", "/v1/workpoint/current"],
        "docs": ["docs/92-agent-first-polish-hooks-efficiency-spec.md", "docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md"],
        "warnings": [],
        "details": details,
    })
}

fn scoped_fencing_token(
    status: &Value,
    writer_id: &str,
    project_root: &str,
    continuity_id: &str,
) -> Option<u64> {
    if status.get("schema").and_then(Value::as_str) != Some("focusa.work_loop_status.v3")
        || status.get("state").and_then(Value::as_str) == Some("unsupported")
    {
        return None;
    }
    let partition = status.get("execution_partition")?;
    (partition.get("writer_key")?.as_str()? == writer_id
        && partition.get("project_root_key")?.as_str()? == project_root
        && partition.get("workstream_key")?.as_str()? == continuity_id
        && partition.get("lease_freshness")?.as_str()? == "current")
        .then(|| partition.get("fencing_token")?.as_u64())
        .flatten()
}

fn operator_surface_details(status: &Value, project_root: &str, continuity_id: &str) -> Value {
    let partition = status
        .get("execution_partition")
        .cloned()
        .unwrap_or_else(|| json!({}));
    json!({
        "project_root": project_root,
        "continuity_id": continuity_id,
        "work_item_id": partition.get("work_item_key"),
        "writer_id": partition.get("writer_key"),
        "lease_freshness": partition.get("lease_freshness"),
        "lease_expires_at": partition.get("lease_expires_at"),
        "fencing_token": partition.get("fencing_token"),
        "partition_status": partition.get("partition_status"),
        "typed_state": status.get("state"),
        "canonical_workpoint": status
            .pointer("/active_workpoint/active/canonical")
            .or_else(|| status.pointer("/active_workpoint/canonical")),
    })
}

pub async fn run(args: ContinueArgs, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let before_status = api
        .get_scoped(
            "/v1/work-loop/status?summary_only=true",
            &args.project_root,
            &args.continuity_id,
        )
        .await
        .unwrap_or_else(|err| json!({"status":"blocked","error":err.to_string()}));
    let workpoint = api
        .get_scoped(
            "/v1/workpoint/current",
            &args.project_root,
            &args.continuity_id,
        )
        .await
        .unwrap_or_else(|err| json!({"status":"blocked","error":err.to_string()}));

    let mut actions = Vec::new();
    let mut fencing_token = scoped_fencing_token(
        &before_status,
        &args.writer_id,
        &args.project_root,
        &args.continuity_id,
    );
    if args.enable {
        let enable_headers = [
            ("x-scope-project-root", args.project_root.as_str()),
            ("x-scope-continuity-id", args.continuity_id.as_str()),
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
        fencing_token = resp.get("fencing_token").and_then(Value::as_u64);
        actions.push(json!({"action":"enable", "response": resp}));
    }

    let fencing_token = fencing_token.ok_or_else(|| {
        anyhow::anyhow!(
            "no current fencing token for writer {}; use --enable to acquire this scoped lease or stop the conflicting writer",
            args.writer_id
        )
    })?;
    let fencing_token_header = fencing_token.to_string();
    let headers = [
        ("x-scope-project-root", args.project_root.as_str()),
        ("x-scope-continuity-id", args.continuity_id.as_str()),
        ("x-focusa-writer-id", args.writer_id.as_str()),
        ("x-focusa-fencing-token", fencing_token_header.as_str()),
    ];

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
        .get_scoped(
            "/v1/work-loop/status?summary_only=true",
            &args.project_root,
            &args.continuity_id,
        )
        .await
        .unwrap_or_else(|err| json!({"status":"blocked","error":err.to_string()}));
    let operator_surface =
        operator_surface_details(&after_status, &args.project_root, &args.continuity_id);
    let response = envelope(
        "completed",
        "Work-loop continue request accepted and current state refreshed".to_string(),
        "Watch the next Pi turn or run focusa work-loop status to confirm follow-on dispatch",
        json!({
            "authority_scope": {
                "project_root": args.project_root,
                "continuity_id": args.continuity_id,
            },
            "operator_surface": operator_surface,
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
        let operator_surface = &response["details"]["operator_surface"];
        println!(
            "Authority: project_root={} continuity_id={} work_item={}",
            operator_surface["project_root"]
                .as_str()
                .unwrap_or("unbound"),
            operator_surface["continuity_id"]
                .as_str()
                .unwrap_or("unbound"),
            operator_surface["work_item_id"]
                .as_str()
                .unwrap_or("unbound")
        );
        println!(
            "Writer lease: writer={} freshness={} fence={} expires={}",
            operator_surface["writer_id"]
                .as_str()
                .unwrap_or("unclaimed"),
            operator_surface["lease_freshness"]
                .as_str()
                .unwrap_or("unknown"),
            operator_surface["fencing_token"]
                .as_u64()
                .map(|value| value.to_string())
                .as_deref()
                .unwrap_or("none"),
            operator_surface["lease_expires_at"]
                .as_str()
                .unwrap_or("unknown")
        );
        println!("Command: focusa work-loop status");
        println!(
            "Recovery: focusa doctor && focusa continue --project-root <abs> --continuity-id <id> --enable --parent-work-item-id <id>"
        );
        println!("Evidence: /v1/work-loop/status?summary_only=true, /v1/workpoint/current");
        println!("Docs: docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fencing_token_requires_matching_scoped_writer() {
        let status = json!({
            "schema": "focusa.work_loop_status.v3",
            "state": "healthy",
            "execution_partition": {
                "project_root_key": "/tmp/focusa",
                "workstream_key": "focusa-continuity",
                "writer_key": "cli-writer",
                "fencing_token": 42,
                "lease_freshness": "current"
            }
        });
        assert_eq!(
            scoped_fencing_token(&status, "cli-writer", "/tmp/focusa", "focusa-continuity"),
            Some(42)
        );
        assert_eq!(
            scoped_fencing_token(&status, "other-writer", "/tmp/focusa", "focusa-continuity"),
            None
        );
        assert_eq!(
            scoped_fencing_token(&status, "cli-writer", "/tmp/other", "focusa-continuity"),
            None
        );
        assert_eq!(
            scoped_fencing_token(&status, "cli-writer", "/tmp/focusa", "other-continuity"),
            None
        );
        let mut stale = status.clone();
        stale["execution_partition"]["lease_freshness"] = json!("expired");
        assert_eq!(
            scoped_fencing_token(&stale, "cli-writer", "/tmp/focusa", "focusa-continuity"),
            None
        );
        let unsupported = json!({
            "schema": "focusa.work_loop_status.v999",
            "state": "healthy",
            "execution_partition": {"writer_key": "cli-writer", "fencing_token": 42}
        });
        assert_eq!(
            scoped_fencing_token(
                &unsupported,
                "cli-writer",
                "/tmp/focusa",
                "focusa-continuity"
            ),
            None
        );

        let details = operator_surface_details(&status, "/tmp/focusa", "focusa-continuity");
        assert_eq!(details["project_root"], json!("/tmp/focusa"));
        assert_eq!(details["continuity_id"], json!("focusa-continuity"));
        assert_eq!(details["writer_id"], json!("cli-writer"));
        assert_eq!(details["lease_freshness"], json!("current"));
        assert_eq!(details["fencing_token"], json!(42));
    }
}
