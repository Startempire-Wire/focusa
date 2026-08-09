import type { PtyEvent, PtyTerminalGeometry } from './pty-contract';
import type { PtyAttachmentIdentity } from './pty-contract';
import type { PtyHandle } from './pty-handle';

/**
 * Virtual PTY handle — the honest browser/preview fallback.
 *
 * It is NOT a terminal and NOT an ordinary pipe: no child process, no pipe
 * file descriptors, no invented output. It acknowledges governed commands
 * with typed events (attached/resized/interrupted/closed/restarted/detached)
 * carrying the exact AttachmentKey, WorkSurfaceId, run generation, and
 * monotonic sequence, and rejects `input`/`write` while unbound — the same
 * fail-closed path the native adapter uses before a real Attachment exists.
 */
export function createVirtualPtyHandle(): PtyHandle {
  let listeners = new Set<(event: PtyEvent) => void>();
  let generation = 0;
  let sequence = 0;
  let attached = false;
  let identity: PtyAttachmentIdentity | undefined;

  function emit(event: PtyEvent): void {
    for (const listener of listeners) listener(event);
  }

  function next(event: PtyEvent): PtyEvent {
    sequence += 1;
    return {
      ...event,
      attachment_key: identity!,
      work_surface_id: identity!.work_surface_id,
      generation,
      sequence
    };
  }

  return {
    label: 'virtual-pty (preview)',
    adapterKind: 'virtual',

    async attach(nextIdentity, geometry) {
      identity = nextIdentity;
      generation += 1;
      sequence = 0;
      attached = true;
      emit(next({ kind: 'attached', geometry } as PtyEvent));
    },

    async write() {
      return false;
    },

    async resize(geometry) {
      if (!attached || !identity) return false;
      emit(next({ kind: 'resized', geometry } as PtyEvent));
      return true;
    },

    async interrupt() {
      if (!attached || !identity) return false;
      emit(next({ kind: 'interrupted' } as PtyEvent));
      return true;
    },

    async detach() {
      if (!identity) return;
      emit(next({ kind: 'detached' } as PtyEvent));
      attached = false;
    },

    async close() {
      if (!identity) return;
      emit(next({ kind: 'closed' } as PtyEvent));
      attached = false;
    },

    async restart() {
      if (!identity) return;
      emit(next({ kind: 'restarted' } as PtyEvent));
    },

    dispose() {
      listeners.clear();
      attached = false;
      identity = undefined;
    },

    onEvent(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    }
  };
}
