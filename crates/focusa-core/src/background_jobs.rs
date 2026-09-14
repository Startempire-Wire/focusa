//! Background job completion events (#311-family): first-class background
//! execution records with a durable completion boundary.
//!
//! `focusa bg run` creates a job row, executes the command as a detached
//! child, and reports the completion back through the daemon — which
//! records it durably and broadcasts it over SSE. Consumers (Pi extension
//! uiCtx.notify, `focusa bg wait`, TUI) all read the same envelope.
//! Nothing here is a shell wrapper: the CLI monitor is the lifecycle owner.

use std::io::{Read, Seek, SeekFrom};

use crate::scoped_state::AttachmentKey;
use serde::{Deserialize, Serialize};

pub const BACKGROUND_JOB_SCHEMA_V1: &str = "focusa.background_job.v1";
pub const BACKGROUND_JOB_SCHEMA_V2: &str = "focusa.background_job.v2";
pub const BACKGROUND_JOB_SCHEMA: &str = "focusa.background_job.v3";
pub const BACKGROUND_JOB_DISPATCH_SCHEMA: &str = "focusa.background_job_dispatch.v1";
pub const BACKGROUND_JOB_COMPLETION_EVENT: &str = "background_job_completion";
/// docs/165 v2: broadcast when a job transitions queued → running so
/// surfaces see dispatch latency, not just completion.
pub const BACKGROUND_JOB_STARTED_EVENT: &str = "background_job_started";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundJobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    MonitorLost,
}

impl BackgroundJobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            BackgroundJobStatus::Queued => "queued",
            BackgroundJobStatus::Running => "running",
            BackgroundJobStatus::Completed => "completed",
            BackgroundJobStatus::Failed => "failed",
            BackgroundJobStatus::MonitorLost => "monitor_lost",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "running" => BackgroundJobStatus::Running,
            "completed" => BackgroundJobStatus::Completed,
            "failed" => BackgroundJobStatus::Failed,
            "monitor_lost" => BackgroundJobStatus::MonitorLost,
            _ => BackgroundJobStatus::Queued,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundJobFailureClass {
    LaunchFailed,
    MonitorFailed,
}

impl BackgroundJobFailureClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LaunchFailed => "launch_failed",
            Self::MonitorFailed => "monitor_failed",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "launch_failed" => Some(Self::LaunchFailed),
            "monitor_failed" => Some(Self::MonitorFailed),
            _ => None,
        }
    }

    pub const fn exit_code(self) -> i32 {
        match self {
            Self::LaunchFailed => 126,
            Self::MonitorFailed => 125,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessIdentityStatus {
    Match,
    Missing,
    Mismatch,
    Unknown,
}

#[cfg(target_os = "linux")]
pub fn process_start_token(pid: u32) -> Option<String> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let mut fields = stat.get(stat.rfind(") ")? + 2..)?.split_whitespace();
    // /proc/<pid>/stat field 22 is process start time. The slice begins
    // at field 3, so start time is the twentieth value in this iterator.
    fields.nth(19).map(str::to_string)
}

#[cfg(not(target_os = "linux"))]
pub fn process_start_token(_pid: u32) -> Option<String> {
    None
}

pub fn current_process_start_token() -> Option<String> {
    process_start_token(std::process::id())
}

#[cfg(target_os = "linux")]
fn pid_alive(pid: u32) -> bool {
    std::path::Path::new(&format!("/proc/{pid}")).exists()
}

#[cfg(all(unix, not(target_os = "linux")))]
fn pid_alive(pid: u32) -> bool {
    unsafe extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }
    // SAFETY: signal zero only checks process existence/permission.
    let result = unsafe { kill(pid as i32, 0) };
    result == 0 || std::io::Error::last_os_error().raw_os_error() == Some(1)
}

#[cfg(windows)]
fn pid_alive(pid: u32) -> bool {
    use std::ffi::c_void;

    const SYNCHRONIZE: u32 = 0x0010_0000;
    const WAIT_OBJECT_0: u32 = 0;
    const WAIT_TIMEOUT: u32 = 0x0000_0102;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn OpenProcess(access: u32, inherit_handle: i32, process_id: u32) -> *mut c_void;
        fn WaitForSingleObject(handle: *mut c_void, milliseconds: u32) -> u32;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }

    // SAFETY: the returned handle is checked and closed exactly once.
    let handle = unsafe { OpenProcess(SYNCHRONIZE, 0, pid) };
    if handle.is_null() {
        return std::io::Error::last_os_error().raw_os_error() != Some(87);
    }
    // SAFETY: handle remains valid until CloseHandle below.
    let result = unsafe { WaitForSingleObject(handle, 0) };
    unsafe {
        CloseHandle(handle);
    }
    match result {
        WAIT_OBJECT_0 => false,
        WAIT_TIMEOUT => true,
        _ => true,
    }
}

#[cfg(not(any(unix, windows)))]
fn pid_alive(_pid: u32) -> bool {
    true
}

pub fn process_identity_status(
    pid: u32,
    expected_start_token: Option<&str>,
) -> ProcessIdentityStatus {
    if !pid_alive(pid) {
        return ProcessIdentityStatus::Missing;
    }
    let Some(expected) = expected_start_token.filter(|value| !value.is_empty()) else {
        return ProcessIdentityStatus::Unknown;
    };
    match process_start_token(pid) {
        Some(actual) if actual == expected => ProcessIdentityStatus::Match,
        Some(_) => ProcessIdentityStatus::Mismatch,
        None => ProcessIdentityStatus::Unknown,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackgroundJobRecord {
    pub schema: String,
    pub job_id: String,
    pub name: String,
    pub command: String,
    pub cwd: String,
    /// Exact producer attachment. Legacy/manual jobs may be unscoped, but Pi
    /// consumers must treat those records as inert rather than infer scope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachment: Option<AttachmentKey>,
    pub status: BackgroundJobStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_class: Option<BackgroundJobFailureClass>,
    pub exit_code: Option<i32>,
    pub pid: Option<u32>,
    /// OS process-start identity paired with `pid` where the platform can
    /// provide one. Older records remain valid with this field absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_start_token: Option<String>,
    pub log_path: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    /// Durable bounded output carried by the completing monitor. This avoids
    /// relying on a shared `/tmp` namespace between the CLI and daemon.
    #[serde(default)]
    pub output_tail: String,
}

/// The completion envelope every consumer reads (SSE + wait + status).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackgroundJobCompletionEvent {
    pub schema: String,
    pub event_type: String,
    pub job_id: String,
    pub name: String,
    pub command: String,
    pub cwd: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachment: Option<AttachmentKey>,
    pub status: BackgroundJobStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_class: Option<BackgroundJobFailureClass>,
    pub exit_code: Option<i32>,
    pub log_path: String,
    pub started_at: String,
    pub completed_at: String,
    /// Bounded tail of the job's output — the agent's front terminal
    /// displays this on completion (never the whole log).
    #[serde(default)]
    pub output_tail: String,
}

/// docs/165 v2 §2 — started envelope (dispatch visibility).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackgroundJobStartedEvent {
    pub schema: String,
    pub event_type: String,
    pub job_id: String,
    pub name: String,
    pub command: String,
    pub cwd: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachment: Option<AttachmentKey>,
    pub pid: Option<u32>,
    pub log_path: String,
    pub started_at: String,
}

impl BackgroundJobStartedEvent {
    pub fn from_record(record: &BackgroundJobRecord) -> Self {
        Self {
            schema: "focusa.stream_event.v1".to_string(),
            event_type: BACKGROUND_JOB_STARTED_EVENT.to_string(),
            job_id: record.job_id.clone(),
            name: record.name.clone(),
            command: record.command.clone(),
            cwd: record.cwd.clone(),
            attachment: record.attachment.clone(),
            pid: record.pid,
            log_path: record.log_path.clone(),
            started_at: record.started_at.clone(),
        }
    }
}

/// Read the bounded tail of a job log (last N bytes, line-aligned) without
/// loading an unbounded job log into the monitor or daemon.
pub fn bounded_log_tail(log_path: &str, max_bytes: usize) -> String {
    if max_bytes == 0 {
        return String::new();
    }
    let Ok(mut file) = std::fs::File::open(log_path) else {
        return String::new();
    };
    let Ok(length) = file.metadata().map(|metadata| metadata.len()) else {
        return String::new();
    };
    let wanted = max_bytes.saturating_add(4).min(i64::MAX as usize) as u64;
    let read_len = length.min(wanted);
    if file
        .seek(SeekFrom::Start(length.saturating_sub(read_len)))
        .is_err()
    {
        return String::new();
    }
    let mut bytes = Vec::with_capacity(read_len as usize);
    if file.take(read_len).read_to_end(&mut bytes).is_err() {
        return String::new();
    }
    bounded_output_tail(&String::from_utf8_lossy(&bytes), max_bytes)
}

/// Bound monitor-provided output without splitting a UTF-8 code point. Align
/// truncated output to the next line when possible.
pub fn bounded_output_tail(output: &str, max_bytes: usize) -> String {
    if output.len() <= max_bytes {
        return output.to_string();
    }
    let mut start = output.len().saturating_sub(max_bytes);
    while start < output.len() && !output.is_char_boundary(start) {
        start += 1;
    }
    let tail = &output[start..];
    tail.find('\n')
        .map_or(tail, |index| &tail[index + 1..])
        .to_string()
}

pub fn resolved_background_job_output_tail(record: &BackgroundJobRecord) -> String {
    if !record.output_tail.is_empty() {
        return bounded_output_tail(&record.output_tail, 4096);
    }
    let direct = bounded_log_tail(&record.log_path, 4096);
    if !direct.is_empty() {
        return direct;
    }
    #[cfg(target_os = "linux")]
    if let Some(pid) = record.pid {
        let monitor_path = format!(
            "/proc/{pid}/root/{}",
            record.log_path.trim_start_matches('/')
        );
        return bounded_log_tail(&monitor_path, 4096);
    }
    String::new()
}

impl BackgroundJobCompletionEvent {
    pub fn from_record(record: &BackgroundJobRecord) -> Self {
        Self {
            schema: "focusa.stream_event.v1".to_string(),
            event_type: BACKGROUND_JOB_COMPLETION_EVENT.to_string(),
            job_id: record.job_id.clone(),
            name: record.name.clone(),
            command: record.command.clone(),
            cwd: record.cwd.clone(),
            attachment: record.attachment.clone(),
            status: record.status,
            failure_class: record.failure_class,
            exit_code: record.exit_code,
            log_path: record.log_path.clone(),
            started_at: record.started_at.clone(),
            completed_at: record
                .completed_at
                .clone()
                .unwrap_or_else(|| record.started_at.clone()),
            output_tail: resolved_background_job_output_tail(record),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoped_state::{ScopeRef, WorkstreamKey};

    fn attachment() -> AttachmentKey {
        let root = std::env::temp_dir().join("focusa-bg-event-project");
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
    fn completion_envelope_carries_every_consumer_field() {
        let record = BackgroundJobRecord {
            schema: BACKGROUND_JOB_SCHEMA.to_string(),
            job_id: "j1".to_string(),
            name: "gate".to_string(),
            command: "cargo test".to_string(),
            cwd: "/root/proj".to_string(),
            attachment: Some(attachment()),
            status: BackgroundJobStatus::Completed,
            failure_class: None,
            exit_code: Some(0),
            pid: Some(42),
            process_start_token: None,
            log_path: "/tmp/j1.log".to_string(),
            started_at: "t0".to_string(),
            completed_at: Some("t1".to_string()),
            output_tail: "typecheck failed".to_string(),
        };
        let envelope = BackgroundJobCompletionEvent::from_record(&record);
        assert_eq!(envelope.event_type, BACKGROUND_JOB_COMPLETION_EVENT);
        assert_eq!(envelope.job_id, "j1");
        assert_eq!(envelope.status, BackgroundJobStatus::Completed);
        let value = serde_json::to_value(&envelope).unwrap();
        assert_eq!(value["event_type"], "background_job_completion");
        assert_eq!(value["failure_class"], serde_json::Value::Null);
        assert_eq!(value["output_tail"], "typecheck failed");
        assert_eq!(value["attachment"]["session_id"], "session-bg");
    }

    #[test]
    fn monitor_output_tail_is_utf8_safe_and_line_aligned() {
        let output = format!("old line\n{}new line", "é".repeat(3000));
        let tail = bounded_output_tail(&output, 4096);
        assert!(tail.len() <= 4096);
        assert!(tail.ends_with("new line"));
    }

    #[test]
    fn bounded_log_tail_does_not_return_an_unbounded_prefix() {
        let path = std::env::temp_dir().join(format!(
            "focusa-bg-tail-{}-{}.log",
            std::process::id(),
            uuid::Uuid::now_v7()
        ));
        let mut output = "discarded-prefix\n".repeat(100_000);
        output.push_str("final diagnostic line");
        std::fs::write(&path, output).unwrap();

        let tail = bounded_log_tail(path.to_str().unwrap(), 128);
        assert!(tail.len() <= 128);
        assert!(tail.ends_with("final diagnostic line"));
        assert!(tail == "final diagnostic line" || tail.starts_with("discarded-prefix\n"));
        assert!(tail.matches("discarded-prefix").count() < 100_000);

        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn durable_tail_outranks_an_inaccessible_log_path() {
        let mut record = BackgroundJobRecord {
            schema: BACKGROUND_JOB_SCHEMA.to_string(),
            job_id: "j1".into(),
            name: "gate".into(),
            command: "false".into(),
            cwd: ".".into(),
            attachment: None,
            status: BackgroundJobStatus::Failed,
            failure_class: Some(BackgroundJobFailureClass::MonitorFailed),
            exit_code: Some(1),
            pid: None,
            process_start_token: None,
            log_path: "/not-visible-across-private-tmp/job.log".into(),
            started_at: "t0".into(),
            completed_at: Some("t1".into()),
            output_tail: "exact compiler failure".into(),
        };
        assert_eq!(
            BackgroundJobCompletionEvent::from_record(&record).output_tail,
            "exact compiler failure"
        );
        record.output_tail.clear();
        assert!(
            BackgroundJobCompletionEvent::from_record(&record)
                .output_tail
                .is_empty()
        );
    }

    #[test]
    fn failure_class_roundtrip_is_typed_and_total() {
        for failure_class in [
            BackgroundJobFailureClass::LaunchFailed,
            BackgroundJobFailureClass::MonitorFailed,
        ] {
            assert_eq!(
                BackgroundJobFailureClass::parse(failure_class.as_str()),
                Some(failure_class)
            );
            assert_eq!(
                serde_json::to_value(failure_class).unwrap(),
                failure_class.as_str()
            );
            assert!(failure_class.exit_code() > 0);
        }
        assert_eq!(BackgroundJobFailureClass::parse("unknown"), None);
    }

    #[test]
    fn process_identity_uses_pid_and_start_token_when_available() {
        let pid = std::process::id();
        let token = current_process_start_token();
        if let Some(token) = token {
            assert_eq!(
                process_identity_status(pid, Some(&token)),
                ProcessIdentityStatus::Match
            );
            assert_eq!(
                process_identity_status(pid, Some("not-the-current-start")),
                ProcessIdentityStatus::Mismatch
            );
        } else {
            assert_eq!(
                process_identity_status(pid, None),
                ProcessIdentityStatus::Unknown
            );
        }
        assert_eq!(
            process_identity_status(u32::MAX, None),
            ProcessIdentityStatus::Missing
        );
    }

    #[test]
    fn status_roundtrip_is_total() {
        for status in [
            BackgroundJobStatus::Queued,
            BackgroundJobStatus::Running,
            BackgroundJobStatus::Completed,
            BackgroundJobStatus::Failed,
            BackgroundJobStatus::MonitorLost,
        ] {
            assert_eq!(
                BackgroundJobStatus::parse(status.as_str()),
                status,
                "{status:?} must round-trip"
            );
        }
    }

    #[test]
    fn missing_completed_at_falls_back_to_started_at() {
        let record = BackgroundJobRecord {
            schema: BACKGROUND_JOB_SCHEMA.to_string(),
            job_id: "j1".to_string(),
            name: "gate".to_string(),
            command: "true".to_string(),
            cwd: ".".to_string(),
            attachment: None,
            status: BackgroundJobStatus::Failed,
            failure_class: Some(BackgroundJobFailureClass::LaunchFailed),
            exit_code: Some(1),
            pid: None,
            process_start_token: None,
            log_path: "/tmp/j1.log".to_string(),
            started_at: "t0".to_string(),
            completed_at: None,
            output_tail: String::new(),
        };
        assert_eq!(
            BackgroundJobCompletionEvent::from_record(&record).completed_at,
            "t0"
        );
    }
}
