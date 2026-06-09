//! Spec96 ProjectIdentity CLI parity commands.

use crate::api_client::ApiClient;
use crate::commands::scope::ensure_project_root_scope_safe;
use clap::Subcommand;
use serde_json::{Value, json};

#[derive(Subcommand)]
pub enum ProjectCmd {
    /// Read hot-path ProjectIdentity for cwd/project_root.
    Identity {
        #[arg(long)]
        cwd: Option<String>,
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        remote_host: Option<String>,
        #[arg(long)]
        remote_user: Option<String>,
        #[arg(long)]
        remote_port: Option<u16>,
        #[arg(long)]
        remote_repo_remote: Option<String>,
        #[arg(long)]
        remote_workspace_kind: Option<String>,
        #[arg(long)]
        remote_deploy_root: Option<String>,
        #[arg(long)]
        persisted_project_root: Option<String>,
        #[arg(long)]
        persisted_project_fingerprint: Option<String>,
        #[arg(long)]
        persisted_project_id: Option<String>,
        #[arg(long)]
        persisted_canonical_name: Option<String>,
    },
    /// Build advisory Project Card from identity, ontology, trajectory, prediction, evidence, and learning-loop signals.
    Card {
        #[arg(long)]
        cwd: Option<String>,
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        current_ask: Option<String>,
        #[arg(long)]
        remote_host: Option<String>,
        #[arg(long)]
        remote_user: Option<String>,
        #[arg(long)]
        remote_port: Option<u16>,
        #[arg(long)]
        remote_repo_remote: Option<String>,
        #[arg(long)]
        remote_workspace_kind: Option<String>,
        #[arg(long)]
        remote_deploy_root: Option<String>,
    },
    /// Attach final outcome/evaluation to a project-card algorithm_run_id.
    CardOutcome {
        #[arg(long)]
        algorithm_run_id: String,
        #[arg(long)]
        actual_outcome: String,
        #[arg(long)]
        score: Option<f64>,
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long = "evidence-ref")]
        evidence_refs: Vec<String>,
        #[arg(long)]
        notes: Option<String>,
    },
    /// Save or continue a Focusa session-transfer packet.
    SessionTransfer {
        #[arg(long, default_value = "status")]
        action: String,
        #[arg(long)]
        cwd: Option<String>,
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        current_ask: Option<String>,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long)]
        mission: Option<String>,
        #[arg(long)]
        next_action: Option<String>,
    },
    /// Verify expected project identity signals against discovered ProjectIdentity.
    Verify {
        #[arg(long)]
        cwd: Option<String>,
        #[arg(long)]
        project_root: Option<String>,
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        canonical_name: Option<String>,
        #[arg(long)]
        repo_remote: Option<String>,
        #[arg(long)]
        remote_host: Option<String>,
        #[arg(long)]
        remote_user: Option<String>,
        #[arg(long)]
        remote_port: Option<u16>,
        #[arg(long)]
        remote_repo_remote: Option<String>,
        #[arg(long)]
        remote_workspace_kind: Option<String>,
        #[arg(long)]
        remote_deploy_root: Option<String>,
        #[arg(long)]
        persisted_project_root: Option<String>,
        #[arg(long)]
        persisted_project_fingerprint: Option<String>,
        #[arg(long)]
        persisted_project_id: Option<String>,
        #[arg(long)]
        persisted_canonical_name: Option<String>,
    },
}

fn encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' | b':' => {
                vec![b as char]
            }
            b' ' => vec!['+'],
            _ => format!("%{b:02X}").chars().collect(),
        })
        .collect()
}

fn push_query(qs: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
        qs.push(format!("{key}={}", encode(value)));
    }
}

fn print_summary(label: &str, resp: &Value) {
    let status = resp
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let canonical = resp
        .get("canonical")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let null_value = Value::Null;
    let project = resp.get("project_identity").unwrap_or(&null_value);
    let root = project
        .get("project_root")
        .and_then(Value::as_str)
        .unwrap_or("unbound");
    let confidence = project
        .get("confidence")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let project_status = project
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    println!(
        "project {label}: status={status} canonical={canonical} project_status={project_status} confidence={confidence}"
    );
    println!("  project_root: {root}");
    if let Some(next) = resp
        .get("verification")
        .and_then(|v| v.get("required_recovery"))
        .and_then(Value::as_str)
    {
        println!("  recovery: {next}");
    }
}

pub async fn run(cmd: ProjectCmd, json_output: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let (label, resp) = match cmd {
        ProjectCmd::Identity {
            cwd,
            project_root,
            remote_host,
            remote_user,
            remote_port,
            remote_repo_remote,
            remote_workspace_kind,
            remote_deploy_root,
            persisted_project_root,
            persisted_project_fingerprint,
            persisted_project_id,
            persisted_canonical_name,
        } => {
            ensure_project_root_scope_safe(cwd.as_deref(), "project identity: cwd")?;
            ensure_project_root_scope_safe(
                project_root.as_deref(),
                "project identity: project_root",
            )?;
            ensure_project_root_scope_safe(
                persisted_project_root.as_deref(),
                "project identity: persisted_project_root",
            )?;
            let mut qs = Vec::new();
            push_query(&mut qs, "cwd", cwd.as_deref());
            push_query(&mut qs, "project_root", project_root.as_deref());
            push_query(&mut qs, "remote_host", remote_host.as_deref());
            push_query(&mut qs, "remote_user", remote_user.as_deref());
            if let Some(port) = remote_port {
                qs.push(format!("remote_port={port}"));
            }
            push_query(&mut qs, "remote_repo_remote", remote_repo_remote.as_deref());
            push_query(
                &mut qs,
                "remote_workspace_kind",
                remote_workspace_kind.as_deref(),
            );
            push_query(&mut qs, "remote_deploy_root", remote_deploy_root.as_deref());
            push_query(
                &mut qs,
                "persisted_project_root",
                persisted_project_root.as_deref(),
            );
            push_query(
                &mut qs,
                "persisted_project_fingerprint",
                persisted_project_fingerprint.as_deref(),
            );
            push_query(
                &mut qs,
                "persisted_project_id",
                persisted_project_id.as_deref(),
            );
            push_query(
                &mut qs,
                "persisted_canonical_name",
                persisted_canonical_name.as_deref(),
            );
            let path = if qs.is_empty() {
                "/v1/project/identity".to_string()
            } else {
                format!("/v1/project/identity?{}", qs.join("&"))
            };
            ("identity", api.get(&path).await?)
        }
        ProjectCmd::Card {
            cwd,
            project_root,
            current_ask,
            remote_host,
            remote_user,
            remote_port,
            remote_repo_remote,
            remote_workspace_kind,
            remote_deploy_root,
        } => {
            ensure_project_root_scope_safe(cwd.as_deref(), "project card: cwd")?;
            ensure_project_root_scope_safe(project_root.as_deref(), "project card: project_root")?;
            let mut qs = Vec::new();
            push_query(&mut qs, "cwd", cwd.as_deref());
            push_query(&mut qs, "project_root", project_root.as_deref());
            push_query(&mut qs, "current_ask", current_ask.as_deref());
            push_query(&mut qs, "remote_host", remote_host.as_deref());
            push_query(&mut qs, "remote_user", remote_user.as_deref());
            if let Some(port) = remote_port {
                qs.push(format!("remote_port={port}"));
            }
            push_query(&mut qs, "remote_repo_remote", remote_repo_remote.as_deref());
            push_query(
                &mut qs,
                "remote_workspace_kind",
                remote_workspace_kind.as_deref(),
            );
            push_query(&mut qs, "remote_deploy_root", remote_deploy_root.as_deref());
            let path = if qs.is_empty() {
                "/v1/project/card".to_string()
            } else {
                format!("/v1/project/card?{}", qs.join("&"))
            };
            ("card", api.get(&path).await?)
        }
        ProjectCmd::CardOutcome {
            algorithm_run_id,
            actual_outcome,
            score,
            project_root,
            evidence_refs,
            notes,
        } => {
            ensure_project_root_scope_safe(
                project_root.as_deref(),
                "project card outcome: project_root",
            )?;
            let body = json!({
                "algorithm_run_id": algorithm_run_id,
                "actual_outcome": actual_outcome,
                "score": score,
                "project_root": project_root,
                "evidence_refs": evidence_refs,
                "notes": notes,
            });
            (
                "card-outcome",
                api.post("/v1/project/card/outcome", &body).await?,
            )
        }
        ProjectCmd::SessionTransfer {
            action,
            cwd,
            project_root,
            current_ask,
            continuity_id,
            mission,
            next_action,
        } => {
            ensure_project_root_scope_safe(cwd.as_deref(), "project session-transfer: cwd")?;
            ensure_project_root_scope_safe(
                project_root.as_deref(),
                "project session-transfer: project_root",
            )?;
            let body = json!({
                "action": action,
                "cwd": cwd,
                "project_root": project_root,
                "current_ask": current_ask,
                "continuity_id": continuity_id,
                "mission": mission,
                "next_action": next_action,
            });
            (
                "session-transfer",
                api.post("/v1/project/session-transfer", &body).await?,
            )
        }
        ProjectCmd::Verify {
            cwd,
            project_root,
            project_id,
            canonical_name,
            repo_remote,
            remote_host,
            remote_user,
            remote_port,
            remote_repo_remote,
            remote_workspace_kind,
            remote_deploy_root,
            persisted_project_root,
            persisted_project_fingerprint,
            persisted_project_id,
            persisted_canonical_name,
        } => {
            ensure_project_root_scope_safe(cwd.as_deref(), "project verify: cwd")?;
            ensure_project_root_scope_safe(
                project_root.as_deref(),
                "project verify: project_root",
            )?;
            ensure_project_root_scope_safe(
                persisted_project_root.as_deref(),
                "project verify: persisted_project_root",
            )?;
            let body = json!({
                "cwd": cwd,
                "project_root": project_root,
                "project_id": project_id,
                "canonical_name": canonical_name,
                "repo_remote": repo_remote,
                "remote_host": remote_host,
                "remote_user": remote_user,
                "remote_port": remote_port,
                "remote_repo_remote": remote_repo_remote,
                "remote_workspace_kind": remote_workspace_kind,
                "remote_deploy_root": remote_deploy_root,
                "persisted_project_root": persisted_project_root,
                "persisted_project_fingerprint": persisted_project_fingerprint,
                "persisted_project_id": persisted_project_id,
                "persisted_canonical_name": persisted_canonical_name,
            });
            ("verify", api.post("/v1/project/verify", &body).await?)
        }
    };
    if json_output {
        println!("{}", serde_json::to_string_pretty(&resp)?);
    } else {
        print_summary(label, &resp);
    }
    Ok(())
}
