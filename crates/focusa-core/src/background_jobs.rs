//! Background job completion events (#311-family): first-class background
//! execution records with a durable completion boundary.
//!
//! `focusa bg run` creates a job row, executes the command as a detached
//! child, and reports the completion back through the daemon — which
//! records it durably and broadcasts it over SSE. Consumers (Pi extension
//! uiCtx.notify, `focusa bg wait`, TUI) all read the same envelope.
//! Nothing here is a shell wrapper: the CLI monitor is the lifecycle owner.

use std::io::{Read, Seek, SeekFrom};

use serde::{Deserialize, Serialize};

pub const BACKGROUND_JOB_SCHEMA: &str = "focusa.background_job.v1";
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackgroundJobRecord {
    pub schema: String,
    pub job_id: String,
    pub name: String,
    pub command: String,
    pub cwd: String,
    pub status: BackgroundJobStatus,
    pub exit_code: Option<i32>,
    pub pid: Option<u32>,
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
    pub status: BackgroundJobStatus,
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
            status: record.status,
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

    #[test]
    fn completion_envelope_carries_every_consumer_field() {
        let record = BackgroundJobRecord {
            schema: BACKGROUND_JOB_SCHEMA.to_string(),
            job_id: "j1".to_string(),
            name: "gate".to_string(),
            command: "cargo test".to_string(),
            cwd: "/root/proj".to_string(),
            status: BackgroundJobStatus::Completed,
            exit_code: Some(0),
            pid: Some(42),
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
        assert_eq!(value["output_tail"], "typecheck failed");
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
        assert_eq!(tail, "final diagnostic line");

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
            status: BackgroundJobStatus::Failed,
            exit_code: Some(1),
            pid: None,
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
            status: BackgroundJobStatus::Failed,
            exit_code: Some(1),
            pid: None,
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
