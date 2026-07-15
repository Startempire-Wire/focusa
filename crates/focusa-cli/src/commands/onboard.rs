//! First-run Operator Preview onboarding flow.

use crate::api_client::ApiClient;
use crate::commands::daemon;
use chrono::Utc;
use clap::Args;
use focusa_core::scope_safety::classify_project_root;
use serde_json::{Value, json};
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Args)]
pub struct OnboardArgs {
    /// Agent harness to orient for; currently informational. Example: pi, claude-code, cursor.
    #[arg(long, default_value = "manual")]
    pub agent: String,

    /// Onboarding scope. Default project preserves existing behavior. Use
    /// --scope host for instance-level setup on a non-Focusa/non-project host.
    #[arg(long, value_name = "SCOPE", default_value = "project")]
    pub scope: OnboardScope,

    /// Explicit project root. Defaults to git root, then current directory.
    #[arg(long)]
    pub project_root: Option<String>,

    /// Remote git URL to record in a local `.focusa-project.json` marker.
    /// This is the low-risk remote/VPS onboarding path: run it from the
    /// remote checkout (or pass --project-root) to bind the project before
    /// Workpoint/Trajectory authority is accepted.
    #[arg(long, value_name = "GIT_URL")]
    pub remote: Option<String>,

    /// Stable continuity id for the demo Workpoint.
    #[arg(long)]
    pub continuity_id: Option<String>,

    /// Skip creating the demo Workpoint.
    #[arg(long)]
    pub no_demo_workpoint: bool,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OnboardScope {
    /// Instance-level setup: start daemon and show host readiness only. Does
    /// not create a project Workpoint and does not require git/license files.
    Host,
    /// Project-level setup: existing behavior. Requires safe project root;
    /// may create a demo Workpoint when daemon is healthy.
    #[default]
    Project,
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

fn safe_project_root(project_root: &Path) -> bool {
    let root = project_root.to_string_lossy();
    let trimmed = root.trim_end_matches('/');
    match trimmed {
        "" | "/" | "/root" | "/home" | "/tmp" | "/var" | "/usr" | "/opt" => false,
        _ => classify_project_root(&root).is_safe(),
    }
}

fn slug_from_remote(remote: &str) -> String {
    let trimmed = remote.trim().trim_end_matches('/').trim_end_matches(".git");
    let tail = trimmed.rsplit('/').next().unwrap_or(trimmed);
    let tail = tail.rsplit(':').next().unwrap_or(tail);
    tail.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn title_from_slug(slug: &str) -> String {
    slug.split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn detect_workspace_kind(project_root: &Path) -> &'static str {
    if project_root.join("Cargo.toml").exists() {
        "rust-monorepo"
    } else if project_root.join("go.mod").exists() || project_root.join("go.work").exists() {
        "go-workspace"
    } else if project_root.join("package.json").exists() {
        "node-workspace"
    } else {
        "unknown"
    }
}

fn ensure_project_marker(project_root: &Path, remote: &str) -> anyhow::Result<Value> {
    if !project_root.exists() {
        std::fs::create_dir_all(project_root).map_err(|e| {
            anyhow::anyhow!(
                "create_dir_all failed for {}: {}",
                project_root.display(),
                e
            )
        })?;
    }
    let marker_path = project_root.join(".focusa-project.json");
    if marker_path.exists() {
        let existing: Value = serde_json::from_slice(&fs::read(&marker_path)?)
            .unwrap_or_else(|_| json!({"status":"unreadable"}));
        if existing
            .get("repo_remote")
            .and_then(Value::as_str)
            .map(|value| value == remote.trim())
            .unwrap_or(false)
        {
            return Ok(json!({"status":"exists","path":marker_path,"repo_remote":remote.trim()}));
        }
        anyhow::bail!(
            "project marker already exists at {} with a different repo_remote; refusing to overwrite",
            marker_path.display()
        );
    }
    fs::create_dir_all(project_root)?;
    let slug = slug_from_remote(remote);
    if slug.is_empty() {
        anyhow::bail!("--remote requires a git URL with a repository name");
    }
    let marker = json!({
        "schema": "focusa.project.v1",
        "project_id": slug.clone(),
        "canonical_name": title_from_slug(&slug),
        "project_root": project_root.display().to_string(),
        "repo_remote": remote.trim(),
        "beads_prefix": slug.clone(),
        "workspace_kind": detect_workspace_kind(project_root),
        "aliases": [],
        "created_at": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    });
    fs::write(&marker_path, serde_json::to_vec_pretty(&marker)?)?;
    Ok(json!({"status":"created","path":marker_path,"repo_remote":remote.trim(),"marker":marker}))
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
    if let Some(marker_status) = response.pointer("/marker/status").and_then(Value::as_str) {
        println!("Project marker: {marker_status}");
    }
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
    let quiet = std::env::args().any(|arg| arg == "--quiet")
        || matches!(
            std::env::var("FOCUSA_QUIET").ok().as_deref(),
            Some("1") | Some("true")
        );
    let tty = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    let scope_label = if matches!(args.scope, OnboardScope::Host) {
        "host"
    } else {
        "project"
    };
    if !json_mode && !quiet && tty {
        println!(
            "{}",
            crate::commands::intro::render_onboard_banner(
                &args
                    .project_root
                    .clone()
                    .unwrap_or_else(|| std::env::current_dir()
                        .map(|d| d.display().to_string())
                        .unwrap_or_else(|_| ".".into())),
                scope_label
            )
        );
    }
    let _scope_idx = if json_mode || quiet || !tty {
        0
    } else {
        let intent = crate::commands::intro::detect_prompt_intent();
        crate::commands::intro::pick_scope_intent(intent, |choices| {
            // Tiny interactive picker: print arrows + read number (1-2) from stdin.
            use std::io::{BufRead, Write};
            for (idx, choice) in choices.iter().enumerate() {
                println!("  {}. {}", idx + 1, choice);
            }
            print!("Choose [1-{}]: ", choices.len());
            let _ = std::io::stdout().flush();
            let mut input = String::new();
            std::io::stdin().lock().read_line(&mut input).unwrap_or(0);
            let n = input.trim().parse::<usize>().unwrap_or(1);
            if n == 0 || n > choices.len() {
                0
            } else {
                n - 1
            }
        })
    };
    let project_root = detect_project_root(args.project_root)?;
    let project_root_str = project_root.display().to_string();
    let project_scope = matches!(args.scope, OnboardScope::Project);
    if project_scope && !safe_project_root(&project_root) {
        anyhow::bail!(
            "unsafe project root for project onboarding: {project_root_str}. Use --scope host for instance-level setup or pass --project-root to a focused project directory."
        );
    }
    let project_marker = if project_scope {
        match args.remote.as_deref() {
            Some(remote) => ensure_project_marker(&project_root, remote)?,
            None => Value::Null,
        }
    } else {
        Value::Null
    };
    let continuity_id = args
        .continuity_id
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("focusa-onboard-{}", chrono::Utc::now().timestamp()));

    let git_repo = project_scope && has_git_repo(&project_root);
    let license = project_scope && license_visible(&project_root);
    let pi_extension = project_scope && pi_extension_visible(&project_root);

    let started = daemon::start().await.unwrap_or(false);
    let api = ApiClient::with_timeout_secs(8);
    let health = api.get("/v1/health").await;
    let health_ok = health.is_ok();

    let project_identity = if project_scope {
        api.get(&format!(
            "/v1/project/identity?project_root={}",
            encode_query(&project_root_str)
        ))
        .await
        .unwrap_or_else(|err| json!({"status":"blocked","error":err.to_string()}))
    } else {
        json!({"status":"skipped","reason":"host-scope onboarding does not bind project identity"})
    };

    let mut workpoint = Value::Null;
    let mut resume = Value::Null;
    if project_scope && !args.no_demo_workpoint && health_ok {
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

    let status = if health_ok && (!project_scope || (git_repo && license)) {
        "ready"
    } else {
        "needs_attention"
    };
    let response = json!({
        "status": status,
        "agent": args.agent,
        "scope": args.scope,
        "project_root": if project_scope { project_root.display().to_string() } else { "".to_string() },
        "continuity_id": continuity_id,
        "checks": {
            "daemon": if started { "started" } else if health_ok { "already_running" } else { "blocked" },
            "api_health": ok_status(health_ok),
            "git_repo": if project_scope { ok_status(git_repo) } else { "skipped" },
            "license": if project_scope { ok_status(license) } else { "skipped" },
            "pi_extension": if project_scope { if pi_extension { "available" } else { "manual_mode_available" } } else { "skipped" }
        },
        "project_identity": project_identity,
        "project_marker": project_marker,
        "workpoint": workpoint,
        "resume": resume,
        "next_command": if project_scope { "focusa workpoint resume --mode compact_prompt" } else { "focusa doctor --scope host" },
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
    } else if !quiet {
        print_human(&response);
    }
    Ok(())
}
