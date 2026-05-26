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
                        "[DAEMON_ALREADY_RUNNING] pid={} lock={}",
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

    let _instance_lock = DaemonInstanceLock::acquire(&config)?;

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

    // Clone persistence for API server (sync routes need direct DB access).
    let persistence = daemon.persistence();

    // Spawn daemon event loop.
    let daemon_handle = tokio::spawn(async move {
        if let Err(e) = daemon.run().await {
            tracing::error!("Daemon error: {}", e);
        }
    });

    // Start API server (blocks until shutdown).
    let api_handle = tokio::spawn(async move {
        if let Err(e) = server::run(
            shared_state,
            command_tx,
            events_tx_for_api,
            config,
            persistence,
            write_serial_lock,
            external_mutation_epoch,
        )
        .await
        {
            tracing::error!("API server error: {}", e);
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
