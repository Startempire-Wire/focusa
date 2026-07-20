//! Canonical success, failure, retry, and recovery envelope shared by Focusa surfaces.

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const TOOL_RESULT_SCHEMA: &str = "focusa.tool_result.v1";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolStatus {
    Accepted,
    Completed,
    NoOp,
    Blocked,
    ValidationRejected,
    Degraded,
    Offline,
    Error,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryPosture {
    SafeRetry,
    RetryWithIdempotencyKey,
    CheckSideEffectsFirst,
    DoNotRetryUnchanged,
    OperatorRequired,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    ValidationRejected,
    SchemaInvalid,
    NotFound,
    FrameUnavailable,
    DaemonUnavailable,
    StaleRuntimeRegistry,
    ResourceExhausted,
    NullResponse,
    HotPathTimeout,
    ColdPathTimeout,
    WriterConflict,
    ScopeMismatch,
    ScopeConflict,
    ApprovalRequired,
    PermissionDenied,
    ProcessControlFailed,
    NoncanonicalFallback,
    ReadModelLag,
    UnknownAmbiguousCompletion,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RetryDirective {
    pub safe: bool,
    pub posture: RetryPosture,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ToolError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_values: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ToolResultV1 {
    pub schema: String,
    pub ok: bool,
    pub status: ToolStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_class: Option<FailureClass>,
    pub canonical: bool,
    pub degraded: bool,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workpoint_id: Option<String>,
    pub retry: RetryDirective,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub misuse_hint: Option<String>,
    #[serde(default)]
    pub side_effects: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub next_tools: Vec<String>,
    #[serde(default)]
    pub reflex_suggestions: Vec<String>,
    #[serde(default)]
    pub ontology_candidate_delta_refs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ToolError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<Value>,
}

impl ToolResultV1 {
    pub fn success(status: ToolStatus, summary: impl Into<String>) -> Self {
        debug_assert!(matches!(
            status,
            ToolStatus::Accepted | ToolStatus::Completed | ToolStatus::NoOp
        ));
        Self {
            schema: TOOL_RESULT_SCHEMA.to_string(),
            ok: true,
            status,
            failure_class: None,
            canonical: true,
            degraded: false,
            summary: summary.into(),
            tool: None,
            family: None,
            endpoint: None,
            workpoint_id: None,
            retry: RetryDirective {
                safe: true,
                posture: RetryPosture::SafeRetry,
                reason: None,
            },
            recovery_hint: None,
            misuse_hint: None,
            side_effects: Vec::new(),
            evidence_refs: Vec::new(),
            next_tools: Vec::new(),
            reflex_suggestions: Vec::new(),
            ontology_candidate_delta_refs: Vec::new(),
            error: None,
            raw: None,
        }
    }

    pub fn failure(
        status: ToolStatus,
        failure_class: FailureClass,
        summary: impl Into<String>,
    ) -> Self {
        debug_assert!(!matches!(
            status,
            ToolStatus::Accepted | ToolStatus::Completed | ToolStatus::NoOp
        ));
        let reason = serde_json::to_value(failure_class)
            .ok()
            .and_then(|value| value.as_str().map(str::to_owned));
        Self {
            schema: TOOL_RESULT_SCHEMA.to_string(),
            ok: false,
            status,
            failure_class: Some(failure_class),
            canonical: false,
            degraded: matches!(status, ToolStatus::Degraded | ToolStatus::Offline),
            summary: summary.into(),
            tool: None,
            family: None,
            endpoint: None,
            workpoint_id: None,
            retry: RetryDirective {
                safe: false,
                posture: RetryPosture::DoNotRetryUnchanged,
                reason,
            },
            recovery_hint: None,
            misuse_hint: None,
            side_effects: Vec::new(),
            evidence_refs: Vec::new(),
            next_tools: Vec::new(),
            reflex_suggestions: Vec::new(),
            ontology_candidate_delta_refs: Vec::new(),
            error: None,
            raw: None,
        }
    }

    pub fn with_recovery(
        mut self,
        recovery_hint: impl Into<String>,
        misuse_hint: impl Into<String>,
        next_tools: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.recovery_hint = Some(recovery_hint.into());
        self.misuse_hint = Some(misuse_hint.into());
        self.next_tools = next_tools.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_error(mut self, code: impl Into<String>, message: impl Into<String>) -> Self {
        self.error = Some(ToolError {
            code: Some(code.into()),
            message: Some(message.into()),
            ..ToolError::default()
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_preserve_success_and_failure_invariants() {
        let success = ToolResultV1::success(ToolStatus::Completed, "done");
        assert!(success.ok && success.canonical && !success.degraded);
        assert_eq!(success.schema, TOOL_RESULT_SCHEMA);
        assert!(success.failure_class.is_none());

        let failure = ToolResultV1::failure(
            ToolStatus::Blocked,
            FailureClass::PermissionDenied,
            "permission denied",
        )
        .with_recovery(
            "request approval",
            "do not retry unchanged",
            ["focusa_tool_doctor"],
        )
        .with_error("forbidden", "permission denied");
        assert!(!failure.ok && !failure.canonical);
        assert_eq!(failure.retry.posture, RetryPosture::DoNotRetryUnchanged);
        assert_eq!(failure.next_tools, ["focusa_tool_doctor"]);
        assert_eq!(
            serde_json::to_value(failure).unwrap()["failure_class"],
            "permission_denied"
        );
    }
}
