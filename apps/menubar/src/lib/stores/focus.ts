// Focus store — reactive state for Focus Stack + Focus State.

export interface FocusFrame {
  frame_id: string;
  intent: string;
  status: string;
  depth: number;
  beads_id: string;
  checkpoint?: {
    intent: string;
    constraints: string[];
    decisions: string[];
    next_steps: string[];
    current_state?: string;
  };
}

export interface FocusStoreState {
  connected: boolean;
  sessionId: string | null;
  activeFrameId: string | null;
  frames: FocusFrame[];
  stackPath: string[];
  version: number;
}

function createFocusStore() {
  let state = $state<FocusStoreState>({
    connected: false,
    sessionId: null,
    activeFrameId: null,
    frames: [],
    stackPath: [],
    version: 0,
  });

  return {
    get connected() { return state.connected; },
    get sessionId() { return state.sessionId; },
    get activeFrameId() { return state.activeFrameId; },
    get frames() { return state.frames; },
    get stackPath() { return state.stackPath; },
    get version() { return state.version; },
    get activeFrame() {
      return state.frames.find(f => f.frame_id === state.activeFrameId) ?? null;
    },
    get inactiveFrames() {
      return state.frames.filter(f => f.frame_id !== state.activeFrameId);
    },

    update(data: any) {
      state.connected = true;
      state.sessionId = data.session?.session_id ?? null;
      state.version = data.version ?? 0;

      const stack = data.focus_stack;
      if (stack) {
        state.activeFrameId = stack.active_id ?? null;
        state.frames = stack.frames ?? [];
        state.stackPath = stack.stack_path ?? [];
      }
    },

    disconnect() {
      state.connected = false;
    },
  };
}

export const focusStore = createFocusStore();
