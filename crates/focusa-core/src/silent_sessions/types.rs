use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{config::SilentSessionConfig, identity::*};

pub const SILENT_SESSION_SCHEMA_VERSION: u32 = 1;
pub const CONFIG_SCHEMA_VERSION: u32 = 1;
pub const EVENT_SCHEMA_VERSION: u32 = 1;
pub const DAEMON_RUNNER_PROTOCOL_VERSION: u32 = 1;
pub const HARNESS_ADAPTER_PROTOCOL_VERSION: u32 = 1;
pub const PROCESS_BACKEND_PROTOCOL_VERSION: u32 = 1;
pub const STREAM_CHUNK_FORMAT_VERSION: u32 = 1;
pub const RECEIPT_MAPPING_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SilentSessionLifecycle {
    Draft,
    Validating,
    Queued,
    Launching,
    Initializing,
    Running,
    WaitingInput,
    Blocked,
    Pausing,
    Paused,
    Resuming,
    Recovering,
    Orphaned,
    Completing,
    Completed,
    Failed,
    Cancelling,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SilentSessionHealth {
    Healthy,
    Degraded,
    Stale,
    Unresponsive,
    ProcessExited,
    TransportLost,
    RunnerLost,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SemanticActivity {
    Working,
    ToolRunning,
    Thinking,
    WaitingForOperator,
    WaitingForProvider,
    WaitingForDependency,
    IdleBetweenTurns,
    Verifying,
    Checkpointing,
    Integrating,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SilentSession {
    pub silent_session_schema_version: u32,
    pub id: SilentSessionId,
    pub authority: SilentSessionAuthority,
    /// Immutable authenticated principal that created the session.
    /// Empty only for legacy projections; authorization treats empty as unknown/fail-closed.
    #[serde(default)]
    pub creator_principal_id: String,
    /// Mutable control owner; adoption may transfer this without rewriting creator identity.
    #[serde(default)]
    pub controller_principal_id: String,
    /// Immutable operating-system account that owns the local runtime process.
    /// Empty only for legacy projections; authorization treats empty as unknown/fail-closed.
    #[serde(default)]
    pub owner_os_user: String,
    pub display_name: String,
    pub work_item_ref: Option<String>,
    pub mission: String,
    pub active_config_revision_id: ConfigRevisionId,
    pub current_run_generation: RunGeneration,
    pub lifecycle: SilentSessionLifecycle,
    pub health: SilentSessionHealth,
    pub semantic_activity: SemanticActivity,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SilentSession {
    pub fn draft(
        authority: SilentSessionAuthority,
        display_name: impl Into<String>,
        mission: impl Into<String>,
        active_config_revision_id: ConfigRevisionId,
        now: DateTime<Utc>,
    ) -> Result<Self, SilentSessionTypeError> {
        let display_name = display_name.into();
        let mission = mission.into();
        require_nonempty("display_name", &display_name)?;
        require_nonempty("mission", &mission)?;
        Ok(Self {
            silent_session_schema_version: SILENT_SESSION_SCHEMA_VERSION,
            id: SilentSessionId::new(),
            authority,
            creator_principal_id: String::new(),
            controller_principal_id: String::new(),
            owner_os_user: String::new(),
            display_name,
            work_item_ref: None,
            mission,
            active_config_revision_id,
            current_run_generation: RunGeneration::first(),
            lifecycle: SilentSessionLifecycle::Draft,
            health: SilentSessionHealth::Unknown,
            semantic_activity: SemanticActivity::Unknown,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draft_owned(
        authority: SilentSessionAuthority,
        creator_principal_id: impl Into<String>,
        owner_os_user: impl Into<String>,
        display_name: impl Into<String>,
        mission: impl Into<String>,
        active_config_revision_id: ConfigRevisionId,
        now: DateTime<Utc>,
    ) -> Result<Self, SilentSessionTypeError> {
        let creator_principal_id = creator_principal_id.into();
        let owner_os_user = owner_os_user.into();
        require_nonempty("creator_principal_id", &creator_principal_id)?;
        require_nonempty("owner_os_user", &owner_os_user)?;
        let mut session = Self::draft(
            authority,
            display_name,
            mission,
            active_config_revision_id,
            now,
        )?;
        session.creator_principal_id = creator_principal_id.clone();
        session.controller_principal_id = creator_principal_id;
        session.owner_os_user = owner_os_user;
        Ok(session)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SilentSessionRun {
    pub silent_session_schema_version: u32,
    pub id: SilentSessionRunId,
    pub silent_session_id: SilentSessionId,
    pub generation: RunGeneration,
    pub actor_instance_id: ActorInstanceId,
    pub config_revision_id: ConfigRevisionId,
    pub protocol_versions: ProtocolVersions,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SilentSessionConfigRevision {
    pub config_schema_version: u32,
    pub id: ConfigRevisionId,
    pub silent_session_id: SilentSessionId,
    pub revision: u64,
    pub config: SilentSessionConfig,
    pub redacted_config_hash: String,
    pub created_by: ActorInstanceId,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolVersions {
    pub daemon_runner_protocol_version: u32,
    pub harness_adapter_protocol_version: u32,
    pub process_backend_protocol_version: u32,
    pub stream_chunk_format_version: u32,
    pub receipt_mapping_version: u32,
}

impl Default for ProtocolVersions {
    fn default() -> Self {
        Self {
            daemon_runner_protocol_version: DAEMON_RUNNER_PROTOCOL_VERSION,
            harness_adapter_protocol_version: HARNESS_ADAPTER_PROTOCOL_VERSION,
            process_backend_protocol_version: PROCESS_BACKEND_PROTOCOL_VERSION,
            stream_chunk_format_version: STREAM_CHUNK_FORMAT_VERSION,
            receipt_mapping_version: RECEIPT_MAPPING_VERSION,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SilentSessionEvent {
    pub event_schema_version: u32,
    pub id: SilentSessionEventId,
    pub silent_session_id: SilentSessionId,
    pub run_id: Option<SilentSessionRunId>,
    pub sequence: u64,
    pub kind: String,
    pub payload: Value,
    pub idempotency_key: String,
    pub previous_event_hash: Option<String>,
    pub event_hash: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeCheckpoint {
    pub schema_version: u32,
    pub id: RuntimeCheckpointId,
    pub silent_session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub run_generation: RunGeneration,
    pub event_sequence: u64,
    pub stream_cursor: String,
    pub runtime_state_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SilentSessionWorkpointCheckpoint {
    pub schema_version: u32,
    pub id: WorkpointCheckpointId,
    pub silent_session_id: SilentSessionId,
    pub workpoint_id: String,
    pub mission: String,
    pub current_action: String,
    pub next_action: String,
    pub evidence_refs: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LeaseStatus {
    Active,
    Released,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SilentSessionLease {
    pub schema_version: u32,
    pub id: SilentSessionLeaseId,
    pub silent_session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub owner_actor_instance_id: ActorInstanceId,
    pub fencing_token: u64,
    pub status: LeaseStatus,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompletionDecision {
    Complete,
    Incomplete,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompletionEvaluation {
    pub schema_version: u32,
    pub id: CompletionEvaluationId,
    pub silent_session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub decision: CompletionDecision,
    pub reason: String,
    pub required_evidence_refs: Vec<String>,
    pub verified_evidence_refs: Vec<String>,
    pub receipt_ready: bool,
    pub evaluated_by: ActorInstanceId,
    pub evaluated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use serde_json::json;

    use super::*;
    use crate::silent_sessions::config::{
        HarnessConfig, HarnessKind, IdentityConfig, ModelConfig, ModelFallbackPolicy, ModelRef,
        ModelSelectionPolicy, NativeResumePolicy,
    };

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 7, 17, 12, 0, 0).unwrap()
    }

    fn project_root() -> String {
        crate::test_support::absolute_path_string("silent-session-types-project")
    }

    fn config() -> SilentSessionConfig {
        SilentSessionConfig::new(
            IdentityConfig {
                display_name: "proof".into(),
                project_root: project_root(),
                continuity_id: "cont-1".into(),
                work_item_ref: Some("focusa-a6yq6.2.1".into()),
                mission: "prove domain".into(),
                agent_identity_ref: "agent:pi".into(),
                role_profile_ref: None,
            },
            HarnessConfig {
                kind: HarnessKind::Pi,
                adapter_version: "1".into(),
                native_resume_policy: NativeResumePolicy::Prefer,
            },
            ModelConfig {
                provider: "anthropic".into(),
                model: "claude-opus".into(),
                thinking: Some("high".into()),
                selection_policy: ModelSelectionPolicy::Exact,
                fallback_policy: ModelFallbackPolicy::Disabled,
                allowed_fallbacks: Vec::<ModelRef>::new(),
                auth_profile_ref: "operator".into(),
                require_entitlement_preflight: true,
                require_runtime_model_confirmation: true,
            },
        )
    }

    #[test]
    fn generated_canonical_ids_are_uuid_v7() {
        assert_eq!(SilentSessionId::new().as_uuid().get_version_num(), 7);
        assert_eq!(SilentSessionRunId::new().as_uuid().get_version_num(), 7);
        assert_eq!(ConfigRevisionId::new().as_uuid().get_version_num(), 7);
    }

    #[test]
    fn authority_requires_absolute_project_and_continuity() {
        assert!(matches!(
            SilentSessionAuthority::new("relative", "cont"),
            Err(SilentSessionTypeError::ProjectRootNotAbsolute)
        ));
        assert!(matches!(
            SilentSessionAuthority::new(project_root(), " "),
            Err(SilentSessionTypeError::EmptyField {
                field: "continuity_id"
            })
        ));
    }

    #[test]
    fn run_generation_is_nonzero_and_monotonic() {
        assert_eq!(RunGeneration::first().get(), 1);
        assert_eq!(RunGeneration::first().next().unwrap().get(), 2);
        assert_eq!(
            RunGeneration::new(0),
            Err(SilentSessionTypeError::ZeroRunGeneration)
        );
    }

    #[test]
    fn draft_session_is_scope_bound_and_versioned() {
        let revision = ConfigRevisionId::new();
        let authority = SilentSessionAuthority::new(project_root(), "cont-1").unwrap();
        let session =
            SilentSession::draft(authority.clone(), "proof", "mission", revision, now()).unwrap();
        assert_eq!(session.authority, authority);
        assert_eq!(session.lifecycle, SilentSessionLifecycle::Draft);
        assert_eq!(session.silent_session_schema_version, 1);
    }

    #[test]
    fn owned_draft_captures_immutable_authorization_facts() {
        let session = SilentSession::draft_owned(
            SilentSessionAuthority::new(project_root(), "cont-1").unwrap(),
            "principal:device:mac",
            "wirebot",
            "proof",
            "mission",
            ConfigRevisionId::new(),
            now(),
        )
        .unwrap();
        assert_eq!(session.creator_principal_id, "principal:device:mac");
        assert_eq!(session.controller_principal_id, "principal:device:mac");
        assert_eq!(session.owner_os_user, "wirebot");
        assert!(
            SilentSession::draft_owned(
                SilentSessionAuthority::new(project_root(), "cont-1").unwrap(),
                "",
                "wirebot",
                "proof",
                "mission",
                ConfigRevisionId::new(),
                now(),
            )
            .is_err()
        );
    }

    #[test]
    fn legacy_projection_without_ownership_deserializes_fail_closed() {
        let authority = SilentSessionAuthority::new(project_root(), "cont-1").unwrap();
        let session = SilentSession::draft(
            authority,
            "proof",
            "mission",
            ConfigRevisionId::new(),
            now(),
        )
        .unwrap();
        let mut value = serde_json::to_value(session).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .remove("creator_principal_id");
        value
            .as_object_mut()
            .unwrap()
            .remove("controller_principal_id");
        value.as_object_mut().unwrap().remove("owner_os_user");
        let restored: SilentSession = serde_json::from_value(value).unwrap();
        assert!(restored.creator_principal_id.is_empty());
        assert!(restored.controller_principal_id.is_empty());
        assert!(restored.owner_os_user.is_empty());
    }

    #[test]
    fn all_independent_protocol_versions_serialize() {
        let value = serde_json::to_value(ProtocolVersions::default()).unwrap();
        for key in [
            "daemon_runner_protocol_version",
            "harness_adapter_protocol_version",
            "process_backend_protocol_version",
            "stream_chunk_format_version",
            "receipt_mapping_version",
        ] {
            assert_eq!(value[key], 1);
        }
    }

    #[test]
    fn event_roundtrip_preserves_unknown_kind_and_payload() {
        let event = SilentSessionEvent {
            event_schema_version: EVENT_SCHEMA_VERSION,
            id: SilentSessionEventId::new(),
            silent_session_id: SilentSessionId::new(),
            run_id: None,
            sequence: 4,
            kind: "future_adapter_observation".into(),
            payload: json!({"future": true}),
            idempotency_key: "event-4".into(),
            previous_event_hash: Some("abc".into()),
            event_hash: "def".into(),
            occurred_at: now(),
        };
        let encoded = serde_json::to_vec(&event).unwrap();
        let decoded: SilentSessionEvent = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded.kind, "future_adapter_observation");
        assert_eq!(decoded.payload, json!({"future": true}));
    }

    #[test]
    fn config_revision_has_independent_schema_and_hash() {
        let revision = SilentSessionConfigRevision {
            config_schema_version: CONFIG_SCHEMA_VERSION,
            id: ConfigRevisionId::new(),
            silent_session_id: SilentSessionId::new(),
            revision: 1,
            config: config(),
            redacted_config_hash: "sha256:abc".into(),
            created_by: ActorInstanceId::new(),
            created_at: now(),
        };
        let value = serde_json::to_value(revision).unwrap();
        assert_eq!(value["config_schema_version"], 1);
        assert_eq!(value["config"]["schema"], "focusa.silent_session_config.v1");
    }
}
