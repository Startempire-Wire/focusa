//! Governed Work Loop CLI parity for the canonical `/v1/work-loop/*` API.
//!
//! The CLI never infers project scope, writer ownership, or fencing tokens.
//! Read operations require explicit scope; progression requires the exact lease
//! returned by `enable`. Pause/stop may omit a fence for unclaimed recovery;
//! the daemon still checks scope and every existing owner, without acquiring a lease.

use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::{Value, json};

use crate::api_client::ApiClient;

#[derive(Subcommand, Debug)]
pub enum WorkLoopCmd {
    /// Read bounded Work Loop state and budgets.
    Status(ScopeArgs),
    /// Read writer ownership/preflight state without mutation.
    WriterStatus(ScopeArgs),
    /// Enable or safely rebind continuous work with explicit approval.
    Enable(EnableArgs),
    /// Create a recovery checkpoint under the current writer lease.
    Checkpoint(CheckpointArgs),
    /// Update the bounded continuation context.
    Context(ContextArgs),
    /// Select the next blocks-ready descendant under a root work item.
    SelectNext(SelectNextArgs),
    /// Pause continuous work.
    Pause(ControlArgs),
    /// Resume continuous work without silently renewing its budget.
    Resume(ResumeArgs),
    /// Stop continuous work and release the writer lease.
    Stop(ControlArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ScopeArgs {
    /// Canonical absolute project root.
    #[arg(long)]
    pub project_root: String,
    /// Stable project workstream/continuity id.
    #[arg(long)]
    pub continuity_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct LeaseArgs {
    /// Exact writer id used when the loop was enabled.
    #[arg(long)]
    pub writer_id: String,
    /// Exact positive fencing token returned by enable.
    #[arg(long)]
    pub fencing_token: u64,
}

#[derive(Args, Debug, Clone)]
pub struct OptionalLeaseArgs {
    /// Actual caller identity; this argument never grants writer ownership.
    #[arg(long)]
    pub writer_id: String,
    /// Existing positive fence, if any. Omission never bypasses a live owner's fence.
    #[arg(long)]
    pub fencing_token: Option<u64>,
}

#[derive(Args, Debug)]
pub struct EnableArgs {
    #[command(flatten)]
    pub scope: ScopeArgs,
    /// Root provider WorkItem whose true blocking descendants form the frontier.
    #[arg(long)]
    pub root_work_item_id: String,
    /// Explicit approval for enable/rebind governance boundary.
    #[arg(long)]
    pub approve: bool,
    /// Writer identity claiming this partition.
    #[arg(long, default_value = "focusa-cli")]
    pub writer_id: String,
    #[arg(long, default_value = "balanced")]
    pub preset: String,
    /// Stable key for signed limit reservation and replay safety.
    #[arg(long)]
    pub idempotency_key: String,
}

#[derive(Args, Debug)]
pub struct CheckpointArgs {
    #[command(flatten)]
    pub scope: ScopeArgs,
    #[command(flatten)]
    pub lease: LeaseArgs,
    #[arg(long)]
    pub summary: String,
    /// Optional UUID for replay-safe checkpoint creation.
    #[arg(long)]
    pub checkpoint_id: Option<String>,
    #[arg(long)]
    pub idempotency_key: String,
}

#[derive(Args, Debug)]
pub struct ContextArgs {
    #[command(flatten)]
    pub scope: ScopeArgs,
    #[command(flatten)]
    pub lease: OptionalLeaseArgs,
    #[arg(long)]
    pub current_ask: String,
    #[arg(long)]
    pub ask_kind: Option<String>,
    #[arg(long)]
    pub scope_kind: Option<String>,
    #[arg(long)]
    pub source_turn_id: Option<String>,
    #[arg(long, default_value_t = false)]
    pub operator_steering_detected: bool,
    #[arg(long)]
    pub idempotency_key: String,
}

#[derive(Args, Debug)]
pub struct SelectNextArgs {
    #[command(flatten)]
    pub scope: ScopeArgs,
    #[command(flatten)]
    pub lease: LeaseArgs,
    #[arg(long)]
    pub parent_work_item_id: String,
    #[arg(long)]
    pub idempotency_key: String,
}

#[derive(Args, Debug)]
pub struct ControlArgs {
    #[command(flatten)]
    pub scope: ScopeArgs,
    #[command(flatten)]
    pub lease: OptionalLeaseArgs,
    #[arg(long, default_value = "operator requested")]
    pub reason: String,
    #[arg(long)]
    pub idempotency_key: String,
}

#[derive(Args, Debug)]
pub struct ResumeArgs {
    #[command(flatten)]
    pub scope: ScopeArgs,
    #[command(flatten)]
    pub lease: LeaseArgs,
    #[arg(long, default_value = "operator requested")]
    pub reason: String,
    /// Start a fresh budget epoch. Requires explicit daemon approval.
    #[arg(long, default_value_t = false)]
    pub renew_budget: bool,
    #[arg(long, requires = "renew_budget")]
    pub approve: bool,
    #[arg(long)]
    pub idempotency_key: String,
}

fn scope_headers(scope: &ScopeArgs) -> [(&str, &str); 2] {
    [
        ("x-scope-project-root", scope.project_root.as_str()),
        ("x-scope-continuity-id", scope.continuity_id.as_str()),
    ]
}

async fn post_with_lease(
    api: &ApiClient,
    path: &str,
    scope: &ScopeArgs,
    lease: &LeaseArgs,
    body: &Value,
    approved: bool,
) -> Result<Value> {
    post_with_writer(
        api,
        path,
        scope,
        &lease.writer_id,
        Some(lease.fencing_token),
        body,
        approved,
    )
    .await
}

fn fencing_header_value(value: Option<u64>) -> Result<Option<String>> {
    value
        .map(|token| -> Result<String> {
            anyhow::ensure!(token > 0, "--fencing-token must be positive");
            Ok(token.to_string())
        })
        .transpose()
}

async fn post_with_writer(
    api: &ApiClient,
    path: &str,
    scope: &ScopeArgs,
    writer_id: &str,
    token: Option<u64>,
    body: &Value,
    approved: bool,
) -> Result<Value> {
    let fencing_token = fencing_header_value(token)?;
    let mut headers = scope_headers(scope).to_vec();
    headers.push(("x-focusa-writer-id", writer_id));
    if let Some(token) = fencing_token.as_deref() {
        headers.push(("x-focusa-fencing-token", token));
    }
    if approved {
        headers.push(("x-focusa-approval", "approved"));
    }
    api.post_with_headers(path, body, &headers).await
}

fn print_result(value: &Value, json_mode: bool, label: &str) -> Result<()> {
    if json_mode {
        println!("{}", serde_json::to_string_pretty(value)?);
    } else {
        println!("Work Loop {label}");
        println!(
            "Status: {}",
            value
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or_else(|| {
                    if value.get("ok").and_then(Value::as_bool) == Some(true) {
                        "accepted"
                    } else {
                        "observed"
                    }
                })
        );
        if let Some(current) = value
            .pointer("/current_task/work_item_id")
            .and_then(Value::as_str)
        {
            println!("Current task: {current}");
        }
        if let Some(writer) = value.get("writer_id").and_then(Value::as_str) {
            println!("Writer: {writer}");
        }
        if let Some(token) = value.get("fencing_token").and_then(Value::as_u64) {
            println!("Fencing token: {token}");
        }
    }
    Ok(())
}

pub async fn run(command: WorkLoopCmd, json_mode: bool) -> Result<()> {
    // Work Loop enable/select may perform bounded provider discovery before replying.
    let api = ApiClient::with_timeout_secs(60);
    let (label, value) = match command {
        WorkLoopCmd::Status(scope) => (
            "status",
            api.get_with_headers(
                "/v1/work-loop/status?summary_only=true",
                &scope_headers(&scope),
            )
            .await?,
        ),
        WorkLoopCmd::WriterStatus(scope) => (
            "writer status",
            api.get_with_headers(
                "/v1/work-loop/status?summary_only=true",
                &scope_headers(&scope),
            )
            .await?,
        ),
        WorkLoopCmd::Enable(args) => {
            anyhow::ensure!(
                args.approve,
                "--approve is required to enable or rebind Work Loop"
            );
            let body = json!({
                "preset": args.preset,
                "root_work_item_id": args.root_work_item_id,
                "idempotency_key": args.idempotency_key,
            });
            let headers = [
                ("x-scope-project-root", args.scope.project_root.as_str()),
                ("x-scope-continuity-id", args.scope.continuity_id.as_str()),
                ("x-focusa-writer-id", args.writer_id.as_str()),
                ("x-focusa-approval", "approved"),
            ];
            (
                "enable",
                api.post_with_headers("/v1/work-loop/enable", &body, &headers)
                    .await?,
            )
        }
        WorkLoopCmd::Checkpoint(args) => {
            let body = json!({"checkpoint_id": args.checkpoint_id, "summary": args.summary, "idempotency_key": args.idempotency_key});
            (
                "checkpoint",
                post_with_lease(
                    &api,
                    "/v1/work-loop/checkpoint",
                    &args.scope,
                    &args.lease,
                    &body,
                    false,
                )
                .await?,
            )
        }
        WorkLoopCmd::Context(args) => {
            let body = json!({
                "current_ask": args.current_ask,
                "ask_kind": args.ask_kind,
                "scope_kind": args.scope_kind,
                "source_turn_id": args.source_turn_id,
                "operator_steering_detected": args.operator_steering_detected,
                "idempotency_key": args.idempotency_key,
            });
            (
                "context",
                post_with_writer(
                    &api,
                    "/v1/work-loop/context",
                    &args.scope,
                    &args.lease.writer_id,
                    args.lease.fencing_token,
                    &body,
                    false,
                )
                .await?,
            )
        }
        WorkLoopCmd::SelectNext(args) => {
            let body = json!({"parent_work_item_id": args.parent_work_item_id, "idempotency_key": args.idempotency_key});
            (
                "select next",
                post_with_lease(
                    &api,
                    "/v1/work-loop/select-next",
                    &args.scope,
                    &args.lease,
                    &body,
                    false,
                )
                .await?,
            )
        }
        WorkLoopCmd::Pause(args) => (
            "pause",
            post_with_writer(
                &api,
                "/v1/work-loop/pause",
                &args.scope,
                &args.lease.writer_id,
                args.lease.fencing_token,
                &json!({"reason": args.reason, "idempotency_key": args.idempotency_key}),
                false,
            )
            .await?,
        ),
        WorkLoopCmd::Resume(args) => (
            "resume",
            post_with_lease(
                &api,
                "/v1/work-loop/resume",
                &args.scope,
                &args.lease,
                &json!({"reason": args.reason, "renew_budget": args.renew_budget, "idempotency_key": args.idempotency_key}),
                args.approve,
            )
            .await?,
        ),
        WorkLoopCmd::Stop(args) => (
            "stop",
            post_with_writer(
                &api,
                "/v1/work-loop/stop",
                &args.scope,
                &args.lease.writer_id,
                args.lease.fencing_token,
                &json!({"reason": args.reason, "idempotency_key": args.idempotency_key}),
                false,
            )
            .await?,
        ),
    };
    print_result(&value, json_mode, label)
}

#[cfg(test)]
mod recovery_argument_tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser, Debug)]
    struct TestCli {
        #[command(subcommand)]
        command: WorkLoopCmd,
    }

    fn args(command: &str) -> Vec<&str> {
        vec![
            "test",
            command,
            "--project-root",
            "/workspace/project",
            "--continuity-id",
            "existing-stream",
            "--writer-id",
            "actual-caller",
            "--idempotency-key",
            "request-1",
        ]
    }

    #[test]
    fn recovery_can_omit_fence_without_manufacturing_one() {
        for command in ["pause", "stop"] {
            let parsed = TestCli::try_parse_from(args(command)).unwrap();
            match parsed.command {
                WorkLoopCmd::Pause(control) | WorkLoopCmd::Stop(control) => {
                    assert_eq!(control.lease.writer_id, "actual-caller");
                    assert_eq!(control.lease.fencing_token, None);
                }
                _ => panic!("wrong recovery command"),
            }
        }
        let mut input = args("context");
        input.extend(["--current-ask", "existing authorized work"]);
        match TestCli::try_parse_from(input).unwrap().command {
            WorkLoopCmd::Context(context) => assert_eq!(context.lease.fencing_token, None),
            _ => panic!("wrong metadata command"),
        }
        assert_eq!(fencing_header_value(None).unwrap(), None);
        assert!(fencing_header_value(Some(0)).is_err());
        assert_eq!(
            fencing_header_value(Some(42)).unwrap().as_deref(),
            Some("42")
        );
    }

    #[test]
    fn progression_and_owned_writes_still_require_fence() {
        for (command, extra) in [
            ("resume", vec![]),
            ("checkpoint", vec!["--summary", "existing-work checkpoint"]),
            (
                "select-next",
                vec!["--parent-work-item-id", "existing-item"],
            ),
        ] {
            let mut input = args(command);
            input.extend(extra);
            let failure = TestCli::try_parse_from(input).unwrap_err();
            assert!(failure.to_string().contains("--fencing-token"));
        }
    }
}
