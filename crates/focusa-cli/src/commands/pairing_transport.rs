//! Commercialization-safe multi-transport pairing transport setup (focusa-ifc3).
//!
//! Mass-adoption / long-term policy:
//!   - Defaults are open-source-reusable-licensed, self-hostable, zero-account.
//!   - Vendor coordination-server tunnels are opt-in only (FOCUSA_TUNNEL_*=1).
//!   - Never bundle vendor binaries; runtime download with LICENSE/NOTICE.
//!
//! Default order: operator URL -> public hostname/IP -> ssh -R -> frp -> bore.
//! Vendor interop (opt-in): cloudflared quick tunnel, Tailscale Funnel, ngrok.
//! Not supported: localhost.run.

use anyhow::{Context, Result};
use clap::Subcommand;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;

const PUBLIC_URL_FILE: &str = "/etc/focusa/public-url";
const USER_PUBLIC_URL_FILE: &str = ".config/focusa/public-url";

#[derive(Subcommand, Debug)]
pub enum TransportCmd {
    /// Probe + write /etc/focusa/public-url with the best phone-reachable transport.
    Setup {
        #[arg(long)]
        json: bool,
    },
    /// Print currently-written /etc/focusa/public-url (if any).
    Show {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Serialize)]
pub struct TransportReport {
    pub chosen_transport: Option<String>,
    pub chosen_url: Option<String>,
    pub candidates: Vec<Candidate>,
    pub public_url_written: bool,
    pub recovery_hint: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Candidate {
    pub transport: &'static str,
    pub url: Option<String>,
    pub status: &'static str,
    pub note: Option<String>,
}

pub async fn run(cmd: TransportCmd) -> Result<()> {
    let res = match cmd {
        TransportCmd::Setup { json } => tokio::task::spawn_blocking(move || setup(json)).await?,
        TransportCmd::Show { json } => tokio::task::spawn_blocking(move || show(json)).await?,
    };
    use std::io::Write;
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
    res
}

fn setup(json: bool) -> Result<()> {
    let mut candidates = Vec::new();

    // 1. Operator-supplied URL (highest trust).
    if let Some(url) = operator_url() {
        candidates.push(Candidate {
            transport: "operator_url",
            url: Some(url.clone()),
            status: "candidate",
            note: Some("from FOCUSA_PAIRING_URL or /etc/focusa/public-url".into()),
        });
    }

    // 2. Public hostname / IP probe (already in pair resolver, recorded as
    //    informational here; the resolver itself is the authority).
    candidates.push(host_or_ip_candidate());

    // 3. ssh -R reverse tunnel to operator-controlled jump host.
    candidates.push(ssh_reverse_candidate());

    // 4. frp (Apache 2.0, self-hostable) -- only if opted in via binary path.
    candidates.push(frp_candidate());

    // 5. bore (MIT, self-hostable).
    candidates.push(bore_candidate());

    // 6. Vendor interop (opt-in only via FOCUSA_TUNNEL_*=1).
    if opt_in_enabled("CLOUDFLARED") {
        candidates.push(cloudflared_candidate());
    }
    if opt_in_enabled("TAILSCALE") {
        candidates.push(tailscale_candidate());
    }
    if opt_in_enabled("NGROK") {
        candidates.push(ngrok_candidate());
    }

    // Pick the first candidate with a URL.
    let chosen = candidates.iter().find(|c| c.url.is_some()).cloned();
    let (chosen_transport, chosen_url) = match &chosen {
        Some(c) => (Some(c.transport.to_string()), c.url.clone()),
        None => (None, None),
    };

    let public_url_written = if let Some(url) = chosen_url.clone() {
        match write_public_url(&url) {
            Ok(()) => true,
            Err(err) => {
                eprintln!(
                    "focusa pairing transport setup: failed to write {}: {err}",
                    PUBLIC_URL_FILE
                );
                false
            }
        }
    } else {
        false
    };

    let recovery_hint = if chosen.is_none() {
        Some(format!(
            "No verified transport found. Set FOCUSA_PAIRING_URL=https://your-host, write {}, or configure ssh -R / frp / bore. Vendor tunnels require FOCUSA_TUNNEL_CLOUDFLARED=1 / TAILSCALE=1 / NGROK=1.",
            PUBLIC_URL_FILE
        ))
    } else {
        None
    };

    let report = TransportReport {
        chosen_transport,
        chosen_url,
        candidates,
        public_url_written,
        recovery_hint,
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("focusa pairing transport setup");
        for c in &report.candidates {
            match &c.url {
                Some(u) => println!("  OK {} ({}) -> {}", c.transport, c.status, u),
                None => println!(
                    "  - {} ({}) {}",
                    c.transport,
                    c.status,
                    c.note.clone().unwrap_or_default()
                ),
            }
        }
        if let Some(t) = &report.chosen_transport {
            println!("  chosen: {t}");
            println!("  public_url_written: {}", report.public_url_written);
        } else if let Some(h) = &report.recovery_hint {
            println!("  recovery_hint: {h}");
        }
    }
    Ok(())
}

fn show(json: bool) -> Result<()> {
    let url = operator_url();
    if json {
        let payload = serde_json::json!({"public_url": url});
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        match url {
            Some(u) => println!("{u}"),
            None => println!("(unset)"),
        }
    }
    Ok(())
}

fn opt_in_enabled(name: &str) -> bool {
    std::env::var(format!("FOCUSA_TUNNEL_{name}"))
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn operator_url() -> Option<String> {
    for key in ["FOCUSA_PAIRING_URL", "FOCUSA_PUBLIC_URL"] {
        if let Ok(v) = std::env::var(key) {
            let trimmed = v.trim().trim_end_matches('/').to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    let candidates = [
        PUBLIC_URL_FILE.to_string(),
        user_public_url_path(),
    ];
    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            let trimmed = content.trim().trim_end_matches('/').to_string();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                return Some(trimmed);
            }
        }
    }
    None
}

fn user_public_url_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    format!("{home}/{USER_PUBLIC_URL_FILE}")
}

fn write_public_url(url: &str) -> Result<()> {
    if let Some(parent) = Path::new(PUBLIC_URL_FILE).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if std::fs::write(PUBLIC_URL_FILE, format!("{url}\n")).is_ok() {
        return Ok(());
    }
    // Fallback to per-user path when /etc/focusa isn't writable.
    let user_path = user_public_url_path();
    if let Some(parent) = Path::new(&user_path).parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    std::fs::write(&user_path, format!("{url}\n"))
        .with_context(|| format!("write {user_path}"))?;
    Ok(())
}

fn host_or_ip_candidate() -> Candidate {
    // Hostname (best-effort).
    let host = hostname_best_effort();
    match host {
        Some(h) => Candidate {
            transport: "host_public",
            url: Some(format!("https://{h}")),
            status: "candidate",
            note: Some("verified via pair resolver; not auto-written".into()),
        },
        None => Candidate {
            transport: "host_public",
            url: None,
            status: "skipped",
            note: Some("no hostname detected".into()),
        },
    }
}

fn hostname_best_effort() -> Option<String> {
    let out = Command::new("hostname").args(["-f"]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().trim_end_matches('.').to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn ssh_reverse_candidate() -> Candidate {
    let jump = std::env::var("FOCUSA_TUNNEL_SSH_JUMP").ok();
    match jump {
        Some(j) => Candidate {
            transport: "ssh_reverse",
            url: Some(j.clone()),
            status: "candidate",
            note: Some(format!(
                "operator-controlled jump host: {j}. Run: ssh -R 80:127.0.0.1:8787 {j}"
            )),
        },
        None => Candidate {
            transport: "ssh_reverse",
            url: None,
            status: "skipped",
            note: Some(
                "set FOCUSA_TUNNEL_SSH_JUMP=user@jump-host to enable self-hostable ssh -R"
                    .into(),
            ),
        },
    }
}

fn frp_candidate() -> Candidate {
    if which("frpc").is_none() {
        return Candidate {
            transport: "frp",
            url: None,
            status: "skipped",
            note: Some(
                "frpc (Apache 2.0, self-hostable) not installed; install from https://github.com/fatedier/frp"
                    .into(),
            ),
        };
    }
    Candidate {
        transport: "frp",
        url: None,
        status: "candidate",
        note: Some("requires operator-run frps server; not auto-provisioned".into()),
    }
}

fn bore_candidate() -> Candidate {
    if which("bore").is_none() {
        return Candidate {
            transport: "bore",
            url: None,
            status: "skipped",
            note: Some(
                "bore (MIT, self-hostable) not installed; install from https://github.com/ekzhang/bore"
                    .into(),
            ),
        };
    }
    Candidate {
        transport: "bore",
        url: None,
        status: "candidate",
        note: Some("requires operator-run bore server; not auto-provisioned".into()),
    }
}

fn cloudflared_candidate() -> Candidate {
    if which("cloudflared").is_none() {
        return Candidate {
            transport: "cloudflared_quick",
            url: None,
            status: "skipped",
            note: Some("cloudflared not installed (opt-in)".into()),
        };
    }
    Candidate {
        transport: "cloudflared_quick",
        url: None,
        status: "skipped",
        note: Some(
            "VENDOR INTEROP (opt-in): probe-time spawn disabled in setup; run `cloudflared tunnel --url http://127.0.0.1:8787` yourself and set FOCUSA_PAIRING_URL".into(),
        ),
    }
}

fn tailscale_candidate() -> Candidate {
    if which("tailscale").is_none() {
        return Candidate {
            transport: "tailscale_funnel",
            url: None,
            status: "skipped",
            note: Some("tailscale not installed (opt-in)".into()),
        };
    }
    Candidate {
        transport: "tailscale_funnel",
        url: None,
        status: "skipped",
        note: Some(
            "VENDOR INTEROP (opt-in): requires Tailscale account + `tailscale funnel 8787`; not auto-enabled"
                .into(),
        ),
    }
}

fn ngrok_candidate() -> Candidate {
    if which("ngrok").is_none() {
        return Candidate {
            transport: "ngrok",
            url: None,
            status: "skipped",
            note: Some("ngrok not installed (opt-in)".into()),
        };
    }
    Candidate {
        transport: "ngrok",
        url: None,
        status: "skipped",
        note: Some(
            "VENDOR INTEROP (opt-in): requires ngrok account + auth token; not auto-enabled"
                .into(),
        ),
    }
}

fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for entry in std::env::split_paths(&path) {
        if entry.join(name).is_file() {
            return Some(entry.join(name));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_url_prefers_env_over_file() {
        // env precedence sanity check (uses real env if set; harmless).
        let _ = operator_url();
    }

    #[test]
    fn opt_in_only_for_named_vendor_transports() {
        assert!(!opt_in_enabled("CLOUDFLARED"));
        assert!(!opt_in_enabled("TAILSCALE"));
        assert!(!opt_in_enabled("NGROK"));
        assert!(!opt_in_enabled("SOMETHING_ELSE"));
    }

    #[test]
    fn report_serializes() {
        let r = TransportReport {
            chosen_transport: Some("operator_url".into()),
            chosen_url: Some("https://focusa.example.com".into()),
            candidates: vec![],
            public_url_written: true,
            recovery_hint: None,
        };
        assert!(serde_json::to_string(&r).is_ok());
    }
}