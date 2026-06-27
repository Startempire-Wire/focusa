//! focusa pairing status + history subcommands (focusa-ui0y v0.9.35-dev G11+G12).
//!
//! Both subcommands are operator-friendly views over the PairingStore.
//! They hit the daemon's REST API (no direct DB access from the CLI) so
//! they work the same way against a local or remote Focusa daemon.

use anyhow::{Context, Result};
use clap::Parser;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;

#[derive(Parser, Debug, Clone)]
pub struct StatusArgs {
    /// Base URL of the Focusa daemon (default 127.0.0.1:8787).
    #[arg(long, default_value = "http://127.0.0.1:8787")]
    pub base_url: String,
    /// JSON output (machine-readable).
    #[arg(long)]
    pub json: bool,
}

#[derive(Parser, Debug, Clone)]
pub struct HistoryArgs {
    #[arg(long, default_value = "http://127.0.0.1:8787")]
    pub base_url: String,
    #[arg(long, default_value_t = 30)]
    pub limit: usize,
    /// Filter by host label (default: all hosts).
    #[arg(long)]
    pub host: Option<String>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Serialize)]
pub struct StatusReport {
    pub daemon_reachable: bool,
    pub daemon_version: Option<String>,
    pub daemon_uptime_ms: Option<u64>,
    pub active_rooms: Option<usize>,
    pub paired_devices: Option<usize>,
    pub expiring_soon: Option<usize>,
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct HistoryEntry {
    pub device_id: String,
    pub host: String,
    pub created_at: String,
    pub revoked: bool,
    pub token_preview: String,
}

#[derive(Debug, Serialize)]
pub struct HistoryReport {
    pub entries: Vec<HistoryEntry>,
    pub total: usize,
}

pub async fn run_status(args: StatusArgs) -> Result<()> {
    let mut report = StatusReport {
        daemon_reachable: false,
        daemon_version: None,
        daemon_uptime_ms: None,
        active_rooms: None,
        paired_devices: None,
        expiring_soon: None,
        notes: Vec::new(),
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()?;

    // Daemon health
    if let Ok(r) = client.get(format!("{}/v1/health", args.base_url)).send().await {
        if r.status().is_success() {
            report.daemon_reachable = true;
            if let Ok(v) = r.json::<Value>().await {
                report.daemon_version = v.get("version").and_then(|x| x.as_str()).map(String::from);
                report.daemon_uptime_ms = v.get("uptime_ms").and_then(|x| x.as_u64());
            }
        }
    }
    if !report.daemon_reachable {
        report.notes.push(format!(
            "daemon unreachable at {}; verify it is running (focusa pairing doctor)",
            args.base_url
        ));
        if args.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            println!("Focusa Pairing Status");
            println!("  daemon:  ✗ unreachable at {}", args.base_url);
            println!("  hint:    focusa pairing doctor");
        }
        return Ok(());
    }

    // Count active rooms (status=waiting_for_phone or mac_seen) via /v1/connect/room/create (no list endpoint yet, so we approximate)
    if let Ok(r) = client.get(format!("{}/v1/device/pair/list?host=default", args.base_url)).send().await {
        if r.status().is_success() {
            if let Ok(v) = r.json::<Value>().await {
                let devices = v.get("devices").and_then(|x| x.as_array()).cloned().unwrap_or_default();
                let paired = devices
                    .iter()
                    .filter(|d| !d.get("revoked").and_then(|x| x.as_bool()).unwrap_or(false))
                    .count();
                report.paired_devices = Some(paired);
            }
        }
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("Focusa Pairing Status");
        println!("  daemon:    {} ({})", report.daemon_version.as_deref().unwrap_or("?"), args.base_url);
        if let Some(up) = report.daemon_uptime_ms {
            println!("  uptime:    {} ms", up);
        }
        if let Some(p) = report.paired_devices {
            println!("  paired:    {} active devices", p);
        }
        if let Some(n) = report.active_rooms {
            println!("  rooms:     {} active", n);
        }
    }
    Ok(())
}

pub async fn run_history(args: HistoryArgs) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()?;
    let host_q = args
        .host
        .as_deref()
        .map(|h| format!("?host={}", h))
        .unwrap_or_default();
    let url = format!("{}/v1/device/pair/list{}", args.base_url, host_q);
    let r = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("GET {url}"))?;
    if !r.status().is_success() {
        anyhow::bail!("GET {url} returned HTTP {}", r.status());
    }
    let v: Value = r.json().await.context("decode /v1/device/pair/list")?;
    let devices = v.get("devices").and_then(|x| x.as_array()).cloned().unwrap_or_default();

    let entries: Vec<HistoryEntry> = devices
        .into_iter()
        .take(args.limit)
        .map(|d| HistoryEntry {
            device_id: d.get("device_id").and_then(|x| x.as_str()).unwrap_or("?").to_string(),
            host: d.get("host").and_then(|x| x.as_str()).unwrap_or("?").to_string(),
            created_at: d.get("created_at").and_then(|x| x.as_str()).unwrap_or("?").to_string(),
            revoked: d.get("revoked").and_then(|x| x.as_bool()).unwrap_or(false),
            token_preview: d
                .get("token")
                .and_then(|x| x.as_str())
                .map(|s| format!("{}…", &s[..s.len().min(6)]))
                .unwrap_or_else(|| "(none)".to_string()),
        })
        .collect();

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&HistoryReport {
                entries: entries.clone(),
                total: entries.len(),
            })?
        );
        return Ok(());
    }
    println!("Focusa Pairing History ({} entries)", entries.len());
    println!("  {:<40}  {:<20}  {:<22}  {:<6}  token", "device_id", "host", "created_at", "revoked");
    for e in &entries {
        println!(
            "  {:<40}  {:<20}  {:<22}  {:<6}  {}",
            &e.device_id[..e.device_id.len().min(40)],
            &e.host[..e.host.len().min(20)],
            &e.created_at,
            if e.revoked { "yes" } else { "no" },
            e.token_preview
        );
    }
    Ok(())
}