import { writable } from 'svelte/store';
import type { CanvasAsccSections, CanvasFocusFrame, CanvasState } from '$lib/types/focus-canvas';

const MOCK_ASCC_1: CanvasAsccSections = {
  intent: 'Build auth module',
  current_focus: 'JWT middleware and refresh token logic',
  decisions: ['Use JWT access tokens', 'Add refresh token rotation'],
  artifacts: [],
  constraints: ['No breaking API changes'],
  open_questions: ['How to store refresh tokens securely?'],
  next_steps: ['Implement refresh token endpoint'],
  recent_results: ['JWT middleware implemented'],
  failures: [],
  notes: ['Need to add token revocation list']
};

const MOCK_ASCC_2: CanvasAsccSections = {
  intent: 'Setup OAuth provider',
  current_focus: 'Google OAuth complete, GitHub pending',
  decisions: ['Use OAuth2 PKCE flow'],
  artifacts: [],
  constraints: ['Must support both providers'],
  open_questions: ['Do we need Apple Sign-In?'],
  next_steps: ['Implement GitHub OAuth'],
  recent_results: ['Google OAuth working'],
  failures: [],
  notes: ['Consider adding provider linking UI']
};

function getMockFrames(nowMs: number): CanvasFocusFrame[] {
  return [
    {
      id: 'frame-001',
      title: 'Build auth module',
      intent: 'Implement user authentication with JWT tokens',
      goal: 'Secure API endpoints with proper auth flow',
      status: 'paused',
      started_at: new Date(nowMs - 86400000).toISOString(),
      ascc: MOCK_ASCC_1
    },
    {
      id: 'frame-002',
      title: 'Setup OAuth provider',
      intent: 'Integrate Google and GitHub OAuth',
      goal: 'Allow users to sign in with external providers',
      status: 'active',
      started_at: new Date(nowMs - 3600000).toISOString(),
      ascc: MOCK_ASCC_2
    }
  ];
}

function createFocusCanvasStore() {
  const { subscribe, set, update } = writable<CanvasState>({
    stack: { frames: [], active_id: null },
    activeFrame: null
  });

  return {
    subscribe,

    loadMock() {
      const frames = getMockFrames(Date.now());

      set({
        stack: {
          frames,
          active_id: 'frame-002'
        },
        activeFrame: frames.find((f) => f.id === 'frame-002') ?? null
      });
    },

    setActiveFrame(frameId: string) {
      update((state) => {
        const frame = state.stack.frames.find((f) => f.id === frameId) ?? null;
        if (!frame) return state;

        return {
          ...state,
          stack: { ...state.stack, active_id: frameId },
          activeFrame: frame,
        };
      });
    }
  };
}

export const focusCanvasStore = createFocusCanvasStore();
