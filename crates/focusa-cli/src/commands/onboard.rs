//! First-run Operator Preview onboarding flow.

use crate::api_client::ApiClient;
use crate::commands::daemon;
use clap::Args;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Args)]
pub struct OnboardArgs {
    /// Agent harness to orient for; currently informational. Example: pi, claude-code, cursor.
    #[arg(long, default_value = "manual")]
    pub agent: String,

    /// Explicit project root. Defaults to git root, then current directory.
    #[arg(long)]
    pub project_root: Option<String>,

    /// Stable continuity id for the demo Workpoint.
    #[arg(long)]
    pub continuity_id: Option<String>,

    /// Skip creating the demo Workpoint.
    #[arg(long)]
    pub no_demo_workpoint: bool,
}

fn shell_output(args: &[&str], cwd: Option<&Path>) -> Option<String> {
    let mut cmd = Command::new(args.first().copied()?);
    cmd.args(&args[1..]);
    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn detect_project_root(explicit: Option<String>) -> anyhow::Result<PathBuf> {
    if let Some(root) = explicit.filter(|value| !value.trim().is_empty()) {
        return Ok(PathBuf::from(root));
    }
    if let Some(root) = shell_output(&["git", "rev-parse", "--show-toplevel"], None) {
        return Ok(PathBuf::from(root));
    }
    Ok(std::env::current_dir()?)
}

fn has_git_repo(project_root: &Path) -> bool {
    project_root.join(".git").exists()
        || shell_output(
            &["git", "rev-parse", "--is-inside-work-tree"],
            Some(project_root),
        )
        .as_deref()
            == Some("true")
}

fn license_visible(project_root: &Path) -> bool {
    project_root.join("LICENSE.md").exists()
        || project_root.join("LICENSE").exists()
        || project_root.join("COMMERCIAL.md").exists()
}

fn pi_extension_visible(project_root: &Path) -> bool {
    project_root.join("apps/pi-extension/package.json").exists()
        || std::env::var("HOME")
            .ok()
            .map(|home| {
                PathBuf::from(home)
                    .join(".pi/skills/focusa/SKILL.md")
                    .exists()
            })
            .unwrap_or(false)
}

fn encode_query(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

fn ok_status(ok: bool) -> &'static str {
    if ok { "ok" } else { "needs_attention" }
}

fn print_human(response: &Value) {
    println!("FOCUSA OPERATOR PREVIEW ONBOARDING");
    println!(
        "Status: {}",
        response["status"].as_str().unwrap_or("unknown")
    );
    println!(
        "Project: {}",
        response["project_root"].as_str().unwrap_or("unknown")
    );
    println!(
        "Agent mode: {}",
        response["agent"].as_str().unwrap_or("manual")
    );
    println!(
        "Daemon: {}",
        response["checks"]["daemon"].as_str().unwrap_or("unknown")
    );
    println!(
        "API health: {}",
        response["checks"]["api_health"]
            .as_str()
            .unwrap_or("unknown")
    );
    println!(
        "Git repo: {}",
        response["checks"]["git_repo"].as_str().unwrap_or("unknown")
    );
    println!(
        "License: {}",
        response["checks"]["license"].as_str().unwrap_or("unknown")
    );
    println!(
        "Pi extension: {}",
        response["checks"]["pi_extension"]
            .as_str()
            .unwrap_or("unknown")
    );
    if let Some(id) = response
        .pointer("/workpoint/workpoint_id")
        .and_then(Value::as_str)
    {
        println!("Demo Workpoint: {id}");
    }
    if let Some(summary) = response
        .pointer("/resume/rendered_summary")
        .and_then(Value::as_str)
    {
        println!("Resume: {summary}");
    }
    println!(
        "Next: {}",
        response["next_command"].as_str().unwrap_or("focusa doctor")
    );
}

pub async fn run(args: OnboardArgs, json_mode: bool) -> anyhow::Result<()> {
    let project_root = detect_project_root(args.project_root)?;
    let project_root_str = project_root.display().to_string();
    let continuity_id = args
        .continuity_id
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("focusa-onboard-{}", chrono::Utc::now().timestamp()));

    let git_repo = has_git_repo(&project_root);
    let license = license_visible(&project_root);
    let pi_extension = pi_extension_visible(&project_root);

    let started = daemon::start().await.unwrap_or(false);
    let api = ApiClient::with_timeout_secs(8);
    let health = api.get("/v1/health").await;
    let health_ok = health.is_ok();

    let project_identity = api
        .get(&format!(
            "/v1/project/identity?project_root={}",
            encode_query(&project_root_str)
        ))
        .await
        .unwrap_or_else(|err| json!({"status":"blocked","error":err.to_string()}));

    let mut workpoint = Value::Null;
    let mut resume = Value::Null;
    if !args.no_demo_workpoint && health_ok {
        workpoint = api
            .post(
                "/v1/workpoint/checkpoint",
                &json!({
                    "mission": "Operator Preview onboarding: prove Workpoint continuity in this project",
                    "next_slice": "Link evidence, simulate compaction, then resume this Workpoint",
                    "work_item_id": "focusa-onboard-demo",
                    "project_root": project_root_str,
                    "continuity_id": continuity_id,
                    "checkpoint_reason": "session_start",
                    "canonical": true,
                    "promote": true,
                    "action_intent": {
                        "action_type": "operator_preview_onboarding",
                        "target_ref": "docs/current/FOCUSA_OPERATOR_PREVIEW_PROOF.md",
                        "verification_hooks": [],
                        "status": "ready"
                    },
                    "active_object_refs": ["docs/current/FOCUSA_OPERATOR_PREVIEW_PROOF.md"]
                }),
            )
            .await
            .unwrap_or_else(|err| json!({"status":"blocked","error":err.to_string()}));

        resume = api
            .post(
                "/v1/workpoint/resume",
                &json!({
                    "mode": "compact_prompt",
                    "project_root": project_root_str,
                    "continuity_id": continuity_id,
                }),
            )
            .await
            .unwrap_or_else(|err| json!({"status":"blocked","error":err.to_string()}));
    }

    let status = if health_ok && git_repo && license {
        "ready"
    } else {
        "needs_attention"
    };
    let response = json!({
        "status": status,
        "agent": args.agent,
        "project_root": project_root.display().to_string(),
        "continuity_id": continuity_id,
        "checks": {
            "daemon": if started { "started" } else if health_ok { "already_running" } else { "blocked" },
            "api_health": ok_status(health_ok),
            "git_repo": ok_status(git_repo),
            "license": ok_status(license),
            "pi_extension": if pi_extension { "available" } else { "manual_mode_available" }
        },
        "project_identity": project_identity,
        "workpoint": workpoint,
        "resume": resume,
        "next_command": "focusa workpoint resume --mode compact_prompt",
        "manual_mode_command": "focusa awareness card --adapter-id manual --workspace-id local --agent-id cli",
        "proof_doc": "docs/current/FOCUSA_OPERATOR_PREVIEW_PROOF.md",
        "known_limits": [
            "Pi is the best-supported deep harness path today.",
            "Manual mode uses CLI-generated continuation cards for non-Pi agents.",
            "Menubar GUI is not the primary preview surface."
        ]
    });

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        print_human(&response);
    }
    Ok(())
}
