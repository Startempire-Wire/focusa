//! focusa device pair-* subcommands (Spec focusa-ui0y, Mac menubar OAuth).
//!
//! `focusa device pair-start [--device-name N] [--platform P] [--scopes ...]`
//! `focusa device pair-complete <code> [--host H] [--operator-id ID]`
//! `focusa device pair-status --code <c>|--device-id <id>`
//! `focusa device pair-list [--host H] [--limit N]`
//! `focusa device pair-revoke --device-id <id> [--host H] [--reason R]`

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::Value;

#[derive(Subcommand)]
#[command(rename_all = "kebab-case")]
#[allow(clippy::enum_variant_names)]
pub enum DeviceCmd {
    /// Start a new device pairing (generates 8-char code).
    #[command(name = "pair-start")]
    PairStart {
        #[arg(long)]
        device_name: Option<String>,
        #[arg(long, default_value = "macos")]
        platform: String,
        #[arg(long, default_value = "http://127.0.0.1:8787")]
        daemon_base_url: String,
        #[arg(long, value_delimiter = ',')]
        scopes: Vec<String>,
    },
    /// Shortcut for `pair-start` with QR handoff: prints pair_url prominently.
    /// focusa-ui0y.11 — Mode B (Telegram/Discord-style QR + phone).
    #[command(name = "pair-qr")]
    PairQr {
        #[arg(long)]
        device_name: Option<String>,
        #[arg(long, default_value = "macos")]
        platform: String,
        #[arg(long, default_value = "http://127.0.0.1:8787")]
        daemon_base_url: String,
        #[arg(long, value_delimiter = ',')]
        scopes: Vec<String>,
    },
    /// Complete a pending pairing (run on the VPS side; returns the token).
    #[command(name = "pair-complete")]
    PairComplete {
        code: String,
        #[arg(long, default_value = "operator-vps")]
        host: String,
        #[arg(long)]
        operator_id: Option<String>,
        #[arg(long, default_value = "vps-cli")]
        completed_by: String,
    },
    /// Check the status of a pending or completed pairing.
    #[command(name = "pair-status")]
    PairStatus {
        #[arg(long, conflicts_with = "device_id")]
        code: Option<String>,
        #[arg(long, conflicts_with = "code")]
        device_id: Option<String>,
    },
    /// List paired devices for a host.
    #[command(name = "pair-list")]
    PairList {
        #[arg(long, default_value = "operator-vps")]
        host: String,
        #[arg(long, default_value = "20")]
        limit: usize,
    },
    /// Revoke a paired device (appends revoked=true to the ledger).
    #[command(name = "pair-revoke")]
    PairRevoke {
        #[arg(long)]
        device_id: String,
        #[arg(long, default_value = "operator-vps")]
        host: String,
        #[arg(long)]
        reason: Option<String>,
    },
}

pub async fn handle(client: &mut ApiClient, cmd: DeviceCmd) -> anyhow::Result<()> {
    match cmd {
        DeviceCmd::PairStart {
            device_name,
            platform,
            daemon_base_url,
            scopes,
        } => {
            let body = serde_json::json!({
                "device_name": device_name.unwrap_or_else(|| "operator-device".to_string()),
                "platform": platform,
                "daemon_base_url": daemon_base_url,
                "scopes": if scopes.is_empty() { vec!["read".to_string(),"write".to_string()] } else { scopes },
            });
            let resp = client.post("/v1/device/pair/start", &body).await?;
            print_pair_start_human(&resp);
            Ok(())
        }
        DeviceCmd::PairQr {
            device_name,
            platform,
            daemon_base_url,
            scopes,
        } => {
            // QR/PWA handoff is licensed; CLI code fallback remains available so operators can pair without blocking on QR entitlement.
            if let Err(e) = focusa_core::license::require_feature("qr_pwa_handoff") {
                anyhow::bail!("{e}");
            }
            let body = serde_json::json!({
                "device_name": device_name.unwrap_or_else(|| "operator-device".to_string()),
                "platform": platform,
                "daemon_base_url": daemon_base_url,
                "scopes": if scopes.is_empty() { vec!["read".to_string(),"write".to_string()] } else { scopes },
            });
            let resp = client.post("/v1/device/pair/start", &body).await?;
            print_pair_qr_human(&resp);
            Ok(())
        }
        DeviceCmd::PairComplete {
            code,
            host,
            operator_id,
            completed_by,
        } => {
            let body = serde_json::json!({
                "code": code,
                "host": host,
                "operator_id": operator_id,
                "completed_by": completed_by,
            });
            let resp = client.post("/v1/device/pair/complete", &body).await?;
            print_pair_complete_human(&resp);
            Ok(())
        }
        DeviceCmd::PairStatus { code, device_id } => {
            if code.is_none() && device_id.is_none() {
                anyhow::bail!("--code or --device-id is required");
            }
            let mut path = String::from("/v1/device/pair/status?");
            if let Some(c) = code.as_deref() {
                path.push_str(&format!("code={}", urlencoding_minimal(c)));
            } else if let Some(d) = device_id.as_deref() {
                path.push_str(&format!("device_id={}", urlencoding_minimal(d)));
            }
            let resp = client.get(&path).await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
            Ok(())
        }
        DeviceCmd::PairList { host, limit } => {
            let path = format!(
                "/v1/device/pair/list?host={}&limit={}",
                urlencoding_minimal(&host),
                limit
            );
            let resp = client.get(&path).await?;
            print_pair_list_human(&resp);
            Ok(())
        }
        DeviceCmd::PairRevoke {
            device_id,
            host,
            reason,
        } => {
            let body = serde_json::json!({
                "device_id": device_id,
                "host": host,
                "reason": reason,
            });
            let resp = client.post("/v1/device/pair/revoke", &body).await?;
            print_pair_revoke_human(&resp);
            Ok(())
        }
    }
}

fn urlencoding_minimal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => out.push(c),
            _ => out.push_str(&format!("%{:02X}", c as u32)),
        }
    }
    out
}

fn print_pair_start_human(payload: &Value) {
    let code = payload.get("code").and_then(Value::as_str).unwrap_or("?");
    let device_id = payload
        .get("device_id")
        .and_then(Value::as_str)
        .unwrap_or("?");
    let scopes = payload.get("scopes").cloned().unwrap_or(Value::Null);
    let expires_in = payload
        .get("expires_in_secs")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let operator_handoff = payload
        .get("operator_handoff")
        .cloned()
        .unwrap_or(Value::Null);
    println!("device pair start | code={code} device_id={device_id} expires_in={expires_in}s");
    println!("fields: scopes={scopes} advisory=true");
    if let Some(on_vps) = operator_handoff
        .get("on_your_vps_run")
        .and_then(Value::as_str)
    {
        println!("on_your_vps_run: {on_vps}");
    }
}

/// focusa-ui0y.11: print QR-handoff output prominently.
fn print_pair_qr_human(payload: &Value) {
    let code = payload.get("code").and_then(Value::as_str).unwrap_or("?");
    let device_id = payload
        .get("device_id")
        .and_then(Value::as_str)
        .unwrap_or("?");
    let expires_in = payload
        .get("expires_in_secs")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let pair_url = payload
        .get("pair_url")
        .and_then(Value::as_str)
        .unwrap_or("");
    let pair_url_qr_payload = payload
        .get("pair_url_qr_payload")
        .and_then(Value::as_str)
        .unwrap_or("");
    let on_vps = payload
        .get("operator_handoff")
        .and_then(|h| h.get("on_your_vps_run"))
        .and_then(Value::as_str)
        .unwrap_or("");

    println!("device pair qr | code={code} device_id={device_id} expires_in={expires_in}s");
    println!();
    println!("  pair_url: {pair_url}");
    if pair_url_qr_payload != pair_url {
        println!("  pair_url_qr_payload: {pair_url_qr_payload}");
    }
    println!();
    println!("  Encode pair_url in a QR. Operator scans with phone, opens");
    println!("  the focusa-pairing PWA helper, and taps \"Complete on this VPS\".");
    println!();
    if !on_vps.is_empty() {
        println!("  Alternative (CLI):  {on_vps}");
    }
    println!();
    println!("See: docs/53-focusa-device-pairing-spec.md#3-handoff-modes");
}

fn print_pair_complete_human(payload: &Value) {
    let status = payload.get("status").and_then(Value::as_str).unwrap_or("?");
    let device_id = payload
        .get("device_id")
        .and_then(Value::as_str)
        .unwrap_or("?");
    let token = payload.get("token").and_then(Value::as_str).unwrap_or("?");
    let host = payload.get("host").and_then(Value::as_str).unwrap_or("?");
    let expires = payload
        .get("token_expires_at")
        .and_then(Value::as_str)
        .unwrap_or("?");
    println!("device pair complete {status} | device_id={device_id} host={host}");
    println!("fields: token={token} token_expires_at={expires}");
}

fn print_pair_list_human(payload: &Value) {
    let count = payload.get("count").and_then(Value::as_u64).unwrap_or(0);
    let host = payload.get("host").and_then(Value::as_str).unwrap_or("?");
    println!("device pair list | host={host} count={count}");
    if let Some(arr) = payload.get("devices").and_then(Value::as_array) {
        for d in arr.iter().take(20) {
            let id = d.get("device_id").and_then(Value::as_str).unwrap_or("?");
            let name = d.get("name").and_then(Value::as_str).unwrap_or("?");
            let revoked = d.get("revoked").and_then(Value::as_bool).unwrap_or(false);
            println!("  - {id} name={name} revoked={revoked}");
        }
    }
}

fn print_pair_revoke_human(payload: &Value) {
    let status = payload.get("status").and_then(Value::as_str).unwrap_or("?");
    let device_id = payload
        .get("device_id")
        .and_then(Value::as_str)
        .unwrap_or("?");
    let appended = payload
        .get("ledger_appended")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    println!("device pair revoke {status} | device_id={device_id} ledger_appended={appended}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urlencoding_minimal_encodes_query_values() {
        assert_eq!(urlencoding_minimal("hello"), "hello");
        assert_eq!(urlencoding_minimal("a/b"), "a%2Fb");
        assert_eq!(urlencoding_minimal("FOCUS-XYZ7-1234"), "FOCUS-XYZ7-1234");
    }

    #[test]
    fn print_human_does_not_panic() {
        print_pair_start_human(&serde_json::json!({
            "code": "FOCUS-TEST-1234", "device_id": "abc", "scopes": ["read"], "expires_in_secs": 300,
            "operator_handoff": {"on_your_vps_run": "focusa device pair-complete FOCUS-TEST-1234 --host vps"}
        }));
        print_pair_complete_human(&serde_json::json!({
            "status": "completed", "device_id": "abc", "host": "vps", "token": "tok", "token_expires_at": "2026"
        }));
        print_pair_list_human(&serde_json::json!({
            "host": "vps", "count": 1, "devices": [{"device_id":"abc","name":"n","revoked":false}]
        }));
        print_pair_revoke_human(&serde_json::json!({
            "status": "completed", "device_id": "abc", "ledger_appended": true
        }));
    }
}
