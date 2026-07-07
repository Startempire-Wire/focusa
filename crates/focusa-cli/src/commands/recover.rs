//! `focusa recover` — evaluator-driven recovery path for crashed daemon / lost Workpoint context.
//!
//! MVP posture: do the smallest safe recovery automatically, explain everything,
//! and return a typed envelope usable by agents and humans.

use crate::api_client::ApiClient;
use crate::commands::scope::ensure_project_root_scope_safe;
use clap::Args;
use serde_json::{Value, json};

#[derive(Args, Debug)]
pub struct RecoverArgs {
    /// Inspect crashed state and proposed recovery without mutating daemon/workpoint state.
    #[arg(long)]
    pub dry_run: bool,

    /// Safe project folder/container for canonical Workpoint recovery.
    #[arg(long)]
    pub project_root: Option<String>,

    /// Stable logical workstream id for same-project Workpoint continuity.
    #[arg(long)]
    pub continuity_id: Option<String>,

    /// Workpoint resume render mode: compact_prompt, full_json, operator_summary.
    #[arg(long, default_value = "operator_summary")]
    pub mode: String,

    /// Do not attempt to start the daemon when health probe fails.
    #[arg(long)]
    pub no_start_daemon: bool,
}

impl Default for RecoverArgs {
    fn default() -> Self {
        Self {
            dry_run: false,
            project_root: None,
            continuity_id: None,
            mode: "operator_summary".to_string(),
            no_start_daemon: false,
        }
    }
}

pub async fn run(json_output: bool, args: RecoverArgs) -> anyhow::Result<()> {
    ensure_project_root_scope_safe(args.project_root.as_deref(), "recover: project_root")?;

    let client = ApiClient::with_timeout_secs(4);
    let health_before = client.get("/v1/health").await;
    let daemon_running_before = health_before.is_ok();
    let crashed_state = if daemon_running_before {
        "daemon_available"
    } else {
        "daemon_unavailable_or_crashed"
    };

    let mut daemon_started = false;
    let mut recovery_errors: Vec<String> = Vec::new();

    if !args.dry_run && !daemon_running_before && !args.no_start_daemon {
        match crate::commands::daemon::start().await {
            Ok(started) => daemon_started = started,
            Err(error) => recovery_errors.push(format!("daemon_start_failed: {error}")),
        }
    }

    let should_probe_resume =
        daemon_running_before || (!args.dry_run && recovery_errors.is_empty());
    let workpoint_resume = if should_probe_resume {
        let resume_client = ApiClient::with_timeout_secs(8);
        match resume_client
            .post(
                "/v1/workpoint/resume",
                &json!({
                    "mode": args.mode,
                    "project_root": args.project_root,
                    "continuity_id": args.continuity_id,
                }),
            )
            .await
        {
            Ok(value) => value,
            Err(error) => {
                recovery_errors.push(format!("workpoint_resume_failed: {error}"));
                json!({
                    "status": "blocked",
                    "canonical": false,
                    "failure_class": "workpoint_resume_failed",
                    "error": error.to_string(),
                })
            }
        }
    } else {
        json!({
            "status": "not_attempted",
            "canonical": false,
            "reason": if args.dry_run { "dry_run" } else { "daemon_unavailable" },
        })
    };

    let proposed_recovery = if daemon_running_before {
        "resume last canonical Workpoint and surface recovery_hint"
    } else if args.no_start_daemon {
        "daemon appears crashed; run focusa start, then focusa recover --project-root <root> --continuity-id <id>"
    } else {
        "start daemon, reload persisted state, resume last canonical Workpoint, surface recovery_hint"
    };

    let status = if args.dry_run {
        "dry_run"
    } else if recovery_errors.is_empty() {
        "completed"
    } else {
        "blocked"
    };

    let envelope = json!({
        "ok": recovery_errors.is_empty() || args.dry_run,
        "status": status,
        "dry_run": args.dry_run,
        "crashed_state": crashed_state,
        "daemon_running_before": daemon_running_before,
        "daemon_started": daemon_started,
        "project_root": args.project_root,
        "continuity_id": args.continuity_id,
        "proposed_recovery": proposed_recovery,
        "workpoint_resume": workpoint_resume,
        "recovery_errors": recovery_errors,
        "recovery_hint": "If blocked, run focusa doctor --scope host, verify project_root, then retry focusa recover --dry-run before mutating recovery.",
        "next_tools": ["focusa doctor --scope host", "focusa workpoint resume", "focusa project identity"],
        "evidence_ref": "crates/focusa-cli/src/commands/recover.rs",
    });

    if json_output {
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        print_human(&envelope);
    }

    Ok(())
}

fn print_human(envelope: &Value) {
    println!("focusa recover");
    println!(
        "  status: {}",
        envelope["status"].as_str().unwrap_or("unknown")
    );
    println!(
        "  crashed_state: {}",
        envelope["crashed_state"].as_str().unwrap_or("unknown")
    );
    println!(
        "  proposed_recovery: {}",
        envelope["proposed_recovery"].as_str().unwrap_or("unknown")
    );
    println!(
        "  recovery_hint: {}",
        envelope["recovery_hint"].as_str().unwrap_or("unknown")
    );
    if let Some(errors) = envelope["recovery_errors"].as_array()
        && !errors.is_empty()
    {
        println!("  recovery_errors:");
        for error in errors {
            println!("    - {}", error.as_str().unwrap_or("unknown"));
        }
    }
    if let Some(workpoint_status) = envelope["workpoint_resume"]["status"].as_str() {
        println!("  workpoint_resume: {workpoint_status}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recover_args_default_to_safe_operator_summary() {
        let args = RecoverArgs::default();
        assert!(!args.dry_run);
        assert_eq!(args.mode, "operator_summary");
        assert!(!args.no_start_daemon);
    }
}
