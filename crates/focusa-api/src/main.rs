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
use focusa_core::daemon_lifecycle::{DaemonLockRecord, DaemonProcessIdentity};
use focusa_core::runtime::daemon::Daemon;
use focusa_core::types::{FocusaConfig, FocusaState};
use rand::{RngCore, rngs::OsRng};
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

fn pi_ota_update_state_root() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))
        .unwrap_or_else(|| PathBuf::from(".local/state"))
        .join("focusa/update")
}

fn bridge_legacy_pi_activation_marker(root: &Path) -> std::io::Result<Option<PathBuf>> {
    let legacy = root.join("pi-extension-restart-required.json");
    if !legacy.is_file() {
        return Ok(None);
    }
    let silent = root.join("pi-extension-silent-restart-required.json");
    let destination = if silent.exists() {
        root.join(format!(
            "pi-extension-legacy-quarantined-{}.json",
            uuid::Uuid::now_v7()
        ))
    } else {
        silent
    };
    std::fs::rename(legacy, &destination)?;
    Ok(Some(destination))
}

async fn run_legacy_pi_activation_bridge() {
    let root = pi_ota_update_state_root();
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
    loop {
        interval.tick().await;
        match bridge_legacy_pi_activation_marker(&root) {
            Ok(Some(destination)) => tracing::info!(
                marker = %destination.display(),
                "quarantined legacy conversational Pi OTA marker"
            ),
            Ok(None) => {}
            Err(error) => tracing::warn!(
                error = %error,
                "legacy Pi OTA marker quarantine deferred"
            ),
        }
    }
}

struct DaemonInstanceLock {
    path: PathBuf,
    record: DaemonLockRecord,
}

fn secure_random_token() -> String {
    let mut bytes = [0_u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn create_new_lock_file(path: &Path) -> std::io::Result<std::fs::File> {
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

impl DaemonInstanceLock {
    fn acquire(config: &FocusaConfig, started_at: String) -> anyhow::Result<Self> {
        let pid = std::process::id();
        let data_dir = resolved_data_dir(config);
        fs::create_dir_all(&data_dir)?;
        let path = data_dir.join("focusa-daemon.lock");
        let record = DaemonLockRecord {
            pid,
            bind: config.api_bind.clone(),
            started_at,
            start_token: secure_random_token(),
            shutdown_token: secure_random_token(),
        };

        for _ in 0..2 {
            match create_new_lock_file(&path) {
                Ok(mut file) => {
                    file.write_all(record.render().as_bytes())?;
                    file.flush()?;
                    file.sync_all()?;
                    tracing::info!(pid, lock = %path.display(), "acquired daemon lock");
                    return Ok(Self { path, record });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    let existing_content = fs::read_to_string(&path).map_err(|read_error| {
                        anyhow!(
                            "[DAEMON_LOCK_UNVERIFIED] existing daemon lock cannot be read: {read_error}"
                        )
                    })?;
                    let existing = DaemonLockRecord::parse(&existing_content).map_err(|parse_error| {
                        anyhow!(
                            "[DAEMON_LOCK_UNVERIFIED] existing daemon lock is not an exact lifecycle record: {parse_error}"
                        )
                    })?;
                    if !process_alive(existing.pid) {
                        fs::remove_file(&path)?;
                        continue;
                    }
                    return Err(anyhow!(
                        r#"[DAEMON_ALREADY_RUNNING] {{"code":"DAEMON_ALREADY_RUNNING","pid":{},"lock":"{}"}}"#,
                        existing.pid,
                        path.display()
                    ));
                }
                Err(error) => return Err(error.into()),
            }
        }

        Err(anyhow!("unable to acquire daemon lock {}", path.display()))
    }

    fn runtime_identity(&self) -> server::DaemonRuntimeIdentity {
        server::DaemonRuntimeIdentity {
            process: DaemonProcessIdentity::new(
                self.record.pid,
                self.record.start_token.clone(),
                self.path.to_string_lossy(),
            ),
            shutdown_token: self.record.shutdown_token.clone(),
        }
    }
}

impl Drop for DaemonInstanceLock {
    fn drop(&mut self) {
        let still_owned = fs::read_to_string(&self.path)
            .ok()
            .and_then(|content| DaemonLockRecord::parse(&content).ok())
            .is_some_and(|record| {
                record.pid == self.record.pid && record.start_token == self.record.start_token
            });
        if still_owned && let Err(error) = fs::remove_file(&self.path) {
            tracing::warn!(
                error = %error,
                lock = %self.path.display(),
                "failed to remove owned daemon lock during shutdown"
            );
        }
    }
}

#[cfg(target_os = "linux")]
fn process_alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

/// Platforms without a safe standard-library process identity probe retain an
/// existing exact lock and fail closed. They never infer staleness by name or
/// signal an unverified PID.
#[cfg(not(target_os = "linux"))]
fn process_alive(_pid: u32) -> bool {
    true
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

/// CLI flags the daemon recognizes before any lock/startup work.
///
/// Fixes #213: `--help` must print usage and exit (it previously started a
/// full daemon instance and could displace the live one), and unknown
/// dash-arguments must fail instead of silently starting.
#[derive(Debug, Clone, PartialEq, Eq)]
enum CliAction {
    Version,
    Help,
    Unknown(String),
}

fn detect_cli_action(args: impl Iterator<Item = String>) -> Option<CliAction> {
    let mut args = args.skip(1);
    let first = args.next()?;
    match first.as_str() {
        "--version" | "-V" => Some(CliAction::Version),
        "--help" | "-h" => Some(CliAction::Help),
        _ if first.starts_with('-') => Some(CliAction::Unknown(first)),
        // Positional arguments are not used by the daemon; preserve the
        // historical behavior of starting normally.
        _ => None,
    }
}

fn print_daemon_usage() {
    println!(
        "Focusa daemon — cognitive governance runtime\n\n\
         USAGE:\n\
             focusa-daemon [OPTIONS]\n\n\
         OPTIONS:\n\
             -h, --help       Print this help and exit\n\
             -V, --version    Print version and exit\n\n\
         ENVIRONMENT:\n\
             FOCUSA_BIND          Bind address (default 127.0.0.1:8787)\n\
             FOCUSA_DATA_DIR      Data directory override\n\
             FOCUSA_AUTH_TOKEN    Enforced auth token for non-loopback binds"
    );
}

async fn wait_for_os_shutdown() -> std::io::Result<()> {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
        tokio::select! {
            result = tokio::signal::ctrl_c() => result,
            _ = terminate.recv() => Ok(()),
        }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match detect_cli_action(std::env::args()) {
        Some(CliAction::Version) => {
            println!("focusa-daemon {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some(CliAction::Help) => {
            print_daemon_usage();
            return Ok(());
        }
        Some(CliAction::Unknown(arg)) => {
            eprintln!("focusa-daemon: unknown argument: {arg}");
            print_daemon_usage();
            std::process::exit(2);
        }
        None => {}
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
                "focusa-daemon startup: prior lock detected; exact lifecycle ownership will be verified before acquisition"
            );
        }
        Some(_) => {
            // Never log untrusted lock content: it may contain credentials.
            tracing::warn!(
                pid,
                "focusa-daemon startup: prior lock is unparseable; acquisition will fail closed"
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

    let instance_lock = DaemonInstanceLock::acquire(&config, started_at.clone())?;
    let daemon_runtime_identity = instance_lock.runtime_identity();
    let _instance_lock = instance_lock;
    let _legacy_pi_activation_bridge = tokio::spawn(run_legacy_pi_activation_bridge());

    // License plane: evaluate tier + log current capability posture.
    // Bead focusa-nbai.1: wire LicenseGuard into daemon startup.
    let mut license_guard = focusa_license::resolve_license_guard();
    // Isolated e2e/CI daemons run with FOCUSA_TEST_MODE=1 and no operator lease;
    // grant a bound test entitlement so value-producing e2e surfaces exercise
    // the full entitlement path instead of failing at the gate.
    if std::env::var("FOCUSA_TEST_MODE")
        .map(|value| value == "1")
        .unwrap_or(false)
        && license_guard.entitlement.as_ref().is_none_or(|snapshot| {
            !matches!(
                snapshot.state,
                focusa_license::authority::EntitlementState::Active
            )
        })
    {
        let mut entitlement =
            focusa_license::authority::EntitlementSnapshot::unactivated("focusa", "test-node");
        entitlement.state = focusa_license::authority::EntitlementState::Active;
        entitlement.lease_id = Some("test-lease".to_string());
        entitlement.sequence = Some(1);
        entitlement.lease_digest = Some("sha256:test-lease-digest".to_string());
        entitlement.expires_at = Some(chrono::Utc::now() + chrono::Duration::hours(1));
        license_guard = focusa_license::LicenseGuard::from_entitlement(entitlement);
    }
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
    // Shared state: daemon writes after every reduction, API reads.
    let shared_state = Arc::new(RwLock::new(FocusaState::default()));

    // Event bus for SSE.
    let (events_tx, _events_rx) = tokio::sync::broadcast::channel::<String>(1024);
    let write_serial_lock = Arc::new(Mutex::new(()));
    let external_mutation_epoch = Arc::new(AtomicU64::new(0));
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let shutdown_tx_for_supervisor = shutdown_tx.clone();
    let shutdown_tx_for_os = shutdown_tx.clone();
    let daemon_shutdown_rx = shutdown_rx.clone();
    let api_shutdown_rx = shutdown_rx;

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

    // SIGTERM is systemd's normal stop path. Convert OS termination into the
    // same governed shutdown signal used by the API so the daemon's bounded
    // state checkpoint and durable event cursor complete before process exit.
    tokio::spawn(async move {
        match wait_for_os_shutdown().await {
            Ok(()) => {
                let _ = shutdown_tx_for_os.send(true);
            }
            Err(error) => tracing::error!(error = %error, "OS shutdown signal listener failed"),
        }
    });

    // Spawn daemon event loop.
    let mut daemon_handle = tokio::spawn(async move {
        if let Err(e) = daemon.run_until_shutdown(daemon_shutdown_rx).await {
            tracing::error!("Daemon error: {}", e);
        }
    });

    // Start API server (blocks until shutdown).
    let mut api_handle = tokio::spawn(async move {
        if let Err(e) = server::run(
            shared_state,
            command_tx,
            events_tx_for_api,
            config,
            (persistence, persistence_actor),
            write_serial_lock,
            external_mutation_epoch,
            license_guard,
            daemon_runtime_identity,
            shutdown_tx,
            api_shutdown_rx,
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
    let _bonjour_handle = tokio::spawn(async move {
        if let Err(e) = focusa_core::bonjour::advertise("_focusa._tcp.local.", bonjour_port).await {
            tracing::warn!(error = %e, "Bonjour advertisement ended (non-fatal)");
        }
    });

    // If either governed plane exits, request its peer to stop and await it.
    // Intentional shutdown therefore cannot end the Tokio runtime before the
    // daemon's final persistence flush completes.
    tokio::select! {
        result = &mut daemon_handle => {
            tracing::warn!(result = ?result, "Daemon exited");
            let _ = shutdown_tx_for_supervisor.send(true);
            let _ = api_handle.await;
        }
        result = &mut api_handle => {
            tracing::warn!(result = ?result, "API server exited");
            let _ = shutdown_tx_for_supervisor.send(true);
            let _ = daemon_handle.await;
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    fn lock_test_config(label: &str) -> (FocusaConfig, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "focusa-daemon-lock-{label}-{}",
            uuid::Uuid::now_v7()
        ));
        let config = FocusaConfig {
            data_dir: root.to_string_lossy().into_owned(),
            ..FocusaConfig::default()
        };
        (config, root)
    }

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

    #[test]
    fn unversioned_existing_lock_fails_closed_without_removal() {
        let (config, root) = lock_test_config("legacy");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("focusa-daemon.lock");
        let content = format!("pid={}\nbind=127.0.0.1:8787\n", std::process::id());
        std::fs::write(&path, &content).unwrap();

        let error = DaemonInstanceLock::acquire(&config, chrono::Utc::now().to_rfc3339())
            .err()
            .expect("unversioned lock must fail closed");
        assert!(error.to_string().contains("DAEMON_LOCK_UNVERIFIED"));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), content);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn exact_dead_lock_is_replaced_without_signaling_and_new_lock_is_private() {
        let (config, root) = lock_test_config("stale");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("focusa-daemon.lock");
        let stale = DaemonLockRecord {
            pid: u32::MAX,
            bind: config.api_bind.clone(),
            started_at: "2026-08-31T00:00:00Z".into(),
            start_token: "stale-start".into(),
            shutdown_token: "stale-shutdown".into(),
        };
        std::fs::write(&path, stale.render()).unwrap();

        let lock = DaemonInstanceLock::acquire(&config, chrono::Utc::now().to_rfc3339()).unwrap();
        assert_eq!(lock.record.pid, std::process::id());
        assert_ne!(lock.record.start_token, stale.start_token);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        drop(lock);
        assert!(!path.exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn legacy_pi_activation_marker_is_atomically_quarantined() {
        let root =
            std::env::temp_dir().join(format!("focusa-legacy-pi-marker-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&root).unwrap();
        let legacy = root.join("pi-extension-restart-required.json");
        std::fs::write(&legacy, r#"{"version":"0.9.135-dev"}"#).unwrap();

        let destination = bridge_legacy_pi_activation_marker(&root)
            .unwrap()
            .expect("legacy marker should move");
        assert_eq!(
            destination,
            root.join("pi-extension-silent-restart-required.json")
        );
        assert!(!legacy.exists());
        assert!(destination.is_file());
        assert!(bridge_legacy_pi_activation_marker(&root).unwrap().is_none());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn legacy_marker_remains_recoverable_when_silent_marker_exists() {
        let root = std::env::temp_dir().join(format!(
            "focusa-legacy-pi-marker-conflict-{}",
            uuid::Uuid::now_v7()
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("pi-extension-restart-required.json"),
            r#"{"version":"0.9.136-dev"}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("pi-extension-silent-restart-required.json"),
            r#"{"version":"0.9.135-dev"}"#,
        )
        .unwrap();

        let destination = bridge_legacy_pi_activation_marker(&root)
            .unwrap()
            .expect("legacy marker should be quarantined");
        assert!(destination.file_name().is_some_and(|name| {
            name.to_string_lossy()
                .starts_with("pi-extension-legacy-quarantined-")
        }));
        assert!(destination.is_file());
        assert!(
            root.join("pi-extension-silent-restart-required.json")
                .is_file()
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod cli_action_tests {
    use super::*;

    fn args(v: &[&str]) -> impl Iterator<Item = String> {
        std::iter::once("focusa-daemon".to_string()).chain(v.iter().map(|s| s.to_string()))
    }

    #[test]
    fn help_flag_prints_action_not_startup() {
        assert_eq!(detect_cli_action(args(&["--help"])), Some(CliAction::Help));
        assert_eq!(detect_cli_action(args(&["-h"])), Some(CliAction::Help));
    }

    #[test]
    fn version_flag_recognized() {
        assert_eq!(
            detect_cli_action(args(&["--version"])),
            Some(CliAction::Version)
        );
        assert_eq!(detect_cli_action(args(&["-V"])), Some(CliAction::Version));
    }

    #[test]
    fn unknown_dash_arg_is_rejected_not_started() {
        assert_eq!(
            detect_cli_action(args(&["--vresion"])),
            Some(CliAction::Unknown("--vresion".to_string()))
        );
    }

    #[test]
    fn no_args_or_positionals_start_normally() {
        assert_eq!(detect_cli_action(args(&[])), None);
        assert_eq!(detect_cli_action(args(&["some-positional"])), None);
    }
}
