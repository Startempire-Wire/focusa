//! Spec103/106 Call Stack CLI — design, verify, list, show.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::{json, Value};

#[derive(Subcommand)]
pub enum CallStackCmd {
    /// Write a typed Call Stack Design before implementation.
    Design {
        #[arg(long)]
        project_root: String,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long)]
        mission: String,
        #[arg(long, default_value = "pi_tool")]
        entry_surface: String,
        #[arg(long)]
        entry_name: String,
        #[arg(long)]
        workpoint_id: Option<String>,
        #[arg(long, default_value_t = false)]
        attach_to_workpoint: bool,
        #[arg(long, default_value_t = false)]
        attach_to_stg: bool,
        #[arg(long)]
        parent_design_id: Option<String>,
        #[arg(long)]
        notes: Option<String>,
    },
    /// Verify a saved Call Stack Design against implementation surfaces.
    Verify {
        #[arg(long)]
        project_root: String,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long)]
        design_id: Option<String>,
        #[arg(long)]
        entry_name: Option<String>,
    },
    /// List saved Call Stack Designs for a project.
    List {
        #[arg(long)]
        project_root: String,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long)]
        entry_name: Option<String>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Show a saved Call Stack Design by id.
    Show {
        #[arg(long)]
        project_root: String,
        #[arg(long)]
        continuity_id: Option<String>,
        #[arg(long)]
        design_id: String,
    },
}

pub async fn run(cmd: CallStackCmd, json_out: bool) -> anyhow::Result<()> {
    let client = ApiClient::new();
    match cmd {
        CallStackCmd::Design {
            project_root,
            continuity_id,
            mission,
            entry_surface,
            entry_name,
            workpoint_id,
            attach_to_workpoint,
            attach_to_stg,
            parent_design_id,
            notes,
        } => {
            let body = json!({
                "project_root": project_root,
                "continuity_id": continuity_id,
                "mission": mission,
                "entry_surface": entry_surface,
                "entry_name": entry_name,
                "workpoint_id": workpoint_id,
                "attach_to_workpoint": attach_to_workpoint,
                "attach_to_stg": attach_to_stg,
                "parent_design_id": parent_design_id,
                "notes": notes,
            });
            let value = client.post("/v1/call-stack/design", &body).await?;
            print_design(value, json_out)
        }
        CallStackCmd::Verify { project_root, continuity_id, design_id, entry_name } => {
            let body = json!({
                "project_root": project_root,
                "continuity_id": continuity_id,
                "design_id": design_id,
                "entry_name": entry_name,
            });
            let value = client.post("/v1/call-stack/verify", &body).await?;
            print_verify(value, json_out)
        }
        CallStackCmd::List { project_root, continuity_id, entry_name, limit } => {
            let mut path = format!("/v1/call-stack/list?project_root={}&limit={}", urlencoding::encode(&project_root), limit);
            if let Some(cid) = continuity_id.as_deref() {
                path.push_str(&format!("&continuity_id={}", urlencoding::encode(cid)));
            }
            if let Some(name) = entry_name.as_deref() {
                path.push_str(&format!("&entry_name={}", urlencoding::encode(name)));
            }
            let value = client.get(&path).await?;
            print_list(value, json_out)
        }
        CallStackCmd::Show { project_root, continuity_id, design_id } => {
            let mut path = format!("/v1/call-stack/show?project_root={}&design_id={}", urlencoding::encode(&project_root), urlencoding::encode(&design_id));
            if let Some(cid) = continuity_id.as_deref() {
                path.push_str(&format!("&continuity_id={}", urlencoding::encode(cid)));
            }
            let value = client.get(&path).await?;
            print_show(value, json_out)
        }
    }
}

fn print_json(value: &Value) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn print_design(value: Value, json_out: bool) -> anyhow::Result<()> {
    if json_out { return print_json(&value); }
    println!(
        "call-stack design: status={} design_id={} entry={} surface={} advisory=true",
        value["status"].as_str().unwrap_or("unknown"),
        value["design_id"].as_str().unwrap_or("unknown"),
        value["design"]["entry_name"].as_str().unwrap_or("unknown"),
        value["design"]["entry_surface"].as_str().unwrap_or("unknown"),
    );
    Ok(())
}

fn print_verify(value: Value, json_out: bool) -> anyhow::Result<()> {
    if json_out { return print_json(&value); }
    println!(
        "call-stack verify: status={} design_id={} drift_status={} failures={} warnings={} advisory=true",
        value["status"].as_str().unwrap_or("unknown"),
        value["design_id"].as_str().unwrap_or("unknown"),
        value["drift_status"].as_str().unwrap_or("unknown"),
        value["failures"].as_u64().unwrap_or(0),
        value["warnings"].as_u64().unwrap_or(0),
    );
    if let Some(checks) = value["checks"].as_array() {
        for check in checks.iter().take(12) {
            println!(
                "- {}: {} — {}",
                check["id"].as_str().unwrap_or("check"),
                check["status"].as_str().unwrap_or("unknown"),
                check["message"].as_str().unwrap_or("")
            );
        }
    }
    Ok(())
}

fn print_list(value: Value, json_out: bool) -> anyhow::Result<()> {
    if json_out { return print_json(&value); }
    println!("call-stack list: count={} advisory=true", value["count"].as_u64().unwrap_or(0));
    if let Some(designs) = value["designs"].as_array() {
        for design in designs.iter().take(50) {
            println!(
                "- {} entry={} surface={} mission={}",
                design["design_id"].as_str().unwrap_or("unknown"),
                design["entry_name"].as_str().unwrap_or("unknown"),
                design["entry_surface"].as_str().unwrap_or("unknown"),
                design["mission"].as_str().unwrap_or("")
            );
        }
    }
    Ok(())
}

fn print_show(value: Value, json_out: bool) -> anyhow::Result<()> {
    if json_out { return print_json(&value); }
    let design = &value["design"];
    println!(
        "call-stack show: design_id={} entry={} surface={} mission={} advisory=true",
        design["design_id"].as_str().unwrap_or("unknown"),
        design["entry_name"].as_str().unwrap_or("unknown"),
        design["entry_surface"].as_str().unwrap_or("unknown"),
        design["mission"].as_str().unwrap_or("")
    );
    Ok(())
}
