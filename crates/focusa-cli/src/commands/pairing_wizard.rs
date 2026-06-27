//! Focusa pairing wizard (focusa-ui0y v0.9.35-dev).
//!
//! Canonical operator entry point for self-host pairing. Replaces the bash
//! script at `crates/focusa-cli/scripts/focusa-pairing-wizard.sh`.
//!
//! Subcommands:
//!   - `focusa pairing wizard` — interactive flow (Tailscale detect, room
//!     create, terminal QR, poll until phone approves, print next steps)
//!   - `focusa pairing create-room` — non-interactive: returns room_id +
//!     pair_url as JSON (no QR rendering, no polling)
//!
//! No bash / python / qrencode dependencies at runtime. QR rendering uses
//! the `qrcode` crate (already a dependency of focusa-cli).
//!
//! Specs:
//!   - docs/55-focusa-self-host-architecture.md §6
//!   - docs/56-focusa-pairing-wizard-spec.md
//!   - docs/57-focusa-pairing-revoke-and-repair.md

use anyhow::{bail, Context, Result};
use clap::{Args, Subcommand};
use qrcode::render::unicode;
use qrcode::QrCode;
use serde::Serialize;
use std::io::{IsTerminal, Write};
use std::time::Duration;
use tracing::{debug, error, info, warn};

#[derive(Subcommand, Debug)]
pub enum WizardCmd {
    /// Interactive pairing wizard: create room, print QR, poll for approval.
    Wizard(WizardArgs),
    /// Non-interactive: create a room and print JSON with room_id + pair_url.
    CreateRoom(CreateRoomArgs),
}

#[derive(Args, Debug, Clone)]
pub struct WizardArgs {
    /// Skip Tailscale detection; use FOCUSA_PUBLIC_URL or daemon URL.
    #[arg(long)]
    no_tailnet: bool,
    /// Poll timeout in seconds (default 300).
    #[arg(long, default_value_t = 300)]
    timeout: u64,
    /// Self-test: auto-approve via local daemon (for the revoke-repair test cycle).
    #[arg(long)]
    demo: bool,
    /// JSON output instead of human-readable terminal UI.
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct CreateRoomArgs {
    /// Optional operator-supplied public VPS URL hint.
    #[arg(long)]
    server_url: Option<String>,
    /// Output as JSON only (default).
    #[arg(long, default_value_t = true)]
    json: bool,
}

#[derive(Debug, Serialize)]
pub struct CreatedRoom {
    pub status: String,
    pub room_id: String,
    pub device_id: String,
    pub server_url: String,
    pub pair_url: String,
    pub expires_in_secs: i64,
    pub join_url: String,
    pub approve_url: String,
    pub poll_url: String,
}

pub async fn run(cmd: WizardCmd) -> Result<()> {
    match cmd {
        WizardCmd::Wizard(args) => run_wizard(args).await,
        WizardCmd::CreateRoom(args) => run_create_room(args).await,
    }
}

fn daemon_url() -> String {
    std::env::var("FOCUSA_DAEMON_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:8787".to_string())
}

fn detect_tailscale_hostname() -> Option<(String, String)> {
    let out = match std::process::Command::new("tailscale")
        .args(["status", "--json"])
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            debug!(error = %e, "tailscale CLI not found; skipping MagicDNS detection");
            return None;
        }
    };
    if !out.status.success() {
        warn!(
            status = ?out.status.code(),
            stderr = %String::from_utf8_lossy(&out.stderr).trim(),
            "tailscale status exited non-zero; skipping MagicDNS detection"
        );
        return None;
    }
    let v: serde_json::Value = match serde_json::from_slice(&out.stdout) {
        Ok(v) => v,
        Err(e) => {
            warn!(error = %e, "tailscale status JSON parse failed; skipping MagicDNS detection");
            return None;
        }
    };
    let name = v.get("Self")?.get("DNSName")?.as_str()?.trim_end_matches('.').to_string();
    let ip = v
        .get("TailscaleIPs")?
        .as_array()?
        .first()?
        .as_str()?
        .to_string();
    if name.is_empty() {
        warn!("tailscale status returned empty DNSName; skipping MagicDNS detection");
        return None;
    }
    Some((name, ip))
}

fn resolve_public_url(no_tailnet: bool) -> (String, String) {
    // (url, source) — source for diagnostics
    // Env vars beat auto-discovery; explicit operator intent > heuristic.
    if let Ok(u) = std::env::var("FOCUSA_PUBLIC_URL") {
        if !u.trim().is_empty() {
            return (u, "FOCUSA_PUBLIC_URL env".to_string());
        }
    }
    if let Ok(u) = std::env::var("FOCUSA_PAIRING_URL") {
        if !u.trim().is_empty() {
            return (u, "FOCUSA_PAIRING_URL env".to_string());
        }
    }
    if !no_tailnet {
        if let Some((name, ip)) = detect_tailscale_hostname() {
            return (format!("https://{name}"), format!("tailscale MagicDNS {name} → {ip}"));
        }
    }
    (daemon_url(), "daemon URL fallback".to_string())
}

async fn daemon_health(url: &str) -> Result<serde_json::Value> {
    let resp = reqwest::Client::new()
        .get(format!("{url}/v1/health"))
        .timeout(Duration::from_secs(3))
        .send()
        .await
        .with_context(|| format!("GET {url}/v1/health"))?;
    if !resp.status().is_success() {
        error!(daemon_url = %url, http_status = %resp.status(), "daemon health check failed");
        bail!("daemon health returned HTTP {}", resp.status());
    }
    Ok(resp.json().await?)
}

async fn create_room(server_url: &str) -> Result<CreatedRoom> {
    let url = format!("{}/v1/connect/room/create", server_url.trim_end_matches('/'));
    let resp = reqwest::Client::new()
        .post(&url)
        .json(&serde_json::json!({}))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .with_context(|| format!("POST {url}"))?;
    if !resp.status().is_success() {
        error!(server_url = %server_url, http_status = %resp.status(), "create-room request failed");
        bail!("create-room returned HTTP {}", resp.status());
    }
    let v: serde_json::Value = resp.json().await?;
    Ok(CreatedRoom {
        status: v.get("status").and_then(|x| x.as_str()).unwrap_or("?").to_string(),
        room_id: v.get("room_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        device_id: v.get("device_id").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        server_url: v.get("server_url").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        pair_url: v.get("pair_url").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        expires_in_secs: v.get("expires_in_secs").and_then(|x| x.as_i64()).unwrap_or(0),
        join_url: v.get("join_url").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        approve_url: v.get("approve_url").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        poll_url: v.get("poll_url").and_then(|x| x.as_str()).unwrap_or("").to_string(),
    })
}

async fn poll_room(poll_url: &str, timeout_secs: u64) -> Result<(String, Option<String>)> {
    let start = std::time::Instant::now();
    let client = reqwest::Client::new();
    while start.elapsed().as_secs() < timeout_secs {
        let resp = client.get(poll_url).timeout(Duration::from_secs(3)).send().await;
        if let Ok(r) = resp {
            if let Ok(v) = r.json::<serde_json::Value>().await {
                let status = v.get("status").and_then(|x| x.as_str()).unwrap_or("?").to_string();
                let token = v.get("token").and_then(|x| x.as_str()).map(str::to_string);
                if status == "completed" {
                    return Ok((status, token));
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    warn!(timeout_secs, "wizard timed out waiting for phone approval");
        bail!("timeout waiting for phone approval ({}s)", timeout_secs)
    }

fn render_terminal_qr(data: &str) -> Result<String> {
    let code = QrCode::new(data.as_bytes()).context("QR encode failed")?;
    let image = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    Ok(image)
}

async fn run_create_room(args: CreateRoomArgs) -> Result<()> {
    let (public_url, _source) = resolve_public_url(false);
    let room = create_room(&public_url).await?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&room)?);
    } else {
        println!("Room:        {}", room.room_id);
        println!("Server URL:  {}", room.server_url);
        println!("Pair URL:    {}", room.pair_url);
        println!("Expires in:  {}s", room.expires_in_secs);
    }
    Ok(())
}

async fn run_wizard(args: WizardArgs) -> Result<()> {
    if args.json {
        // JSON mode: just create the room and print
        let (public_url, source) = resolve_public_url(args.no_tailnet);
        let room = create_room(&public_url).await?;
        let mut obj = serde_json::to_value(&room)?;
        obj.as_object_mut()
            .unwrap()
            .insert("public_url_source".to_string(), serde_json::json!(source));
        println!("{}", serde_json::to_string_pretty(&obj)?);
        return Ok(());
    }

    // Human-friendly banner
    println!();
    println!("  ╔══════════════════════════════════════════════════════════╗");
    println!("  ║          Focusa Pairing Wizard                           ║");
    println!("  ║          focusa-pairing-wizard v0.9.35-dev               ║");
    println!("  ╚══════════════════════════════════════════════════════════╝");
    println!();

    let base_url = daemon_url();
    println!("▶  Welcome to Focusa pairing.");
    match daemon_health(&base_url).await {
        Ok(h) => println!(
            "✓  Focusa daemon detected (v{}) at {}",
            h.get("version").and_then(|x| x.as_str()).unwrap_or("?"),
            base_url
        ),
        Err(e) => {
            error!(daemon_url = %base_url, error = %e, "cannot reach Focusa daemon");
            eprintln!("✗  Cannot reach Focusa daemon at {base_url}: {e}");
            eprintln!("   recovery_hint: systemctl --user restart focusa-daemon");
            std::process::exit(2);
        }
    }

    println!();
    println!("▶  Resolving phone-reachable URL…");
    let (public_url, source) = resolve_public_url(args.no_tailnet);
    println!("✓  {source}");
    println!("   Pairing URL: {public_url}");
    println!();

    if std::io::stdin().is_terminal() {
        print!("▶  Pair your Mac now? [Y/n]: ");
        std::io::stdout().flush().ok();
        let mut reply = String::new();
        std::io::stdin().read_line(&mut reply).ok();
        let reply = reply.trim().to_ascii_lowercase();
        if !matches!(reply.as_str(), "" | "y" | "yes") {
            println!("  Skipped. Run 'focusa pairing wizard' any time.");
            return Ok(());
        }
    } else if args.demo {
        println!("▶  [FOCUSA_WIZARD_DEMO=1] non-interactive; auto-approving.");
    } else {
        println!("▶  Non-interactive (no TTY); proceeding.");
    }

    println!();
    println!("▶  Creating pairing room…");
    let room = create_room(&public_url).await?;
    println!(
        "✓  Room {}…  expires in {}s",
        &room.room_id.chars().take(8).collect::<String>(),
        room.expires_in_secs
    );

    println!();
    println!("  Scan this QR with your iPhone or Android camera:");
    println!();
    match render_terminal_qr(&room.pair_url) {
        Ok(qr) => {
            for line in qr.lines() {
                println!("  {line}");
            }
        }
        Err(e) => {
            warn!(error = %e, pair_url = %room.pair_url, "QR render failed; URL still printed below for manual paste");
            eprintln!("✗  QR render failed: {e}");
        }
    }
    println!();
    println!("  URL: {}", room.pair_url);
    println!();

    if args.demo {
        // Self-test: simulate phone-side actions
        println!("  [demo mode] simulating phone-side approval");
        let _ = reqwest::Client::new()
            .post(&room.join_url)
            .json(&serde_json::json!({
                "mac_name": "operator-mac",
                "mac_nonce": "demo",
            }))
            .send()
            .await;
        let _ = reqwest::Client::new()
            .post(&room.approve_url)
            .json(&serde_json::json!({
                "host": "127.0.0.1",
                "operator_id": "phone",
                "completed_by": "demo",
            }))
            .send()
            .await;
    }

    println!();
    println!("▶  Waiting for Mac to join + phone to approve (timeout {}s)…", args.timeout);
    let poll_result = poll_room(&room.poll_url, args.timeout).await;
    match poll_result {
        Ok((status, token)) => {
            let token_chars = token.as_deref().unwrap_or("").len();
            println!();
            println!("✓  Pairing complete (status={status}, token {token_chars} chars).");
            info!(
                room_id = %room.room_id,
                status = %status,
                token_chars = token_chars,
                "wizard pairing completed"
            );
            println!();
            println!("  Next:");
            println!("    1. On your Mac: open /Applications/Focusa.app");
            println!("       (the wizard will detect this VPS and connect automatically)");
            println!("    2. Verify:      focusa doctor");
            Ok(())
        }
        Err(e) => {
            error!(error = %e, "wizard polling failed");
            eprintln!("✗  {e}");
            eprintln!("   recovery_hint: re-run 'focusa pairing wizard' to create a fresh room.");
            std::process::exit(1);
        }
    }
}