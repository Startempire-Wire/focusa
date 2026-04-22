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
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

struct DaemonInstanceLock {
    path: PathBuf,
    pid: u32,
}

impl DaemonInstanceLock {
    fn acquire(config: &FocusaConfig) -> anyhow::Result<Self> {
        let pid = std::process::id();
        let data_dir = PathBuf::from(&config.data_dir);
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
        if let Some(rest) = line.strip_prefix("pid=") {
            if let Ok(pid) = rest.trim().parse::<u32>() {
                return Some(pid);
            }
        }
    }
    None
}

fn process_alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
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

    let _instance_lock = DaemonInstanceLock::acquire(&config)?;

    // Shared state: daemon writes after every reduction, API reads.
    let shared_state = Arc::new(RwLock::new(FocusaState::default()));

    // Event bus for SSE.
    let (events_tx, _events_rx) = tokio::sync::broadcast::channel::<String>(1024);
    let write_serial_lock = Arc::new(Mutex::new(()));

    // Initialize daemon (loads saved state from disk, syncs to shared_state on run).
    let mut daemon = Daemon::new(
        config.clone(),
        shared_state.clone(),
        write_serial_lock.clone(),
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
