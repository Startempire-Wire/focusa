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
    pub handles: Vec<HandleRef>,
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FocusState {
    pub intent: String,
    pub decisions: Vec<String>,
    pub constraints: Vec<String>,
    /// References only — no inline content.
    pub artifacts: Vec<ArtifactLine>,
    pub failures: Vec<String>,
    pub next_steps: Vec<String>,
    pub current_state: String,
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FocusStateDelta {
    pub intent: Option<String>,
    pub decisions: Option<Vec<String>>,
    pub constraints: Option<Vec<String>>,
    pub artifacts: Option<Vec<ArtifactLine>>,
    pub failures: Option<Vec<String>>,
    pub next_steps: Option<Vec<String>>,
    pub current_state: Option<String>,
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
