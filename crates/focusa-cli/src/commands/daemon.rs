//! Daemon control commands — start/stop.

use crate::api_client::ApiClient;
use std::process::Stdio;

fn running_version_matches(health: &serde_json::Value) -> bool {
    health.get("version").and_then(|v| v.as_str()) == Some(env!("CARGO_PKG_VERSION"))
}

async fn wait_until_stopped(client: &ApiClient) -> bool {
    for _ in 0..50 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        if client.get("/v1/health").await.is_err() {
            return true;
        }
    }
    false
}

fn kill_daemon_processes() {
    let _ = std::process::Command::new("pkill")
        .arg("-f")
        .arg("focusa-daemon")
        .status();
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
        let _ = stop().await;
        if client.get("/v1/health").await.is_ok() {
            kill_daemon_processes();
            let _ = wait_until_stopped(&client).await;
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

/// Stop the Focusa daemon.
pub async fn stop() -> anyhow::Result<()> {
    let client = ApiClient::new();

    // Check if running.
    if client.get("/v1/health").await.is_err() {
        anyhow::bail!("Focusa daemon is not running");
    }

    // Send shutdown request (if endpoint exists).
    // For now, use pkill as fallback.
    if client
        .post("/v1/shutdown", &serde_json::json!({}))
        .await
        .is_err()
    {
        // Fallback: kill by name.
        kill_daemon_processes();
    }

    // Wait for daemon to stop (max 5s).
    for _ in 0..50 {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        if client.get("/v1/health").await.is_err() {
            return Ok(());
        }
    }

    anyhow::bail!("Daemon did not stop within timeout")
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
