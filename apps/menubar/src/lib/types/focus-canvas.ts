// Types used by the Visual Focus Canvas feature.
// Intentionally separate from the app's main focus store types,
// which match /v1/state/dump.

export interface CanvasAsccSections {
  intent: string;
  current_focus: string;
  decisions: string[];
  artifacts: Array<{ label: string; handle_id: string }>;
  constraints: string[];
  open_questions: string[];
  next_steps: string[];
  recent_results: string[];
  failures: string[];
  notes: string[];
}

export interface CanvasFocusFrame {
  id: string;
  title: string;
  intent: string;
  goal: string;
  status: 'active' | 'paused' | 'completed';
  started_at: string;
  completed_at?: string;
  beads_issue_id?: string;
  ascc?: CanvasAsccSections;
  parent_id?: string;
}

export interface CanvasFocusStack {
  frames: CanvasFocusFrame[];
  active_id: string | null;
}

export interface CanvasEvent {
  id: string;
  timestamp: string;
  type: string;
  summary: string;
  frame_id?: string;
}

export interface CanvasState {
  stack: CanvasFocusStack;
  activeFrame: CanvasFocusFrame | null;
  events: CanvasEvent[];
  error: string | null;
}

export type SemanticPairTruthState =
  | 'schema_only' | 'pack_missing' | 'migration_required' | 'verification_required'
  | 'verification_blocked' | 'operator_required' | 'unsupported_future_definition'
  | 'writer_blocked' | 'degraded' | 'stale' | 'conflicted' | 'quarantined';

export interface SemanticPairOperation {
  operation_id: string;
  kind: 'read' | 'mutation';
  availability: 'available' | 'writer_blocked';
  truthful_states: SemanticPairTruthState[];
}

export interface SemanticPairStatus {
  schema: string;
  state: SemanticPairTruthState;
  scope: { project_root: string; continuity_id: string };
  operations: SemanticPairOperation[];
  evidence_refs?: string[];
  receipt_refs?: string[];
}

export interface SemanticPairActionRequest {
  operation_id: string;
  project_root: string;
  continuity_id: string;
  pair_id?: string;
  idempotency_key?: string;
  confirmation?: string;
  payload?: Record<string, unknown>;
}
