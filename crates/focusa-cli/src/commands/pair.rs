//! Apple-like Mac Pairing Room entry command.

use crate::commands::daemon;
use clap::Args;
use qrcode::QrCode;
use qrcode::render::unicode;
use serde_json::json;

#[derive(Args)]
pub struct PairArgs {
    /// Public URL for this Focusa server/VPS, e.g. https://focusa.example.com.
    #[arg(long)]
    pub url: Option<String>,

    /// Do not print the terminal QR.
    #[arg(long)]
    pub no_qr: bool,
}

fn normalize_base(url: &str) -> String {
    url.trim().trim_end_matches('/').to_string()
}

fn server_url(explicit: Option<String>) -> (String, &'static str) {
    if let Some(url) = explicit.filter(|v| !v.trim().is_empty()) {
        return (normalize_base(&url), "--url");
    }
    if let Ok(url) = std::env::var("FOCUSA_PAIRING_URL") {
        if !url.trim().is_empty() {
            return (normalize_base(&url), "FOCUSA_PAIRING_URL");
        }
    }
    if let Ok(url) = std::env::var("FOCUSA_PUBLIC_URL") {
        if !url.trim().is_empty() {
            return (normalize_base(&url), "FOCUSA_PUBLIC_URL");
        }
    }
    ("http://127.0.0.1:8787".to_string(), "local_default")
}

fn terminal_qr(payload: &str) -> anyhow::Result<String> {
    let code = QrCode::new(payload.as_bytes())?;
    Ok(code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Dark)
        .light_color(unicode::Dense1x2::Light)
        .quiet_zone(true)
        .build())
}

pub async fn run(args: PairArgs, json_mode: bool) -> anyhow::Result<()> {
    let (server_url, source) = server_url(args.url);
    let connect_url = format!("{server_url}/connect");
    let daemon_started = daemon::start().await.unwrap_or(false);
    let warning = if source == "local_default" {
        Some("Set FOCUSA_PAIRING_URL or pass --url for a phone-reachable VPS URL.")
    } else {
        None
    };

    if json_mode {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": "ready",
                "command": "focusa pair",
                "server_url": server_url,
                "server_url_source": source,
                "connect_url": connect_url,
                "daemon": if daemon_started { "started" } else { "already_running_or_external" },
                "warning": warning,
                "next_steps": [
                    "Scan connect_url with your phone to open Focusa Connect.",
                    "In the phone PWA, scan the QR shown in the Mac menubar app.",
                    "Approve the Mac from the phone PWA."
                ],
            }))?
        );
        return Ok(());
    }

    println!("Focusa Pairing Room");
    println!();
    println!("Open on phone:");
    println!("{connect_url}");
    if let Some(warning) = warning {
        println!();
        println!("Warning: {warning}");
    }
    if !args.no_qr {
        println!();
        println!("{}", terminal_qr(&connect_url)?);
    }
    println!();
    println!("Then open the Mac menubar app and scan its code from this phone page.");
    Ok(())
}
