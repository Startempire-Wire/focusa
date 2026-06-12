//! Apple-like Mac Phone Bridge Flow entry command.

use crate::api_client::ApiClient;
use crate::commands::daemon;
use clap::Args;
use qrcode::QrCode;
use qrcode::render::unicode;
use serde_json::{Value, json};
use std::net::{IpAddr, Ipv4Addr};

#[derive(Args)]
pub struct PairArgs {
    /// Public URL for this Focusa server/VPS, e.g. https://focusa.example.com.
    #[arg(long)]
    pub url: Option<String>,

    /// Do not print the terminal QR.
    #[arg(long)]
    pub no_qr: bool,
}

#[derive(Debug, Clone)]
struct UrlChoice {
    url: String,
    source: &'static str,
    warning: Option<String>,
    checked_candidates: Vec<Value>,
}

fn normalize_base(url: &str) -> String {
    url.trim().trim_end_matches('/').to_string()
}

fn env_url(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|url| normalize_base(&url))
        .filter(|url| !url.is_empty())
}

fn file_url(path: &str) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|url| normalize_base(&url))
        .filter(|url| !url.is_empty() && !url.starts_with('#'))
}

fn is_local_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.contains("://127.")
        || lower.contains("://localhost")
        || lower.contains("://0.0.0.0")
        || lower.contains("://[::1]")
}

fn command_output(command: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(command)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn detected_hostname() -> Option<String> {
    let hostname =
        command_output("hostname", &["-f"]).or_else(|| command_output("hostname", &[]))?;
    let host = hostname.trim().trim_end_matches('.');
    if host.is_empty()
        || host.eq_ignore_ascii_case("localhost")
        || host.ends_with(".local")
        || host.ends_with(".localdomain")
    {
        return None;
    }
    Some(host.to_string())
}

fn public_ipv4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    let carrier_grade_nat = octets[0] == 100 && (64..=127).contains(&octets[1]);
    !(ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_multicast()
        || ip.is_broadcast()
        || ip.is_unspecified()
        || carrier_grade_nat)
}

fn private_or_tailscale_ipv4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    let carrier_grade_nat = octets[0] == 100 && (64..=127).contains(&octets[1]);
    !ip.is_loopback()
        && !ip.is_link_local()
        && !ip.is_multicast()
        && !ip.is_broadcast()
        && !ip.is_unspecified()
        && (ip.is_private() || carrier_grade_nat)
}

fn detected_ips(filter: fn(Ipv4Addr) -> bool) -> Vec<String> {
    command_output("hostname", &["-I"])
        .unwrap_or_default()
        .split_whitespace()
        .filter_map(|value| match value.parse::<IpAddr>().ok()? {
            IpAddr::V4(v4) if filter(v4) => Some(v4.to_string()),
            _ => None,
        })
        .collect()
}

async fn connect_probe(url: &str) -> bool {
    let connect_url = format!("{}/connect", normalize_base(url));
    let Ok(client) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .danger_accept_invalid_certs(true)
        .build()
    else {
        return false;
    };
    let Ok(resp) = client.get(connect_url).send().await else {
        return false;
    };
    if !resp.status().is_success() {
        return false;
    }
    resp.text()
        .await
        .map(|body| body.contains("Focusa Connect") && body.contains("Connect Mac to Focusa"))
        .unwrap_or(false)
}

async fn bridge_api_probe(url: &str) -> bool {
    let probe_url = format!(
        "{}/v1/connect/room/focusa-probe/status",
        normalize_base(url)
    );
    let Ok(client) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .danger_accept_invalid_certs(true)
        .build()
    else {
        return false;
    };
    let Ok(resp) = client.get(probe_url).send().await else {
        return false;
    };
    resp.text()
        .await
        .map(|body| {
            body.contains("connect_room_not_found") || body.contains("\"status\":\"not_found\"")
        })
        .unwrap_or(false)
}

fn push_candidate(candidates: &mut Vec<(String, &'static str)>, url: String, source: &'static str) {
    let normalized = normalize_base(&url);
    if !normalized.is_empty() && !candidates.iter().any(|(seen, _)| seen == &normalized) {
        candidates.push((normalized, source));
    }
}

async fn resolve_server_url(explicit: Option<String>) -> UrlChoice {
    if let Some(url) = explicit.filter(|v| !v.trim().is_empty()) {
        return UrlChoice {
            url: normalize_base(&url),
            source: "--url",
            warning: None,
            checked_candidates: vec![],
        };
    }

    let mut candidates = vec![];

    for (key, source) in [
        ("FOCUSA_PAIRING_URL", "FOCUSA_PAIRING_URL"),
        ("FOCUSA_PUBLIC_URL", "FOCUSA_PUBLIC_URL"),
    ] {
        if let Some(url) = env_url(key) {
            push_candidate(&mut candidates, url, source);
        }
    }

    for path in [
        "/etc/focusa/pairing-url",
        "/etc/focusa/public-url",
        ".focusa-pairing-url",
        ".focusa-public-url",
    ] {
        if let Some(url) = file_url(path) {
            push_candidate(&mut candidates, url, "install_config");
        }
    }

    for key in ["FOCUSA_API_URL", "FOCUSA_BASE_URL"] {
        if let Some(url) = env_url(key)
            && !is_local_url(&url)
        {
            push_candidate(&mut candidates, url, key);
        }
    }

    if let Some(host) = detected_hostname() {
        push_candidate(&mut candidates, format!("https://{host}"), "hostname_https");
        push_candidate(&mut candidates, format!("http://{host}"), "hostname_http");
        push_candidate(
            &mut candidates,
            format!("http://{host}:8787"),
            "hostname_daemon_port",
        );
    }
    for ip in detected_ips(public_ipv4) {
        push_candidate(&mut candidates, format!("https://{ip}"), "public_ip_https");
        push_candidate(&mut candidates, format!("http://{ip}"), "public_ip_http");
        push_candidate(
            &mut candidates,
            format!("http://{ip}:8787"),
            "public_ip_daemon_port",
        );
    }
    for ip in detected_ips(private_or_tailscale_ipv4) {
        push_candidate(
            &mut candidates,
            format!("http://{ip}:8787"),
            "private_or_tailscale_daemon_port",
        );
    }
    push_candidate(
        &mut candidates,
        "http://127.0.0.1:8787".to_string(),
        "local_default",
    );

    let mut checked = Vec::new();
    for (url, source) in candidates {
        let connect_ok = connect_probe(&url).await;
        let bridge_api_ok = if connect_ok {
            bridge_api_probe(&url).await
        } else {
            false
        };
        checked.push(json!({
            "url": url,
            "source": source,
            "connect_route_reachable": connect_ok,
            "bridge_api_reachable": bridge_api_ok,
        }));
        if connect_ok && bridge_api_ok {
            let warning = (source == "local_default").then(|| {
                "Auto-detected local daemon URL. This works for same-machine/dev use; phones on another device need a shared network, tunnel, or public URL.".to_string()
            });
            return UrlChoice {
                url,
                source,
                warning,
                checked_candidates: checked,
            };
        }
    }

    UrlChoice {
        url: "http://127.0.0.1:8787".to_string(),
        source: "local_default_unverified",
        warning: Some(
            "Focusa could not auto-detect a phone-reachable transport. Ensure the daemon is running and that the phone shares a reachable network, tunnel, or public URL."
                .to_string(),
        ),
        checked_candidates: checked,
    }
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

async fn create_room(server_url: &str) -> anyhow::Result<Value> {
    ApiClient::new()
        .post(
            "/v1/connect/room/start",
            &json!({
                "server_url": server_url,
                "scopes": ["read", "write"],
            }),
        )
        .await
}

async fn start_room(server_url: &str) -> (Value, Option<String>) {
    match create_room(server_url).await {
        Ok(payload) => (payload, None),
        Err(first_err) => {
            // Dead-simple repair path: if the daemon is stale or was just rebuilt,
            // try the idempotent starter once, then retry room creation.
            let _ = daemon::start().await;
            if let Ok(payload) = create_room(server_url).await {
                return (
                    payload,
                    Some("Updated Focusa daemon detected; Phone Bridge Flow is ready.".to_string()),
                );
            }

            let connect_url = format!("{server_url}/connect");
            (
                json!({
                    "status": "fallback_static_connect",
                    "server_url": server_url,
                    "connect_url": connect_url,
                    "room_id": null,
                    "failure_class": "connect_room_start_unavailable",
                    "details": first_err.to_string(),
                }),
                Some("Focusa Connect needs the current daemon. Run `focusa stop && focusa start`, then `focusa pair`.".to_string()),
            )
        }
    }
}

pub async fn run(args: PairArgs, json_mode: bool) -> anyhow::Result<()> {
    let daemon_started = daemon::start().await.unwrap_or(false);
    let choice = resolve_server_url(args.url).await;
    let server_url = choice.url.clone();
    let source = choice.source;
    let (room_payload, room_warning) = start_room(&server_url).await;
    let connect_url = room_payload
        .get("connect_url")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("{server_url}/connect"));
    let room_id = room_payload.get("room_id").and_then(Value::as_str);
    let warning = room_warning.or(choice.warning);

    if json_mode {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "status": room_payload.get("status").and_then(Value::as_str).unwrap_or("ready"),
                "command": "focusa pair",
                "server_url": server_url,
                "server_url_source": source,
                "checked_candidates": choice.checked_candidates,
                "room_id": room_id,
                "connect_url": connect_url,
                "daemon": if daemon_started { "started" } else { "already_running_or_external" },
                "warning": warning,
                "room": room_payload,
                "next_steps": [
                    "Scan connect_url with your phone to open Focusa Connect.",
                    "In the Focusa Connect Page, scan the QR shown in the Mac Menubar App.",
                    "Approve the Mac from the Focusa Connect Page."
                ],
            }))?
        );
        return Ok(());
    }

    println!("Focusa Phone Bridge Flow");
    if let Some(room_id) = room_id {
        println!("Room: {room_id}");
    }
    println!();
    println!("Open on phone:");
    println!("{connect_url}");
    println!("Detected from: {source}");
    if let Some(warning) = warning.as_deref() {
        println!();
        println!("Setup needed: {warning}");
    }
    if !args.no_qr {
        println!();
        println!("{}", terminal_qr(&connect_url)?);
    }
    println!();
    println!("Then open the Mac Menubar App and scan its code from the Focusa Connect Page.");
    Ok(())
}
