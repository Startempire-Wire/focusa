import { writable, derived } from 'svelte/store';
import type { FocusFrame, FocusStack } from '$lib/types/focus';

function createFocusStore() {
  const { subscribe, set, update } = writable<{
    stack: FocusStack;
    activeFrame: FocusFrame | null;
  }>({
    stack: {
      frames: [],
      active_id: null,
      depth: 0
    },
    activeFrame: null
  });

  return {
    subscribe,
    
    async loadStack() {
      // In real implementation, fetch from API
      // For now, use mock data
      const mockFrames: FocusFrame[] = [
        {
          id: 'frame-001',
          title: 'Build auth module',
          intent: 'Implement user authentication with JWT tokens',
          goal: 'Secure API endpoints with proper auth flow',
          status: 'paused',
          started_at: new Date(Date.now() - 86400000).toISOString(),
          ascc_preview: 'JWT middleware implemented, need refresh token logic'
        },
        {
          id: 'frame-002',
          title: 'Setup OAuth provider',
          intent: 'Integrate Google and GitHub OAuth',
          goal: 'Allow users to sign in with external providers',
          status: 'active',
          started_at: new Date(Date.now() - 3600000).toISOString(),
          ascc_preview: 'Google OAuth working, GitHub pending'
        }
      ];
      
      update(state => ({
        ...state,
        stack: {
          frames: mockFrames,
          active_id: 'frame-002',
          depth: 2
        },
        activeFrame: mockFrames[1]
      }));
    },
    
    setActiveFrame(frameId: string) {
      update(state => {
        const frame = state.stack.frames.find(f => f.id === frameId) || null;
        return {
          ...state,
          stack: {
            ...state.stack,
            active_id: frameId
          },
          activeFrame: frame
        };
      });
    },
    
    async pushFrame(title: string, intent: string, goal: string) {
      // API call to push frame
      console.log('Push frame:', { title, intent, goal });
    },
    
    async popFrame() {
      // API call to pop frame
      console.log('Pop frame');
    }
  };
}

export const focusStore = createFocusStore();
