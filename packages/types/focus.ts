// Canonical TypeScript types — mirrors focusa-core/src/types.rs

export interface FocusFrame {
  frame_id: string;
  beads_id: string;
  intent: string;
  status: 'Active' | 'Paused' | 'Suspended' | 'Completed' | 'Archived';
  depth: number;
  parent_id: string | null;
  handles: HandleRef[];
  checkpoint: FocusState;
  created_at: string;
  updated_at: string;
}

// 10 canonical ASCC slots per G1-07-ascc.md
export interface FocusState {
  intent: string;                  // Slot 1
  current_state: string;           // Slot 2
  decisions: string[];             // Slot 3 (cap 30)
  artifacts: ArtifactRef[];        // Slot 4 (cap 50)
  constraints: string[];           // Slot 5 (cap 30)
  open_questions: string[];        // Slot 6 (cap 20)
  next_steps: string[];            // Slot 7 (cap 15)
  recent_results: string[];        // Slot 8 (cap 10)
  failures: string[];              // Slot 9 (cap 20)
  notes: string[];                 // Slot 10 (cap 20)
}

export interface ArtifactRef {
  kind: string;
  label: string;
  path: string | null;
}

export interface HandleRef {
  id: string;
  kind: 'Log' | 'Diff' | 'Snapshot' | 'Text' | 'Binary';
  label: string;
  size: number;
  sha256: string;
  created_at: string;
  pinned: boolean;
}

export interface SessionState {
  session_id: string;
  started_at: string;
  ended_at: string | null;
  is_active: boolean;
}

// CLT
export interface CltNode {
  node_id: string;
  node_type: 'interaction' | 'summary' | 'branch_marker';
  parent_id: string | null;
  created_at: string;
  session_id: string | null;
  payload: CltPayload;
  metadata: CltMetadata;
}

export type CltPayload =
  | { type: 'Interaction'; role: string; content_ref: string | null }
  | { type: 'Summary'; summary: string; covered_range: string[]; compression_ratio: number }
  | { type: 'BranchMarker'; reason: string; branches: string[] };

export interface CltMetadata {
  task_id: string | null;
  agent_id: string | null;
  model_id: string | null;
}

// UXP / UFI
export interface UxpDimension {
  value: number;
  confidence: number;
  citations: string[];
  learning_rate: number;
  window_size: number;
  frozen: boolean;
}

export interface UxpProfile {
  autonomy_tolerance: UxpDimension;
  verbosity_preference: UxpDimension;
  interruption_sensitivity: UxpDimension;
  explanation_depth: UxpDimension;
  confirmation_preference: UxpDimension;
  risk_tolerance: UxpDimension;
  review_cadence: UxpDimension;
}

export type UfiSignalType =
  | 'task_reopened' | 'manual_override' | 'immediate_correction'
  | 'undo_or_revert' | 'explicit_rejection'
  | 'rephrase' | 'repeat_request' | 'scope_clarification' | 'forced_simplification'
  | 'negation_language' | 'meta_language' | 'impatience_marker'
  | 'frustration_indicator' | 'escalation_event';

// Autonomy
export type AutonomyLevel = 'AL0' | 'AL1' | 'AL2' | 'AL3' | 'AL4' | 'AL5';

export interface AutonomyState {
  level: AutonomyLevel;
  ari_score: number;
  dimensions: {
    correctness: number;
    stability: number;
    efficiency: number;
    trust: number;
    grounding: number;
    recovery: number;
  };
  sample_count: number;
}

// Constitution
export interface Constitution {
  version: string;
  created_at: string;
  agent_id: string;
  principles: { id: string; text: string; priority: number; rationale: string }[];
  active: boolean;
}

// RFM
export type RfmLevel = 'R0' | 'R1' | 'R2' | 'R3';

// Telemetry
export interface TelemetryState {
  total_events: number;
  total_prompt_tokens: number;
  total_completion_tokens: number;
}

// Proposals
export type ProposalStatus = 'pending' | 'accepted' | 'rejected' | 'expired';
export type ProposalKind = 'focus_change' | 'thesis_update' | 'autonomy_adjustment' | 'constitution_revision' | 'memory_write';

export interface Proposal {
  id: string;
  kind: ProposalKind;
  source: string;
  status: ProposalStatus;
  score: number;
  created_at: string;
  deadline: string;
}
