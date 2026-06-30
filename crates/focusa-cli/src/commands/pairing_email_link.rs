//! focusa pairing email-link subcommand (focusa-ui0y v0.9.39-dev G13).
//!
//! Operator-side helper for the "phone camera broken" / "operator on a
//! different device" fallback. Generates a one-time deep link and
//! delivers it to the operator's email (or prints it for mailto:).
//!
//! Delivery modes (auto-detected):
//!   1. --mailto: prints a mailto: URL to stdout (operator pastes into
//!      their email client; works on any Mac/Linux without SMTP setup).
//!   2. --sendmail-command: pipes a complete RFC822 message to an
//!      external TLS-capable relay command such as `sendmail -t` or
//!      `msmtp --read-envelope-from --read-recipients`.
//!   3. Neither: prints the deep link to stdout; operator copies it.
//!
//! Built-in raw SMTP is intentionally disabled. Pairing links now carry
//! room_claim_secret bootstrap material and must not be sent by an
//! internal plaintext SMTP sender.
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
    /// SMTP server hint (env SMTP_HOST). Built-in SMTP is disabled; if set
    /// without --sendmail-command we fail closed with guidance.
    #[arg(long, env = "SMTP_HOST")]
    pub smtp_host: Option<String>,
    /// SMTP port hint (env SMTP_PORT). Kept for compatibility/guidance only.
    #[arg(long, env = "SMTP_PORT", default_value = "587")]
    pub smtp_port: u16,
    /// SMTP user hint (env SMTP_USER). Kept for compatibility/guidance only.
    #[arg(long, env = "SMTP_USER")]
    pub smtp_user: Option<String>,
    /// SMTP password hint (env SMTP_PASS). Kept for compatibility/guidance only.
    #[arg(long, env = "SMTP_PASS")]
    pub smtp_pass: Option<String>,
    /// From address used with --sendmail-command. Default: SMTP_FROM, then
    /// SMTP_USER, then focusa@localhost.
    #[arg(long, env = "SMTP_FROM")]
    pub smtp_from: Option<String>,
    /// External mailer command, e.g. `sendmail -t` or
    /// `msmtp --read-envelope-from --read-recipients`.
    #[arg(long, env = "FOCUSA_PAIRING_SENDMAIL_COMMAND")]
    pub sendmail_command: Option<String>,
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
        "Open this link on your phone to pair it with your Focusa VPS:\r\n\r\n{}\r\n\r\n\
         This link expires in 5 minutes.\r\n\
         Room ID: {}\r\n\r\n\
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
        report
            .instructions
            .push(format!("Open in your email client: {mailto}"));
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

    if let Some(command) = args.sendmail_command.as_deref() {
        let to = args
            .to
            .as_deref()
            .context("--to is required with --sendmail-command")?;
        let smtp_from = args
            .smtp_from
            .as_deref()
            .or(args.smtp_user.as_deref())
            .unwrap_or("focusa@localhost");
        send_via_sendmail(command, smtp_from, to, &args.subject, &body)
            .await
            .with_context(|| format!("external mailer failed: {command}"))?;
        report.delivered_via = "sendmail".to_string();
        report
            .instructions
            .push(format!("Sent via external mailer to {to}"));
        if args.json {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            println!("Focusa pairing email sent");
            println!("  Room:    {room_id}");
            println!("  To:      {to}");
            println!("  Subject: {}", args.subject);
            println!("  URL:     {deliverable_url}");
            println!("  Mailer:  {command}");
        }
        return Ok(());
    }

    if args.smtp_host.is_some() || args.smtp_user.is_some() || args.smtp_pass.is_some() {
        anyhow::bail!(
            "Built-in SMTP delivery is disabled because pairing links now carry room_claim_secret bootstrap material. \
             Use --mailto, omit --to for stdout, or pass --sendmail-command 'sendmail -t' / \
             --sendmail-command 'msmtp --read-envelope-from --read-recipients'."
        );
    }

    if args.to.is_some() {
        anyhow::bail!(
            "--to without --mailto or --sendmail-command would require the disabled built-in SMTP path. \
             Use --mailto, stdout mode, or --sendmail-command."
        );
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("Focusa pairing link");
        println!("  Room:    {room_id}");
        println!("  URL:     {deliverable_url}");
    }
    Ok(())
}

async fn send_via_sendmail(
    command: &str,
    from: &str,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<()> {
    use std::process::Stdio;
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command;

    let message = format!(
        "From: {from}\r\n\
         To: {to}\r\n\
         Subject: {subject}\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         \r\n\
         {body}\r\n"
    );

    let mut child = Command::new("sh")
        .arg("-lc")
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("spawn external mailer: {command}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(message.as_bytes()).await?;
        stdin.shutdown().await.ok();
    }

    let output = child.wait_with_output().await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        anyhow::bail!(
            "external mailer exited with {}{}",
            output.status,
            if stderr.is_empty() {
                "".to_string()
            } else {
                format!(": {stderr}")
            }
        );
    }
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
