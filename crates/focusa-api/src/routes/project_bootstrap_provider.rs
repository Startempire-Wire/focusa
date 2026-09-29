//! Bounded local provider invocation. This never grants cross-owner execution.
use super::project_bootstrap_support::reject;
use axum::{Json, http::StatusCode};
use serde_json::{Value, json};
use std::{
    io::Read,
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

const PROVIDER_TIMEOUT: Duration = Duration::from_secs(120);
pub(super) const BOOTSTRAP_PROVIDER_BUDGET: Duration = Duration::from_secs(300);
const OUTPUT_PARSE_LIMIT: usize = 64 * 1024;
const STDERR_TAIL_LIMIT: usize = 8 * 1024;

pub(super) fn provider_rejection(
    fallback: &'static str,
    error: String,
) -> (StatusCode, Json<Value>) {
    let (status, code) = match error.split(':').next().unwrap_or("") {
        "provider_timeout" => (StatusCode::GATEWAY_TIMEOUT, "provider_timeout"),
        "provider_output_limit_exceeded" => {
            (StatusCode::BAD_GATEWAY, "provider_output_limit_exceeded")
        }
        "provider_output_stream_unclosed" => {
            (StatusCode::BAD_GATEWAY, "provider_output_stream_unclosed")
        }
        "provider_failed" => (StatusCode::SERVICE_UNAVAILABLE, "provider_failed"),
        "provider_spawn_failed" => (StatusCode::SERVICE_UNAVAILABLE, "provider_spawn_failed"),
        "provider_wait_failed"
        | "provider_reap_failed"
        | "provider_output_read_failed"
        | "provider_stdout_unavailable"
        | "provider_stderr_unavailable" => (StatusCode::BAD_GATEWAY, "provider_io_failed"),
        _ => (StatusCode::SERVICE_UNAVAILABLE, fallback),
    };
    reject(status, code, error)
}

/// A request-wide deadline prevents N independent Beads calls from multiplying
/// the transport wait without a typed terminal outcome.
pub(super) fn run_before_deadline(
    root: &Path,
    binary: &str,
    args: &[&str],
    deadline: Instant,
) -> Result<Value, String> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return Err("provider_timeout: total bootstrap provider deadline exceeded".into());
    }
    run_with_timeout(root, binary, args, remaining.min(PROVIDER_TIMEOUT))
}

type Captured = Result<(Vec<u8>, bool), String>;

/// Drain even after reaching the cap, so a verbose child cannot deadlock on a full pipe.
fn drain<R: Read + Send + 'static>(
    mut source: R,
    limit: usize,
    tail: bool,
) -> mpsc::Receiver<Captured> {
    let (sender, receiver) = mpsc::sync_channel(1);
    thread::spawn(move || {
        let mut captured = Vec::new();
        let mut overflow = false;
        let mut chunk = [0_u8; 4096];
        loop {
            match source.read(&mut chunk) {
                Ok(0) => break,
                Ok(count) => {
                    if captured.len().saturating_add(count) > limit {
                        overflow = true;
                    }
                    if tail {
                        captured.extend_from_slice(&chunk[..count]);
                        if captured.len() > limit {
                            captured.drain(..captured.len() - limit);
                        }
                    } else if captured.len() < limit {
                        captured.extend_from_slice(&chunk[..count.min(limit - captured.len())]);
                    }
                }
                Err(error) => {
                    let _ = sender.send(Err(format!("provider_output_read_failed: {error}")));
                    return;
                }
            }
        }
        let _ = sender.send(Ok((captured, overflow)));
    });
    receiver
}

#[cfg(unix)]
fn kill_provider_group(pid: u32) {
    let _ = nix::sys::signal::killpg(
        nix::unistd::Pid::from_raw(pid as i32),
        nix::sys::signal::Signal::SIGKILL,
    );
}

pub(super) fn run(root: &Path, binary: &str, args: &[&str]) -> Result<Value, String> {
    run_with_timeout(root, binary, args, PROVIDER_TIMEOUT)
}

fn run_with_timeout(
    root: &Path,
    binary: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<Value, String> {
    let mut command = Command::new(binary);
    command
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command
        .spawn()
        .map_err(|e| format!("provider_spawn_failed: {e}"))?;
    let stdout = drain(
        child.stdout.take().ok_or("provider_stdout_unavailable")?,
        OUTPUT_PARSE_LIMIT,
        false,
    );
    let stderr = drain(
        child.stderr.take().ok_or("provider_stderr_unavailable")?,
        STDERR_TAIL_LIMIT,
        true,
    );
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if start.elapsed() < timeout => thread::sleep(Duration::from_millis(20)),
            Ok(None) => {
                #[cfg(unix)]
                kill_provider_group(child.id());
                let _ = child.kill();
                child
                    .wait()
                    .map_err(|e| format!("provider_reap_failed: {e}"))?;
                let tail = stderr
                    .recv_timeout(Duration::from_millis(500))
                    .ok()
                    .and_then(Result::ok)
                    .map(|(bytes, _)| String::from_utf8_lossy(&bytes).trim().to_string())
                    .unwrap_or_default();
                return Err(format!(
                    "provider_timeout: {} exceeded {} seconds; stderr_tail={tail}",
                    args.first().copied().unwrap_or("unknown"),
                    timeout.as_secs()
                ));
            }
            Err(e) => return Err(format!("provider_wait_failed: {e}")),
        }
    };
    // If an exited provider left descendants holding output pipes, kill only its
    // isolated group, then return an uncertain error rather than hang indefinitely.
    let stdout_result = stdout.recv_timeout(Duration::from_secs(2));
    let stderr_result = stderr.recv_timeout(Duration::from_secs(2));
    if stdout_result.is_err() || stderr_result.is_err() {
        #[cfg(unix)]
        kill_provider_group(child.id());
        return Err(
            "provider_output_stream_unclosed: child exited but descendants retained output pipes"
                .into(),
        );
    }
    let (bytes, oversized) = stdout_result.map_err(|e| e.to_string())??;
    let (stderr_bytes, _) = stderr_result.map_err(|e| e.to_string())??;
    let stderr_tail = String::from_utf8_lossy(&stderr_bytes).trim().to_string();
    if !status.success() {
        return Err(format!(
            "provider_failed: exit={status}; stderr_tail={stderr_tail}"
        ));
    }
    if oversized {
        return Err("provider_output_limit_exceeded: stdout exceeds 64 KiB".into());
    }
    serde_json::from_slice(&bytes)
        .or_else(|_| {
            Ok::<Value, serde_json::Error>(
                json!({"status":"ok","stdout":String::from_utf8_lossy(&bytes).trim()}),
            )
        })
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_terminal_failures_remain_distinct_from_generic_health() {
        for (input, expected_status, expected_code) in [
            (
                "provider_timeout: expired",
                StatusCode::GATEWAY_TIMEOUT,
                "provider_timeout",
            ),
            (
                "provider_output_limit_exceeded: huge",
                StatusCode::BAD_GATEWAY,
                "provider_output_limit_exceeded",
            ),
            (
                "provider_failed: exit=1",
                StatusCode::SERVICE_UNAVAILABLE,
                "provider_failed",
            ),
            (
                "provider_spawn_failed: missing",
                StatusCode::SERVICE_UNAVAILABLE,
                "provider_spawn_failed",
            ),
            (
                "not installed",
                StatusCode::SERVICE_UNAVAILABLE,
                "task_provider_unhealthy",
            ),
        ] {
            let (status, Json(body)) = provider_rejection("task_provider_unhealthy", input.into());
            assert_eq!(status, expected_status);
            assert_eq!(body["failure_class"], expected_code);
        }
    }

    #[test]
    fn expired_request_budget_fails_before_spawning_provider() {
        let result = run_before_deadline(
            Path::new("/"),
            "/bin/false",
            &[],
            Instant::now() - Duration::from_secs(1),
        );
        assert!(result.unwrap_err().starts_with("provider_timeout:"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn provider_timeout_reaps_process_group() {
        let temp = tempfile::tempdir().unwrap();
        let script = temp.path().join("provider.sh");
        std::fs::write(
            &script,
            "#!/bin/sh\nsleep 30 &\necho $! > child.pid\nwait\n",
        )
        .unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
        let result = run_with_timeout(
            temp.path(),
            script.to_str().unwrap(),
            &[],
            Duration::from_millis(100),
        );
        let error = result.unwrap_err();
        assert!(
            error.starts_with("provider_timeout:"),
            "unexpected: {error}"
        );
        let pid: i32 = std::fs::read_to_string(temp.path().join("child.pid"))
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).unwrap_or_default();
            if stat.is_empty() || stat.split_whitespace().nth(2) == Some("Z") {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "descendant survived timeout: {stat}"
            );
            thread::sleep(Duration::from_millis(20));
        }
    }
    #[test]
    fn provider_failure_success_and_output_limit_are_bounded() {
        let temp = tempfile::tempdir().unwrap();
        let value = run_with_timeout(
            temp.path(),
            "/bin/sh",
            &["-c", "printf '{\"status\":\"ok\"}'"],
            Duration::from_secs(2),
        )
        .unwrap();
        assert_eq!(value["status"], "ok");
        let error = run_with_timeout(
            temp.path(),
            "/bin/sh",
            &["-c", "echo invalid >&2; exit 7"],
            Duration::from_secs(2),
        )
        .unwrap_err();
        assert!(error.contains("provider_failed:") && error.contains("invalid"));
        let oversized = run_with_timeout(
            temp.path(),
            "/bin/sh",
            &["-c", "head -c 70000 /dev/zero"],
            Duration::from_secs(2),
        )
        .unwrap_err();
        assert!(oversized.starts_with("provider_output_limit_exceeded:"));
    }
}
