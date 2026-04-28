//! Canonical shared types for Focusa.
//!
//! Source: core-reducer.md, 00-glossary.md, 03-focus-stack.md,
//!         04-focus-gate.md, 06-focus-state.md, 07-reference-store.md,
//!         G1-09-memory.md
//!
//! INVARIANT: If a cognition change cannot be expressed as a reducer event,
//!            it does not belong in Focusa.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Identifiers ────────────────────────────────────────────────────────────

/// Frame identifier (UUIDv7 preferred).
pub type FrameId = Uuid;

/// Session identifier.
pub type SessionId = Uuid;

/// Signal identifier.
pub type SignalId = Uuid;

/// Candidate identifier.
pub type CandidateId = Uuid;

/// Handle identifier (UUIDv7 for uniqueness, sha256 stored in metadata).
pub type HandleId = Uuid;

/// Artifact identifier.
pub type ArtifactId = Uuid;

/// Project-level autonomous execution run identifier.
pub type ProjectRunId = Uuid;
/// Tranche-level autonomous execution run identifier.
pub type TrancheRunId = Uuid;
/// Task-level autonomous execution run identifier.
pub type TaskRunId = Uuid;
/// Checkpoint identifier for continuous work recovery.
pub type CheckpointId = Uuid;
/// Workpoint identifier for Spec88 continuity checkpoints.
pub type WorkpointId = Uuid;

// ─── Continuous Work Loop (spec 79) ─────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkLoopStatus {
    #[default]
    Idle,
    SelectingReadyWork,
    PreparingTurn,
    AwaitingHarnessTurn,
    EvaluatingOutcome,
    AdvancingTask,
    Paused,
    Blocked,
    Completed,
    Aborted,
    TransportDegraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkLoopPreset {
    Conservative,
    #[default]
    Balanced,
    Push,
    Audit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskClass {
    #[default]
    Unknown,
    Code,
    Refactor,
    DocSpec,
    Architecture,
    Integration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkItemLifecycle {
    #[default]
    Ready,
    InProgress,
    Deferred,
    Blocked,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BlockerClass {
    #[default]
    Unknown,
    Tooling,
    Environment,
    Dependency,
    SpecGap,
    Verification,
    Governance,
    Permission,
    Transport,
    ModelQuality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuthorshipMode {
    #[default]
    OperatorOnly,
    Delegated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunIdentityState {
    pub project_run_id: ProjectRunId,
    pub tranche_run_id: Option<TrancheRunId>,
    pub task_run_id: Option<TaskRunId>,
    pub worker_session_id: Option<String>,
    pub last_checkpoint_id: Option<CheckpointId>,
}

impl Default for RunIdentityState {
    fn default() -> Self {
        Self {
            project_run_id: Uuid::now_v7(),
            tranche_run_id: None,
            task_run_id: None,
            worker_session_id: None,
            last_checkpoint_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkerCapabilityProfile {
    pub worker_id: String,
    pub tool_use_supported: bool,
    pub edit_reliable: bool,
    pub structured_output_reliable: bool,
    pub code_generation_strong: bool,
    pub context_window_class: Option<String>,
    pub latency_class: Option<String>,
    pub cost_tier: Option<String>,
    pub fallback_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkLoopPolicy {
    pub preset: WorkLoopPreset,
    pub max_turns: Option<u32>,
    pub max_wall_clock_ms: Option<u64>,
    pub max_retries: u32,
    pub cooldown_ms: u64,
    pub allow_destructive_actions: bool,
    pub require_operator_for_governance: bool,
    pub require_operator_for_scope_change: bool,
    pub require_verification_before_persist: bool,
    pub max_consecutive_low_productivity_turns: u32,
    pub max_consecutive_failures: u32,
    pub auto_pause_on_operator_message: bool,
    pub require_explainable_continue_reason: bool,
    pub max_same_subproblem_retries: u32,
    pub status_heartbeat_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkLoopPolicyOverrides {
    pub max_turns: Option<u32>,
    pub max_wall_clock_ms: Option<u64>,
    pub max_retries: Option<u32>,
    pub cooldown_ms: Option<u64>,
    pub allow_destructive_actions: Option<bool>,
    pub require_operator_for_governance: Option<bool>,
    pub require_operator_for_scope_change: Option<bool>,
    pub require_verification_before_persist: Option<bool>,
    pub max_consecutive_low_productivity_turns: Option<u32>,
    pub max_consecutive_failures: Option<u32>,
    pub auto_pause_on_operator_message: Option<bool>,
    pub require_explainable_continue_reason: Option<bool>,
    pub max_same_subproblem_retries: Option<u32>,
    pub status_heartbeat_ms: Option<u64>,
}

impl WorkLoopPolicy {
    pub fn for_preset(preset: WorkLoopPreset) -> Self {
        match preset {
            WorkLoopPreset::Conservative => Self {
                preset,
                max_turns: Some(6),
                max_wall_clock_ms: Some(15 * 60 * 1_000),
                max_retries: 2,
                cooldown_ms: 2_000,
                allow_destructive_actions: false,
                require_operator_for_governance: true,
                require_operator_for_scope_change: true,
                require_verification_before_persist: true,
                max_consecutive_low_productivity_turns: 2,
                max_consecutive_failures: 2,
                auto_pause_on_operator_message: false,
                require_explainable_continue_reason: true,
                max_same_subproblem_retries: 1,
                status_heartbeat_ms: 3_000,
            },
            WorkLoopPreset::Balanced => Self {
                preset,
                max_turns: Some(12),
                max_wall_clock_ms: Some(30 * 60 * 1_000),
                max_retries: 3,
                cooldown_ms: 1_000,
                allow_destructive_actions: false,
                require_operator_for_governance: true,
                require_operator_for_scope_change: true,
                require_verification_before_persist: true,
                max_consecutive_low_productivity_turns: 3,
                max_consecutive_failures: 3,
                auto_pause_on_operator_message: false,
                require_explainable_continue_reason: true,
                max_same_subproblem_retries: 2,
                status_heartbeat_ms: 5_000,
            },
            WorkLoopPreset::Push => Self {
                preset,
                max_turns: Some(24),
                max_wall_clock_ms: Some(60 * 60 * 1_000),
                max_retries: 4,
                cooldown_ms: 500,
                allow_destructive_actions: false,
                require_operator_for_governance: true,
                require_operator_for_scope_change: true,
                require_verification_before_persist: true,
                max_consecutive_low_productivity_turns: 4,
                max_consecutive_failures: 4,
                auto_pause_on_operator_message: false,
                require_explainable_continue_reason: true,
                max_same_subproblem_retries: 3,
                status_heartbeat_ms: 5_000,
            },
            WorkLoopPreset::Audit => Self {
                preset,
                max_turns: Some(10),
                max_wall_clock_ms: Some(20 * 60 * 1_000),
                max_retries: 2,
                cooldown_ms: 1_500,
                allow_destructive_actions: false,
                require_operator_for_governance: true,
                require_operator_for_scope_change: true,
                require_verification_before_persist: true,
                max_consecutive_low_productivity_turns: 2,
                max_consecutive_failures: 2,
                auto_pause_on_operator_message: false,
                require_explainable_continue_reason: true,
                max_same_subproblem_retries: 1,
                status_heartbeat_ms: 2_000,
            },
        }
    }

    pub fn with_overrides(preset: WorkLoopPreset, overrides: WorkLoopPolicyOverrides) -> Self {
        let mut policy = Self::for_preset(preset);
        if let Some(v) = overrides.max_turns {
            policy.max_turns = Some(v);
        }
        if let Some(v) = overrides.max_wall_clock_ms {
            policy.max_wall_clock_ms = Some(v);
        }
        if let Some(v) = overrides.max_retries {
            policy.max_retries = v;
        }
        if let Some(v) = overrides.cooldown_ms {
            policy.cooldown_ms = v;
        }
        if let Some(v) = overrides.allow_destructive_actions {
            policy.allow_destructive_actions = v;
        }
        if let Some(v) = overrides.require_operator_for_governance {
            policy.require_operator_for_governance = v;
        }
        if let Some(v) = overrides.require_operator_for_scope_change {
            policy.require_operator_for_scope_change = v;
        }
        if let Some(v) = overrides.require_verification_before_persist {
            policy.require_verification_before_persist = v;
        }
        if let Some(v) = overrides.max_consecutive_low_productivity_turns {
            policy.max_consecutive_low_productivity_turns = v;
        }
        if let Some(v) = overrides.max_consecutive_failures {
            policy.max_consecutive_failures = v;
        }
        if let Some(v) = overrides.auto_pause_on_operator_message {
            policy.auto_pause_on_operator_message = v;
        }
        if let Some(v) = overrides.require_explainable_continue_reason {
            policy.require_explainable_continue_reason = v;
        }
        if let Some(v) = overrides.max_same_subproblem_retries {
            policy.max_same_subproblem_retries = v;
        }
        if let Some(v) = overrides.status_heartbeat_ms {
            policy.status_heartbeat_ms = v;
        }
        policy
    }
}

impl Default for WorkLoopPolicy {
    fn default() -> Self {
        Self::for_preset(WorkLoopPreset::Balanced)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpecLinkedTaskPacket {
    pub work_item_id: String,
    pub title: String,
    pub task_class: TaskClass,
    pub linked_spec_refs: Vec<String>,
    pub acceptance_criteria: Vec<String>,
    pub required_verification_tier: Option<String>,
    pub allowed_scope: Vec<String>,
    pub dependencies: Vec<String>,
    pub tranche_id: Option<String>,
    pub blocker_class: Option<BlockerClass>,
    pub checkpoint_summary: Option<String>,
}

impl SpecLinkedTaskPacket {
    pub fn has_authoritative_grounding(&self) -> bool {
        !self.linked_spec_refs.is_empty()
    }

    pub fn has_acceptance_criteria(&self) -> bool {
        !self.acceptance_criteria.is_empty()
    }

    pub fn requires_verification(&self) -> bool {
        self.required_verification_tier.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DelegatedAuthorshipState {
    pub delegate_id: String,
    pub scope: String,
    pub amendment_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkLoopPauseFlags {
    pub destructive_confirmation_required: bool,
    pub governance_decision_pending: bool,
    pub operator_override_active: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkLoopDecisionContext {
    pub current_ask: Option<String>,
    pub ask_kind: Option<String>,
    pub scope_kind: Option<String>,
    pub carryover_policy: Option<String>,
    pub excluded_context_reason: Option<String>,
    pub excluded_context_labels: Vec<String>,
    pub source_turn_id: Option<String>,
    pub operator_steering_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkLoopState {
    pub enabled: bool,
    pub status: WorkLoopStatus,
    pub authorship_mode: AuthorshipMode,
    pub policy: WorkLoopPolicy,
    pub run: RunIdentityState,
    pub current_task: Option<SpecLinkedTaskPacket>,
    pub last_completed_task_id: Option<String>,
    pub last_recorded_bd_transition_id: Option<String>,
    pub last_blocker_class: Option<BlockerClass>,
    pub last_blocker_reason: Option<String>,
    pub last_continue_reason: Option<String>,
    pub last_observed_summary: Option<String>,
    pub last_safe_reentry_prompt_basis: Option<String>,
    pub restored_context_summary: Option<String>,
    pub transport_adapter: Option<String>,
    pub transport_session_state: Option<String>,
    pub last_transport_event_kind: Option<String>,
    pub last_transport_event_summary: Option<String>,
    pub last_transport_event_sequence: u64,
    pub transport_abort_reason: Option<String>,
    pub enabled_at: Option<DateTime<Utc>>,
    pub last_turn_requested_at: Option<DateTime<Utc>>,
    pub turn_count: u32,
    pub consecutive_failures_for_task_class: u32,
    pub consecutive_low_productivity_turns: u32,
    pub consecutive_same_work_item_retries: u32,
    pub last_observed_work_item_id: Option<String>,
    pub pause_flags: WorkLoopPauseFlags,
    pub decision_context: WorkLoopDecisionContext,
    pub pending_proposals_requiring_resolution: usize,
    pub next_work_risk_class: Option<String>,
    pub current_autonomy_level: Option<AutonomyLevel>,
    pub delegated_authorship: Option<DelegatedAuthorshipState>,
    pub active_worker: Option<WorkerCapabilityProfile>,
}

// ─── Canonical State (from core-reducer.md) ─────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OntologyProposalRecord {
    pub proposal_id: Uuid,
    pub proposal_kind: String,
    pub target_class: String,
    pub status: String,
    pub source: Option<String>,
    pub object_type: Option<String>,
    pub object_id: Option<String>,
    pub link_type: Option<String>,
    pub source_id: Option<String>,
    pub target_id: Option<String>,
    pub notes: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OntologyVerificationRecord {
    pub proposal_id: Option<Uuid>,
    pub verification: String,
    pub outcome: String,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OntologyWorkingSetRefreshRecord {
    pub scope: String,
    pub reason: String,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OntologyDeltaRecord {
    pub delta_kind: String,
    pub payload: serde_json::Value,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OntologyState {
    #[serde(default)]
    pub objects: Vec<serde_json::Value>,
    #[serde(default)]
    pub links: Vec<serde_json::Value>,
    #[serde(default)]
    pub proposals: Vec<OntologyProposalRecord>,
    #[serde(default)]
    pub verifications: Vec<OntologyVerificationRecord>,
    #[serde(default)]
    pub working_set_refreshes: Vec<OntologyWorkingSetRefreshRecord>,
    #[serde(default)]
    pub delta_log: Vec<OntologyDeltaRecord>,
}


// ─── Workpoint Continuity (Spec88) ─────────────────────────────────────────

pub mod workpoint_caps {
    pub const RECORDS: usize = 32;
    pub const OBJECT_REFS: usize = 32;
    pub const VERIFICATIONS: usize = 24;
    pub const BLOCKERS: usize = 16;
    pub const RESUME_EVENTS: usize = 24;
    pub const DRIFT_EVENTS: usize = 32;
    pub const DEGRADED_FALLBACKS: usize = 16;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkpointStatus {
    #[default]
    Proposed,
    Active,
    Superseded,
    Rejected,
    DegradedFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkpointCheckpointReason {
    SessionStart,
    SessionResume,
    BeforeCompact,
    AfterCompact,
    ContextOverflow,
    ModelSwitch,
    Fork,
    Manual,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkpointConfidence {
    Low,
    #[default]
    Medium,
    High,
    Verified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkpointDriftSeverity {
    Info,
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkpointActionIntentRecord {
    pub action_type: String,
    pub target_ref: Option<String>,
    #[serde(default)]
    pub verification_hooks: Vec<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkpointVerificationRecord {
    pub target_ref: String,
    pub result: String,
    pub evidence_ref: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkpointBlockerRecord {
    pub reason: String,
    pub severity: Option<String>,
    pub target_ref: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkpointRecord {
    pub workpoint_id: WorkpointId,
    pub work_item_id: Option<String>,
    pub session_id: Option<String>,
    pub frame_id: Option<FrameId>,
    pub status: WorkpointStatus,
    pub checkpoint_reason: WorkpointCheckpointReason,
    pub confidence: WorkpointConfidence,
    pub canonical: bool,
    pub mission: Option<String>,
    #[serde(default)]
    pub active_object_refs: Vec<String>,
    pub action_intent: Option<WorkpointActionIntentRecord>,
    #[serde(default)]
    pub verification_records: Vec<WorkpointVerificationRecord>,
    #[serde(default)]
    pub blockers: Vec<WorkpointBlockerRecord>,
    pub next_slice: Option<String>,
    pub source_turn_id: Option<String>,
    pub supersedes: Option<WorkpointId>,
    pub rejection_reason: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Default for WorkpointRecord {
    fn default() -> Self {
        Self {
            workpoint_id: Uuid::now_v7(),
            work_item_id: None,
            session_id: None,
            frame_id: None,
            status: WorkpointStatus::Proposed,
            checkpoint_reason: WorkpointCheckpointReason::Unknown,
            confidence: WorkpointConfidence::Medium,
            canonical: false,
            mission: None,
            active_object_refs: vec![],
            action_intent: None,
            verification_records: vec![],
            blockers: vec![],
            next_slice: None,
            source_turn_id: None,
            supersedes: None,
            rejection_reason: None,
            created_at: None,
            updated_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkpointResumeRenderRecord {
    pub workpoint_id: Option<WorkpointId>,
    pub mode: String,
    pub rendered_summary: String,
    pub rendered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkpointDriftRecord {
    pub workpoint_id: Option<WorkpointId>,
    pub severity: WorkpointDriftSeverity,
    pub reason: String,
    pub recovery_hint: Option<String>,
    pub detected_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkpointDegradedFallbackRecord {
    pub workpoint_id: WorkpointId,
    pub reason: String,
    pub packet: serde_json::Value,
    pub recorded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkpointState {
    pub active_workpoint_id: Option<WorkpointId>,
    #[serde(default)]
    pub records: Vec<WorkpointRecord>,
    #[serde(default)]
    pub resume_events: Vec<WorkpointResumeRenderRecord>,
    #[serde(default)]
    pub drift_events: Vec<WorkpointDriftRecord>,
    #[serde(default)]
    pub degraded_fallbacks: Vec<WorkpointDegradedFallbackRecord>,
}

/// The complete cognitive state of a Focusa instance.
///
/// INVARIANT: Conversation history is NEVER part of FocusaState.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusaState {
    pub session: Option<SessionState>,
    pub focus_stack: FocusStackState,
    pub focus_gate: FocusGateState,
    pub reference_index: ReferenceIndex,
    pub memory: ExplicitMemory,
    pub clt: CltState,
    pub uxp: UxpProfile,
    pub ufi: UfiState,
    pub autonomy: AutonomyState,
    pub constitution: ConstitutionState,
    pub telemetry: TelemetryState,
    pub rfm: RfmState,
    pub pre: PreState,
    #[serde(default)]
    pub ontology: OntologyState,
    #[serde(default)]
    pub workpoint: WorkpointState,
    pub contribution: ContributionState,
    /// Canonical continuous work loop state (spec 79).
    #[serde(default)]
    pub work_loop: WorkLoopState,

    /// Runtime reality (docs/40): instances, sessions, attachments.
    pub instances: Vec<Instance>,
    pub attachments: Vec<Attachment>,

    /// Thread index for ownership enforcement (docs/43 Policy #5).
    /// Threads are the unit of ownership; each has an owner_machine_id.
    pub threads: Vec<Thread>,

    /// Active turn from Mode A adapter (if any).
    pub active_turn: Option<ActiveTurn>,
    /// Anticipated context from DEEP PATH pre-turn enrichment (§11.7).
    /// Populated after each turn by LLM predicting next user query.
    /// Used by next turn's pre-enrichment before Mem0/Wiki queries.
    #[serde(default)]
    pub anticipated_context: Vec<String>,
    /// Monotonic version — incremented on every successful reduction.
    pub version: u64,
}

impl FocusaState {
    /// Create a new empty state for a fresh session.
    pub fn new() -> Self {
        Self {
            session: None,
            focus_stack: FocusStackState::default(),
            focus_gate: FocusGateState::default(),
            reference_index: ReferenceIndex::default(),
            memory: ExplicitMemory::default(),
            clt: CltState::default(),
            uxp: UxpProfile::default(),
            ufi: UfiState::default(),
            autonomy: AutonomyState::default(),
            constitution: ConstitutionState::default(),
            telemetry: TelemetryState::default(),
            rfm: RfmState::default(),
            pre: PreState::default(),
            ontology: OntologyState::default(),
            workpoint: WorkpointState::default(),
            contribution: ContributionState::default(),
            work_loop: WorkLoopState::default(),
            instances: vec![],
            attachments: vec![],
            threads: vec![],
            active_turn: None,
            anticipated_context: vec![],
            version: 0,
        }
    }
}

impl Default for FocusaState {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Session ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: SessionId,
    pub created_at: DateTime<Utc>,
    pub adapter_id: Option<String>,
    pub workspace_id: Option<String>,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Closed,
}

// ─── Focus Stack (from 03-focus-stack.md) ───────────────────────────────────

/// Hierarchical Execution Context — models nested work.
///
/// INVARIANT: Exactly one active Focus Frame exists at any time.
/// INVARIANT: Every Focus Frame maps to a Beads issue.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FocusStackState {
    pub root_id: Option<FrameId>,
    pub active_id: Option<FrameId>,
    pub frames: Vec<FrameRecord>,
    /// Derived, cached for fast reads: frame IDs from root to active.
    pub stack_path_cache: Vec<FrameId>,
    /// Monotonic; increments on mutation.
    pub version: u64,
}

/// A single unit of focused work.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameRecord {
    pub id: FrameId,
    pub parent_id: Option<FrameId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: FrameStatus,
    /// Short title.
    pub title: String,
    /// One sentence goal.
    pub goal: String,
    /// Beads issue ID (required).
    pub beads_issue_id: String,
    pub tags: Vec<String>,
    pub priority_hint: Option<String>,
    pub ascc_checkpoint_id: Option<String>,
    pub stats: FrameStats,
    pub constraints: Vec<String>,
    /// The frame's current cognitive state (updated incrementally via deltas).
    pub focus_state: FocusState,
    /// When the frame was completed (G1-detail-05 UPDATE §Completion Semantics).
    #[serde(default)]
    pub completed_at: Option<DateTime<Utc>>,
    /// Why the frame was completed (G1-detail-05 UPDATE §Completion Semantics).
    #[serde(default)]
    pub completion_reason: Option<CompletionReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameStatus {
    Active,
    Paused,
    Completed,
    Archived,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrameStats {
    pub turn_count: u64,
    pub last_turn_id: Option<String>,
    pub last_token_estimate: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompletionReason {
    GoalAchieved,
    Blocked,
    Abandoned,
    Superseded,
    Error,
}

// ─── Focus State (from 06-focus-state.md) ───────────────────────────────────

/// The system's current state of mind.
///
/// INVARIANT: Focus State is explicit and structured.
/// INVARIANT: Focus State is deterministic.
/// INVARIANT: Focus State is incrementally updated.
/// INVARIANT: Focus State is injected every turn.
/// INVARIANT: Focus State never inferred implicitly.
/// 10 canonical ASCC slots per G1-07-ascc.md.
/// All slots always exist (may be empty, never absent).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FocusState {
    /// Slot 1: Current intent.
    pub intent: String,
    /// Slot 2: Current focus / state description.
    #[serde(alias = "current_focus")]
    pub current_state: String,
    /// Slot 3: Decisions made (cap 30).
    pub decisions: Vec<String>,
    /// Slot 4: Artifact references (cap 50).
    pub artifacts: Vec<ArtifactLine>,
    /// Slot 5: Active constraints (cap 30).
    pub constraints: Vec<String>,
    /// Slot 6: Open questions (cap 20).
    pub open_questions: Vec<String>,
    /// Slot 7: Next steps (cap 15).
    pub next_steps: Vec<String>,
    /// Slot 8: Recent results (cap 10, newest-first).
    pub recent_results: Vec<String>,
    /// Slot 9: Failures (cap 20).
    pub failures: Vec<String>,
    /// Slot 10: Freeform notes (cap 20).
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactLine {
    pub kind: ArtifactLineKind,
    pub label: String,
    pub handle_ref: Option<HandleRef>,
    pub path_or_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactLineKind {
    File,
    Diff,
    Log,
    Url,
    Handle,
    Other,
}

// ─── Focus Gate (from 04-focus-gate.md) ─────────────────────────────────────

/// Pre-conscious salience filter.
///
/// INVARIANT: Focus Gate never mutates Focus State or Focus Stack.
/// INVARIANT: Focus Gate never triggers actions.
/// INVARIANT: Focus Gate only surfaces candidates.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FocusGateState {
    pub signals: Vec<Signal>,
    pub candidates: Vec<Candidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub id: SignalId,
    pub ts: DateTime<Utc>,
    pub origin: SignalOrigin,
    pub kind: SignalKind,
    pub frame_context: Option<FrameId>,
    /// Short, <= 200 chars.
    pub summary: String,
    pub payload_ref: Option<HandleRef>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalOrigin {
    Adapter,
    Worker,
    Daemon,
    Cli,
    Gui,
    /// Remote sync import (docs/43 Policy #2).
    /// Events from peers are tagged with Sync origin and is_observation=true.
    Sync,
}

/// MVP — 9 signal kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalKind {
    UserInput,
    AssistantOutput,
    ToolOutput,
    Error,
    Warning,
    ArtifactChanged,
    RepeatedPattern,
    DeadlineTick,
    ManualPin,
    /// Per G1-detail-06 UPDATE §Time as First-Class Signal:
    /// Emitted when no user activity for threshold period.
    InactivityTick,
    /// Per G1-detail-06 UPDATE §Time as First-Class Signal:
    /// Emitted when frame open > N minutes.
    LongRunningFrame,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub id: CandidateId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub kind: CandidateKind,
    /// User-facing label.
    pub label: String,
    pub origin_signal_ids: Vec<SignalId>,
    pub related_frame_id: Option<FrameId>,
    pub state: CandidateState,
    /// Internal pressure score.
    pub pressure: f32,
    pub last_seen_at: DateTime<Utc>,
    pub times_seen: u32,
    pub suppressed_until: Option<DateTime<Utc>>,
    pub resolution: Option<String>,
    pub pinned: bool,
}

/// MVP — 5 candidate kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateKind {
    SuggestPushFrame,
    SuggestResumeFrame,
    SuggestCheckArtifact,
    SuggestFixError,
    SuggestPinMemory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateState {
    Latent,
    Surfaced,
    Suppressed,
    Resolved,
}

// ─── Reference Store / ECS (from 07-reference-store.md) ─────────────────────

/// Index of all known handles.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReferenceIndex {
    pub handles: Vec<HandleRef>,
}

/// Prompt-safe handle reference.
///
/// Prompt representation: `[HANDLE:<kind>:<id> "<label>"]`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandleRef {
    pub id: HandleId,
    pub kind: HandleKind,
    /// Short label.
    pub label: String,
    /// Size in bytes.
    pub size: u64,
    /// Hex-encoded SHA-256.
    pub sha256: String,
    pub created_at: DateTime<Utc>,
    pub session_id: Option<SessionId>,
    pub pinned: bool,
}

/// MVP — 7 handle kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandleKind {
    Log,
    Diff,
    Text,
    Json,
    Url,
    FileSnapshot,
    Other,
}

// ─── Memory (from G1-09-memory.md) ──────────────────────────────────────────

/// Explicit, small, user-approved memory.
///
/// INVARIANT: Memory is opt-in.
/// INVARIANT: Never inferred automatically.
/// INVARIANT: No automatic personality drift.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExplicitMemory {
    pub semantic: Vec<SemanticRecord>,
    pub procedural: Vec<RuleRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticRecord {
    pub key: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source: MemorySource,
    pub confidence: f32,
    pub ttl: Option<chrono::Duration>,
    pub tags: Vec<String>,
    pub pinned: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemorySource {
    User,
    Worker,
    Manual,
    Operator,
    Constitution,
    FocusState,
    ContextCore,
    Mem0,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleRecord {
    pub id: String,
    /// Compact imperative rule text.
    pub rule: String,
    pub weight: f32,
    pub reinforced_count: u32,
    pub last_reinforced_at: DateTime<Utc>,
    pub scope: RuleScope,
    pub enabled: bool,
    pub pinned: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleScope {
    Global,
    Frame(FrameId),
    Project(String),
}

// ─── ASCC (from G1-07-ascc.md) ─────────────────────────────────────────────

/// Anchored Structured Context Checkpoint — replaces chat history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointRecord {
    pub frame_id: FrameId,
    pub revision: u64,
    pub updated_at: DateTime<Utc>,
    /// Last processed turn.
    pub anchor_turn_id: String,
    pub sections: AsccSections,
    pub breadcrumbs: Vec<HandleRef>,
}

/// Per-section pinning metadata.
///
/// Per G1-07 UPDATE §Pinning: Any ASCC section may be marked pinned.
/// Pinned sections cannot be dropped during prompt degradation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsccSlotMeta {
    pub pinned: bool,
    pub last_updated_at: Option<DateTime<Utc>>,
}

/// 10 fixed semantic slots with pinning metadata.
///
/// Per G1-07 UPDATE §Pinning: Each slot has pinned: bool and last_updated_at.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsccSections {
    /// 1–3 sentences.
    pub intent: String,
    /// 1–3 sentences.
    pub current_focus: String,
    /// Bullets; each <= 160 chars. Cap: 30.
    pub decisions: Vec<String>,
    /// Typed artifact lines. Cap: 50.
    pub artifacts: Vec<ArtifactLine>,
    /// Short constraints. Cap: 30.
    pub constraints: Vec<String>,
    /// Cap: 20.
    pub open_questions: Vec<String>,
    /// Cap: 15.
    pub next_steps: Vec<String>,
    /// Cap: 10, newest first.
    pub recent_results: Vec<String>,
    /// What failed and why. Cap: 20.
    pub failures: Vec<String>,
    /// Misc, bounded. Cap: 20.
    pub notes: Vec<String>,
    /// Per-section pinning metadata (G1-07 UPDATE §Pinning).
    #[serde(default)]
    pub slot_meta: AsccSlotMetadata,
}

/// Pinning metadata for all ASCC slots.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsccSlotMetadata {
    pub intent: AsccSlotMeta,
    pub current_focus: AsccSlotMeta,
    pub decisions: AsccSlotMeta,
    pub artifacts: AsccSlotMeta,
    pub constraints: AsccSlotMeta,
    pub open_questions: AsccSlotMeta,
    pub next_steps: AsccSlotMeta,
    pub recent_results: AsccSlotMeta,
    pub failures: AsccSlotMeta,
    pub notes: AsccSlotMeta,
}

// ─── Events (from core-reducer.md) ──────────────────────────────────────────

/// Canonical event types (15 total).
///
/// If a cognition change cannot be expressed as one of these,
/// it does not belong in Focusa.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FocusaEvent {
    // Instance lifecycle (multi-device / multi-surface observability)
    InstanceConnected {
        instance_id: Uuid,
        kind: InstanceKind,
    },
    InstanceDisconnected {
        instance_id: Uuid,
        reason: String,
    },

    ThreadAttached {
        instance_id: Uuid,
        session_id: SessionId,
        thread_id: Uuid,
        role: AttachmentRole,
    },
    ThreadDetached {
        instance_id: Uuid,
        session_id: SessionId,
        thread_id: Uuid,
        reason: String,
    },

    ThreadOwnershipTransferred {
        thread_id: Uuid,
        from_machine_id: Option<String>,
        to_machine_id: String,
        reason: String,
    },

    ThreadCreated {
        thread_id: Uuid,
        name: String,
        primary_intent: String,
        owner_machine_id: Option<String>,
    },
    ThreadForked {
        source_thread_id: Uuid,
        thread_id: Uuid,
        name: String,
        owner_machine_id: Option<String>,
    },
    ThreadThesisUpdated {
        thread_id: Uuid,
        thesis: ThreadThesis,
    },

    // Session lifecycle
    SessionStarted {
        session_id: SessionId,
        adapter_id: Option<String>,
        workspace_id: Option<String>,
    },
    SessionRestored {
        session_id: SessionId,
    },
    SessionClosed {
        reason: String,
    },

    // Turn lifecycle (Mode A adapter integration)
    TurnStarted {
        turn_id: TurnId,
        harness_name: String,
        adapter_id: String,
        raw_user_input: Option<String>,
    },
    TurnCompleted {
        turn_id: TurnId,
        harness_name: String,
        raw_user_input: Option<String>,
        assistant_output: Option<String>,
        artifacts_used: Vec<HandleRef>,
        errors: Vec<String>,
        prompt_tokens: Option<u32>,
        completion_tokens: Option<u32>,
    },

    // Continuous work loop (spec 79)
    ContinuousWorkModeEnabled {
        project_run_id: ProjectRunId,
        policy: WorkLoopPolicy,
    },
    ContinuousWorkModeDisabled {
        reason: String,
    },
    ContinuousWorkItemSelected {
        task_run_id: Option<TaskRunId>,
        packet: SpecLinkedTaskPacket,
    },
    ContinuousTurnRequested {
        task_run_id: Option<TaskRunId>,
        work_item_id: Option<String>,
        reason: String,
    },
    ContinuousTurnStarted {
        task_run_id: Option<TaskRunId>,
        work_item_id: Option<String>,
    },
    ContinuousTurnObserved {
        task_run_id: Option<TaskRunId>,
        summary: String,
    },
    ContinuousTurnCompleted {
        task_run_id: Option<TaskRunId>,
        work_item_id: Option<String>,
        continue_reason: Option<String>,
        verification_satisfied: bool,
        spec_conformant: bool,
    },
    /// Replay/audit event for doc78 secondary-loop comparative evidence.
    ContinuousSecondaryLoopOutcomeRecorded {
        task_run_id: Option<TaskRunId>,
        work_item_id: Option<String>,
        promotion_status: String,
        verification_satisfied: bool,
        spec_conformant: bool,
        trace_id: String,
    },
    ContinuousTurnPaused {
        reason: String,
    },
    ContinuousTurnBlocked {
        blocker_class: BlockerClass,
        reason: String,
        work_item_id: Option<String>,
    },
    ContinuousTurnEscalated {
        reason: String,
        work_item_id: Option<String>,
    },
    ContinuousTrancheCompleted {
        tranche_id: Option<String>,
        reason: String,
    },
    ContinuousLoopBudgetExhausted {
        reason: String,
    },
    ContinuousLoopTransportDegraded {
        reason: String,
    },
    ContinuousLoopResumed {
        reason: String,
    },
    ContinuousAuthorshipDelegated {
        delegate_id: String,
        scope: String,
        amendment_summary: Option<String>,
    },
    ContinuousAuthorshipDelegationCleared {
        reason: String,
    },
    ContinuousPauseFlagsUpdated {
        destructive_confirmation_required: bool,
        governance_decision_pending: bool,
        operator_override_active: bool,
        reason: Option<String>,
    },
    ContinuousDecisionContextUpdated {
        current_ask: Option<String>,
        ask_kind: Option<String>,
        scope_kind: Option<String>,
        carryover_policy: Option<String>,
        excluded_context_reason: Option<String>,
        excluded_context_labels: Option<Vec<String>>,
        source_turn_id: Option<String>,
        operator_steering_detected: Option<bool>,
    },
    ContinuousTransportSessionAttached {
        adapter: String,
        session_id: String,
    },
    ContinuousTransportAbortForwarded {
        reason: String,
    },
    ContinuousTransportEventIngested {
        sequence: u64,
        kind: String,
        session_id: Option<String>,
        turn_id: Option<String>,
        summary: Option<String>,
    },
    ContinuousLoopRecoveryCheckpointed {
        checkpoint_id: CheckpointId,
        summary: String,
    },

    // Focus Stack
    FocusFramePushed {
        frame_id: FrameId,
        beads_issue_id: String,
        title: String,
        goal: String,
        constraints: Vec<String>,
        tags: Vec<String>,
    },
    FocusFrameCompleted {
        frame_id: FrameId,
        completion_reason: CompletionReason,
    },
    FocusFrameSuspended {
        frame_id: FrameId,
        reason: String,
    },
    FocusFrameResumed {
        frame_id: FrameId,
    },

    // Focus State
    FocusStateUpdated {
        frame_id: FrameId,
        delta: FocusStateDelta,
    },

    // Intuition → Gate
    IntuitionSignalObserved {
        signal_id: SignalId,
        signal_type: SignalKind,
        severity: String,
        summary: String,
        related_frame_id: Option<FrameId>,
    },
    CandidateSurfaced {
        candidate_id: CandidateId,
        kind: CandidateKind,
        description: String,
        pressure: f32,
        related_frame_id: Option<FrameId>,
    },
    CandidatePinned {
        candidate_id: CandidateId,
    },
    CandidateSuppressed {
        candidate_id: CandidateId,
        scope: String,
        /// Concrete deadline computed from scope at command time.
        /// None = permanent (session scope). Stored in event for replay correctness.
        suppressed_until: Option<DateTime<Utc>>,
    },

    // Reference Store
    ArtifactRegistered {
        handle: HandleRef,
        storage_uri: String,
    },
    ArtifactPinned {
        artifact_id: ArtifactId,
    },
    ArtifactGarbageCollected {
        artifact_id: ArtifactId,
    },

    // Workers
    WorkerJobEnqueued {
        job_id: Uuid,
        kind: WorkerJobKind,
        correlation_id: Option<String>,
    },
    WorkerJobStarted {
        job_id: Uuid,
        kind: WorkerJobKind,
    },
    WorkerJobCompleted {
        job_id: Uuid,
        kind: WorkerJobKind,
        duration_ms: u64,
    },
    WorkerJobFailed {
        job_id: Uuid,
        kind: WorkerJobKind,
        duration_ms: u64,
        error: String,
    },

    // Prompt Assembly
    /// Emitted when a prompt is assembled.
    /// Per G1-detail-11 §Events: prompt.assembled with telemetry.
    PromptAssembled {
        turn_id: Option<TurnId>,
        estimated_tokens: u32,
        budget_target: u32,
        dropped_sections: Vec<String>,
        degraded: bool,
    },

    AutonomyAdjusted {
        level: AutonomyLevel,
        scope: Option<String>,
        ttl: Option<DateTime<Utc>>,
        reason: String,
    },

    // Memory
    /// Per G1-09 §Memory Operations: semantic memory upserted.
    SemanticMemoryUpserted {
        key: String,
        value: String,
        source: String,
    },

    // PRE / governance
    ProposalSubmitted {
        proposal_id: Uuid,
        kind: ProposalKind,
        source: String,
        payload: serde_json::Value,
        deadline_ms: u64,
        score: Option<f64>,
    },
    ProposalStatusChanged {
        proposal_id: Uuid,
        status: ProposalStatus,
    },
    ConstitutionLoaded {
        version: String,
        agent_id: String,
        principles: Vec<ConstitutionPrinciple>,
        safety_rules: Vec<String>,
        expression_rules: Vec<String>,
    },
    /// Per G1-09 §Memory Operations: procedural rule reinforced.
    RuleReinforced {
        rule_id: String,
        new_weight: f32,
        reinforced_count: u32,
    },
    /// Per G1-09 §Memory Operations: periodic decay tick applied.
    MemoryDecayTick {
        decay_factor: f32,
        rules_affected: usize,
    },

    // ─── RFM ─────────────────────────────────────────────────────────
    /// Per 36-reliability-focus-mode §6: RFM triggered regeneration.
    RfmRegenerationTriggered {
        frame_id: Option<FrameId>,
        ais_score: f64,
        rfm_level: u8,
        reason: String,
    },

    // ─── Ontology Classification / Reducer (docs/50) ─────────────────
    #[serde(rename = "ontology_object_upsert_proposed")]
    OntologyObjectUpsertProposed {
        proposal_id: Uuid,
        object_type: String,
        object_id: Option<String>,
        source: String,
    },
    #[serde(rename = "ontology_link_upsert_proposed")]
    OntologyLinkUpsertProposed {
        proposal_id: Uuid,
        link_type: String,
        source_id: String,
        target_id: String,
        source: String,
    },
    #[serde(rename = "ontology_status_change_proposed")]
    OntologyStatusChangeProposed {
        proposal_id: Uuid,
        subject: String,
        from_status: Option<String>,
        to_status: String,
        source: String,
    },
    #[serde(rename = "ontology_working_set_membership_proposed")]
    OntologyWorkingSetMembershipProposed {
        proposal_id: Uuid,
        subject: String,
        operation: String,
        source: String,
    },
    #[serde(rename = "ontology_proposal_promoted")]
    OntologyProposalPromoted {
        proposal_id: Uuid,
        target_class: String,
        applied_kind: String,
    },
    #[serde(rename = "ontology_proposal_rejected")]
    OntologyProposalRejected {
        proposal_id: Uuid,
        target_class: String,
        reason: String,
    },
    #[serde(rename = "ontology_verification_applied")]
    OntologyVerificationApplied {
        proposal_id: Option<Uuid>,
        verification: String,
        outcome: String,
    },
    #[serde(rename = "ontology_working_set_refreshed")]
    OntologyWorkingSetRefreshed {
        scope: String,
        reason: String,
    },

    // ─── Workpoint Continuity (Spec88) ─────────────────────────────────
    #[serde(rename = "workpoint_checkpoint_proposed")]
    WorkpointCheckpointProposed {
        workpoint: WorkpointRecord,
    },
    #[serde(rename = "workpoint_checkpoint_promoted")]
    WorkpointCheckpointPromoted {
        workpoint_id: WorkpointId,
        confidence: WorkpointConfidence,
        reason: String,
    },
    #[serde(rename = "workpoint_checkpoint_rejected")]
    WorkpointCheckpointRejected {
        workpoint_id: WorkpointId,
        reason: String,
    },
    #[serde(rename = "workpoint_superseded")]
    WorkpointSuperseded {
        old_workpoint_id: WorkpointId,
        new_workpoint_id: WorkpointId,
        reason: String,
    },
    #[serde(rename = "workpoint_resume_rendered")]
    WorkpointResumeRendered {
        workpoint_id: Option<WorkpointId>,
        mode: String,
        rendered_summary: String,
    },
    #[serde(rename = "workpoint_drift_detected")]
    WorkpointDriftDetected {
        workpoint_id: Option<WorkpointId>,
        severity: WorkpointDriftSeverity,
        reason: String,
        recovery_hint: Option<String>,
    },
    #[serde(rename = "workpoint_degraded_fallback_recorded")]
    WorkpointDegradedFallbackRecorded {
        workpoint_id: WorkpointId,
        reason: String,
        packet: serde_json::Value,
    },
    #[serde(rename = "ontology_action_intent_bound")]
    OntologyActionIntentBound {
        workpoint_id: WorkpointId,
        action_intent: WorkpointActionIntentRecord,
    },
    #[serde(rename = "ontology_verification_linked")]
    OntologyVerificationLinked {
        workpoint_id: WorkpointId,
        verification: WorkpointVerificationRecord,
    },

    // Errors
    InvariantViolation {
        invariant: String,
        details: String,
    },
}

/// Incremental Focus State delta — only changed fields.
///
/// Source: G1-07-ascc.md — 10 canonical ASCC slots.
/// All 10 slots must exist. May be empty, never absent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FocusStateDelta {
    /// Slot 1: Current intent.
    pub intent: Option<String>,
    /// Slot 2: Current focus / state description.
    /// §AsccSections: exposed to adapters as `current_focus` (aliased for serde).
    #[serde(alias = "current_focus")]
    pub current_state: Option<String>,
    /// Slot 3: Decisions made (cap 30, dedup).
    pub decisions: Option<Vec<String>>,
    /// Slot 4: Artifact references (cap 50, dedup by kind+path+label).
    pub artifacts: Option<Vec<ArtifactLine>>,
    /// Slot 5: Active constraints (cap 30, dedup).
    pub constraints: Option<Vec<String>>,
    /// Slot 6: Open questions (cap 20, remove when answered).
    pub open_questions: Option<Vec<String>>,
    /// Slot 7: Next steps (cap 15, replaced with latest).
    pub next_steps: Option<Vec<String>>,
    /// Slot 8: Recent results (cap 10, newest-first).
    pub recent_results: Option<Vec<String>>,
    /// Slot 9: Failures encountered (cap 20, append-only).
    pub failures: Option<Vec<String>>,
    /// Slot 10: Freeform notes (cap 20, append/decay oldest).
    pub notes: Option<Vec<String>>,
}

/// ASCC per-slot capacity limits from G1-07-ascc.md.
pub mod ascc_caps {
    pub const DECISIONS: usize = 30;
    pub const ARTIFACTS: usize = 50;
    pub const CONSTRAINTS: usize = 30;
    pub const OPEN_QUESTIONS: usize = 20;
    pub const NEXT_STEPS: usize = 15;
    pub const RECENT_RESULTS: usize = 10;
    pub const FAILURES: usize = 20;
    pub const NOTES: usize = 20;
}

// ─── Reduction Result (from core-reducer.md) ────────────────────────────────

/// Output of the core reducer.
#[derive(Debug, Clone)]
pub struct ReductionResult {
    pub new_state: FocusaState,
    pub emitted_events: Vec<FocusaEvent>,
}

// ─── Event Log Entry (from G1-detail-03-runtime-daemon.md) ──────────────────

/// Persisted event log entry (JSONL).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    #[serde(flatten)]
    pub event: FocusaEvent,
    pub correlation_id: Option<String>,
    pub origin: SignalOrigin,

    /// Multi-device sync fields (docs/40 + docs/43).
    /// These are duplicated into indexed SQLite columns for efficient sync.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<Uuid>,

    /// Policy #2: observations-only import (docs/43).
    /// Remote events imported from peers are tagged as observations.
    /// Observations are recorded but do not mutate canonical Focus Stack/State.
    #[serde(default)]
    pub is_observation: bool,
}

// ─── Workers (from G1-10-workers.md) ────────────────────────────────────────

/// MVP — 5 worker job kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerJobKind {
    ClassifyTurn,
    ExtractAsccDelta,
    DetectRepetition,
    ScanForErrors,
    SuggestMemory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobPriority {
    Low,
    Normal,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerJob {
    pub id: Uuid,
    pub kind: WorkerJobKind,
    pub created_at: DateTime<Utc>,
    pub priority: JobPriority,
    pub payload_ref: Option<HandleRef>,
    pub frame_context: Option<FrameId>,
    pub correlation_id: Option<String>,
    pub timeout_ms: u64,
}

// ─── Daemon Actions (from G1-detail-03-runtime-daemon.md) ───────────────────

/// Commands dispatched to the reducer via mpsc channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    // Focus
    PushFrame {
        title: String,
        goal: String,
        beads_issue_id: String,
        constraints: Vec<String>,
        tags: Vec<String>,
    },
    PopFrame {
        completion_reason: CompletionReason,
    },
    SetActiveFrame {
        frame_id: FrameId,
    },

    // Gate
    IngestSignal {
        signal: Signal,
    },
    SurfaceCandidate {
        candidate_id: CandidateId,
        boost: f32,
    },
    PinCandidate {
        candidate_id: CandidateId,
    },
    SuppressCandidate {
        candidate_id: CandidateId,
        scope: String,
    },

    // ASCC
    UpdateCheckpointDelta {
        frame_id: FrameId,
        turn_id: String,
        delta: FocusStateDelta,
    },

    // ECS
    StoreArtifact {
        kind: HandleKind,
        label: String,
        content: Vec<u8>,
    },
    ResolveHandle {
        handle_id: HandleId,
    },
    CacheBust {
        category: CacheBustCategory,
    },

    // Memory
    UpsertSemantic {
        key: String,
        value: String,
        source: MemorySource,
    },
    ReinforceRule {
        rule_id: String,
    },
    DecayTick,

    // Worker
    WorkerEnqueue {
        job: WorkerJob,
    },
    WorkerComplete {
        job_id: Uuid,
    },

    // Instance/Session
    InstanceConnect {
        kind: InstanceKind,
    },
    InstanceDisconnect {
        instance_id: Uuid,
        reason: String,
    },

    StartSession {
        adapter_id: Option<String>,
        workspace_id: Option<String>,
        instance_id: Option<Uuid>,
    },
    ResumeSession {
        session_id: Uuid,
    },
    CloseSession {
        reason: String,
        instance_id: Option<Uuid>,
    },
    CheckpointFrame {
        frame_id: Option<Uuid>,
        reason: String,
    },
    CompactContext {
        force: bool,
        tier: String,
        turn_count: Option<u64>,
        surface: Option<String>,
    },

    ThreadAttach {
        instance_id: Uuid,
        session_id: SessionId,
        thread_id: Uuid,
        role: AttachmentRole,
    },
    ThreadDetach {
        instance_id: Uuid,
        session_id: SessionId,
        thread_id: Uuid,
        reason: String,
    },

    // Proposals
    SubmitProposal {
        kind: ProposalKind,
        source: String,
        payload: serde_json::Value,
        deadline_ms: u64,
        score: Option<f64>,
    },

    // Thesis
    UpdateThesis {
        frame_id: FrameId,
        thesis: ThreadThesis,
    },

    // Confidence calibration
    LogConfidence {
        prediction_type: String,
        confidence: f64,
        context: String,
    },

    // Continuous work loop
    EnableContinuousWork {
        project_run_id: ProjectRunId,
        policy: WorkLoopPolicy,
    },
    PauseContinuousWork {
        reason: String,
    },
    ResumeContinuousWork {
        reason: String,
    },
    StopContinuousWork {
        reason: String,
    },
    SetContinuousWorkItem {
        task_run_id: Option<TaskRunId>,
        packet: SpecLinkedTaskPacket,
    },
    SelectNextContinuousSubtask {
        parent_work_item_id: String,
    },
    SetDelegatedContinuousAuthorship {
        delegate_id: Option<String>,
        scope: Option<String>,
        amendment_summary: Option<String>,
    },
    SetContinuousPauseFlags {
        destructive_confirmation_required: bool,
        governance_decision_pending: bool,
        operator_override_active: bool,
        reason: Option<String>,
    },
    SetContinuousDecisionContext {
        current_ask: Option<String>,
        ask_kind: Option<String>,
        scope_kind: Option<String>,
        carryover_policy: Option<String>,
        excluded_context_reason: Option<String>,
        excluded_context_labels: Option<Vec<String>>,
        source_turn_id: Option<String>,
        operator_steering_detected: Option<bool>,
    },
    AttachContinuousTransportSession {
        adapter: String,
        session_id: String,
    },
    AbortContinuousTransportSession {
        reason: String,
    },
    IngestContinuousTransportEvent {
        sequence: u64,
        kind: String,
        session_id: Option<String>,
        turn_id: Option<String>,
        summary: Option<String>,
    },
    RequestNextContinuousTurn {
        task_run_id: Option<TaskRunId>,
        work_item_id: Option<String>,
        reason: String,
    },
    ObserveContinuousTurnOutcome {
        task_run_id: Option<TaskRunId>,
        work_item_id: Option<String>,
        summary: String,
        continue_reason: Option<String>,
        verification_satisfied: bool,
        spec_conformant: bool,
    },
    CheckpointContinuousLoop {
        checkpoint_id: CheckpointId,
        summary: String,
    },
    MarkContinuousLoopTransportDegraded {
        reason: String,
    },

    // Events
    EmitEvent {
        event: FocusaEvent,
    },
}

// ─── Config ─────────────────────────────────────────────────────────────────

/// Runtime configuration defaults from spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusaConfig {
    /// Default: "~/.focusa/"
    pub data_dir: String,
    /// Default: 127.0.0.1:8787
    pub api_bind: String,
    /// Default: 6000
    pub max_prompt_tokens: u32,
    /// Default: 2000
    pub reserve_for_response: u32,
    /// Default: 8192 (8KB)
    pub ecs_externalize_bytes_threshold: u64,
    /// Default: 800
    pub ecs_externalize_token_threshold: u32,
    /// Default: 2.2
    pub gate_surface_threshold: f32,
    /// Default: 0.98
    pub gate_decay_factor: f32,
    /// Default: 200
    pub gate_max_candidates: usize,
    /// Default: 100
    pub worker_queue_size: usize,
    /// Default: 200
    pub worker_job_timeout_ms: u64,
    /// Enable redaction of sensitive content in transcripts.
    /// Default: false
    pub redaction_enabled: bool,
    /// Patterns to redact (regex strings).
    /// Default: ["\\b\\d{3}-\\d{2}-\\d{4}\\b", "\\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\\.[A-Z|a-z]{2,}\\b"]
    pub redaction_patterns: Vec<String>,
    /// Auth token for API access (None = no auth).
    /// Default: None
    pub auth_token: Option<String>,
    /// Seconds of inactivity before emitting InactivityTick signal.
    /// Default: 300 (5 minutes)
    pub inactivity_threshold_secs: Option<i64>,
    /// Seconds before a frame is considered long-running.
    /// Default: 1800 (30 minutes)
    pub long_running_frame_secs: Option<i64>,
}

impl Default for FocusaConfig {
    fn default() -> Self {
        Self {
            data_dir: "~/.focusa".to_string(),
            api_bind: "127.0.0.1:8787".to_string(),
            max_prompt_tokens: 6000,
            reserve_for_response: 2000,
            ecs_externalize_bytes_threshold: 8192,
            ecs_externalize_token_threshold: 800,
            gate_surface_threshold: 2.2,
            gate_decay_factor: 0.98,
            gate_max_candidates: 200,
            worker_queue_size: 100,
            worker_job_timeout_ms: 200,
            redaction_enabled: false,
            redaction_patterns: vec![
                r"\b\d{3}-\d{2}-\d{4}\b".to_string(), // SSN
                r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string(), // Email
            ],
            auth_token: None,
            inactivity_threshold_secs: Some(300),
            long_running_frame_secs: Some(1800),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONTEXT LINEAGE TREE (CLT) — docs/17-context-lineage-tree.md
// ═══════════════════════════════════════════════════════════════════════════════

/// CLT Node — append-only, immutable once written.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CltNode {
    pub node_id: String,
    pub node_type: CltNodeType,
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub session_id: Option<SessionId>,
    pub payload: CltPayload,
    pub metadata: CltMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CltNodeType {
    Interaction,
    Summary,
    BranchMarker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CltPayload {
    Interaction {
        role: String,
        content_ref: Option<String>,
    },
    Summary {
        summary: String,
        covered_range: Vec<String>,
        compression_ratio: f64,
    },
    BranchMarker {
        reason: String,
        branches: Vec<String>,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CltMetadata {
    pub task_id: Option<String>,
    pub agent_id: Option<String>,
    pub model_id: Option<String>,
}

/// CLT tree state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CltState {
    pub nodes: Vec<CltNode>,
    pub head_id: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// UXP / UFI — docs/14-uxp-ufi-schema.md
// ═══════════════════════════════════════════════════════════════════════════════

/// UXP — User Experience Profile (slow-moving calibration).
/// 7 canonical dimensions, α ≤ 0.1, window ≥ 30.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UxpProfile {
    pub autonomy_tolerance: UxpDimension,
    pub verbosity_preference: UxpDimension,
    pub interruption_sensitivity: UxpDimension,
    pub explanation_depth: UxpDimension,
    pub confirmation_preference: UxpDimension,
    pub risk_tolerance: UxpDimension,
    pub review_cadence: UxpDimension,
}

impl Default for UxpProfile {
    fn default() -> Self {
        let dim = || UxpDimension {
            value: 0.5,
            confidence: 0.0,
            citations: vec![],
            learning_rate: 0.1,
            window_size: 30,
            frozen: false,
        };
        Self {
            autonomy_tolerance: dim(),
            verbosity_preference: dim(),
            interruption_sensitivity: dim(),
            explanation_depth: dim(),
            confirmation_preference: dim(),
            risk_tolerance: dim(),
            review_cadence: dim(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UxpDimension {
    pub value: f64,
    pub confidence: f64,
    pub citations: Vec<String>,
    pub learning_rate: f64,
    pub window_size: u32,
    /// User override freezes learning.
    pub frozen: bool,
}

/// UFI — User Friction Index (fast-moving interaction cost).
/// 14 signal types in 3 weight tiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UfiSignal {
    pub signal_type: UfiSignalType,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<SessionId>,
    pub weight_tier: UfiWeightTier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UfiSignalType {
    // High tier (objective)
    TaskReopened,
    ManualOverride,
    ImmediateCorrection,
    UndoOrRevert,
    ExplicitRejection,
    // Medium tier
    Rephrase,
    RepeatRequest,
    ScopeClarification,
    ForcedSimplification,
    // Low tier (language-only — NEVER dominate aggregate)
    NegationLanguage,
    MetaLanguage,
    ImpatienceMarker,
    FrustrationIndicator,
    EscalationEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UfiWeightTier {
    High,
    Medium,
    Low,
}

/// UFI aggregate state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UfiState {
    pub signals: Vec<UfiSignal>,
    pub aggregate: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// AUTONOMY CALIBRATION — docs/37-autonomy-calibration-spec.md
// ═══════════════════════════════════════════════════════════════════════════════

/// Autonomy Reliability Index (ARI) — 0 to 100.
/// 6 dimensions: Correctness, Stability, Efficiency, Trust, Grounding, Recovery.
/// ARI weights: Outcome 50%, Efficiency 20%, Discipline 15%, Safety 15%.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomyState {
    pub level: AutonomyLevel,
    pub ari_score: f64,
    pub dimensions: AutonomyDimensions,
    pub sample_count: u64,
    pub granted_scope: Option<String>,
    pub granted_ttl: Option<DateTime<Utc>>,
    pub history: Vec<AutonomyEvent>,
}

impl Default for AutonomyState {
    fn default() -> Self {
        Self {
            level: AutonomyLevel::AL0,
            ari_score: 0.0,
            dimensions: AutonomyDimensions::default(),
            sample_count: 0,
            granted_scope: None,
            granted_ttl: None,
            history: vec![],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AutonomyLevel {
    /// Advisory only.
    AL0 = 0,
    /// Suggest + confirm.
    AL1 = 1,
    /// Execute with undo window.
    AL2 = 2,
    /// Execute, report after.
    AL3 = 3,
    /// Full autonomy in scope.
    AL4 = 4,
    /// Long-horizon autonomy.
    AL5 = 5,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AutonomyDimensions {
    pub correctness: f64,
    pub stability: f64,
    pub efficiency: f64,
    pub trust: f64,
    pub grounding: f64,
    pub recovery: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomyEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub from_level: AutonomyLevel,
    pub to_level: AutonomyLevel,
    pub reason: String,
    pub evidence: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// AGENT CONSTITUTION (ACP) — docs/16-agent-constitution.md
// ═══════════════════════════════════════════════════════════════════════════════

/// Agent Constitution — versioned, immutable reasoning charter.
/// SemVer. One active per agent. Never self-modifies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constitution {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub agent_id: String,
    pub principles: Vec<ConstitutionPrinciple>,
    pub self_eval_heuristics: Vec<String>,
    pub autonomy_posture: String,
    pub safety_rules: Vec<String>,
    pub expression_rules: Vec<String>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstitutionPrinciple {
    pub id: String,
    pub text: String,
    pub priority: u32,
    pub rationale: String,
}

/// Constitution store — version history.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConstitutionState {
    pub versions: Vec<Constitution>,
    pub active_version: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// THREAD / THESIS / INSTANCE / SESSION — docs/38-39-40
// ═══════════════════════════════════════════════════════════════════════════════

/// Thread — persistent cognitive workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: ThreadStatus,
    pub thesis: ThreadThesis,
    pub clt_head: Option<String>,
    pub autonomy_history: Vec<AutonomyEvent>,
    /// Per-thread ownership (docs/43): machine_id of the canonical writer.
    pub owner_machine_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreadStatus {
    Active,
    Saved,
    Archived,
    Forked,
}

/// Thread Thesis — living semantic anchor per thread.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThreadThesis {
    pub primary_intent: String,
    pub secondary_goals: Vec<String>,
    pub constraints: ThesisConstraints,
    pub open_questions: Vec<String>,
    pub assumptions: Vec<String>,
    pub confidence: ThesisConfidence,
    pub scope: ThesisScope,
    pub sources: Vec<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThesisConstraints {
    pub explicit: Vec<String>,
    pub implicit: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThesisConfidence {
    pub score: f64,
    pub rationale: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThesisScope {
    pub domain: String,
    pub time_horizon: String,
    pub risk_level: String,
}

/// Instance — where (runtime integration point).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: Uuid,
    pub kind: InstanceKind,
    pub created_at: DateTime<Utc>,
    pub thread_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceKind {
    Acp,
    Cli,
    Tui,
    Gui,
    Background,
}

/// Session attachment — binds instance/session to thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub session_id: SessionId,
    pub thread_id: Uuid,
    pub role: AttachmentRole,
    pub attached_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentRole {
    Active,
    Assistant,
    Observer,
    Background,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PROPOSAL RESOLUTION ENGINE (PRE) — docs/41-proposal-resolution-engine.md
// ═══════════════════════════════════════════════════════════════════════════════

/// Proposal — timestamped async request for state change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: Uuid,
    pub kind: ProposalKind,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub deadline: DateTime<Utc>,
    pub payload: serde_json::Value,
    pub score: f64,
    pub status: ProposalStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalKind {
    FocusChange,
    ThesisUpdate,
    AutonomyAdjustment,
    ConstitutionRevision,
    MemoryWrite,
    OntologyMutation,
    QueryScopeMutation,
    ReferenceResolutionMutation,
    ProjectionViewMutation,
    OntologyGovernanceMutation,
    IdentityModelMutation,
    VisualModelMutation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Pending,
    Accepted,
    Rejected,
    Expired,
}

/// PRE state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreState {
    pub proposals: Vec<Proposal>,
    pub resolution_window_ms: u64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// TELEMETRY / CTL — docs/29-30-31-32
// ═══════════════════════════════════════════════════════════════════════════════

/// Cognitive Telemetry event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub event_id: Uuid,
    pub event_type: TelemetryEventType,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<SessionId>,
    pub agent_id: Option<String>,
    pub model_id: Option<String>,
    pub clt_id: Option<String>,
    pub focus_frame_id: Option<FrameId>,
    pub payload: serde_json::Value,
    pub schema_version: String,
}

/// Trace dimension events for SPEC 56.
/// These track the 18 required dimensions for trace inspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryEventType {
    ModelTokens,
    FocusTransition,
    LineageNodeCreated,
    GateDecision,
    ToolCall,
    UxSignal,
    AutonomyUpdate,
    // SPEC 56 Trace Dimensions:
    MissionFrameContext,
    WorkingSetUsed,
    ConstraintsConsulted,
    DecisionsConsulted,
    ActionIntentsProposed,
    ToolsInvoked,
    VerificationResult,
    OntologyDeltaApplied,
    BlockersFailuresEmitted,
    FinalStateTransition,
    OperatorSubject,
    ActiveSubjectAfterRouting,
    SteeringDetected,
    SubjectHijackPrevented,
    SubjectHijackOccurred,
    PriorMissionReused,
    FocusSliceSize,
    FocusSliceRelevanceScore,
    // Docs 67/69 scope-routing + scope-failure trace surfaces.
    CurrentAskDetermined,
    QueryScopeBuilt,
    RelevantContextSelected,
    IrrelevantContextExcluded,
    ScopeVerified,
    ScopeContaminationDetected,
    WrongQuestionDetected,
    AnswerBroadeningDetected,
    ScopeFailureRecorded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondaryLoopLedgerEntry {
    pub proposal_id: String,
    pub source_function: String,
    pub actor_instance_id: Option<String>,
    pub role_profile_id: String,
    pub current_ask_id: Option<String>,
    pub query_scope_id: Option<String>,
    pub input_window_ref: Option<String>,
    pub evidence_refs: Vec<String>,
    pub proposed_delta: String,
    pub verification_status: String,
    pub promotion_status: String,
    pub confidence: f64,
    pub impact_metrics: serde_json::Value,
    pub failure_class: Option<String>,
    pub description: String,
    pub trace_id: String,
    pub correlation_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Telemetry aggregate state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelemetryState {
    pub total_events: u64,
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub tokens_per_task: Vec<TokensPerTask>,
    /// Tool call names for autonomy analysis (§33.4).
    #[serde(default)]
    pub tool_calls: Vec<String>,
    /// Trace dimension events for SPEC 56 (18 dimensions).
    #[serde(default)]
    pub trace_events: Vec<serde_json::Value>,
    /// Count of verification_result trace events for eval coverage tracking (doc 78 §15.3).
    #[serde(default)]
    pub verification_result_events: u64,
    /// Count of verification_result events that consulted decisions.
    #[serde(default)]
    pub decision_consult_events: u64,
    /// Count of scope_contamination_detected trace events.
    #[serde(default)]
    pub scope_contamination_events: u64,
    /// Count of subject_hijack_prevented trace events.
    #[serde(default)]
    pub subject_hijack_prevented_events: u64,
    /// Count of subject_hijack_occurred trace events.
    #[serde(default)]
    pub subject_hijack_occurred_events: u64,
    /// Count of secondary-loop quality traces graded as useful.
    #[serde(default)]
    pub secondary_loop_useful_events: u64,
    /// Count of secondary-loop quality traces graded as low_quality.
    #[serde(default)]
    pub secondary_loop_low_quality_events: u64,
    /// Durable secondary-cognition proposal advancement ledger (doc 78 §10).
    #[serde(default)]
    pub secondary_loop_ledger: Vec<SecondaryLoopLedgerEntry>,
    /// Count of ledger entries archived out of the active window.
    #[serde(default)]
    pub secondary_loop_archived_events: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokensPerTask {
    pub task_id: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub turns: u32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// CACHE PERMISSION MATRIX — docs/18-19
// ═══════════════════════════════════════════════════════════════════════════════

/// Cache class from 5-tier permission matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheClass {
    /// C0: Immutable, always safe to cache.
    C0,
    /// C1: Stable, TTL-based.
    C1,
    /// C2: Session-scoped.
    C2,
    /// C3: Turn-scoped (short-lived).
    C3,
    /// C4: Forbidden — never cache.
    C4,
}

/// Cache bust categories (A–F).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheBustCategory {
    /// A: Fresh evidence.
    FreshEvidence,
    /// B: Authority change.
    AuthorityChange,
    /// C: Compaction.
    Compaction,
    /// D: Staleness.
    Staleness,
    /// E: Salience collapse.
    SalienceCollapse,
    /// F: Provider mismatch.
    ProviderMismatch,
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAINING DATASET EXPORT — docs/20-21
// ═══════════════════════════════════════════════════════════════════════════════

/// Training dataset families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetFamily {
    FocusaSft,
    FocusaPreference,
    FocusaContrastive,
    FocusaLongHorizon,
}

/// Single training example.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingExample {
    pub family: DatasetFamily,
    pub session_id: SessionId,
    pub turn_id: String,
    pub input: String,
    pub output: String,
    pub focus_state_before: Option<FocusState>,
    pub focus_state_after: Option<FocusState>,
    pub uxp_signals: Vec<String>,
    pub ufi_signals: Vec<String>,
    pub lineage_path: Vec<String>,
    pub created_at: DateTime<Utc>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// CAPABILITY PERMISSIONS — docs/25-26
// ═══════════════════════════════════════════════════════════════════════════════

/// Permission scope: <domain>:<action>.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionScope {
    pub domain: String,
    pub action: String,
}

/// 3 permission classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionClass {
    Read,
    Command,
    Administrative,
}

/// 3 token types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiToken {
    pub token_id: Uuid,
    pub token_type: ApiTokenType,
    pub scopes: Vec<PermissionScope>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiTokenType {
    /// Full access.
    Owner,
    /// Scoped, revocable.
    Agent,
    /// Read-only, expirable.
    Integration,
}

// ═══════════════════════════════════════════════════════════════════════════════
// AGENT SKILLS — docs/34-35
// ═══════════════════════════════════════════════════════════════════════════════

/// Agent Skill — 18 skills in 4 categories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSkill {
    pub id: String,
    pub name: String,
    pub category: SkillCategory,
    pub description: String,
    pub api_endpoint: String,
    pub permission_class: PermissionClass,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    /// 8 read-only cognition inspection skills.
    CognitionInspection,
    /// 4 read-only telemetry skills.
    TelemetryMetrics,
    /// 2 read-only explanation skills.
    ExplanationTraceability,
    /// 4 guarded proposal skills.
    ProposalRequest,
}

/// Prohibited skills — agents CANNOT execute these.
pub const PROHIBITED_SKILLS: &[&str] = &[
    "set_focus_state",
    "modify_lineage",
    "write_reference",
    "activate_constitution",
    "escalate_autonomy",
    "approve_export",
];

// ═══════════════════════════════════════════════════════════════════════════════
// RELIABILITY FOCUS MODE (RFM) — docs/36-reliability-focus-mode.md
// ═══════════════════════════════════════════════════════════════════════════════

/// RFM level.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RfmLevel {
    /// Normal operation.
    #[default]
    R0 = 0,
    /// Validation mode.
    R1 = 1,
    /// Regeneration mode.
    R2 = 2,
    /// Ensemble mode.
    R3 = 3,
}

/// Microcell validators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MicrocellValidator {
    Schema,
    Constraint,
    Consistency,
    ReferenceGrounding,
}

/// Artifact Integrity Score.
/// ≥0.90 safe, 0.70–0.90 degraded, <0.70 triggers RFM.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RfmState {
    pub level: RfmLevel,
    pub ais_score: f64,
    pub validator_results: Vec<ValidatorResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorResult {
    pub validator: MicrocellValidator,
    pub passed: bool,
    pub details: String,
    pub timestamp: DateTime<Utc>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// DATA CONTRIBUTION — docs/22
// ═══════════════════════════════════════════════════════════════════════════════

/// Contribution pipeline state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContributionState {
    pub enabled: bool,
    pub queue: Vec<ContributionItem>,
    pub total_contributed: u64,
    pub policy: ContributionPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionItem {
    pub id: Uuid,
    pub dataset_family: DatasetFamily,
    pub status: ContributionStatus,
    pub created_at: DateTime<Utc>,
    pub reviewed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContributionStatus {
    Pending,
    Approved,
    Rejected,
    Submitted,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContributionPolicy {
    pub auto_contribute: bool,
    pub require_review: bool,
    pub anonymize: bool,
    pub allowed_families: Vec<DatasetFamily>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// TURN LIFECYCLE — docs/G1-detail-04-proxy-adapter.md (Mode A)
// ═══════════════════════════════════════════════════════════════════════════════

/// Turn identifier.
pub type TurnId = String;

/// Harness adapter identifier.
pub type AdapterId = String;

/// Turn start request from adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnStart {
    pub turn_id: TurnId,
    pub adapter_id: AdapterId,
    pub harness_name: String,
    pub timestamp: DateTime<Utc>,
}

/// Prompt assembly request from adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptAssembleRequest {
    pub turn_id: TurnId,
    pub raw_user_input: String,
    /// Output format: "string" or "messages".
    #[serde(default, alias = "harness_context")]
    pub format: Option<String>,
    /// Token budget target.
    #[serde(default, alias = "max_tokens_budget")]
    pub budget: Option<u32>,
    /// Eval-only assembly strategy override.
    /// "focusa" (default) or "baseline_raw".
    #[serde(default)]
    pub strategy: Option<String>,
}

/// Prompt assembly response to adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptAssembleResponse {
    /// The assembled prompt (per spec: "assembled").
    #[serde(alias = "assembled")]
    pub assembled_prompt: AssembledPromptOutput,
    pub handles_used: Vec<HandleRef>,
    /// Token statistics (per spec: "stats").
    #[serde(alias = "stats")]
    pub context_stats: ContextStats,
}

/// Assembled prompt — either plain string or chat messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AssembledPromptOutput {
    Plain(String),
    Messages(Vec<ChatMessage>),
}

/// Chat message for structured output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Context statistics from prompt assembly.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextStats {
    pub estimated_tokens: u32,
    pub focus_state_tokens: u32,
    pub rules_tokens: u32,
    pub handles_tokens: u32,
    pub user_input_tokens: u32,
}

/// Turn completion from adapter.
/// Supports both formats:
///   1. Canonical: { prompt_tokens, completion_tokens }
///   2. Extension: { tokens: { input, output, cache_read, cache_write } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnComplete {
    pub turn_id: TurnId,
    /// The user's original input for this turn (if available).
    #[serde(default)]
    pub raw_user_input: Option<String>,
    /// The assistant's output for this turn.
    pub assistant_output: String,
    #[serde(default)]
    pub artifacts: Vec<HandleRef>,
    #[serde(default)]
    pub errors: Vec<String>,
    /// Canonical token counts.
    #[serde(default)]
    pub prompt_tokens: Option<u32>,
    #[serde(default)]
    pub completion_tokens: Option<u32>,
    /// Extension format: { input, output, cache_read, cache_write }.
    #[serde(default)]
    pub tokens: Option<TurnTokens>,
}

/// Extension turn tokens format.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TurnTokens {
    #[serde(alias = "input", default)]
    pub input_tokens: Option<u32>,
    #[serde(alias = "output", default)]
    pub output_tokens: Option<u32>,
    #[serde(default)]
    pub cache_read: Option<u32>,
    #[serde(default)]
    pub cache_write: Option<u32>,
}

impl TurnComplete {
    /// Resolve prompt_tokens (canonical or extension).
    pub fn resolved_prompt_tokens(&self) -> u32 {
        self.prompt_tokens
            .or(self.tokens.as_ref().and_then(|t| t.input_tokens))
            .unwrap_or(0)
    }
    /// Resolve completion_tokens (canonical or extension).
    pub fn resolved_completion_tokens(&self) -> u32 {
        self.completion_tokens
            .or(self.tokens.as_ref().and_then(|t| t.output_tokens))
            .unwrap_or(0)
    }
}

/// Active turn state (daemon-side).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTurn {
    pub turn_id: TurnId,
    pub adapter_id: AdapterId,
    pub harness_name: String,
    pub started_at: DateTime<Utc>,
    pub raw_user_input: Option<String>,
    pub assembled_prompt: Option<String>,
}
