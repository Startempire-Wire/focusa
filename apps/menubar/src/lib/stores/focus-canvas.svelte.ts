import { writable } from 'svelte/store';
import { fetchJson } from '$lib/api';
import type { CanvasAsccSections, CanvasEvent, CanvasFocusFrame, CanvasState } from '$lib/types/focus-canvas';

function asStrings(value: unknown): string[] {
  return Array.isArray(value) ? value.map((item) => String(item)).filter(Boolean) : [];
}

function mapAscc(frame: any): CanvasAsccSections {
  const focusState = frame?.focus_state ?? {};
  return {
    intent: String(focusState.intent ?? frame?.intent ?? ''),
    current_focus: String(focusState.current_state ?? focusState.current_focus ?? ''),
    decisions: asStrings(focusState.decisions),
    artifacts: Array.isArray(focusState.artifacts) ? focusState.artifacts : [],
    constraints: asStrings(focusState.constraints),
    open_questions: asStrings(focusState.open_questions),
    next_steps: asStrings(focusState.next_steps),
    recent_results: asStrings(focusState.recent_results),
    failures: asStrings(focusState.failures?.map?.((failure: any) => failure?.failure ?? failure) ?? focusState.failures),
    notes: asStrings(focusState.notes),
  };
}

function mapFrame(frame: any): CanvasFocusFrame {
  return {
    id: String(frame.id),
    title: String(frame.title ?? frame.focus_state?.intent ?? 'Untitled frame'),
    intent: String(frame.focus_state?.intent ?? ''),
    goal: String(frame.goal ?? ''),
    status: frame.status ?? 'active',
    started_at: String(frame.created_at ?? frame.updated_at ?? new Date().toISOString()),
    completed_at: frame.completed_at ?? undefined,
    beads_issue_id: frame.beads_issue_id ?? undefined,
    parent_id: frame.parent_id ?? undefined,
    ascc: mapAscc(frame),
  };
}

function mapEvent(event: any): CanvasEvent {
  return {
    id: String(event.id),
    timestamp: String(event.timestamp ?? event.ts ?? new Date().toISOString()),
    type: String(event.type ?? event.event_type ?? 'event'),
    summary: String(event.summary ?? event.type ?? event.event_type ?? 'Focusa event'),
    frame_id: event.frame_id ?? event.frame_context ?? undefined,
  };
}

function createFocusCanvasStore() {
  const { subscribe, set, update } = writable<CanvasState>({
    stack: { frames: [], active_id: null },
    activeFrame: null,
    events: [],
    error: null,
  });

  return {
    subscribe,

    async loadLive() {
      try {
        const [state, recent] = await Promise.all([
          fetchJson('/v1/state/dump', 5000),
          fetchJson('/v1/events/recent?limit=20'),
        ]);
        const rawFrames = Array.isArray(state.focus_stack?.frames) ? state.focus_stack.frames : [];
        const frames: CanvasFocusFrame[] = rawFrames.slice(-80).map(mapFrame);
        const activeId = state.focus_stack?.active_id ?? frames.at(-1)?.id ?? null;
        set({
          stack: { frames, active_id: activeId },
          activeFrame: frames.find((frame) => frame.id === activeId) ?? frames.at(-1) ?? null,
          events: Array.isArray(recent.events) ? recent.events.map(mapEvent) : [],
          error: null,
        });
      } catch (e) {
        const msg = e instanceof Error ? e.message : 'Failed to load live Focusa canvas';
        update((state) => ({ ...state, error: msg }));
      }
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
