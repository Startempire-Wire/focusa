use focusa_core::background_jobs::{
    BackgroundJobFailureClass, BackgroundJobStatus, ProcessIdentityStatus, bounded_output_tail,
    process_identity_status, process_start_token,
};
use focusa_core::scoped_state::{AttachmentKey, ScopeKind};
use serde_json::{Value, json};

use crate::api_client::ApiClient;

const FOCUSA_ATTACHMENT_KEY_ENV_V1: &str = "FOCUSA_ATTACHMENT_KEY_V1";
const QUEUED_RECONCILE_GRACE_SECONDS: i64 = 30;

fn parse_published_attachment(raw: Option<&str>) -> anyhow::Result<Option<AttachmentKey>> {
    let Some(raw) = raw.filter(|value| !value.trim().is_empty()) else {
        return Ok(None);
    };
    let attachment: AttachmentKey = serde_json::from_str(raw)
        .map_err(|error| anyhow::anyhow!("invalid {FOCUSA_ATTACHMENT_KEY_ENV_V1}: {error}"))?;
    attachment.validate()?;
    anyhow::ensure!(
        attachment.workstream.root_scope.scope_kind == ScopeKind::Project,
        "{FOCUSA_ATTACHMENT_KEY_ENV_V1} must carry a verified project attachment"
    );
    Ok(Some(attachment))
}

pub(super) fn published_attachment() -> anyhow::Result<Option<AttachmentKey>> {
    parse_published_attachment(std::env::var(FOCUSA_ATTACHMENT_KEY_ENV_V1).ok().as_deref())
}

pub(super) fn response_has_job_status(response: &Value, job_id: &str, expected: &str) -> bool {
    response.get("job").is_some_and(|job| {
        job.get("job_id").and_then(Value::as_str) == Some(job_id)
            && job.get("status").and_then(Value::as_str) == Some(expected)
    })
}

pub(super) fn response_has_job_pid(response: &Value, job_id: &str, expected: u32) -> bool {
    response.get("job").is_some_and(|job| {
        job.get("job_id").and_then(Value::as_str) == Some(job_id)
            && job.get("pid").and_then(Value::as_u64) == Some(u64::from(expected))
    })
}

fn failure_payload(
    failure_class: BackgroundJobFailureClass,
    stage: &str,
    error: &dyn std::fmt::Display,
) -> Value {
    let diagnostic = bounded_output_tail(
        &format!("[{}:{stage}] {error}", failure_class.as_str()),
        4096,
    );
    json!({
        "status": BackgroundJobStatus::Failed.as_str(),
        "failure_class": failure_class,
        "exit_code": failure_class.exit_code(),
        "output_tail": diagnostic,
    })
}

pub(super) async fn settle_failure(
    api: &ApiClient,
    job_id: &str,
    failure_class: BackgroundJobFailureClass,
    stage: &str,
    error: &dyn std::fmt::Display,
) -> anyhow::Result<Value> {
    let result: Value = api
        .post(
            &format!("/v1/background-jobs/{job_id}/complete"),
            &failure_payload(failure_class, stage, error),
        )
        .await?;
    anyhow::ensure!(
        response_has_job_status(&result, job_id, "failed")
            && result["job"]["failure_class"] == failure_class.as_str(),
        "daemon did not durably settle background job {job_id} as {}",
        failure_class.as_str()
    );
    Ok(result)
}

fn queued_job_lapsed(job: &Value, now: chrono::DateTime<chrono::Utc>) -> bool {
    if job.get("status").and_then(Value::as_str) != Some("queued") {
        return false;
    }
    if let Some(pid) = job.get("pid").and_then(Value::as_u64) {
        let token = job.get("process_start_token").and_then(Value::as_str);
        return matches!(
            process_identity_status(pid as u32, token),
            ProcessIdentityStatus::Missing | ProcessIdentityStatus::Mismatch
        );
    }
    job.get("started_at")
        .and_then(Value::as_str)
        .and_then(|value| value.parse::<chrono::DateTime<chrono::Utc>>().ok())
        .is_some_and(|started| (now - started).num_seconds() >= QUEUED_RECONCILE_GRACE_SECONDS)
}

pub(super) async fn reconcile_lapsed_job(
    api: &ApiClient,
    job: &Value,
) -> anyhow::Result<Option<Value>> {
    let Some(job_id) = job.get("job_id").and_then(Value::as_str) else {
        return Ok(None);
    };
    if queued_job_lapsed(job, chrono::Utc::now()) {
        let error = anyhow::anyhow!("creator monitor exited before the job reached running");
        settle_failure(
            api,
            job_id,
            BackgroundJobFailureClass::LaunchFailed,
            "queued_monitor_lost",
            &error,
        )
        .await?;
        return Ok(Some(
            api.get(&format!("/v1/background-jobs/{job_id}")).await?,
        ));
    }
    if job.get("status").and_then(Value::as_str) == Some("running") {
        if let Some(pid) = job.get("pid").and_then(Value::as_u64) {
            let token = job.get("process_start_token").and_then(Value::as_str);
            if matches!(
                process_identity_status(pid as u32, token),
                ProcessIdentityStatus::Missing | ProcessIdentityStatus::Mismatch
            ) {
                let updated: Value = api
                    .post(
                        &format!("/v1/background-jobs/{job_id}"),
                        &json!({ "status": "monitor_lost" }),
                    )
                    .await?;
                anyhow::ensure!(
                    response_has_job_status(&updated, job_id, "monitor_lost"),
                    "daemon did not durably mark background job {job_id} monitor_lost"
                );
                return Ok(Some(
                    api.get(&format!("/v1/background-jobs/{job_id}")).await?,
                ));
            }
        }
    }
    Ok(None)
}

pub(super) async fn bind_detached_monitor(
    api: &ApiClient,
    job_id: &str,
    child: &mut std::process::Child,
) -> anyhow::Result<()> {
    use std::io::Write;

    let monitor_pid = child.id();
    let response = api
        .post(
            &format!("/v1/background-jobs/{job_id}"),
            &json!({
                "pid": monitor_pid,
                "process_start_token": process_start_token(monitor_pid),
            }),
        )
        .await?;
    anyhow::ensure!(
        response_has_job_pid(&response, job_id, monitor_pid),
        "daemon did not bind background job {job_id} to monitor pid {monitor_pid}"
    );
    let mut gate = child
        .stdin
        .take()
        .ok_or_else(|| anyhow::anyhow!("detached monitor registration pipe missing"))?;
    writeln!(gate, "{monitor_pid}")?;
    Ok(())
}

pub(super) fn await_monitor_binding() -> anyhow::Result<()> {
    let mut line = String::new();
    let bytes = std::io::stdin().read_line(&mut line)?;
    anyhow::ensure!(bytes > 0, "detached monitor registration pipe closed");
    anyhow::ensure!(
        line.trim() == std::process::id().to_string(),
        "detached monitor registration pid mismatch"
    );
    Ok(())
}

pub(super) fn terminate_unregistered_child(child: &mut std::process::Child) -> anyhow::Result<()> {
    if child.try_wait()?.is_none() {
        child.kill()?;
        child.wait()?;
    }
    Ok(())
}

#[cfg(unix)]
pub(super) fn configure_detached_monitor(command: &mut std::process::Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(windows)]
pub(super) fn configure_detached_monitor(command: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
}

#[cfg(not(any(unix, windows)))]
pub(super) fn configure_detached_monitor(_command: &mut std::process::Command) {}

#[cfg(unix)]
pub(super) fn build_child(
    command: &[String],
    log_path: &str,
    cwd: Option<&str>,
) -> anyhow::Result<std::process::Child> {
    use std::os::unix::process::CommandExt;
    use std::process::Stdio;
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    let log_clone = log.try_clone()?;
    let (program, rest) = command.split_first().expect("command non-empty");
    let mut child = std::process::Command::new(program);
    child.args(rest);
    if let Some(cwd) = cwd {
        child.current_dir(cwd);
    }
    Ok(child
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_clone))
        .stdin(Stdio::null())
        .process_group(0)
        .spawn()?)
}

#[cfg(not(unix))]
pub(super) fn build_child(
    command: &[String],
    log_path: &str,
    cwd: Option<&str>,
) -> anyhow::Result<std::process::Child> {
    use std::process::Stdio;
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    let log_clone = log.try_clone()?;
    let (program, rest) = command.split_first().expect("command non-empty");
    let mut child = std::process::Command::new(program);
    child.args(rest);
    if let Some(cwd) = cwd {
        child.current_dir(cwd);
    }
    Ok(child
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_clone))
        .stdin(Stdio::null())
        .spawn()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use focusa_core::scoped_state::{ScopeRef, WorkstreamKey};

    fn attachment() -> AttachmentKey {
        let root = std::env::temp_dir().join("focusa-bg-cli-project");
        let scope =
            ScopeRef::project("project:bg", root, "Background Project", "fingerprint:bg").unwrap();
        AttachmentKey::new(
            WorkstreamKey::new(scope, "continuity-bg").unwrap(),
            "pi-42",
            "session-bg",
            "attachment-bg",
        )
        .unwrap()
    }

    #[test]
    fn published_attachment_parser_is_versioned_exact_and_fail_closed() {
        assert_eq!(parse_published_attachment(None).unwrap(), None);
        let expected = attachment();
        let encoded = serde_json::to_string(&expected).unwrap();
        assert_eq!(
            parse_published_attachment(Some(&encoded)).unwrap(),
            Some(expected)
        );
        assert!(parse_published_attachment(Some("{broken")).is_err());
    }

    #[test]
    fn launch_failure_payload_is_typed_and_bounded() {
        let error = "x".repeat(5000);
        let payload = failure_payload(
            BackgroundJobFailureClass::LaunchFailed,
            "command_spawn",
            &error,
        );
        assert_eq!(payload["status"], "failed");
        assert_eq!(payload["failure_class"], "launch_failed");
        assert_eq!(
            payload["exit_code"],
            BackgroundJobFailureClass::LaunchFailed.exit_code()
        );
        assert!(payload["output_tail"].as_str().unwrap().len() <= 4096);
    }

    #[test]
    fn queued_row_without_monitor_reconciles_only_after_grace() {
        let now = chrono::Utc::now();
        let fresh = json!({
            "status": "queued",
            "started_at": now.to_rfc3339(),
        });
        let stale = json!({
            "status": "queued",
            "started_at": (now - chrono::Duration::seconds(31)).to_rfc3339(),
        });
        assert!(!queued_job_lapsed(&fresh, now));
        assert!(queued_job_lapsed(&stale, now));
    }

    #[test]
    fn queued_row_with_current_monitor_remains_live() {
        let job = json!({
            "status": "queued",
            "pid": std::process::id(),
            "started_at": chrono::Utc::now().to_rfc3339(),
        });
        assert!(!queued_job_lapsed(&job, chrono::Utc::now()));
    }

    #[test]
    fn response_pid_binding_requires_exact_job_and_pid() {
        let response = json!({"job": {"job_id": "job-1", "pid": 42}});
        assert!(response_has_job_pid(&response, "job-1", 42));
        assert!(!response_has_job_pid(&response, "job-2", 42));
        assert!(!response_has_job_pid(&response, "job-1", 43));
    }
}
