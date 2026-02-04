// Focus store — maps to /v1/state/dump response.
// Field names match the actual Rust API JSON output.

export interface FocusFrame {
  id: string;
  title: string;
  goal: string;
  status: string; // "active" | "paused" | "suspended" | "completed" | "archived"
  parent_id: string | null;
  beads_issue_id: string;
  tags: string[];
  created_at: string;
  updated_at: string;
  focus_state: {
    intent: string;
    current_state: string;
    constraints: string[];
    decisions: string[];
    next_steps: string[];
    artifacts: any[];
    failures: any[];
    open_questions: string[];
    recent_results: string[];
    notes: string[];
  };
  stats: {
    turn_count: number;
    last_turn_id: string | null;
    last_token_estimate: number | null;
  };
}

export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

function createFocusStore() {
  let connected = $state<ConnectionStatus>('disconnected');
  let errorMsg = $state<string | null>(null);
  let version = $state(0);
  let activeId = $state<string | null>(null);
  let frames = $state<FocusFrame[]>([]);
  let stackPath = $state<string[]>([]);

  return {
    get connected() { return connected; },
    get errorMsg() { return errorMsg; },
    get version() { return version; },
    get activeId() { return activeId; },
    get frames() { return frames; },
    get stackPath() { return stackPath; },
    get frameCount() { return frames.length; },

    get activeFrame(): FocusFrame | null {
      return frames.find(f => f.id === activeId) ?? null;
    },

    get inactiveFrames(): FocusFrame[] {
      return frames.filter(f => f.id !== activeId);
    },

    get pausedFrames(): FocusFrame[] {
      return frames.filter(f => f.status === 'paused');
    },

    setConnecting() {
      connected = 'connecting';
      errorMsg = null;
    },

    update(data: any) {
      connected = 'connected';
      errorMsg = null;
      version = data.version ?? 0;

      const stack = data.focus_stack;
      if (stack) {
        activeId = stack.active_id ?? null;
        frames = stack.frames ?? [];
        stackPath = stack.stack_path_cache ?? [];
      }
    },

    disconnect() {
      connected = 'disconnected';
    },

    setError(msg: string) {
      connected = 'error';
      errorMsg = msg;
    },
  };
}

export const focusStore = createFocusStore();
