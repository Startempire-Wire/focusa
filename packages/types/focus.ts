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

export interface FocusState {
  intent: string;
  constraints: string[];
  decisions: string[];
  next_steps: string[];
  current_state: string | null;
  artifacts: string[];
  failures: string[];
  open_questions: string[];
  confidence: number;
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
