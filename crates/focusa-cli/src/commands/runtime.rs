//! Runtime inventory and daemon hygiene CLI.

use crate::api_client::ApiClient;
use clap::{Args, Subcommand};
use serde::Serialize;

#[derive(Subcommand)]
pub enum RuntimeCmd {
    /// Report CLI/daemon runtime inventory.
    Inventory(RuntimeInventoryArgs),
}

#[derive(Debug, Clone, Args)]
pub struct RuntimeInventoryArgs {
    /// Expected daemon owner; mismatches become hygiene warnings.
    #[arg(long)]
    pub owner: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RuntimeInventory {
    pub schema: &'static str,
    pub daemon: DaemonInventory,
    pub cli: CliInventory,
    pub hygiene: RuntimeHygiene,
}

#[derive(Debug, Serialize)]
pub struct DaemonInventory {
    pub running: bool,
    pub pid: Option<u32>,
    pub user: Option<String>,
    pub bind: String,
    pub version: Option<String>,
    pub lock_pid: Option<u32>,
    pub lock_matches_process: Option<bool>,
    pub one_listener_per_bind: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct CliInventory {
    pub path: Option<String>,
    pub version: &'static str,
}

#[derive(Debug, Serialize)]
pub struct RuntimeHygiene {
    pub status: &'static str,
    pub warnings: Vec<String>,
    pub recommended_action: Option<String>,
}

pub async fn run(cmd: RuntimeCmd, json_mode: bool) -> anyhow::Result<()> {
    match cmd {
        RuntimeCmd::Inventory(args) => {
            let inventory = collect_inventory(args.owner.as_deref()).await;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&inventory)?);
            } else {
                println!("runtime inventory");
                println!("  cli: {} ({})", inventory.cli.path.as_deref().unwrap_or("unknown"), inventory.cli.version);
                println!("  daemon running: {}", inventory.daemon.running);
                println!("  daemon version: {}", inventory.daemon.version.as_deref().unwrap_or("unknown"));
                println!("  daemon pid: {}", inventory.daemon.pid.map(|pid| pid.to_string()).unwrap_or_else(|| "unknown".to_string()));
                println!("  hygiene: {}", inventory.hygiene.status);
                for warning in &inventory.hygiene.warnings {
                    println!("  warning: {warning}");
                }
                if let Some(action) = &inventory.hygiene.recommended_action {
                    println!("  recommended_action: {action}");
                }
            }
        }
    }
    Ok(())
}

pub async fn collect_inventory(expected_owner: Option<&str>) -> RuntimeInventory {
    let bind = std::env::var("FOCUSA_BIND").unwrap_or_else(|_| "127.0.0.1:8787".to_string());
    let client = ApiClient::new();
    let health = client.get("/v1/health").await.ok();
    let daemon_running = health.is_some();
    let daemon_version = health
        .as_ref()
        .and_then(|value| value.get("version"))
        .and_then(|value| value.as_str())
        .map(ToString::to_string);

    let pid = pgrep_focusa_daemon();
    let user = pid.and_then(process_user);
    let lock_pid = read_lock_pid();
    let lock_matches_process = match (lock_pid, pid) {
        (Some(lock_pid), Some(pid)) => Some(lock_pid == pid),
        (Some(_), None) => Some(false),
        _ => None,
    };

    let mut warnings = Vec::new();
    if daemon_running && daemon_version.as_deref() != Some(env!("CARGO_PKG_VERSION")) {
        warnings.push(format!(
            "daemon version {} differs from CLI {}",
            daemon_version.as_deref().unwrap_or("unknown"),
            env!("CARGO_PKG_VERSION")
        ));
    }
    if lock_matches_process == Some(false) {
        warnings.push("daemon lock PID does not match live focusa-daemon process".to_string());
    }
    if daemon_running && pid.is_none() {
        warnings.push("daemon health endpoint responds but process PID was not found by pgrep".to_string());
    }
    if let (Some(expected_owner), Some(actual_user)) = (expected_owner, user.as_deref()) {
        if actual_user != expected_owner {
            warnings.push(format!(
                "daemon user {actual_user} differs from expected owner {expected_owner}"
            ));
        }
    }

    let status = if warnings.is_empty() { "ok" } else { "degraded" };
    let recommended_action = if warnings.is_empty() {
        None
    } else {
        Some("run focusa doctor; if this is a live build host, repair from local repo and restart daemon as owner".to_string())
    };

    RuntimeInventory {
        schema: "focusa.runtime_inventory.v1",
        daemon: DaemonInventory {
            running: daemon_running,
            pid,
            user,
            bind,
            version: daemon_version,
            lock_pid,
            lock_matches_process,
            one_listener_per_bind: Some(true),
        },
        cli: CliInventory {
            path: std::env::current_exe().ok().map(|path| path.display().to_string()),
            version: env!("CARGO_PKG_VERSION"),
        },
        hygiene: RuntimeHygiene {
            status,
            warnings,
            recommended_action,
        },
    }
}

fn pgrep_focusa_daemon() -> Option<u32> {
    let output = std::process::Command::new("pgrep")
        .arg("-f")
        .arg("focusa-daemon")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()?
        .lines()
        .filter_map(|line| line.trim().parse::<u32>().ok())
        .next()
}

fn process_user(pid: u32) -> Option<String> {
    let output = std::process::Command::new("ps")
        .arg("-o")
        .arg("user=")
        .arg("-p")
        .arg(pid.to_string())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn read_lock_pid() -> Option<u32> {
    for path in [
        "/tmp/focusa-daemon.lock",
        "/tmp/focusa/focusa-daemon.lock",
        "runtime/focusa-daemon.lock",
    ] {
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        if let Ok(pid) = content.trim().parse::<u32>() {
            return Some(pid);
        }
    }
    None
}
