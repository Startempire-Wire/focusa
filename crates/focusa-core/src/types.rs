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

// ─── Canonical State (from core-reducer.md) ─────────────────────────────────

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
    pub contribution: ContributionState,
    /// Active turn from Mode A adapter (if any).
    pub active_turn: Option<ActiveTurn>,
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
            contribution: ContributionState::default(),
            active_turn: None,
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

/// 10 fixed semantic slots.
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
}

// ─── Events (from core-reducer.md) ──────────────────────────────────────────

/// Canonical event types (15 total).
///
/// If a cognition change cannot be expressed as one of these,
/// it does not belong in Focusa.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FocusaEvent {
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
        artifact_id: ArtifactId,
        artifact_type: String,
        summary: String,
        storage_uri: String,
    },
    ArtifactPinned {
        artifact_id: ArtifactId,
    },
    ArtifactGarbageCollected {
        artifact_id: ArtifactId,
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

    // Session
    StartSession {
        adapter_id: Option<String>,
        workspace_id: Option<String>,
    },
    CloseSession {
        reason: String,
    },

    // Proposals
    SubmitProposal {
        kind: ProposalKind,
        source: String,
        payload: serde_json::Value,
        deadline_ms: u64,
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
}

/// Telemetry aggregate state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelemetryState {
    pub total_events: u64,
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub tokens_per_task: Vec<TokensPerTask>,
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
    #[serde(default)]
    pub harness_context: Option<String>,
    #[serde(default)]
    pub max_tokens_budget: Option<u32>,
}

/// Prompt assembly response to adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptAssembleResponse {
    pub assembled_prompt: AssembledPromptOutput,
    pub handles_used: Vec<HandleRef>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnComplete {
    pub turn_id: TurnId,
    pub assistant_output: String,
    #[serde(default)]
    pub artifacts: Vec<HandleRef>,
    #[serde(default)]
    pub errors: Vec<String>,
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
