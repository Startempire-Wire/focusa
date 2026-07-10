//! Spec 128 read-only update inventory/status/check.
//!
//! This command intentionally performs no mutation: no downloads, no binary
//! replacement, no daemon restart. It only inventories local Focusa surfaces
//! and reports stale parts against an operator-supplied or environment-supplied
//! latest version placeholder until the release manifest resolver is wired.

use anyhow::Context;
use clap::{Args, Subcommand};
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

#[derive(Subcommand, Debug)]
pub enum UpdateCmd {
    /// Read-only installed-surface inventory and stale-part summary.
    Status(UpdateStatusArgs),
    /// Read-only update check. Same inventory as status plus channel/latest context.
    Check(UpdateStatusArgs),
}

#[derive(Args, Debug, Clone)]
pub struct UpdateStatusArgs {
    /// Release channel to compare against.
    #[arg(long, default_value = "dev")]
    pub channel: String,

    /// Latest eligible version/tag override. Defaults to FOCUSA_LATEST_VERSION,
    /// then FOCUSA_UPDATE_LATEST_TAG, then this CLI package version.
    #[arg(long, value_name = "VERSION_OR_TAG")]
    pub latest_version: Option<String>,

    /// Daemon health URL used for safe daemon version probing.
    #[arg(long, default_value = "http://127.0.0.1:8787/v1/health")]
    pub daemon_health_url: String,
}

#[derive(Debug, Serialize)]
struct UpdateInventoryEnvelope {
    schema: &'static str,
    status: &'static str,
    command: &'static str,
    read_only: bool,
    mutations_performed: bool,
    channel: String,
    latest: LatestVersion,
    policy: UpdatePolicySummary,
    license: LicenseSummary,
    parts: Vec<InstalledPart>,
    stale_parts: Vec<String>,
    warnings: Vec<String>,
    next_actions: Vec<String>,
}

#[derive(Debug, Serialize)]
struct LatestVersion {
    version: String,
    source: String,
    release_manifest_required: bool,
    eligibility_status: &'static str,
}

#[derive(Debug, Serialize)]
struct UpdatePolicySummary {
    path: String,
    exists: bool,
    enabled: bool,
    mode: &'static str,
    note: &'static str,
}

#[derive(Debug, Serialize)]
struct LicenseSummary {
    level: &'static str,
    dev_mode: bool,
    source: &'static str,
    note: &'static str,
}

#[derive(Debug, Serialize)]
struct InstalledPart {
    part: &'static str,
    expected_path: String,
    resolved_path: Option<String>,
    exists: bool,
    version: Option<String>,
    version_source: &'static str,
    version_probe_safe: bool,
    sha256: Option<String>,
    stale: Option<bool>,
    stale_reason: String,
    notes: Vec<String>,
}

pub async fn run(cmd: UpdateCmd, json_mode: bool) -> anyhow::Result<()> {
    let (command_name, args) = match cmd {
        UpdateCmd::Status(args) => ("status", args),
        UpdateCmd::Check(args) => ("check", args),
    };
    let envelope = build_inventory(command_name, args).await?;
    if json_mode {
        println!("{}", serde_json::to_string_pretty(&envelope)?);
    } else {
        print_human(&envelope);
    }
    Ok(())
}

async fn build_inventory(
    command_name: &'static str,
    args: UpdateStatusArgs,
) -> anyhow::Result<UpdateInventoryEnvelope> {
    let latest = resolve_latest(args.latest_version.as_deref());
    let daemon_health = probe_daemon_health(&args.daemon_health_url).await;
    let parts = vec![
        inspect_cli(&latest.version).await?,
        inspect_daemon(&latest.version, daemon_health).await?,
        inspect_tui(&latest.version).await?,
    ];
    let stale_parts = parts
        .iter()
        .filter(|part| part.stale == Some(true))
        .map(|part| part.part.to_string())
        .collect::<Vec<_>>();
    let mut warnings = vec![
        "read-only inventory only; no update apply, download, binary replacement, or daemon restart was attempted".to_string(),
        "release manifest eligibility/signature/provenance is required before trusted auto-apply".to_string(),
    ];
    for part in &parts {
        if part.stale == Some(true) {
            warnings.push(format!("{} is stale: {}", part.part, part.stale_reason));
        }
        if part.version.is_none() {
            warnings.push(format!(
                "{} version unknown: {}",
                part.part, part.stale_reason
            ));
        }
    }
    let next_actions = if stale_parts.is_empty() {
        vec!["Implement Spec128 policy/license/dev_mode defaults before auto-apply.".to_string()]
    } else {
        vec![
            "Use this stale-part report as input to focusa update plan once Spec128 planning is implemented.".to_string(),
            "Do not manually replace binaries from this command; it is read-only by design.".to_string(),
        ]
    };
    Ok(UpdateInventoryEnvelope {
        schema: "focusa.update_inventory.v1",
        status: "completed",
        command: command_name,
        read_only: true,
        mutations_performed: false,
        channel: args.channel,
        latest,
        policy: update_policy_summary(),
        license: license_summary(),
        parts,
        stale_parts,
        warnings,
        next_actions,
    })
}

fn resolve_latest(override_value: Option<&str>) -> LatestVersion {
    if let Some(v) = override_value.filter(|s| !s.trim().is_empty()) {
        return LatestVersion {
            version: normalize_version(v),
            source: "--latest-version".into(),
            release_manifest_required: true,
            eligibility_status: "placeholder_until_manifest_resolver",
        };
    }
    for env_key in ["FOCUSA_LATEST_VERSION", "FOCUSA_UPDATE_LATEST_TAG"] {
        if let Ok(v) = std::env::var(env_key) {
            if !v.trim().is_empty() {
                return LatestVersion {
                    version: normalize_version(&v),
                    source: env_key.into(),
                    release_manifest_required: true,
                    eligibility_status: "placeholder_until_manifest_resolver",
                };
            }
        }
    }
    LatestVersion {
        version: env!("CARGO_PKG_VERSION").into(),
        source: "current_cli_package_version".into(),
        release_manifest_required: true,
        eligibility_status: "placeholder_until_manifest_resolver",
    }
}

fn update_policy_summary() -> UpdatePolicySummary {
    let path = "/usr/local/lib/focusa/update-policy.json";
    let exists = Path::new(path).exists();
    UpdatePolicySummary {
        path: path.into(),
        exists,
        enabled: false,
        mode: "manual",
        note: "Spec128 policy read/write is not implemented yet; auto-apply remains disabled",
    }
}

fn license_summary() -> LicenseSummary {
    LicenseSummary {
        level: "unknown",
        dev_mode: false,
        source: "not_wired_in_update_status_yet",
        note: "Spec128 license/dev_mode policy integration is next; this command does not grant auto-update authority",
    }
}

async fn inspect_cli(latest: &str) -> anyhow::Result<InstalledPart> {
    let path = resolve_path("focusa", "/usr/local/bin/focusa");
    inspect_executable_part("cli", "/usr/local/bin/focusa", path, latest, true).await
}

async fn inspect_tui(latest: &str) -> anyhow::Result<InstalledPart> {
    let path = resolve_path("focusa-tui", "/usr/local/bin/focusa-tui");
    inspect_executable_part("tui", "/usr/local/bin/focusa-tui", path, latest, true).await
}

async fn inspect_daemon(latest: &str, health: Option<String>) -> anyhow::Result<InstalledPart> {
    let path = resolve_path("focusa-daemon", "/usr/local/bin/focusa-daemon");
    let sha256 = path.as_deref().and_then(|p| sha256_file(Path::new(p)).ok());
    let exists = path.is_some();
    let version = health.as_deref().map(normalize_version);
    let stale = version.as_ref().map(|v| v != latest);
    let stale_reason = match (&version, stale) {
        (Some(v), Some(true)) => format!("running daemon health version {v} differs from latest {latest}"),
        (Some(v), Some(false)) => format!("running daemon health version {v} matches latest {latest}"),
        _ => "daemon version unknown; safe probe uses /v1/health because focusa-daemon --version starts the server".into(),
    };
    Ok(InstalledPart {
        part: "daemon",
        expected_path: "/usr/local/bin/focusa-daemon".into(),
        resolved_path: path,
        exists,
        version,
        version_source: "daemon_health_endpoint",
        version_probe_safe: true,
        sha256,
        stale,
        stale_reason,
        notes: vec!["binary --version intentionally not invoked; current daemon binary treats --version as startup input".into()],
    })
}

async fn inspect_executable_part(
    part: &'static str,
    expected_path: &str,
    path: Option<String>,
    latest: &str,
    probe_version: bool,
) -> anyhow::Result<InstalledPart> {
    let sha256 = path.as_deref().and_then(|p| sha256_file(Path::new(p)).ok());
    let version = if probe_version {
        match path.as_deref() {
            Some(p) => probe_version_command(p)
                .await
                .map(|s| normalize_version(&s)),
            None => None,
        }
    } else {
        None
    };
    let stale = version.as_ref().map(|v| v != latest);
    let stale_reason = match (&version, stale, &path) {
        (_, _, None) => format!("{part} binary not found"),
        (Some(v), Some(true), _) => {
            format!("installed {part} version {v} differs from latest {latest}")
        }
        (Some(v), Some(false), _) => {
            format!("installed {part} version {v} matches latest {latest}")
        }
        _ => format!("{part} version probe unavailable"),
    };
    Ok(InstalledPart {
        part,
        expected_path: expected_path.into(),
        resolved_path: path,
        exists: sha256.is_some(),
        version,
        version_source: "binary_--version",
        version_probe_safe: true,
        sha256,
        stale,
        stale_reason,
        notes: Vec::new(),
    })
}

fn resolve_path(command: &str, canonical: &str) -> Option<String> {
    if Path::new(canonical).exists() {
        return Some(canonical.into());
    }
    which::which(command)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

async fn probe_version_command(path: &str) -> Option<String> {
    let output = timeout(
        Duration::from_secs(3),
        Command::new(path).arg("--version").output(),
    )
    .await
    .ok()?
    .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        None
    } else {
        Some(stdout)
    }
}

async fn probe_daemon_health(url: &str) -> Option<String> {
    if let Some(version) = probe_daemon_health_reqwest(url).await {
        return Some(version);
    }
    probe_local_http_health(url)
}

async fn probe_daemon_health_reqwest(url: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .no_proxy()
        .build()
        .ok()?;
    let body: serde_json::Value = client.get(url).send().await.ok()?.json().await.ok()?;
    body.get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn probe_local_http_health(url: &str) -> Option<String> {
    let rest = url.strip_prefix("http://")?;
    let (host_port, path) = rest.split_once('/')?;
    if !host_port.starts_with("127.0.0.1:") && !host_port.starts_with("localhost:") {
        return None;
    }
    let port = host_port.split_once(':')?.1.parse::<u16>().ok()?;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(1)).ok()?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
    let request = format!("GET /{path} HTTP/1.1\r\nHost: {host_port}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).ok()?;
    let mut response = String::new();
    stream.read_to_string(&mut response).ok()?;
    let (_, body) = response.split_once("\r\n\r\n")?;
    let json: serde_json::Value = serde_json::from_str(body).ok()?;
    json.get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn sha256_file(path: &Path) -> anyhow::Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex::encode(hasher.finalize()))
}

fn normalize_version(raw: &str) -> String {
    let trimmed = raw.trim();
    let last = trimmed.split_whitespace().last().unwrap_or(trimmed);
    last.trim_start_matches('v').to_string()
}

fn print_human(envelope: &UpdateInventoryEnvelope) {
    println!("Focusa update {} (read-only)", envelope.command);
    println!("channel: {}", envelope.channel);
    println!(
        "latest: {} ({})",
        envelope.latest.version, envelope.latest.source
    );
    println!(
        "policy: enabled={} mode={} path={} exists={}",
        envelope.policy.enabled, envelope.policy.mode, envelope.policy.path, envelope.policy.exists
    );
    println!("parts:");
    for part in &envelope.parts {
        println!(
            "  - {} path={} version={} stale={} sha256={}",
            part.part,
            part.resolved_path.as_deref().unwrap_or("missing"),
            part.version.as_deref().unwrap_or("unknown"),
            part.stale
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unknown".into()),
            part.sha256
                .as_deref()
                .map(|s| &s[..12.min(s.len())])
                .unwrap_or("unknown")
        );
        println!("    {}", part.stale_reason);
    }
    if envelope.stale_parts.is_empty() {
        println!("stale_parts: none");
    } else {
        println!("stale_parts: {}", envelope.stale_parts.join(", "));
    }
    for warning in &envelope.warnings {
        println!("warning: {warning}");
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_version;

    #[test]
    fn normalizes_common_version_outputs() {
        assert_eq!(normalize_version("focusa 0.9.74-dev"), "0.9.74-dev");
        assert_eq!(normalize_version("v0.9.80-dev"), "0.9.80-dev");
        assert_eq!(normalize_version("0.9.80-dev"), "0.9.80-dev");
    }
}
