use serde::{Deserialize, Serialize};

pub const SILENT_SESSION_CONFIG_SCHEMA_V1: &str = "focusa.silent_session_config.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SilentSessionConfig {
    pub schema: String,
    pub identity: IdentityConfig,
    pub harness: HarnessConfig,
    pub model: ModelConfig,
    pub workspace: WorkspaceConfig,
    pub bootstrap: BootstrapConfig,
    pub supervision: SupervisionConfig,
    pub resources: ResourceConfig,
    pub output: OutputConfig,
    pub governance: GovernanceConfig,
    pub notifications: NotificationConfig,
    pub retention: RetentionConfig,
}

impl SilentSessionConfig {
    pub fn new(identity: IdentityConfig, harness: HarnessConfig, model: ModelConfig) -> Self {
        Self {
            schema: SILENT_SESSION_CONFIG_SCHEMA_V1.into(),
            identity,
            harness,
            model,
            workspace: WorkspaceConfig::default(),
            bootstrap: BootstrapConfig::default(),
            supervision: SupervisionConfig::default(),
            resources: ResourceConfig::default(),
            output: OutputConfig::default(),
            governance: GovernanceConfig::default(),
            notifications: NotificationConfig::default(),
            retention: RetentionConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentityConfig {
    pub display_name: String,
    pub project_root: String,
    pub continuity_id: String,
    pub work_item_ref: Option<String>,
    pub mission: String,
    pub agent_identity_ref: String,
    pub role_profile_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HarnessConfig {
    pub kind: HarnessKind,
    pub adapter_version: String,
    pub native_resume_policy: NativeResumePolicy,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HarnessKind {
    Pi,
    Codex,
    Claude,
    Opencode,
    GenericRpc,
    GenericPty,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NativeResumePolicy {
    Prefer,
    Require,
    Disable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelConfig {
    pub provider: String,
    pub model: String,
    pub thinking: Option<String>,
    pub selection_policy: ModelSelectionPolicy,
    pub fallback_policy: ModelFallbackPolicy,
    pub allowed_fallbacks: Vec<ModelRef>,
    pub auth_profile_ref: String,
    pub require_entitlement_preflight: bool,
    pub require_runtime_model_confirmation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelRef {
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelSelectionPolicy {
    Exact,
    AllowList,
    Adaptive,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModelFallbackPolicy {
    Disabled,
    ExplicitAllowList,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceConfig {
    pub strategy: WorkspaceStrategy,
    pub source_root: Option<String>,
    pub worktree_root: Option<String>,
    pub base_ref: Option<String>,
    pub branch_name: Option<String>,
    pub integration_policy: IntegrationPolicy,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            strategy: WorkspaceStrategy::IsolatedWorktree,
            source_root: None,
            worktree_root: None,
            base_ref: None,
            branch_name: None,
            integration_policy: IntegrationPolicy::Manual,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceStrategy {
    IsolatedWorktree,
    ExclusiveExisting,
    ReadOnlyShared,
    ExplicitShared,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationPolicy {
    Manual,
    VerifiedFastForward,
    GovernedMerge,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BootstrapConfig {
    pub target_profile: String,
    pub packet_mode: String,
    pub verification_required: bool,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            target_profile: "rules_and_context".into(),
            packet_mode: "session_start".into(),
            verification_required: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupervisionConfig {
    pub restart_policy: String,
    pub max_process_restarts: u32,
    pub max_transport_retries: u32,
    pub retry_backoff_seconds: u64,
    pub soft_pause_timeout_seconds: u64,
    pub graceful_stop_timeout_seconds: u64,
    pub checkpoint_interval_seconds: u64,
    pub checkpoint_event_interval: u64,
    pub waiting_input_timeout_seconds: u64,
    pub silent_output_warning_seconds: u64,
}

impl Default for SupervisionConfig {
    fn default() -> Self {
        Self {
            restart_policy: "on_failure".into(),
            max_process_restarts: 3,
            max_transport_retries: 5,
            retry_backoff_seconds: 2,
            soft_pause_timeout_seconds: 30,
            graceful_stop_timeout_seconds: 30,
            checkpoint_interval_seconds: 300,
            checkpoint_event_interval: 250,
            waiting_input_timeout_seconds: 900,
            silent_output_warning_seconds: 300,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ResourceConfig {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutputConfig {
    pub persist_stdout: bool,
    pub persist_stderr: bool,
    pub persist_semantic_events: bool,
    pub chunk_max_bytes: u64,
    pub chunk_max_seconds: u64,
    pub redaction_profile_ref: String,
    pub operator_projection_budget: u64,
    pub raw_retention_policy_ref: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            persist_stdout: true,
            persist_stderr: true,
            persist_semantic_events: true,
            chunk_max_bytes: 1_048_576,
            chunk_max_seconds: 60,
            redaction_profile_ref: "default".into(),
            operator_projection_budget: 4_096,
            raw_retention_policy_ref: "raw-default".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernanceConfig {
    pub context_authority_required: bool,
    pub risky_mutation_preflight_required: bool,
    pub destructive_actions_allowed: bool,
    pub writer_lease_required: bool,
    pub completion_receipt_required: bool,
    pub evidence_policy_ref: String,
    pub policy_locks: Vec<String>,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            context_authority_required: true,
            risky_mutation_preflight_required: true,
            destructive_actions_allowed: false,
            writer_lease_required: true,
            completion_receipt_required: true,
            evidence_policy_ref: "required".into(),
            policy_locks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotificationConfig {
    pub waiting_input: bool,
    pub blocked: bool,
    pub failed: bool,
    pub completed: bool,
    pub model_mismatch: bool,
    pub budget_pressure: bool,
    pub channels: Vec<String>,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            waiting_input: true,
            blocked: true,
            failed: true,
            completed: true,
            model_mismatch: true,
            budget_pressure: true,
            channels: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetentionConfig {
    pub policy_ref: String,
    pub evidence_hold: bool,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            policy_ref: "default".into(),
            evidence_hold: false,
        }
    }
}
