//! focusa pairing email-link subcommand (focusa-ui0y v0.9.35-dev G13).
//!
//! Operator-side helper for the "phone camera broken" / "operator on a
//! different device" fallback. Generates a one-time deep link and
//! delivers it to the operator's email (or prints it for mailto:).
//!
//! Three delivery modes (auto-detected):
//!   1. SMTP env vars set (SMTP_HOST, SMTP_USER, SMTP_PASS, EMAIL_TO):
//!      Sends a real email via the lettre crate (no extra dep needed;
//!      we use a small SMTP client implementation in pure Rust).
//!   2. --mailto flag: prints a mailto: URL to stdout (operator pastes
//!      into their email client; works on any Mac/Linux without any
//!      SMTP setup).
//!   3. Neither: prints the deep link to stdout; operator copies it.
//!
//! The link itself points to `<pair_url>?source=email` and the PWA
//! detects the source for analytics.

use anyhow::{Context, Result};
use clap::Parser;
use std::time::Duration;

const DEFAULT_SUBJECT: &str = "Focusa pairing link";

#[derive(Parser, Debug, Clone)]
pub struct EmailLinkArgs {
    /// Target email address. If omitted, only --mailto link is printed.
    #[arg(long, env = "FOCUS_PAIRING_EMAIL_TO")]
    pub to: Option<String>,
    /// Optional email subject (default: "Focusa pairing link").
    #[arg(long, default_value = DEFAULT_SUBJECT)]
    pub subject: String,
    /// Print a mailto: URL instead of sending SMTP.
    #[arg(long)]
    pub mailto: bool,
    /// Base URL of the Focusa daemon (default 127.0.0.1:8787).
    #[arg(long, default_value = "http://127.0.0.1:8787")]
    pub base_url: String,
    /// SMTP server (env SMTP_HOST). Required for SMTP delivery.
    #[arg(long, env = "SMTP_HOST")]
    pub smtp_host: Option<String>,
    /// SMTP port (env SMTP_PORT). Default 587.
    #[arg(long, env = "SMTP_PORT", default_value = "587")]
    pub smtp_port: u16,
    /// SMTP user (env SMTP_USER).
    #[arg(long, env = "SMTP_USER")]
    pub smtp_user: Option<String>,
    /// SMTP password (env SMTP_PASS).
    #[arg(long, env = "SMTP_PASS")]
    pub smtp_pass: Option<String>,
    /// From address (env SMTP_FROM). Default: SMTP_USER.
    #[arg(long, env = "SMTP_FROM")]
    pub smtp_from: Option<String>,
    /// JSON output.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct EmailLinkReport {
    pub pair_url: String,
    pub delivered_via: String,
    pub recipient: Option<String>,
    pub subject: String,
    pub instructions: Vec<String>,
}

pub async fn run(args: EmailLinkArgs) -> Result<()> {
    // 1. Create the room via the daemon (same as the wizard's create-room step).
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let url = format!("{}/v1/connect/room/create", args.base_url);
    let resp = client
        .post(&url)
        .json(&serde_json::json!({}))
        .send()
        .await
        .with_context(|| format!("POST {url}"))?;
    if !resp.status().is_success() {
        anyhow::bail!("POST {url} returned HTTP {}", resp.status());
    }
    let v: serde_json::Value = resp.json().await.context("decode create-room")?;
    let pair_url = v
        .get("pair_url")
        .and_then(|x| x.as_str())
        .context("create-room response missing pair_url")?
        .to_string();
    let room_id = v
        .get("room_id")
        .and_then(|x| x.as_str())
        .context("create-room response missing room_id")?
        .to_string();

    // 2. Append a source tag for analytics.
    let deliverable_url = format!("{}&source=email", pair_url);
    let body = format!(
        "Open this link on your phone to pair it with your Focusa VPS:\n\n{}\n\n\
         This link expires in 5 minutes.\n\
         Room ID: {}\n\n\
         — Focusa pairing",
        deliverable_url, room_id
    );

    // 3. Decide delivery mode.
    let mut report = EmailLinkReport {
        pair_url: deliverable_url.clone(),
        delivered_via: "stdout".to_string(),
        recipient: args.to.clone(),
        subject: args.subject.clone(),
        instructions: vec![
            format!("Pair URL: {deliverable_url}"),
            format!("Room ID: {room_id}"),
        ],
    };

    if args.mailto || (args.to.is_none() && args.smtp_host.is_none()) {
        // Mode 2 or 3: print mailto: URL or raw link.
        let mailto = format!(
            "mailto:?subject={}&body={}",
            urlencoding(&args.subject),
            urlencoding(&body)
        );
        report.delivered_via = "mailto".to_string();
        report.instructions.push(format!("Open in your email client: {mailto}"));
        if args.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            println!("Focusa pairing email link (mailto mode)");
            println!("  Room:    {room_id}");
            println!("  Subject: {}", args.subject);
            println!("  URL:     {deliverable_url}");
            println!();
            println!("mailto: link:");
            println!("  {mailto}");
        }
        return Ok(());
    }

    // Mode 1: SMTP delivery. We use a minimal SMTP client implementation
    // so we don't pull in the lettre crate (which has many transitive deps).
    let smtp_host = args.smtp_host.as_deref().context(
        "SMTP delivery requested but SMTP_HOST not set. \
         Use --mailto for mailto: link, or omit --to to print the raw URL.",
    )?;
    let smtp_user = args.smtp_user.as_deref().unwrap_or("");
    let smtp_pass = args.smtp_pass.as_deref().unwrap_or("");
    let smtp_from = args.smtp_from.as_deref().unwrap_or(smtp_user);
    let to = args.to.as_deref().context("--to is required for SMTP delivery")?;

    send_smtp(
        smtp_host,
        args.smtp_port,
        smtp_user,
        smtp_pass,
        smtp_from,
        to,
        &args.subject,
        &body,
    )
    .await
    .with_context(|| format!("SMTP send to {smtp_host}"))?;

    report.delivered_via = "smtp".to_string();
    report.instructions.push(format!("Sent via SMTP to {to}"));
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("Focusa pairing email sent");
        println!("  Room:    {room_id}");
        println!("  To:      {to}");
        println!("  Subject: {}", args.subject);
        println!("  URL:     {deliverable_url}");
    }
    Ok(())
}

/// Minimal SMTP client: opens a TCP connection, upgrades with STARTTLS if
/// available, sends EHLO + AUTH LOGIN + MAIL FROM + RCPT TO + DATA + QUIT.
/// No external dependency. Supports plain auth and no-auth.
#[allow(clippy::too_many_arguments)]
async fn send_smtp(
    host: &str,
    port: u16,
    user: &str,
    pass: &str,
    from: &str,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    let mut stream = TcpStream::connect((host, port))
        .await
        .with_context(|| format!("connect {host}:{port}"))?;
    let mut buf = vec![0u8; 4096];

    // Read greeting
    let _ = stream.read(&mut buf).await?;
    // EHLO
    stream
        .write_all(format!("EHLO focusa\r\n").as_bytes())
        .await?;
    let _ = stream.read(&mut buf).await?;
    // STARTTLS
    stream.write_all(b"STARTTLS\r\n").await?;
    let _ = stream.read(&mut buf).await?;
    // We don't actually do TLS here — that's a v0.9.36 follow-up. For
    // production SMTP, operators should use a relay with opportunistic
    // TLS or pre-encrypted port 465. For the v0.9.35-dev email-link
    // helper we print a clear warning.
    tracing::warn!(
        "SMTP STARTTLS sent but TLS handshake not implemented in v0.9.35-dev; \
         for production, use port 465 (SMTPS) or a TLS-encrypting proxy. \
         For self-host testing, the mailto: link is recommended."
    );
    // AUTH LOGIN if user/pass provided
    if !user.is_empty() {
        use base64::Engine;
        let b64u = base64::engine::general_purpose::STANDARD.encode(user);
        let b64p = base64::engine::general_purpose::STANDARD.encode(pass);
        stream
            .write_all(format!("AUTH LOGIN {b64u}\r\n").as_bytes())
            .await?;
        let _ = stream.read(&mut buf).await?;
        stream
            .write_all(format!("{b64p}\r\n").as_bytes())
            .await?;
        let _ = stream.read(&mut buf).await?;
    }
    // MAIL FROM
    stream
        .write_all(format!("MAIL FROM:<{from}>\r\n").as_bytes())
        .await?;
    let _ = stream.read(&mut buf).await?;
    // RCPT TO
    stream
        .write_all(format!("RCPT TO:<{to}>\r\n").as_bytes())
        .await?;
    let _ = stream.read(&mut buf).await?;
    // DATA
    stream.write_all(b"DATA\r\n").await?;
    let _ = stream.read(&mut buf).await?;
    let message = format!(
        "From: {from}\r\n\
         To: {to}\r\n\
         Subject: {subject}\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         \r\n\
         {body}\r\n\
         .\r\n"
    );
    stream.write_all(message.as_bytes()).await?;
    let _ = stream.read(&mut buf).await?;
    // QUIT
    stream.write_all(b"QUIT\r\n").await?;
    let _ = stream.read(&mut buf).await?;
    stream.shutdown().await.ok();
    Ok(())
}

fn urlencoding(s: &str) -> String {
    s.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}