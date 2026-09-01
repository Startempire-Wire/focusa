//! Spec 133 canonical daemon-native Silent Session domain types.
//! Backend handles (tmux names, PIDs, panes, terminal ids) are observations only.

use crate::silent_session_retry::{RetryBudgetPolicy, RetryClass, default_retry_budgets};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Component, PathBuf};
use uuid::{Uuid, Version};

pub const SILENT_SESSION_SCHEMA: &str = "focusa.silent_session.v1";
pub const SILENT_SESSION_RUN_SCHEMA: &str = "focusa.silent_session_run.v1";
pub const SILENT_SESSION_CONFIG_SCHEMA: &str = "focusa.silent_session_config.v1";
pub const SILENT_SESSION_CONFIG_REVISION_SCHEMA: &str = "focusa.silent_session_config_revision.v1";
pub const SILENT_SESSION_EVENT_SCHEMA: &str = "focusa.silent_session_event.v1";
pub const SILENT_SESSION_CHECKPOINT_SCHEMA: &str = "focusa.silent_session_checkpoint.v1";
pub const SILENT_SESSION_LEASE_SCHEMA: &str = "focusa.silent_session_lease.v1";
pub const SILENT_SESSION_COMPLETION_SCHEMA: &str = "focusa.silent_session_completion_evaluation.v1";

macro_rules! uuid_v7_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            pub fn is_uuid_v7(self) -> bool {
                self.0.get_version() == Some(Version::SortRand)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

uuid_v7_id!(SilentSessionId);
uuid_v7_id!(SilentSessionRunId);
uuid_v7_id!(SilentSessionEventId);
uuid_v7_id!(SilentSessionConfigRevisionId);
uuid_v7_id!(SilentSessionCheckpointId);
uuid_v7_id!(SilentSessionLeaseId);
uuid_v7_id!(SilentSessionCompletionEvaluationId);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SilentSessionLifecycleState {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SilentSessionSemanticActivity {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationProvenance {
    ModelInferred,
    TerminalInferred,
    RuntimeObserved,
    VerificationConfirmed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticObservation {
    pub activity: SilentSessionSemanticActivity,
    pub source: String,
    pub confidence: f64,
    pub observed_at: DateTime<Utc>,
    pub fresh_until: DateTime<Utc>,
    pub provenance: ObservationProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorAskBinding {
    pub ask_ref: String,
    pub exact_text: String,
    pub text_sha256: String,
    pub revision: u64,
    pub captured_at: DateTime<Utc>,
}

impl OperatorAskBinding {
    pub fn capture(
        ask_ref: impl Into<String>,
        exact_text: impl Into<String>,
        revision: u64,
        captured_at: DateTime<Utc>,
    ) -> Self {
        let exact_text = exact_text.into();
        let text_sha256 = hex::encode(Sha256::digest(exact_text.as_bytes()));
        Self {
            ask_ref: ask_ref.into(),
            exact_text,
            text_sha256,
            revision,
            captured_at,
        }
    }

    pub fn validate(&self) -> Result<(), SilentSessionInvariantError> {
        if self.ask_ref.trim().is_empty() || self.exact_text.trim().is_empty() {
            return Err(SilentSessionInvariantError::MissingField("operator_ask"));
        }
        if self.revision == 0 {
            return Err(SilentSessionInvariantError::InvalidOperatorAsk);
        }
        let expected = hex::encode(Sha256::digest(self.exact_text.as_bytes()));
        if self.text_sha256 != expected {
            return Err(SilentSessionInvariantError::InvalidOperatorAsk);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkpointBinding {
    pub workpoint_id: String,
    pub revision: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentSessionVersions {
    pub silent_session_schema_version: u32,
    pub config_schema_version: u32,
    pub event_schema_version: u32,
    pub daemon_runner_protocol_version: u32,
    pub harness_adapter_protocol_version: u32,
    pub process_backend_protocol_version: u32,
    pub stream_chunk_format_version: u32,
    pub receipt_mapping_version: u32,
}

impl Default for SilentSessionVersions {
    fn default() -> Self {
        Self {
            silent_session_schema_version: 1,
            config_schema_version: 1,
            event_schema_version: 1,
            daemon_runner_protocol_version: 1,
            harness_adapter_protocol_version: 1,
            process_backend_protocol_version: 1,
            stream_chunk_format_version: 1,
            receipt_mapping_version: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SilentSession {
    pub schema: String,
    pub versions: SilentSessionVersions,
    pub session_id: SilentSessionId,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub created_by_actor_ref: String,
    pub operator_principal_ref: String,
    pub os_execution_user: String,
    pub project_root: PathBuf,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub trajectory_ref: Option<String>,
    pub workpoint_ref: Option<WorkpointBinding>,
    pub work_item_ref: Option<String>,
    pub operator_ask: OperatorAskBinding,
    pub mission: String,
    pub lifecycle_state: SilentSessionLifecycleState,
    pub health: SilentSessionHealth,
    pub semantic_observation: Option<SemanticObservation>,
    pub active_run_id: Option<SilentSessionRunId>,
    pub config_revision_id: SilentSessionConfigRevisionId,
    pub writer_lease_ref: Option<SilentSessionLeaseId>,
    pub retention_policy_ref: String,
    pub receipt_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessIdentity {
    pub pid: u32,
    pub execution_user: String,
    pub executable_ref: String,
    pub signed_runner_record_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceBinding {
    pub workspace_id: String,
    pub root: PathBuf,
    pub strategy: WorkspaceStrategy,
    pub branch_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelBinding {
    pub provider: String,
    pub model: String,
    pub thinking: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SilentSessionRun {
    pub schema: String,
    pub versions: SilentSessionVersions,
    pub run_id: SilentSessionRunId,
    pub session_id: SilentSessionId,
    pub generation: u64,
    pub runner_id: String,
    pub adapter_id: String,
    pub process_backend_id: String,
    pub requested_model_binding: ModelBinding,
    pub effective_model_binding: Option<ModelBinding>,
    pub observed_model_binding: Option<ModelBinding>,
    pub workspace_binding: WorkspaceBinding,
    pub process_identity: Option<ProcessIdentity>,
    pub harness_native_session_ref: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub exit_status: Option<i32>,
    pub current_event_seq: u64,
    pub output_stream_refs: Vec<String>,
    pub runtime_checkpoint_refs: Vec<SilentSessionCheckpointId>,
    pub workpoint_checkpoint_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessKind {
    Pi,
    Codex,
    Claude,
    Opencode,
    GenericRpc,
    GenericPty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeResumePolicy {
    Prefer,
    Require,
    Disable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelSelectionPolicy {
    Exact,
    AllowList,
    Adaptive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFallbackPolicy {
    Disabled,
    ExplicitAllowList,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceStrategy {
    IsolatedWorktree,
    ExclusiveExisting,
    ReadOnlyShared,
    ExplicitShared,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationPolicy {
    Manual,
    VerifiedFastForward,
    GovernedMerge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentSessionIdentityConfig {
    pub display_name: String,
    pub project_root: PathBuf,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub work_item_ref: Option<String>,
    pub mission: String,
    pub agent_identity_ref: String,
    pub role_profile_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessConfig {
    pub kind: HarnessKind,
    pub adapter_version: String,
    pub native_resume_policy: NativeResumePolicy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SilentSessionModelConfig {
    pub requested: ModelBinding,
    pub selection_policy: ModelSelectionPolicy,
    pub fallback_policy: ModelFallbackPolicy,
    pub allowed_fallbacks: Vec<ModelBinding>,
    pub auth_profile_ref: String,
    pub require_entitlement_preflight: bool,
    pub require_runtime_model_confirmation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub strategy: WorkspaceStrategy,
    pub source_root: PathBuf,
    pub worktree_root: Option<PathBuf>,
    pub base_ref: Option<String>,
    pub branch_name: Option<String>,
    pub integration_policy: IntegrationPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupervisionConfig {
    pub restart_policy: String,
    pub max_process_restarts: u32,
    pub max_transport_retries: u32,
    pub retry_backoff_ms: u64,
    #[serde(default = "default_retry_budgets")]
    pub retry_budgets: BTreeMap<RetryClass, RetryBudgetPolicy>,
    pub soft_pause_timeout_ms: u64,
    pub graceful_stop_timeout_ms: u64,
    pub checkpoint_interval_seconds: u64,
    pub checkpoint_event_interval: u64,
    pub waiting_input_timeout_seconds: u64,
    pub silent_output_warning_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub priority: i32,
    pub max_wall_clock_seconds: Option<u64>,
    pub max_cpu_percent: Option<f64>,
    pub max_memory_bytes: Option<u64>,
    pub max_pids: Option<u32>,
    pub max_disk_bytes: Option<u64>,
    pub max_output_bytes: Option<u64>,
    pub max_tokens: Option<u64>,
    pub max_cost_usd: Option<f64>,
    pub max_turns: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputPolicy {
    pub persist_stdout: bool,
    pub persist_stderr: bool,
    pub persist_semantic_events: bool,
    pub chunk_max_bytes: u64,
    pub chunk_max_seconds: u64,
    pub redaction_profile_ref: String,
    pub operator_projection_budget: u64,
    pub raw_retention_policy_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernancePolicy {
    pub context_authority_required: bool,
    pub risky_mutation_preflight_required: bool,
    pub destructive_actions_allowed: bool,
    pub writer_lease_required: bool,
    pub completion_receipt_required: bool,
    pub evidence_policy_ref: String,
    pub policy_locks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationPolicy {
    pub waiting_input: bool,
    pub blocked: bool,
    pub failed: bool,
    pub completed: bool,
    pub model_mismatch: bool,
    pub budget_pressure: bool,
    pub channels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionConfig {
    pub policy_ref: String,
    pub evidence_hold: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SilentSessionConfig {
    pub schema: String,
    pub identity: SilentSessionIdentityConfig,
    pub harness: HarnessConfig,
    pub model: SilentSessionModelConfig,
    pub workspace: WorkspaceConfig,
    pub bootstrap_target_profile: String,
    pub bootstrap_packet_mode: String,
    pub bootstrap_verification_required: bool,
    pub supervision: SupervisionConfig,
    pub resources: ResourceLimits,
    pub output: OutputPolicy,
    pub governance: GovernancePolicy,
    pub notifications: NotificationPolicy,
    pub retention: RetentionConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SilentSessionConfigRevision {
    pub schema: String,
    pub revision_id: SilentSessionConfigRevisionId,
    pub session_id: SilentSessionId,
    pub parent_revision_id: Option<SilentSessionConfigRevisionId>,
    pub config: SilentSessionConfig,
    pub requested_changes: Value,
    pub effective_diff: Value,
    pub field_provenance: BTreeMap<String, String>,
    pub policy_lock_results: BTreeMap<String, bool>,
    pub operator_approval_ref: Option<String>,
    pub validation_result: ConfigValidationResult,
    pub applied_at: Option<DateTime<Utc>>,
    pub rollback_target: Option<SilentSessionConfigRevisionId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionMetadata {
    pub applied: bool,
    pub classes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SilentSessionEvent {
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
    pub payload: Value,
    pub artifact_refs: Vec<String>,
    pub correlation_id: Uuid,
    pub redaction: RedactionMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "checkpoint_class", rename_all = "snake_case")]
pub enum SilentSessionCheckpointBody {
    Runtime {
        process_position: Value,
        protocol_position: Value,
        stream_cursor: String,
        harness_session_ref: Option<String>,
        resource_counters: BTreeMap<String, u64>,
        retry_state: Value,
    },
    CanonicalWorkpoint {
        workpoint_ref: WorkpointBinding,
        mission: String,
        action_intent: String,
        active_objects: Vec<String>,
        blockers: Vec<String>,
        verified_evidence: Vec<String>,
        next_slice: String,
        do_not_drift: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SilentSessionCheckpoint {
    pub schema: String,
    pub checkpoint_id: SilentSessionCheckpointId,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub created_at: DateTime<Utc>,
    pub body: SilentSessionCheckpointBody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WriterMutationMode {
    ReadOnlyShared,
    #[default]
    ExclusiveExisting,
    IsolatedWorktree,
    ExplicitShared,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentSessionLease {
    pub schema: String,
    pub lease_id: SilentSessionLeaseId,
    pub session_id: SilentSessionId,
    pub project_root: PathBuf,
    pub project_identity_ref: String,
    pub continuity_id: String,
    pub work_item_ref: Option<String>,
    pub workspace_ref: String,
    pub path_intents: Vec<PathBuf>,
    #[serde(default)]
    pub mutation_mode: WriterMutationMode,
    pub writer_role: String,
    pub owner_actor_instance_ref: String,
    pub fencing_token: u64,
    pub acquired_at: DateTime<Utc>,
    pub heartbeat_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub adoption_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionDecision {
    Completed,
    Blocked,
    Failed,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SilentSessionCompletionEvaluation {
    pub schema: String,
    pub evaluation_id: SilentSessionCompletionEvaluationId,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub evaluated_at: DateTime<Utc>,
    pub process_result: Value,
    pub workpoint_status: String,
    pub work_item_acceptance: BTreeMap<String, bool>,
    pub evidence_classes: Vec<String>,
    pub test_results: Vec<Value>,
    pub diff_refs: Vec<String>,
    pub commit_refs: Vec<String>,
    pub unresolved_blockers: Vec<String>,
    pub adversarial_verifier_verdict: Option<String>,
    pub receipt_ready: bool,
    pub decision: CompletionDecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SilentSessionInvariantError {
    UnsupportedSchema,
    UnsupportedVersion,
    NonV7Identity(&'static str),
    MissingField(&'static str),
    UnsafeProjectRoot,
    InvalidConfidence,
    InvalidGeneration,
    InvalidSequence,
    InvalidLease,
    InvalidOperatorAsk,
    ScopeMutation,
}

impl SilentSession {
    pub fn validate(&self) -> Result<(), SilentSessionInvariantError> {
        self.operator_ask.validate()?;
        if self.schema != SILENT_SESSION_SCHEMA {
            return Err(SilentSessionInvariantError::UnsupportedSchema);
        }
        if self.versions != SilentSessionVersions::default() {
            return Err(SilentSessionInvariantError::UnsupportedVersion);
        }
        if !self.session_id.is_uuid_v7() {
            return Err(SilentSessionInvariantError::NonV7Identity("session_id"));
        }
        if !self.project_root.is_absolute() {
            return Err(SilentSessionInvariantError::UnsafeProjectRoot);
        }
        for (name, value) in [
            ("project_identity_ref", self.project_identity_ref.as_str()),
            ("continuity_id", self.continuity_id.as_str()),
            ("created_by_actor_ref", self.created_by_actor_ref.as_str()),
            (
                "operator_principal_ref",
                self.operator_principal_ref.as_str(),
            ),
            ("os_execution_user", self.os_execution_user.as_str()),
            ("mission", self.mission.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(SilentSessionInvariantError::MissingField(name));
            }
        }
        if let Some(observation) = &self.semantic_observation
            && (!(0.0..=1.0).contains(&observation.confidence)
                || observation.fresh_until < observation.observed_at)
        {
            return Err(SilentSessionInvariantError::InvalidConfidence);
        }
        Ok(())
    }

    pub fn scope_matches_config(&self, config: &SilentSessionConfig) -> bool {
        self.project_root == config.identity.project_root
            && self.project_identity_ref == config.identity.project_identity_ref
            && self.continuity_id == config.identity.continuity_id
    }
}

impl SilentSessionRun {
    pub fn validate(&self, session: &SilentSession) -> Result<(), SilentSessionInvariantError> {
        if self.schema != SILENT_SESSION_RUN_SCHEMA {
            return Err(SilentSessionInvariantError::UnsupportedSchema);
        }
        if self.versions != SilentSessionVersions::default() {
            return Err(SilentSessionInvariantError::UnsupportedVersion);
        }
        if !self.run_id.is_uuid_v7() {
            return Err(SilentSessionInvariantError::NonV7Identity("run_id"));
        }
        if self.session_id != session.session_id {
            return Err(SilentSessionInvariantError::ScopeMutation);
        }
        if self.generation == 0 {
            return Err(SilentSessionInvariantError::InvalidGeneration);
        }
        Ok(())
    }
}

impl SilentSessionConfigRevision {
    pub fn validate_for_session(
        &self,
        session: &SilentSession,
    ) -> Result<(), SilentSessionInvariantError> {
        if self.schema != SILENT_SESSION_CONFIG_REVISION_SCHEMA
            || self.config.schema != SILENT_SESSION_CONFIG_SCHEMA
        {
            return Err(SilentSessionInvariantError::UnsupportedSchema);
        }
        if !self.revision_id.is_uuid_v7() {
            return Err(SilentSessionInvariantError::NonV7Identity("revision_id"));
        }
        if self.session_id != session.session_id || !session.scope_matches_config(&self.config) {
            return Err(SilentSessionInvariantError::ScopeMutation);
        }
        Ok(())
    }
}

impl SilentSessionEvent {
    pub fn validate(
        &self,
        session: &SilentSession,
        run: &SilentSessionRun,
    ) -> Result<(), SilentSessionInvariantError> {
        if self.schema != SILENT_SESSION_EVENT_SCHEMA {
            return Err(SilentSessionInvariantError::UnsupportedSchema);
        }
        if !self.event_id.is_uuid_v7() {
            return Err(SilentSessionInvariantError::NonV7Identity("event_id"));
        }
        if self.session_id != session.session_id || self.run_id != run.run_id {
            return Err(SilentSessionInvariantError::ScopeMutation);
        }
        if self.seq == 0 {
            return Err(SilentSessionInvariantError::InvalidSequence);
        }
        Ok(())
    }
}

impl SilentSessionCheckpoint {
    pub fn validate(
        &self,
        session: &SilentSession,
        run: &SilentSessionRun,
    ) -> Result<(), SilentSessionInvariantError> {
        if self.schema != SILENT_SESSION_CHECKPOINT_SCHEMA {
            return Err(SilentSessionInvariantError::UnsupportedSchema);
        }
        if !self.checkpoint_id.is_uuid_v7() {
            return Err(SilentSessionInvariantError::NonV7Identity("checkpoint_id"));
        }
        if self.session_id != session.session_id || self.run_id != run.run_id {
            return Err(SilentSessionInvariantError::ScopeMutation);
        }
        Ok(())
    }
}

impl SilentSessionLease {
    pub fn validate(&self, session: &SilentSession) -> Result<(), SilentSessionInvariantError> {
        if self.schema != SILENT_SESSION_LEASE_SCHEMA {
            return Err(SilentSessionInvariantError::UnsupportedSchema);
        }
        if !self.lease_id.is_uuid_v7() {
            return Err(SilentSessionInvariantError::NonV7Identity("lease_id"));
        }
        if self.session_id != session.session_id
            || self.project_root != session.project_root
            || self.project_identity_ref != session.project_identity_ref
            || self.continuity_id != session.continuity_id
        {
            return Err(SilentSessionInvariantError::ScopeMutation);
        }
        if self.workspace_ref.trim().is_empty()
            || self.writer_role.trim().is_empty()
            || self.owner_actor_instance_ref.trim().is_empty()
            || self.path_intents.iter().any(|path| {
                path.as_os_str().is_empty()
                    || path.components().any(|component| {
                        matches!(
                            component,
                            Component::ParentDir | Component::RootDir | Component::Prefix(_)
                        )
                    })
            })
            || self.fencing_token == 0
            || self.heartbeat_at < self.acquired_at
            || self.expires_at <= self.heartbeat_at
        {
            return Err(SilentSessionInvariantError::InvalidLease);
        }
        Ok(())
    }
}

impl SilentSessionCompletionEvaluation {
    pub fn validate(
        &self,
        session: &SilentSession,
        run: &SilentSessionRun,
    ) -> Result<(), SilentSessionInvariantError> {
        if self.schema != SILENT_SESSION_COMPLETION_SCHEMA {
            return Err(SilentSessionInvariantError::UnsupportedSchema);
        }
        if !self.evaluation_id.is_uuid_v7() {
            return Err(SilentSessionInvariantError::NonV7Identity("evaluation_id"));
        }
        if self.session_id != session.session_id || self.run_id != run.run_id {
            return Err(SilentSessionInvariantError::ScopeMutation);
        }
        if self.decision == CompletionDecision::Completed && !self.receipt_ready {
            return Err(SilentSessionInvariantError::MissingField(
                "completion_receipt",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config(root: &str) -> SilentSessionConfig {
        SilentSessionConfig {
            schema: SILENT_SESSION_CONFIG_SCHEMA.into(),
            identity: SilentSessionIdentityConfig {
                display_name: "worker".into(),
                project_root: PathBuf::from(root),
                project_identity_ref: "project:focusa".into(),
                continuity_id: "workloop-completion".into(),
                work_item_ref: Some("focusa-a6yq6.2.1".into()),
                mission: "implement domain types".into(),
                agent_identity_ref: "agent:pi".into(),
                role_profile_ref: "role:implementer".into(),
            },
            harness: HarnessConfig {
                kind: HarnessKind::Pi,
                adapter_version: "1".into(),
                native_resume_policy: NativeResumePolicy::Prefer,
            },
            model: SilentSessionModelConfig {
                requested: ModelBinding {
                    provider: "test".into(),
                    model: "test-model".into(),
                    thinking: None,
                },
                selection_policy: ModelSelectionPolicy::Exact,
                fallback_policy: ModelFallbackPolicy::Disabled,
                allowed_fallbacks: vec![],
                auth_profile_ref: "auth:test".into(),
                require_entitlement_preflight: true,
                require_runtime_model_confirmation: true,
            },
            workspace: WorkspaceConfig {
                strategy: WorkspaceStrategy::IsolatedWorktree,
                source_root: PathBuf::from(root),
                worktree_root: Some(PathBuf::from(format!("{root}-session"))),
                base_ref: Some("HEAD".into()),
                branch_name: Some("focusa/silent/test/item".into()),
                integration_policy: IntegrationPolicy::Manual,
            },
            bootstrap_target_profile: "rules_and_context".into(),
            bootstrap_packet_mode: "session_start".into(),
            bootstrap_verification_required: true,
            supervision: SupervisionConfig {
                restart_policy: "bounded".into(),
                max_process_restarts: 2,
                max_transport_retries: 3,
                retry_backoff_ms: 1000,
                retry_budgets: default_retry_budgets(),
                soft_pause_timeout_ms: 5000,
                graceful_stop_timeout_ms: 5000,
                checkpoint_interval_seconds: 30,
                checkpoint_event_interval: 100,
                waiting_input_timeout_seconds: 300,
                silent_output_warning_seconds: 60,
            },
            resources: ResourceLimits {
                priority: 0,
                max_wall_clock_seconds: Some(3600),
                max_cpu_percent: Some(100.0),
                max_memory_bytes: Some(1_000_000_000),
                max_pids: Some(64),
                max_disk_bytes: Some(1_000_000_000),
                max_output_bytes: Some(10_000_000),
                max_tokens: Some(100_000),
                max_cost_usd: Some(10.0),
                max_turns: Some(100),
            },
            output: OutputPolicy {
                persist_stdout: true,
                persist_stderr: true,
                persist_semantic_events: true,
                chunk_max_bytes: 65_536,
                chunk_max_seconds: 5,
                redaction_profile_ref: "redaction:default".into(),
                operator_projection_budget: 8_000,
                raw_retention_policy_ref: "retention:default".into(),
            },
            governance: GovernancePolicy {
                context_authority_required: true,
                risky_mutation_preflight_required: true,
                destructive_actions_allowed: false,
                writer_lease_required: true,
                completion_receipt_required: true,
                evidence_policy_ref: "evidence:default".into(),
                policy_locks: vec![],
            },
            notifications: NotificationPolicy {
                waiting_input: true,
                blocked: true,
                failed: true,
                completed: true,
                model_mismatch: true,
                budget_pressure: true,
                channels: vec![],
            },
            retention: RetentionConfig {
                policy_ref: "retention:default".into(),
                evidence_hold: true,
            },
        }
    }

    fn sample_session() -> SilentSession {
        SilentSession {
            schema: SILENT_SESSION_SCHEMA.into(),
            versions: SilentSessionVersions::default(),
            session_id: SilentSessionId::new(),
            display_name: "worker".into(),
            created_at: Utc::now(),
            created_by_actor_ref: "actor:pi".into(),
            operator_principal_ref: "operator:test".into(),
            os_execution_user: "test".into(),
            project_root: crate::test_support::absolute_path("silent-session-project"),
            project_identity_ref: "project:focusa".into(),
            continuity_id: "workloop-completion".into(),
            trajectory_ref: None,
            workpoint_ref: None,
            work_item_ref: Some("focusa-a6yq6.2.1".into()),
            operator_ask: OperatorAskBinding::capture(
                "ask:domain-test",
                "implement domain types",
                1,
                Utc::now(),
            ),
            mission: "implement domain types".into(),
            lifecycle_state: SilentSessionLifecycleState::Draft,
            health: SilentSessionHealth::Unknown,
            semantic_observation: None,
            active_run_id: None,
            config_revision_id: SilentSessionConfigRevisionId::new(),
            writer_lease_ref: None,
            retention_policy_ref: "retention:default".into(),
            receipt_refs: vec![],
        }
    }

    fn sample_run(session: &SilentSession) -> SilentSessionRun {
        let model = ModelBinding {
            provider: "test".into(),
            model: "test-model".into(),
            thinking: None,
        };
        SilentSessionRun {
            schema: SILENT_SESSION_RUN_SCHEMA.into(),
            versions: SilentSessionVersions::default(),
            run_id: SilentSessionRunId::new(),
            session_id: session.session_id,
            generation: 1,
            runner_id: "runner:one".into(),
            adapter_id: "pi_rpc".into(),
            process_backend_id: "native".into(),
            requested_model_binding: model.clone(),
            effective_model_binding: Some(model),
            observed_model_binding: None,
            workspace_binding: WorkspaceBinding {
                workspace_id: "workspace:test".into(),
                root: session.project_root.clone(),
                strategy: WorkspaceStrategy::IsolatedWorktree,
                branch_ref: None,
            },
            process_identity: None,
            harness_native_session_ref: None,
            started_at: None,
            ended_at: None,
            exit_status: None,
            current_event_seq: 0,
            output_stream_refs: vec![],
            runtime_checkpoint_refs: vec![],
            workpoint_checkpoint_refs: vec![],
        }
    }

    #[test]
    fn identities_are_uuid_v7_and_round_trip() {
        let session = sample_session();
        assert!(session.session_id.is_uuid_v7());
        assert_eq!(session.versions.daemon_runner_protocol_version, 1);
        let encoded = serde_json::to_string(&session).unwrap();
        let decoded: SilentSession = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, session);
        assert!(decoded.validate().is_ok());
    }

    #[test]
    fn lifecycle_health_and_activity_are_orthogonal() {
        let mut session = sample_session();
        session.lifecycle_state = SilentSessionLifecycleState::WaitingInput;
        session.health = SilentSessionHealth::TransportLost;
        session.semantic_observation = Some(SemanticObservation {
            activity: SilentSessionSemanticActivity::WaitingForOperator,
            source: "pi_rpc".into(),
            confidence: 0.99,
            observed_at: Utc::now(),
            fresh_until: Utc::now() + chrono::Duration::seconds(30),
            provenance: ObservationProvenance::RuntimeObserved,
        });
        assert!(session.validate().is_ok());
    }

    #[test]
    fn config_edit_cannot_change_project_or_continuity_scope() {
        let session = sample_session();
        let project_root = crate::test_support::absolute_path_string("silent-session-project");
        let mut config = sample_config(&project_root);
        assert!(session.scope_matches_config(&config));
        config.identity.continuity_id = "other".into();
        assert!(!session.scope_matches_config(&config));
        let revision = SilentSessionConfigRevision {
            schema: SILENT_SESSION_CONFIG_REVISION_SCHEMA.into(),
            revision_id: SilentSessionConfigRevisionId::new(),
            session_id: session.session_id,
            parent_revision_id: None,
            config,
            requested_changes: Value::Null,
            effective_diff: Value::Null,
            field_provenance: BTreeMap::new(),
            policy_lock_results: BTreeMap::new(),
            operator_approval_ref: None,
            validation_result: ConfigValidationResult {
                valid: true,
                errors: vec![],
                warnings: vec![],
            },
            applied_at: None,
            rollback_target: None,
        };
        assert_eq!(
            revision.validate_for_session(&session),
            Err(SilentSessionInvariantError::ScopeMutation)
        );
    }

    #[test]
    fn run_generation_is_new_identity_under_stable_session() {
        let session = sample_session();
        let workspace = WorkspaceBinding {
            workspace_id: "workspace:test".into(),
            root: session.project_root.clone(),
            strategy: WorkspaceStrategy::IsolatedWorktree,
            branch_ref: None,
        };
        let model = ModelBinding {
            provider: "test".into(),
            model: "test-model".into(),
            thinking: None,
        };
        let first = SilentSessionRun {
            schema: SILENT_SESSION_RUN_SCHEMA.into(),
            versions: SilentSessionVersions::default(),
            run_id: SilentSessionRunId::new(),
            session_id: session.session_id,
            generation: 1,
            runner_id: "runner:one".into(),
            adapter_id: "pi_rpc".into(),
            process_backend_id: "native".into(),
            requested_model_binding: model.clone(),
            effective_model_binding: Some(model.clone()),
            observed_model_binding: None,
            workspace_binding: workspace.clone(),
            process_identity: None,
            harness_native_session_ref: None,
            started_at: None,
            ended_at: None,
            exit_status: None,
            current_event_seq: 0,
            output_stream_refs: vec![],
            runtime_checkpoint_refs: vec![],
            workpoint_checkpoint_refs: vec![],
        };
        let mut second = first.clone();
        second.run_id = SilentSessionRunId::new();
        second.generation = 2;
        assert_ne!(first.run_id, second.run_id);
        assert_eq!(first.session_id, second.session_id);
        assert!(first.validate(&session).is_ok());
        assert!(second.validate(&session).is_ok());
    }

    #[test]
    fn lease_and_completion_enforce_scope_fencing_and_receipt_truth() {
        let session = sample_session();
        let run = sample_run(&session);
        let now = Utc::now();
        let mut lease = SilentSessionLease {
            schema: SILENT_SESSION_LEASE_SCHEMA.into(),
            lease_id: SilentSessionLeaseId::new(),
            session_id: session.session_id,
            project_root: session.project_root.clone(),
            project_identity_ref: session.project_identity_ref.clone(),
            continuity_id: session.continuity_id.clone(),
            work_item_ref: session.work_item_ref.clone(),
            workspace_ref: "workspace:test".into(),
            path_intents: vec![],
            mutation_mode: WriterMutationMode::ExclusiveExisting,
            writer_role: "implementer".into(),
            owner_actor_instance_ref: "actor:pi".into(),
            fencing_token: 1,
            acquired_at: now,
            heartbeat_at: now,
            expires_at: now + chrono::Duration::seconds(30),
            adoption_policy: "signed_match_only".into(),
        };
        assert!(lease.validate(&session).is_ok());
        lease.continuity_id = "other".into();
        assert_eq!(
            lease.validate(&session),
            Err(SilentSessionInvariantError::ScopeMutation)
        );

        let evaluation = SilentSessionCompletionEvaluation {
            schema: SILENT_SESSION_COMPLETION_SCHEMA.into(),
            evaluation_id: SilentSessionCompletionEvaluationId::new(),
            session_id: session.session_id,
            run_id: run.run_id,
            evaluated_at: now,
            process_result: Value::Null,
            workpoint_status: "completed".into(),
            work_item_acceptance: BTreeMap::new(),
            evidence_classes: vec![],
            test_results: vec![],
            diff_refs: vec![],
            commit_refs: vec![],
            unresolved_blockers: vec![],
            adversarial_verifier_verdict: None,
            receipt_ready: false,
            decision: CompletionDecision::Completed,
        };
        assert_eq!(
            evaluation.validate(&session, &run),
            Err(SilentSessionInvariantError::MissingField(
                "completion_receipt"
            ))
        );
    }

    #[test]
    fn runtime_checkpoint_is_distinct_from_canonical_workpoint_checkpoint() {
        let runtime = SilentSessionCheckpointBody::Runtime {
            process_position: Value::Null,
            protocol_position: Value::Null,
            stream_cursor: "run:1".into(),
            harness_session_ref: None,
            resource_counters: BTreeMap::new(),
            retry_state: Value::Null,
        };
        let workpoint = SilentSessionCheckpointBody::CanonicalWorkpoint {
            workpoint_ref: WorkpointBinding {
                workpoint_id: "wp-1".into(),
                revision: Some(2),
            },
            mission: "mission".into(),
            action_intent: "verify".into(),
            active_objects: vec![],
            blockers: vec![],
            verified_evidence: vec![],
            next_slice: "next".into(),
            do_not_drift: vec![],
        };
        assert_ne!(
            serde_json::to_value(runtime).unwrap(),
            serde_json::to_value(workpoint).unwrap()
        );
    }
}
