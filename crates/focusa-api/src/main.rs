//! Focusa Daemon — long-lived process hosting cognitive state.
//!
//! Source: docs/G1-12-api.md
//!
//! Runs two concurrent tasks:
//!   1. Daemon event loop (single-writer state machine)
//!   2. HTTP API server (read state + dispatch commands)
//!
//! Default bind: 127.0.0.1:8787
//! No auth in MVP (localhost only).

#![recursion_limit = "256"]

mod middleware;
mod routes;
mod scope;
mod scoped_store;
mod server;

use anyhow::anyhow;
use focusa_core::runtime::daemon::Daemon;
use focusa_core::types::{FocusaConfig, FocusaState};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::net::ToSocketAddrs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use tokio::sync::{Mutex, RwLock};

fn expand_home_dir(path: &str, home: Option<&Path>) -> PathBuf {
    match path {
        "~" => home
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(path)),
        _ => path
            .strip_prefix("~/")
            .and_then(|rest| home.map(|home| home.join(rest)))
            .unwrap_or_else(|| PathBuf::from(path)),
    }
}

fn resolved_data_dir(config: &FocusaConfig) -> PathBuf {
    let home_var = std::env::var_os("HOME");
    let home = home_var.as_deref().map(Path::new);
    expand_home_dir(&config.data_dir, home)
}

struct DaemonInstanceLock {
    path: PathBuf,
    pid: u32,
}

impl DaemonInstanceLock {
    fn acquire(config: &FocusaConfig) -> anyhow::Result<Self> {
        let pid = std::process::id();
        let data_dir = resolved_data_dir(config);
        fs::create_dir_all(&data_dir)?;
        let path = data_dir.join("focusa-daemon.lock");

        for _ in 0..2 {
            let opened = OpenOptions::new().create_new(true).write(true).open(&path);
            match opened {
                Ok(mut f) => {
                    let started = chrono::Utc::now().to_rfc3339();
                    writeln!(f, "pid={pid}")?;
                    writeln!(f, "bind={}", config.api_bind)?;
                    writeln!(f, "started_at={started}")?;
                    f.flush()?;
                    tracing::info!(pid, lock = %path.display(), "acquired daemon lock");
                    return Ok(Self { path, pid });
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    let stale = match read_lock_pid(&path) {
                        Some(existing_pid) => !process_alive(existing_pid),
                        None => true,
                    };
                    if stale {
                        let _ = fs::remove_file(&path);
                        continue;
                    }
                    let owner = read_lock_pid(&path).unwrap_or(0);
                    return Err(anyhow!(
                        r#"[DAEMON_ALREADY_RUNNING] {{"code":"DAEMON_ALREADY_RUNNING","pid":{},"lock":"{}"}}"#,
                        owner,
                        path.display()
                    ));
                }
                Err(e) => return Err(e.into()),
            }
        }

        Err(anyhow!("unable to acquire daemon lock {}", path.display()))
    }
}

impl Drop for DaemonInstanceLock {
    fn drop(&mut self) {
        let owner = read_lock_pid(&self.path);
        if owner == Some(self.pid) {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn read_lock_pid(path: &Path) -> Option<u32> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("pid=")
            && let Ok(pid) = rest.trim().parse::<u32>()
        {
            return Some(pid);
        }
    }
    None
}

fn process_alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

fn enforced_auth_token_configured() -> bool {
    std::env::var("FOCUSA_AUTH_TOKEN")
        .map(|token| !token.trim().is_empty())
        .unwrap_or(false)
}

fn bind_is_loopback(bind: &str) -> bool {
    bind.to_socket_addrs()
        .map(|mut addrs| addrs.all(|addr| addr.ip().is_loopback()))
        .unwrap_or(false)
}

fn enforce_bind_auth_guard(config: &FocusaConfig) -> anyhow::Result<()> {
    if bind_is_loopback(&config.api_bind) || enforced_auth_token_configured() {
        return Ok(());
    }
    Err(anyhow!(
        "[INSECURE_BIND_WITHOUT_AUTH] FOCUSA_BIND={} is non-loopback but no enforced FOCUSA_AUTH_TOKEN is configured; set FOCUSA_AUTH_TOKEN or bind to 127.0.0.1",
        config.api_bind
    ))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::args()
        .skip(1)
        .any(|arg| arg == "--version" || arg == "-V")
    {
        println!("focusa-daemon {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "focusa=info".into()),
        )
        .init();

    let mut config = FocusaConfig::default();

    // Allow overriding bind address via env (e.g., for Tailscale access from Mac).
    // FOCUSA_BIND=0.0.0.0:8787 or FOCUSA_BIND=100.94.238.56:8787
    if let Ok(bind) = std::env::var("FOCUSA_BIND") {
        config.api_bind = bind;
    }
    // Allow overriding data dir for isolated test runs.
    if let Ok(data_dir) = std::env::var("FOCUSA_DATA_DIR") {
        config.data_dir = data_dir;
    }
    config.data_dir = resolved_data_dir(&config).to_string_lossy().into_owned();
    enforce_bind_auth_guard(&config)?;

    // Startup banner: distinguish crash restart from intentional restart by
    // inspecting the previous lock file before we overwrite it.
    let pid = std::process::id();
    let lock_path = std::path::PathBuf::from(&config.data_dir).join("focusa-daemon.lock");
    let previous = std::fs::read_to_string(&lock_path).ok();
    let started_at = chrono::Utc::now().to_rfc3339();
    match previous.as_deref() {
        Some(prev) if prev.lines().any(|l| l.starts_with("pid=")) => {
            let prev_pid = prev.lines().find_map(|l| {
                l.strip_prefix("pid=")
                    .and_then(|s| s.trim().parse::<u32>().ok())
            });
            let prev_started = prev
                .lines()
                .find_map(|l| l.strip_prefix("started_at=").map(|s| s.trim().to_string()));
            tracing::warn!(
                pid,
                prev_pid = ?prev_pid,
                prev_started_at = ?prev_started,
                new_started_at = %started_at,
                "focusa-daemon startup: replacing prior lock file (was the previous instance an intentional shutdown or a crash?)"
            );
        }
        Some(prev) => {
            // Lock file existed but no pid line — unparseable, treat as suspect.
            tracing::warn!(
                pid,
                prev_contents = %prev.lines().next().unwrap_or(""),
                "focusa-daemon startup: prior lock file was unparseable; replacing"
            );
        }
        None => {
            tracing::info!(
                pid,
                "focusa-daemon startup: no prior lock file (fresh install)"
            );
        }
    }
    tracing::info!(
        pid,
        version = env!("CARGO_PKG_VERSION"),
        bind = %config.api_bind,
        data_dir = %config.data_dir,
        started_at = %started_at,
        "focusa-daemon starting",
    );

    let _instance_lock = DaemonInstanceLock::acquire(&config)?;

    // License plane: evaluate tier + log current capability posture.
    // Bead focusa-nbai.1: wire LicenseGuard into daemon startup.
    let license_guard = focusa_license::resolve_license_guard();
    tracing::info!(
        tier = license_guard.tier.label(),
        issued_at = %license_guard.issued_at,
        expires_at = ?license_guard.expires_at,
        bsl_change_date = %license_guard.bsl_change_date,
        customer_email = ?license_guard.customer_email,
        key_hash = ?license_guard.key_hash,
        expired = license_guard.is_expired(),
        "focusa-daemon license plane ready (focusa-license crate)"
    );
    // Soft-warn when commercial use is requested but license is eval.
    if let Some(warn) = license_guard
        .require(focusa_license::Capability::CommercialUse)
        .ok()
        .flatten()
    {
        tracing::warn!(warning = %warn, "focusa-daemon running under eval tier with commercial capability requested");
    }
    // Register the LicenseGuard so /v1/license/status can serve it.
    crate::routes::license::init_guard(license_guard);

    // Shared state: daemon writes after every reduction, API reads.
    let shared_state = Arc::new(RwLock::new(FocusaState::default()));

    // Event bus for SSE.
    let (events_tx, _events_rx) = tokio::sync::broadcast::channel::<String>(1024);
    let write_serial_lock = Arc::new(Mutex::new(()));
    let external_mutation_epoch = Arc::new(AtomicU64::new(0));

    // Initialize daemon (loads saved state from disk, syncs to shared_state on run).
    let mut daemon = Daemon::new(
        config.clone(),
        shared_state.clone(),
        write_serial_lock.clone(),
        external_mutation_epoch.clone(),
    )?;
    daemon.attach_event_bus(focusa_core::runtime::event_bus::EventBus::new(
        events_tx.clone(),
    ));
    let command_tx = daemon.command_sender();
    let events_tx_for_api = events_tx.clone();
    let events_tx_for_completion_sweep = events_tx.clone();

    // One bounded persistence actor serves daemon and direct API writers.
    let persistence = daemon.persistence();
    let persistence_actor =
        focusa_core::runtime::persistence_actor::PersistenceActor::start(persistence.clone());
    daemon.attach_persistence_actor(persistence_actor.clone());

    // V2: rehydrate PairingStore state from SQLite at startup so the in-memory
    // maps (connect_sessions, tokens) are not empty after a daemon restart.
    // The actual call happens inside server::run after AppState is built.

    // Bonjour / mDNS advertise port (read before config is moved below).
    let bonjour_port = config
        .api_bind
        .rsplit(':')
        .next()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8787);

    // #307: surface the developer-origin entitlement state once at startup.
    {
        let origin = focusa_core::license_developer_origin::developer_origin_report();
        tracing::info!(
            active = origin.active,
            agent_kb_known = origin.agent_kb_known,
            tailnet_member = origin.tailnet_member,
            tailnet_suffix = %origin.tailnet_suffix,
            "developer-origin entitlement resolved"
        );
    }

    // Spawn daemon event loop.
    let daemon_handle = tokio::spawn(async move {
        if let Err(e) = daemon.run().await {
            tracing::error!("Daemon error: {}", e);
        }
    });

    let completion_sweep_db = resolved_data_dir(&config).join("focusa.sqlite");
    let retention_data_dir_hoisted = resolved_data_dir(&config);

    // Start API server (blocks until shutdown).
    let api_handle = tokio::spawn(async move {
        if let Err(e) = server::run(
            shared_state,
            command_tx,
            events_tx_for_api,
            config,
            (persistence, persistence_actor),
            write_serial_lock,
            external_mutation_epoch,
        )
        .await
        {
            tracing::error!("API server error: {}", e);
        }
    });

    // Advertise _focusa._tcp.local via Bonjour / mDNS so the Mac menubar
    // wizard can auto-discover this daemon on the LAN without operator input.
    // The TXT record carries the `url` so the Mac can skip the Tailscale
    // round-trip when on the same LAN. (G08)
    //
    // #251: mDNS availability must never threaten the daemon. Honoring
    // FOCUSA_DISABLE_MDNS skips advertisement entirely; any registration
    // failure is logged and the daemon keeps serving.
    let mdns_disabled = std::env::var("FOCUSA_DISABLE_MDNS")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let _bonjour_handle = tokio::spawn(async move {
        if mdns_disabled {
            tracing::info!("mDNS advertisement disabled via FOCUSA_DISABLE_MDNS");
            return;
        }
        match focusa_core::bonjour::advertise("_focusa._tcp.local.", bonjour_port).await {
            Ok(_service) => {}
            Err(error) => {
                tracing::warn!(error = %error, "Bonjour advertisement failed (non-fatal); daemon continues without LAN discovery. Set FOCUSA_DISABLE_MDNS=1 to silence.");
            }
        }
    });

    // Event-ledger retention sweep. First tick fires immediately; subsequent
    // sweeps run daily. Bounded batches keep the daemon writer responsive and
    // cold export lands in <data>/events-cold. Disable with
    // FOCUSA_EVENT_RETENTION_DISABLED=1; window via FOCUSA_EVENT_RETENTION_DAYS.
    let retention_data_dir = retention_data_dir_hoisted;
    let _retention_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(86_400));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            if std::env::var(focusa_core::runtime::event_retention::RETENTION_ENV_DISABLED)
                .map(|value| value == "1" || value == "true")
                .unwrap_or(false)
            {
                tracing::info!("event retention disabled via environment");
            } else {
                let data_dir = retention_data_dir.clone();
                let result = tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
                    let db_path = data_dir.join("focusa.sqlite");
                    let conn = rusqlite::Connection::open(&db_path)?;
                    conn.busy_timeout(std::time::Duration::from_secs(30))?;
                    let days: u32 = std::env::var(
                        focusa_core::runtime::event_retention::RETENTION_ENV_DAYS,
                    )
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(focusa_core::runtime::event_retention::DEFAULT_RETENTION_DAYS);
                    let cutoff = focusa_core::runtime::event_retention::retention_cutoff(days);
                    let export = data_dir.join("events-cold");
                    let summary = focusa_core::runtime::event_retention::prune_before(
                        &conn,
                        &cutoff,
                        Some(&export),
                        5_000,
                    )?;
                    tracing::info!(
                        deleted = summary.deleted_events,
                        exported = summary.exported_events,
                        anchor = summary.anchor_chain_index,
                        cutoff = %summary.cutoff_ts,
                        "event retention sweep complete"
                    );
                    Ok(())
                })
                .await;
                if let Err(error) = result {
                    tracing::warn!(error = %error, "event retention sweep failed");
                }
            }
            interval.tick().await;
        }
    });

    // Silent-session completion sweeper (issue #311): every 30 seconds, scan
    // settled sessions, record their durable completion events, and broadcast
    // them over SSE so agents are notified instead of polling. Missed events
    // remain recoverable through the completions backfill route.
    let _completion_sweep_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            interval.tick().await;
            let db = completion_sweep_db.clone();
            let tx = events_tx_for_completion_sweep.clone();
            let result = tokio::task::spawn_blocking(move || {
                crate::routes::silent_sessions_wait::sweep_completions(&db, &tx)
            })
            .await;
            match result {
                Ok(Ok(emitted)) if emitted > 0 => {
                    tracing::info!(emitted, "silent session completion sweep emitted events");
                }
                Ok(Ok(_)) => {}
                Ok(Err(error)) => {
                    tracing::warn!(error = %error, "silent session completion sweep failed");
                }
                Err(error) => {
                    tracing::warn!(error = %error, "silent session completion sweep join failed");
                }
            }
        }
    });

    // Wait for either to finish (normally neither should).
    tokio::select! {
        _ = daemon_handle => tracing::warn!("Daemon exited"),
        _ = api_handle => tracing::warn!("API server exited"),
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn expand_home_dir_expands_tilde_prefix() {
        let home = Path::new("/home/focusa-user");
        assert_eq!(
            expand_home_dir("~", Some(home)),
            PathBuf::from("/home/focusa-user")
        );
        assert_eq!(
            expand_home_dir("~/focusa-data", Some(home)),
            home.join("focusa-data")
        );
    }

    #[test]
    fn expand_home_dir_preserves_literal_path_without_home() {
        assert_eq!(
            expand_home_dir("~/focusa-data", None),
            PathBuf::from("~/focusa-data")
        );
        assert_eq!(
            expand_home_dir("/tmp/focusa", Some(Path::new("/home/focusa-user"))),
            PathBuf::from("/tmp/focusa")
        );
    }

    #[test]
    fn bind_auth_guard_allows_loopback_without_token() {
        let mut config = FocusaConfig::default();
        config.api_bind = "127.0.0.1:8787".to_string();
        assert!(enforce_bind_auth_guard(&config).is_ok());
        config.api_bind = "[::1]:8787".to_string();
        assert!(enforce_bind_auth_guard(&config).is_ok());
    }

    #[test]
    fn bind_auth_guard_rejects_non_loopback_with_config_only_token() {
        if std::env::var("FOCUSA_AUTH_TOKEN").is_ok() {
            return;
        }
        let config = FocusaConfig {
            api_bind: "0.0.0.0:8787".to_string(),
            auth_token: Some("configured-token-not-enforced-by-middleware".to_string()),
            ..FocusaConfig::default()
        };
        let err = enforce_bind_auth_guard(&config).expect_err("config-only token is not enforced");
        assert!(err.to_string().contains("FOCUSA_AUTH_TOKEN"));
    }

    #[test]
    fn bind_auth_guard_rejects_non_loopback_without_token_when_env_absent() {
        if std::env::var("FOCUSA_AUTH_TOKEN").is_ok() {
            return;
        }
        let config = FocusaConfig {
            api_bind: "0.0.0.0:8787".to_string(),
            auth_token: None,
            ..FocusaConfig::default()
        };
        let err = enforce_bind_auth_guard(&config).expect_err("non-loopback requires auth");
        assert!(err.to_string().contains("INSECURE_BIND_WITHOUT_AUTH"));
    }
}
