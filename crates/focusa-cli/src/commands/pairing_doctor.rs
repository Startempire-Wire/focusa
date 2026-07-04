//! focusa pairing doctor — single-command root-cause report (focusa-gkrj).

use anyhow::{Context, Result};
use clap::Parser;
use serde::Serialize;
use serde_json::Value;

#[derive(Parser, Debug)]
pub struct DoctorArgs {
    /// Print machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Serialize)]
pub struct DoctorReport {
    pub version: VersionInfo,
    pub daemon: Option<DaemonInfo>,
    pub transport: TransportInfo,
    pub codesign: CodesignInfo,
    pub service_install: ServiceInfo,
    pub next_action: String,
}

#[derive(Debug, Serialize)]
pub struct VersionInfo {
    pub cli: String,
    pub daemon: Option<String>,
    pub matched: bool,
}

#[derive(Debug, Serialize)]
pub struct DaemonInfo {
    pub reachable: bool,
    pub version: Option<String>,
    pub uptime_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct TransportInfo {
    pub public_url: Option<String>,
    pub connect_url: Option<String>,
    pub candidates: Value,
}

#[derive(Debug, Serialize)]
pub struct CodesignInfo {
    pub platform: String,
    pub host_supported: bool,
    pub codesign_present: bool,
    pub notarized: bool,
}

#[derive(Debug, Serialize)]
pub struct ServiceInfo {
    pub systemd_unit_written: bool,
    pub launchd_plist_written: bool,
    pub enabled: bool,
}

fn which(name: &str) -> bool {
    let path = std::env::var_os("PATH");
    if let Some(p) = path {
        for entry in std::env::split_paths(&p) {
            if entry.join(name).is_file() {
                return true;
            }
        }
    }
    false
}

pub async fn run(args: DoctorArgs) -> Result<()> {
    let cli_version = env!("CARGO_PKG_VERSION").to_string();

    let client = crate::api_client::ApiClient::new();
    let daemon_reachable = client.get("/v1/health").await.is_ok();
    let mut daemon_version: Option<String> = None;
    let mut uptime_ms: Option<u64> = None;
    if daemon_reachable {
        if let Ok(h) = client.get("/v1/health").await {
            daemon_version = h.get("version").and_then(|v| v.as_str()).map(String::from);
            uptime_ms = h.get("uptime_ms").and_then(|v| v.as_u64());
        }
    }
    let matched = daemon_version
        .as_deref()
        .map(|v| v == cli_version)
        .unwrap_or(false);

    let public_url = std::env::var("FOCUSA_PAIRING_URL").ok().or_else(|| {
        std::fs::read_to_string("/etc/focusa/public-url")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    });

    // Probe a connect_room_start call if public_url is available.
    let connect_url = if let Some(base) = &public_url {
        let body = serde_json::json!({
            "server_url": base,
            "scopes": ["read", "write"],
        });
        client
            .post("/v1/connect/room/start", &body)
            .await
            .ok()
            .and_then(|v| {
                v.get("connect_url")
                    .and_then(|x| x.as_str())
                    .map(String::from)
            })
    } else {
        None
    };

    let candidates = serde_json::json!({
        "transport_setup": "focusa pairing transport setup",
        "operator_url": public_url.clone(),
        "note": "run `focusa pairing transport setup` to see full candidate list",
    });

    let report = DoctorReport {
        version: VersionInfo {
            cli: cli_version.clone(),
            daemon: daemon_version.clone(),
            matched,
        },
        daemon: Some(DaemonInfo {
            reachable: daemon_reachable,
            version: daemon_version.clone(),
            uptime_ms,
        }),
        transport: TransportInfo {
            public_url: public_url.clone(),
            connect_url: connect_url.clone(),
            candidates,
        },
        codesign: CodesignInfo {
            platform: std::env::consts::OS.to_string(),
            host_supported: cfg!(target_os = "macos"),
            codesign_present: which("codesign"),
            notarized: false,
        },
        service_install: ServiceInfo {
            systemd_unit_written: std::path::Path::new(
                "$HOME/.config/systemd/user/focusa-daemon.service",
            )
            .exists(),
            launchd_plist_written: std::path::Path::new(
                "/Users/wirebot/Library/LaunchAgents/com.startempire.focusa-daemon.plist",
            )
            .exists(),
            enabled: which("systemctl") || which("launchctl"),
        },
        next_action: next_action_recommendation(
            daemon_reachable,
            matched,
            public_url.is_some(),
            connect_url.is_some(),
        ),
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("focusa pairing doctor");
        println!("  cli:           {}", report.version.cli);
        if let Some(d) = &report.version.daemon {
            println!(
                "  daemon:        {} (matched={})",
                d,
                if report.version.matched { "yes" } else { "NO" }
            );
        } else {
            println!("  daemon:        unreachable");
        }
        println!(
            "  transport:     {}",
            report.transport.public_url.as_deref().unwrap_or("(unset)")
        );
        if let Some(cu) = &report.transport.connect_url {
            println!("  connect_url:   {cu}");
        }
        println!(
            "  codesign:      platform={} host_supported={} codesign_present={}",
            report.codesign.platform,
            report.codesign.host_supported,
            report.codesign.codesign_present
        );
        println!("  next:          {}", report.next_action);
    }
    Ok(())
}

fn next_action_recommendation(
    daemon_reachable: bool,
    matched: bool,
    has_public_url: bool,
    has_connect_url: bool,
) -> String {
    if !daemon_reachable {
        return "Daemon not reachable. Run `focusa start` or `systemctl --user start focusa-daemon.service`.".to_string();
    }
    if !matched {
        return format!(
            "Daemon version differs from CLI version. Stop the running daemon and start it from the matching release. recovery_hint: focusa stop && focusa start."
        );
    }
    if !has_public_url {
        return "No FOCUSA_PAIRING_URL or /etc/focusa/public-url. Run `focusa pairing transport setup` to discover or generate a phone-reachable transport.".to_string();
    }
    if !has_connect_url {
        return format!(
            "Public URL {} is set but the daemon Connect API is not reachable from it. Check daemon health with `curl $FOCUSA_PAIRING_URL/v1/health`.",
            std::env::var("FOCUSA_PAIRING_URL").unwrap_or_default()
        );
    }
    "Pairing ready. Run `focusa pair` to print the connect URL and QR.".to_string()
}
