use chrono::{DateTime, Utc};
use focusa_core::silent_sessions::{
    ApprovalId, RunGeneration, SilentSessionAction, SilentSessionId, SilentSessionRouteScope,
    SilentSessionRun, SilentSessionRunId, StreamCursor,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum SilentSessionApiMethod {
    Get,
    Post,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SilentSessionRouteSpec {
    pub name: &'static str,
    pub method: SilentSessionApiMethod,
    pub path: &'static str,
    pub scope: SilentSessionRouteScope,
    pub mutates: bool,
    pub exact_run_required: bool,
}

macro_rules! route {
    ($name:literal, $method:ident, $path:literal, $scope:ident, $mutates:literal, $run:literal) => {
        SilentSessionRouteSpec {
            name: $name,
            method: SilentSessionApiMethod::$method,
            path: $path,
            scope: SilentSessionRouteScope::$scope,
            mutates: $mutates,
            exact_run_required: $run,
        }
    };
}

pub const SILENT_SESSION_PHASE2_ROUTES: [SilentSessionRouteSpec; 36] = [
    route!(
        "preflight",
        Post,
        "/v1/silent-sessions/preflight",
        Create,
        false,
        false
    ),
    route!("create", Post, "/v1/silent-sessions", Create, true, false),
    route!("list", Get, "/v1/silent-sessions", Read, false, false),
    route!(
        "show",
        Get,
        "/v1/silent-sessions/{session_id}",
        Read,
        false,
        false
    ),
    route!(
        "start",
        Post,
        "/v1/silent-sessions/{session_id}/start",
        Create,
        true,
        true
    ),
    route!(
        "approval_create",
        Post,
        "/v1/silent-sessions/{session_id}/approvals",
        Control,
        true,
        true
    ),
    route!(
        "pause",
        Post,
        "/v1/silent-sessions/{session_id}/pause",
        Control,
        true,
        true
    ),
    route!(
        "resume",
        Post,
        "/v1/silent-sessions/{session_id}/resume",
        Control,
        true,
        true
    ),
    route!(
        "interrupt",
        Post,
        "/v1/silent-sessions/{session_id}/interrupt",
        Control,
        true,
        true
    ),
    route!(
        "cancel",
        Post,
        "/v1/silent-sessions/{session_id}/cancel",
        Control,
        true,
        true
    ),
    route!(
        "restart",
        Post,
        "/v1/silent-sessions/{session_id}/restart",
        Control,
        true,
        true
    ),
    route!(
        "adopt",
        Post,
        "/v1/silent-sessions/{session_id}/adopt",
        Admin,
        true,
        true
    ),
    route!(
        "events",
        Get,
        "/v1/silent-sessions/{session_id}/events",
        Stream,
        false,
        true
    ),
    route!(
        "output",
        Get,
        "/v1/silent-sessions/{session_id}/output",
        Stream,
        false,
        true
    ),
    route!(
        "status",
        Get,
        "/v1/silent-sessions/{session_id}/status",
        Read,
        false,
        true
    ),
    route!(
        "usage",
        Get,
        "/v1/silent-sessions/{session_id}/usage",
        Read,
        false,
        true
    ),
    route!(
        "checkpoints",
        Get,
        "/v1/silent-sessions/{session_id}/checkpoints",
        Read,
        false,
        true
    ),
    route!(
        "artifacts",
        Get,
        "/v1/silent-sessions/{session_id}/artifacts",
        Read,
        false,
        true
    ),
    route!(
        "receipts",
        Get,
        "/v1/silent-sessions/{session_id}/receipts",
        Read,
        false,
        true
    ),
    route!(
        "input",
        Post,
        "/v1/silent-sessions/{session_id}/input",
        Control,
        true,
        true
    ),
    route!(
        "steer",
        Post,
        "/v1/silent-sessions/{session_id}/steer",
        Control,
        true,
        true
    ),
    route!(
        "follow_up",
        Post,
        "/v1/silent-sessions/{session_id}/follow-up",
        Control,
        true,
        true
    ),
    route!(
        "keys",
        Post,
        "/v1/silent-sessions/{session_id}/keys",
        Control,
        true,
        true
    ),
    route!(
        "profiles",
        Get,
        "/v1/silent-sessions/profiles",
        Read,
        false,
        false
    ),
    route!(
        "presets",
        Get,
        "/v1/silent-sessions/presets",
        Read,
        false,
        false
    ),
    route!(
        "config_resolve",
        Post,
        "/v1/silent-sessions/config/resolve",
        Config,
        false,
        false
    ),
    route!(
        "config_preview",
        Post,
        "/v1/silent-sessions/{session_id}/config/preview",
        Config,
        false,
        true
    ),
    route!(
        "config_revision",
        Post,
        "/v1/silent-sessions/{session_id}/config/revisions",
        Config,
        true,
        true
    ),
    route!(
        "config_rollback",
        Post,
        "/v1/silent-sessions/{session_id}/config/rollback",
        Config,
        true,
        true
    ),
    route!(
        "capabilities",
        Get,
        "/v1/silent-sessions/capabilities",
        Read,
        false,
        false
    ),
    route!("harnesses", Get, "/v1/harnesses", Read, false, false),
    route!(
        "harness_capabilities",
        Get,
        "/v1/harnesses/{harness}/capabilities",
        Read,
        false,
        false
    ),
    route!(
        "harness_preflight",
        Post,
        "/v1/harnesses/{harness}/preflight",
        Create,
        false,
        false
    ),
    route!("providers", Get, "/v1/providers", Read, false, false),
    route!(
        "provider_models",
        Get,
        "/v1/providers/{provider}/models",
        Read,
        false,
        false
    ),
    route!(
        "provider_model_preflight",
        Post,
        "/v1/providers/{provider}/models/preflight",
        Create,
        false,
        false
    ),
];

pub const SILENT_SESSION_APPROVAL_REQUEST_SCHEMA_V1: &str =
    "focusa.silent_session_approval_request.v1";
pub const SILENT_SESSION_APPROVAL_RESPONSE_SCHEMA_V1: &str =
    "focusa.silent_session_approval_response.v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalRequestAction {
    Start,
    Input,
    Steer,
    FollowUp,
    Keys,
    Cancel,
}

impl ApprovalRequestAction {
    pub fn silent_session_action(self) -> SilentSessionAction {
        match self {
            Self::Start => SilentSessionAction::Start,
            Self::Input | Self::Steer | Self::FollowUp | Self::Keys => {
                SilentSessionAction::SendInput
            }
            Self::Cancel => SilentSessionAction::Cancel,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ApprovalCreateRequest {
    pub schema: String,
    pub action: ApprovalRequestAction,
    pub run_id: SilentSessionRunId,
    pub generation: RunGeneration,
    pub idempotency_key: String,
    pub risk_acknowledged: bool,
    #[serde(default)]
    pub payload: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovalCreateResponse {
    pub schema: String,
    pub status: String,
    pub approval_id: ApprovalId,
    pub action: ApprovalRequestAction,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: RunGeneration,
    pub expires_at: DateTime<Utc>,
    pub receipt_ref: String,
    pub action_idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetryDirective {
    pub retryable: bool,
    pub after_ms: Option<u64>,
    pub idempotency_key_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiSideEffect {
    pub effect: String,
    pub status: String,
    pub target_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SilentSessionApiEnvelope<T> {
    pub ok: bool,
    pub status: String,
    pub canonical: bool,
    pub advisory: bool,
    pub degraded: bool,
    pub stale: bool,
    pub failure_class: Option<String>,
    pub retry: RetryDirective,
    pub side_effects: Vec<ApiSideEffect>,
    pub evidence_refs: Vec<String>,
    pub receipt_refs: Vec<String>,
    pub next_tools: Vec<String>,
    pub recovery_hint: Option<String>,
    pub misuse_hint: Option<String>,
    pub data: Option<T>,
}

impl<T> SilentSessionApiEnvelope<T> {
    pub fn canonical(status: impl Into<String>, data: T) -> Self {
        Self {
            ok: true,
            status: status.into(),
            canonical: true,
            advisory: false,
            degraded: false,
            stale: false,
            failure_class: None,
            retry: RetryDirective {
                retryable: false,
                after_ms: None,
                idempotency_key_required: false,
            },
            side_effects: Vec::new(),
            evidence_refs: Vec::new(),
            receipt_refs: Vec::new(),
            next_tools: Vec::new(),
            recovery_hint: None,
            misuse_hint: None,
            data: Some(data),
        }
    }

    pub fn failure(
        status: impl Into<String>,
        failure_class: impl Into<String>,
        retry: RetryDirective,
    ) -> Self {
        Self {
            ok: false,
            status: status.into(),
            canonical: true,
            advisory: false,
            degraded: false,
            stale: false,
            failure_class: Some(failure_class.into()),
            retry,
            side_effects: Vec::new(),
            evidence_refs: Vec::new(),
            receipt_refs: Vec::new(),
            next_tools: Vec::new(),
            recovery_hint: None,
            misuse_hint: None,
            data: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExactSessionRunTarget {
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub generation: RunGeneration,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExactTargetError {
    SessionMismatch,
    RunMismatch,
    StaleGeneration,
    InvalidResumeCursor,
}

pub fn guard_exact_target(
    target: ExactSessionRunTarget,
    run: &SilentSessionRun,
) -> Result<(), ExactTargetError> {
    if target.session_id != run.silent_session_id {
        return Err(ExactTargetError::SessionMismatch);
    }
    if target.run_id != run.id {
        return Err(ExactTargetError::RunMismatch);
    }
    if target.generation != run.generation {
        return Err(ExactTargetError::StaleGeneration);
    }
    Ok(())
}

pub fn resume_sequence(
    last_event_id: Option<&str>,
    expected_run_id: SilentSessionRunId,
) -> Result<u64, ExactTargetError> {
    let Some(cursor) = last_event_id else {
        return Ok(0);
    };
    let decoded =
        StreamCursor::decode(cursor).map_err(|_| ExactTargetError::InvalidResumeCursor)?;
    if decoded.run_id != expected_run_id {
        return Err(ExactTargetError::InvalidResumeCursor);
    }
    Ok(decoded.sequence)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use focusa_core::silent_sessions::{
        ActorInstanceId, ConfigRevisionId, ProtocolVersions, RunGeneration, SilentSessionRun,
        StreamCursor,
    };

    use super::*;

    fn run() -> SilentSessionRun {
        SilentSessionRun {
            silent_session_schema_version: 1,
            id: SilentSessionRunId::new(),
            silent_session_id: SilentSessionId::new(),
            generation: RunGeneration::new(1).unwrap(),
            actor_instance_id: ActorInstanceId::new(),
            config_revision_id: ConfigRevisionId::new(),
            protocol_versions: ProtocolVersions::default(),
            started_at: Utc::now(),
            ended_at: None,
        }
    }

    #[test]
    fn registry_covers_all_owned_routes_without_duplicates() {
        assert_eq!(SILENT_SESSION_PHASE2_ROUTES.len(), 36);
        let unique = SILENT_SESSION_PHASE2_ROUTES
            .iter()
            .map(|route| (route.method, route.path))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(unique.len(), 36);
        assert!(SILENT_SESSION_PHASE2_ROUTES.iter().all(|route| {
            !route.mutates || matches!(route.method, SilentSessionApiMethod::Post)
        }));
    }

    #[test]
    fn envelope_always_serializes_every_shared_field() {
        let envelope = SilentSessionApiEnvelope::canonical("ready", serde_json::json!({"id":1}));
        let value = serde_json::to_value(envelope).unwrap();
        for field in [
            "ok",
            "status",
            "canonical",
            "advisory",
            "degraded",
            "stale",
            "failure_class",
            "retry",
            "side_effects",
            "evidence_refs",
            "receipt_refs",
            "next_tools",
            "recovery_hint",
            "misuse_hint",
        ] {
            assert!(value.get(field).is_some(), "missing {field}");
        }
    }

    #[test]
    fn approval_contract_is_versioned_bounded_and_deny_unknown() {
        let run = run();
        let raw = serde_json::json!({
            "schema": SILENT_SESSION_APPROVAL_REQUEST_SCHEMA_V1,
            "action": "start",
            "run_id": run.id,
            "generation": run.generation,
            "idempotency_key": "approval:test:1",
            "risk_acknowledged": true,
            "payload": null
        });
        let request: ApprovalCreateRequest = serde_json::from_value(raw).unwrap();
        assert_eq!(
            request.action.silent_session_action(),
            SilentSessionAction::Start
        );
        for supported in ["input", "steer", "follow_up", "keys"] {
            let mut value = serde_json::to_value(&request).unwrap();
            value["action"] = serde_json::Value::String(supported.into());
            value["payload"] = serde_json::json!({"content": "bound"});
            let delivery: ApprovalCreateRequest = serde_json::from_value(value).unwrap();
            assert_eq!(
                delivery.action.silent_session_action(),
                SilentSessionAction::SendInput
            );
        }
        for unsupported in ["send_input", "interrupt", "adopt", "force_kill", "release"] {
            let mut value = serde_json::to_value(&request).unwrap();
            value["action"] = serde_json::Value::String(unsupported.into());
            assert!(serde_json::from_value::<ApprovalCreateRequest>(value).is_err());
        }
        let mut injected = serde_json::to_value(&request).unwrap();
        injected["action_digest"] = serde_json::Value::String("client-controlled".into());
        assert!(serde_json::from_value::<ApprovalCreateRequest>(injected).is_err());
        assert_eq!(
            SILENT_SESSION_APPROVAL_RESPONSE_SCHEMA_V1,
            "focusa.silent_session_approval_response.v1"
        );
    }

    #[test]
    fn exact_target_and_resume_cursor_are_run_generation_bound() {
        let run = run();
        let target = ExactSessionRunTarget {
            session_id: run.silent_session_id,
            run_id: run.id,
            generation: run.generation,
        };
        assert_eq!(guard_exact_target(target, &run), Ok(()));
        let cursor = StreamCursor::new(run.id, 42).encode().unwrap();
        assert_eq!(resume_sequence(Some(&cursor), run.id), Ok(42));
        assert_eq!(
            resume_sequence(Some(&cursor), SilentSessionRunId::new()),
            Err(ExactTargetError::InvalidResumeCursor)
        );
    }
}
