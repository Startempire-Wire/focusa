// Focus-related types for Canvas visualization

export interface FocusFrame {
  id: string;
  title: string;
  intent: string;
  goal: string;
  status: 'active' | 'paused' | 'completed';
  started_at: string;
  completed_at?: string;
  beads_issue_id?: string;
  ascc_preview?: string;
  parent_id?: string;
}

export interface AsccSections {
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

export interface FocusStack {
  frames: FocusFrame[];
  active_id: string | null;
  depth: number;
}

export interface FocusState {
  stack: FocusStack;
  activeFrame: FocusFrame | null;
}
