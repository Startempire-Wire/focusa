//! Installed CallGraph capability probe used inside the system-service rollback boundary.

use anyhow::{Context, Result, anyhow, bail};
use serde_json::Value;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

pub(super) fn verify(health_url: &str) -> Result<()> {
    let validator_url = if let Some(base) = health_url.strip_suffix("/v1/health") {
        format!("{base}/v1/callgraphs/validate")
    } else {
        format!(
            "{}/v1/callgraphs/validate",
            health_url.trim_end_matches('/')
        )
    };
    let graph: Value = serde_json::from_str(include_str!("install_callgraph_probe.json"))
        .context("decode embedded canonical install CallGraph probe")?;
    let auth_header = write_auth_header()?;
    let mut command = Command::new("curl");
    command.args([
        "-fsS",
        "--max-time",
        "5",
        "-H",
        "Content-Type: application/json",
        "--data-binary",
        "@-",
        &validator_url,
    ]);
    if let Some(header) = &auth_header {
        command.args(["-H", &format!("@{}", header.path.display())]);
    }
    let child_result = command
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("start installed CallGraph validator probe");
    let mut child = child_result?;
    child
        .stdin
        .as_mut()
        .ok_or_else(|| anyhow!("open CallGraph validator request pipe"))?
        .write_all(&serde_json::to_vec(&graph)?)
        .context("write CallGraph validator request")?;
    let output = child
        .wait_with_output()
        .context("wait for installed CallGraph validator probe")?;
    if !output.status.success() {
        bail!(
            "installed CallGraph validator verification failed: exit={} ({})",
            output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&output.stderr)
                .trim()
                .chars()
                .take(240)
                .collect::<String>()
        );
    }
    let payload: Value = serde_json::from_slice(&output.stdout)
        .context("installed CallGraph validator returned non-JSON output")?;
    if payload.get("canonical").and_then(Value::as_bool) != Some(true)
        || payload.get("valid").and_then(Value::as_bool) != Some(true)
        || payload.get("status").and_then(Value::as_str) != Some("valid")
        || payload
            .get("issues")
            .and_then(Value::as_array)
            .is_none_or(|issues| !issues.is_empty())
    {
        bail!("installed CallGraph validator returned a non-canonical or invalid envelope");
    }
    Ok(())
}

struct AuthHeader {
    path: PathBuf,
}

impl Drop for AuthHeader {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_file(&self.path) {
            eprintln!(
                "warning: remove private CallGraph authorization header {}: {error}",
                self.path.display()
            );
        }
    }
}

fn write_auth_header() -> Result<Option<AuthHeader>> {
    use std::os::unix::fs::OpenOptionsExt;

    let Some(token) = std::env::var_os("FOCUSA_AGENT_TOKEN") else {
        return Ok(None);
    };
    if token.is_empty() {
        return Ok(None);
    }
    if token
        .as_encoded_bytes()
        .iter()
        .any(|byte| matches!(byte, b'\r' | b'\n'))
    {
        bail!("FOCUSA_AGENT_TOKEN contains an invalid header delimiter");
    }
    let path = PathBuf::from(format!(
        "/run/focusa-callgraph-auth-{}.headers",
        std::process::id()
    ));
    let write_result = (|| -> Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(&path)
            .context("create private CallGraph authorization header")?;
        file.write_all(b"Authorization: Bearer ")?;
        file.write_all(token.as_encoded_bytes())?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        Ok(())
    })();
    if let Err(error) = write_result {
        let _ = std::fs::remove_file(&path);
        return Err(error);
    }
    Ok(Some(AuthHeader { path }))
}
