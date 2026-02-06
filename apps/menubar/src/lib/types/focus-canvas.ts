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

export interface CanvasState {
  stack: CanvasFocusStack;
  activeFrame: CanvasFocusFrame | null;
}
