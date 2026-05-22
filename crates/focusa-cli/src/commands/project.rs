//! Spec96 ProjectIdentity CLI parity commands.

use crate::api_client::ApiClient;
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
    },
}

fn encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' | b':' => vec![b as char],
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
    let status = resp.get("status").and_then(Value::as_str).unwrap_or("unknown");
    let canonical = resp.get("canonical").and_then(Value::as_bool).unwrap_or(false);
    let null_value = Value::Null;
    let project = resp.get("project_identity").unwrap_or(&null_value);
    let root = project.get("project_root").and_then(Value::as_str).unwrap_or("unbound");
    let confidence = project.get("confidence").and_then(Value::as_str).unwrap_or("unknown");
    let project_status = project.get("status").and_then(Value::as_str).unwrap_or("unknown");
    println!("project {label}: status={status} canonical={canonical} project_status={project_status} confidence={confidence}");
    println!("  project_root: {root}");
    if let Some(next) = resp.get("verification").and_then(|v| v.get("required_recovery")).and_then(Value::as_str) {
        println!("  recovery: {next}");
    }
}

pub async fn run(cmd: ProjectCmd, json_output: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let (label, resp) = match cmd {
        ProjectCmd::Identity { cwd, project_root } => {
            let mut qs = Vec::new();
            push_query(&mut qs, "cwd", cwd.as_deref());
            push_query(&mut qs, "project_root", project_root.as_deref());
            let path = if qs.is_empty() { "/v1/project/identity".to_string() } else { format!("/v1/project/identity?{}", qs.join("&")) };
            ("identity", api.get(&path).await?)
        }
        ProjectCmd::Verify { cwd, project_root, project_id, canonical_name, repo_remote } => {
            let body = json!({
                "cwd": cwd,
                "project_root": project_root,
                "project_id": project_id,
                "canonical_name": canonical_name,
                "repo_remote": repo_remote,
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
