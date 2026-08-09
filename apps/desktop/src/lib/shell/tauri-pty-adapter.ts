import type { PtyEvent, PtyTerminalGeometry } from './pty-contract';
import type { PtyAttachmentIdentity } from './pty-contract';
import type { PtyHandle, PtyHandleOptions } from './pty-handle';

/**
 * Native PTY adapter (Tauri).
 *
 * The 5% shell owns presentation only; the real PTY process is owned by the
 * Cargo runtime (PTY-004/005/006: one persistent Pi process per governed
 * Attachment, real PTY library — never ordinary pipes). This adapter forwards
 * the typed PtyCommand surface to the `focusa_pty_*` Tauri commands and
 * re-emits their events with the exact AttachmentKey, WorkSurfaceId, run
 * generation, and monotonic sequence. If the native command is unavailable
 * (host without the Cargo runtime), it fails closed with a typed error event
 * instead of inventing a terminal.
 */
const COMMAND_PREFIX = 'focusa_pty';

export function createTauriPtyHandle(options: PtyHandleOptions = {}): PtyHandle {
  const invoke = options.invoke ?? (window as unknown as { __TAURI_INTERNALS__: { invoke: (c: string, a: Record<string, unknown>) => Promise<unknown> } }).__TAURI_INTERNALS__.invoke;

  let listeners = new Set<(event: PtyEvent) => void>();
  let generation = 0;
  let sequence = 0;
  let attached = false;
  let identity: PtyAttachmentIdentity | undefined;

  function emit(event: PtyEvent): void {
    for (const listener of listeners) listener(event);
  }

  function next(kind: PtyEvent['kind'], extra?: Partial<PtyEvent>): PtyEvent {
    sequence += 1;
    return {
      kind,
      ...extra,
      attachment_key: identity!,
      work_surface_id: identity!.work_surface_id,
      generation,
      sequence
    } as PtyEvent;
  }

  function failClosed(message: string): PtyEvent {
    sequence += 1;
    return {
      kind: 'error',
      message,
      attachment_key: identity!,
      work_surface_id: identity!.work_surface_id,
      generation,
      sequence
    } as PtyEvent;
  }

  async function call(name: string, args: Record<string, unknown>): Promise<unknown> {
    try {
      return await invoke(`${COMMAND_PREFIX}_${name}`, args);
    } catch (error) {
      emit(failClosed(error instanceof Error ? error.message : String(error)));
      return undefined;
    }
  }

  return {
    label: 'tauri-pty (native)',
    adapterKind: 'tauri',

    async attach(nextIdentity, geometry) {
      identity = nextIdentity;
      generation += 1;
      sequence = 0;
      const result = await call('attach', { identity: nextIdentity, geometry });
      if (result !== null && result !== undefined && typeof result === 'object' && 'ok' in result && (result as { ok: boolean }).ok) {
        attached = true;
        emit(next('attached', { geometry }));
      }
    },

    async write(data) {
      if (!attached || !identity) return false;
      const result = await call('input', { attachment_id: identity.attachment_id, data });
      return result !== null && result !== undefined && typeof result === 'object' && (result as { ok?: boolean }).ok !== false;
    },

    async resize(geometry) {
      if (!attached || !identity) return false;
      const result = await call('resize', { attachment_id: identity.attachment_id, geometry });
      if (result !== null && result !== undefined && typeof result === 'object' && (result as { ok?: boolean }).ok !== false) {
        emit(next('resized', { geometry }));
        return true;
      }
      return false;
    },

    async interrupt() {
      if (!attached || !identity) return false;
      const result = await call('interrupt', { attachment_id: identity.attachment_id });
      if (result !== null && result !== undefined && typeof result === 'object' && (result as { ok?: boolean }).ok !== false) {
        emit(next('interrupted'));
        return true;
      }
      return false;
    },

    async detach() {
      if (!identity) return;
      await call('detach', { attachment_id: identity.attachment_id });
      emit(next('detached'));
      attached = false;
    },

    async close() {
      if (!identity) return;
      await call('close', { attachment_id: identity.attachment_id });
      emit(next('closed'));
      attached = false;
    },

    async restart() {
      if (!identity) return;
      const result = await call('restart', { attachment_id: identity.attachment_id });
      if (result !== null && result !== undefined && typeof result === 'object' && (result as { ok?: boolean }).ok !== false) {
        emit(next('restarted'));
      }
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
