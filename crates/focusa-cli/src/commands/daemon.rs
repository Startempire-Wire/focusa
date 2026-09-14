//! Daemon control commands — start/stop.

use crate::api_client::ApiClient;
use anyhow::Context;
use focusa_core::daemon_lifecycle::{
    DAEMON_PROCESS_IDENTITY_SCHEMA, DaemonLockRecord, DaemonProcessIdentity, DaemonShutdownRequest,
};
use focusa_core::types::FocusaConfig;
use std::path::{Path, PathBuf};
use std::process::Stdio;

fn running_version_matches(health: &serde_json::Value) -> bool {
    health.get("version").and_then(|v| v.as_str()) == Some(env!("CARGO_PKG_VERSION"))
}

fn health_process_identity(health: &serde_json::Value) -> anyhow::Result<DaemonProcessIdentity> {
    let identity: DaemonProcessIdentity = serde_json::from_value(
        health
            .get("daemon")
            .cloned()
            .context("daemon health is missing exact process identity")?,
    )
    .context("daemon health contains invalid process identity")?;
    anyhow::ensure!(
        identity.schema == DAEMON_PROCESS_IDENTITY_SCHEMA && identity.pid > 0,
        "daemon health process identity is unsupported"
    );
    Ok(identity)
}

fn configured_lock_path() -> PathBuf {
    let data_dir = std::env::var_os("FOCUSA_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(FocusaConfig::default().data_dir));
    let data_dir = if data_dir == Path::new("~") {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or(data_dir)
    } else if let Ok(relative) = data_dir.strip_prefix("~/") {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(relative))
            .unwrap_or(data_dir)
    } else {
        data_dir
    };
    data_dir.join("focusa-daemon.lock")
}

fn bind_port(bind: &str) -> Option<u16> {
    bind.rsplit(':').next()?.parse().ok()
}

fn validate_lock_identity(
    record: &DaemonLockRecord,
    identity: &DaemonProcessIdentity,
    base_url: &str,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        record.pid == identity.pid && record.start_token == identity.start_token,
        "daemon health and lock process identities do not match"
    );
    let target_port = reqwest::Url::parse(base_url)
        .ok()
        .and_then(|url| url.port_or_known_default());
    anyhow::ensure!(
        bind_port(&record.bind).is_some() && bind_port(&record.bind) == target_port,
        "daemon lock port does not match the requested endpoint"
    );
    Ok(())
}

fn shutdown_bearer(identity: &DaemonProcessIdentity, base_url: &str) -> anyhow::Result<String> {
    if let Ok(admin_token) = std::env::var("FOCUSA_AUTH_TOKEN")
        && !admin_token.is_empty()
    {
        return Ok(admin_token);
    }

    let expected_path = configured_lock_path();
    anyhow::ensure!(
        Path::new(&identity.lock_path) == expected_path,
        "daemon-advertised lock path does not match configured local data directory"
    );
    let content = std::fs::read_to_string(&expected_path)
        .context("read exact daemon lock for shutdown authorization")?;
    let record = DaemonLockRecord::parse(&content)
        .context("parse exact daemon lock for shutdown authorization")?;
    validate_lock_identity(&record, identity, base_url)?;
    Ok(record.shutdown_token)
}

async fn wait_until_identity_stopped(client: &ApiClient, expected: &DaemonProcessIdentity) -> bool {
    for _ in 0..50 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        match client.get("/v1/health").await {
            Err(_) => return true,
            Ok(health) => match health_process_identity(&health) {
                Ok(running)
                    if running.pid == expected.pid
                        && running.start_token == expected.start_token => {}
                _ => return true,
            },
        }
    }
    false
}

/// Start the Focusa daemon.
/// Returns true when daemon was started in this call, false when already running.
pub async fn start() -> anyhow::Result<bool> {
    let client = ApiClient::new();

    // Check if already running (idempotent start). If the daemon is stale,
    // restart it before commands such as `focusa pair` probe current routes/UX.
    if let Ok(health) = client.get("/v1/health").await {
        if running_version_matches(&health) {
            return Ok(false);
        }
        eprintln!(
            "Focusa daemon version mismatch: running={} cli={}; repairing daemon before continuing.",
            health
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown"),
            env!("CARGO_PKG_VERSION")
        );
        stop()
            .await
            .context("exact stale-daemon shutdown failed; refusing broad process repair")?;
        if client.get("/v1/health").await.is_ok() {
            anyhow::bail!(
                "daemon endpoint is still occupied after exact shutdown; refusing to signal by process name"
            );
        }
    }

    // Find and start daemon.
    let daemon_path = find_daemon_binary()?;
    let focusa_url =
        std::env::var("FOCUSA_API_URL").unwrap_or_else(|_| "http://127.0.0.1:8787".into());
    let bind = focusa_url
        .strip_prefix("http://")
        .unwrap_or("127.0.0.1:8787");

    let mut cmd = std::process::Command::new(&daemon_path);
    cmd.env("FOCUSA_BIND", bind);

    // Pass through API keys.
    for key in [
        "ANTHROPIC_API_KEY",
        "OPENAI_API_KEY",
        "FOCUSA_MESSAGES_API_KEY",
        "FOCUSA_ANTHROPIC_KEY", // backward compat
        "FOCUSA_API_KEY",
    ] {
        if let Ok(val) = std::env::var(key) {
            cmd.env(key, val);
        }
    }

    // Redirect output to avoid cluttering terminal.
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    cmd.spawn()?;

    // Wait for daemon to be ready (max 5s).
    for _ in 0..50 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        if let Ok(health) = client.get("/v1/health").await
            && running_version_matches(&health)
        {
            return Ok(true);
        }
    }

    anyhow::bail!("Daemon started but health check failed")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopOutcome {
    Stopped,
    AlreadyStopped,
}

/// Stop the Focusa daemon.
pub async fn stop() -> anyhow::Result<StopOutcome> {
    let client = ApiClient::new();

    let health = match client.get("/v1/health").await {
        Ok(health) => health,
        Err(_) => return Ok(StopOutcome::AlreadyStopped),
    };
    let identity = health_process_identity(&health)?;
    let bearer = shutdown_bearer(&identity, client.base_url())?;
    let request = DaemonShutdownRequest::new(identity.pid, identity.start_token.clone());
    let body = serde_json::to_value(&request)?;
    let authorization = format!("Bearer {bearer}");
    let response = client
        .post_with_headers(
            "/v1/shutdown",
            &body,
            &[("authorization", authorization.as_str())],
        )
        .await
        .context("authenticated exact-daemon shutdown request failed")?;
    anyhow::ensure!(
        response.get("status").and_then(serde_json::Value::as_str) == Some("accepted")
            && response.get("pid").and_then(serde_json::Value::as_u64) == Some(identity.pid as u64)
            && response
                .get("start_token")
                .and_then(serde_json::Value::as_str)
                == Some(identity.start_token.as_str()),
        "daemon returned an invalid shutdown acceptance receipt"
    );

    if wait_until_identity_stopped(&client, &identity).await {
        return Ok(StopOutcome::Stopped);
    }

    anyhow::bail!("exact Focusa daemon instance still responds after shutdown timeout")
}

/// Find the daemon binary.
fn find_daemon_binary() -> anyhow::Result<std::path::PathBuf> {
    // Prefer the daemon next to this CLI binary so release/dev installs stay paired.
    if let Ok(exe) = std::env::current_exe() {
        let dir = exe.parent().unwrap_or(std::path::Path::new("."));
        let candidate = dir.join("focusa-daemon");
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    // Check common dev locations before PATH to avoid picking stale system daemons.
    for path in [
        "./target/release/focusa-daemon",
        "./target/debug/focusa-daemon",
        "/tmp/focusa-target/release/focusa-daemon",
        "/tmp/focusa-target/debug/focusa-daemon",
        "/usr/local/bin/focusa-daemon",
    ] {
        let p = std::path::PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    // Last resort: PATH.
    if let Ok(path) = which::which("focusa-daemon") {
        return Ok(path);
    }

    anyhow::bail!("Could not find focusa-daemon binary. Install it or add to PATH.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use focusa_core::daemon_lifecycle::DAEMON_PROCESS_IDENTITY_SCHEMA;

    fn identity() -> DaemonProcessIdentity {
        DaemonProcessIdentity::new(4242, "start-token", "/tmp/focusa-daemon.lock")
    }

    fn lock() -> DaemonLockRecord {
        DaemonLockRecord {
            pid: 4242,
            bind: "127.0.0.1:18787".into(),
            started_at: "2026-08-31T00:00:00Z".into(),
            start_token: "start-token".into(),
            shutdown_token: "shutdown-token".into(),
        }
    }

    #[test]
    fn health_requires_versioned_exact_process_identity() {
        let health = serde_json::json!({"daemon": identity()});
        assert_eq!(health_process_identity(&health).unwrap(), identity());
        let legacy = serde_json::json!({"ok": true, "version": "0.9.177"});
        assert!(health_process_identity(&legacy).is_err());
        let wrong_schema = serde_json::json!({
            "daemon": {
                "schema": "focusa.daemon_process_identity.v0",
                "pid": 4242,
                "start_token": "start-token",
                "lock_path": "/tmp/focusa-daemon.lock"
            }
        });
        assert!(health_process_identity(&wrong_schema).is_err());
        assert_eq!(identity().schema, DAEMON_PROCESS_IDENTITY_SCHEMA);
    }

    #[test]
    fn lock_validation_binds_pid_start_token_and_target_port() {
        assert!(validate_lock_identity(&lock(), &identity(), "http://127.0.0.1:18787").is_ok());
        let mut foreign_pid = lock();
        foreign_pid.pid = 5252;
        assert!(
            validate_lock_identity(&foreign_pid, &identity(), "http://127.0.0.1:18787").is_err()
        );
        let mut foreign_start = lock();
        foreign_start.start_token = "other-start".into();
        assert!(
            validate_lock_identity(&foreign_start, &identity(), "http://127.0.0.1:18787").is_err()
        );
        assert!(validate_lock_identity(&lock(), &identity(), "http://127.0.0.1:28787").is_err());
    }
}
