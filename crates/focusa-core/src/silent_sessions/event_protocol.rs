use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{ObservationProvenance, SilentSessionEventId, SilentSessionId, SilentSessionRunId};

pub const SILENT_SESSION_EVENT_SCHEMA_V1: &str = "focusa.silent_session_event.v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum OutputChannel {
    Stdout,
    Stderr,
    StructuredHarnessEvents,
    AssistantText,
    ThinkingText,
    ToolCalls,
    ToolOutput,
    FocusaControlEvents,
    OperatorInput,
    SystemDiagnostics,
}

impl OutputChannel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
            Self::StructuredHarnessEvents => "structured_harness_events",
            Self::AssistantText => "assistant_text",
            Self::ThinkingText => "thinking_text",
            Self::ToolCalls => "tool_calls",
            Self::ToolOutput => "tool_output",
            Self::FocusaControlEvents => "focusa_control_events",
            Self::OperatorInput => "operator_input",
            Self::SystemDiagnostics => "system_diagnostics",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RedactionReport {
    pub applied: bool,
    pub classes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CanonicalStreamEvent {
    pub schema: String,
    pub event_id: SilentSessionEventId,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub seq: u64,
    pub occurred_at: DateTime<Utc>,
    pub observed_at: DateTime<Utc>,
    pub kind: String,
    pub source: String,
    pub provenance: ObservationProvenance,
    pub canonical: bool,
    pub channel: OutputChannel,
    pub payload: Value,
    pub artifact_refs: Vec<String>,
    pub correlation_id: SilentSessionEventId,
    pub redaction: RedactionReport,
}

impl CanonicalStreamEvent {
    pub fn validate(&self) -> Result<(), EventProtocolError> {
        if self.schema != SILENT_SESSION_EVENT_SCHEMA_V1 {
            return Err(EventProtocolError::UnsupportedSchema(self.schema.clone()));
        }
        if self.seq == 0 {
            return Err(EventProtocolError::ZeroSequence);
        }
        if self.source.trim().is_empty() {
            return Err(EventProtocolError::EmptySource);
        }
        if !is_known_event_kind(&self.kind) {
            return Err(EventProtocolError::UnknownEventKind(self.kind.clone()));
        }
        if !self.redaction.applied {
            return Err(EventProtocolError::RedactionRequired);
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EventProtocolError {
    #[error("unsupported event schema: {0}")]
    UnsupportedSchema(String),
    #[error("event sequence must be greater than zero")]
    ZeroSequence,
    #[error("event source must not be empty")]
    EmptySource,
    #[error("unknown Spec133 event kind: {0}")]
    UnknownEventKind(String),
    #[error("redaction must be applied before durable stream storage")]
    RedactionRequired,
}

pub fn is_known_event_kind(kind: &str) -> bool {
    matches!(
        kind,
        "session.created"
            | "session.validation_started"
            | "session.validation_failed"
            | "session.admitted"
            | "session.queued"
            | "session.launching"
            | "session.initializing"
            | "session.started"
            | "session.pausing"
            | "session.paused"
            | "session.resuming"
            | "session.recovering"
            | "session.orphaned"
            | "session.completing"
            | "session.completed"
            | "session.failed"
            | "session.cancelling"
            | "session.cancelled"
            | "config.resolved"
            | "config.revision_proposed"
            | "config.revision_applied"
            | "config.revision_rolled_back"
            | "model.preflight_started"
            | "model.preflight_passed"
            | "model.preflight_failed"
            | "model.requested"
            | "model.effective"
            | "model.observed"
            | "model.mismatch"
            | "model.fallback_proposed"
            | "model.fallback_applied"
            | "harness.connected"
            | "harness.disconnected"
            | "agent.started"
            | "agent.working"
            | "agent.waiting_input"
            | "agent.blocked"
            | "agent.idle"
            | "agent.turn_started"
            | "agent.turn_ended"
            | "agent.settled"
            | "agent.error"
            | "stream.stdout"
            | "stream.stderr"
            | "assistant.text_delta"
            | "assistant.thinking_delta"
            | "tool.started"
            | "tool.output"
            | "tool.completed"
            | "tool.failed"
            | "prompt.detected"
            | "input.sent"
            | "key.sent"
            | "interrupt.sent"
            | "project_identity.verified"
            | "trajectory.bound"
            | "workpoint.bound"
            | "workpoint.checkpoint_requested"
            | "workpoint.checkpoint_linked"
            | "context_cognition.packet_bound"
            | "context_authority.preflight"
            | "evidence.captured"
            | "receipt.previewed"
            | "receipt.committed"
            | "writer_lease.acquired"
            | "writer_lease.renewed"
            | "writer_lease.released"
            | "writer_lease.conflict"
            | "resource.admitted"
            | "resource.sample"
            | "resource.pressure"
            | "resource.limit_approaching"
            | "resource.limit_exceeded"
            | "retry.scheduled"
            | "retry.exhausted"
            | "backpressure.applied"
            | "process.spawned"
            | "process.exited"
            | "process.signal_sent"
            | "process_group.terminated"
            | "child_leak.detected"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_output_channels_have_stable_names() {
        let channels = [
            OutputChannel::Stdout,
            OutputChannel::Stderr,
            OutputChannel::StructuredHarnessEvents,
            OutputChannel::AssistantText,
            OutputChannel::ThinkingText,
            OutputChannel::ToolCalls,
            OutputChannel::ToolOutput,
            OutputChannel::FocusaControlEvents,
            OutputChannel::OperatorInput,
            OutputChannel::SystemDiagnostics,
        ];
        assert_eq!(channels.len(), 10);
        assert!(channels.iter().all(|channel| !channel.as_str().is_empty()));
    }

    #[test]
    fn required_event_families_are_known() {
        for kind in [
            "session.created",
            "config.resolved",
            "model.effective",
            "agent.waiting_input",
            "stream.stdout",
            "tool.completed",
            "writer_lease.acquired",
            "resource.limit_exceeded",
            "process.exited",
        ] {
            assert!(is_known_event_kind(kind), "{kind}");
        }
        assert!(!is_known_event_kind("future.unknown"));
    }
}
