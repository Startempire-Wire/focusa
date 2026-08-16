//! Background job completion events (#311-family): first-class background
//! execution records with a durable completion boundary.
//!
//! `focusa bg run` creates a job row, executes the command as a detached
//! child, and reports the completion back through the daemon — which
//! records it durably and broadcasts it over SSE. Consumers (Pi extension
//! uiCtx.notify, `focusa bg wait`, TUI) all read the same envelope.
//! Nothing here is a shell wrapper: the CLI monitor is the lifecycle owner.

use serde::{Deserialize, Serialize};

pub const BACKGROUND_JOB_SCHEMA: &str = "focusa.background_job.v1";
pub const BACKGROUND_JOB_COMPLETION_EVENT: &str = "background_job_completion";

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

/// Read the bounded tail of a job log (last N bytes, line-aligned).
pub fn bounded_log_tail(log_path: &str, max_bytes: usize) -> String {
    let Ok(bytes) = std::fs::read(log_path) else {
        return String::new();
    };
    if bytes.len() <= max_bytes {
        return String::from_utf8_lossy(&bytes).to_string();
    }
    let start = bytes.len() - max_bytes;
    let tail = &bytes[start..];
    // Align to the next newline boundary.
    let aligned = match tail.iter().position(|b| *b == b'\n') {
        Some(index) => &tail[index + 1..],
        None => tail,
    };
    String::from_utf8_lossy(aligned).to_string()
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
            output_tail: bounded_log_tail(&record.log_path, 4096),
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
        };
        let envelope = BackgroundJobCompletionEvent::from_record(&record);
        assert_eq!(envelope.event_type, BACKGROUND_JOB_COMPLETION_EVENT);
        assert_eq!(envelope.job_id, "j1");
        assert_eq!(envelope.status, BackgroundJobStatus::Completed);
        let value = serde_json::to_value(&envelope).unwrap();
        assert_eq!(value["event_type"], "background_job_completion");
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
        };
        assert_eq!(
            BackgroundJobCompletionEvent::from_record(&record).completed_at,
            "t0"
        );
    }
}
