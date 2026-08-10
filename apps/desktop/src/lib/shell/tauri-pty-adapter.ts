import type { PtyAttachmentIdentity, PtyEvent, PtyTerminalGeometry } from './pty-contract';
import type { PtyHandle, PtyHandleOptions } from './pty-handle';

/** Native Tauri adapter backed only by focusa-pty/portable-pty. */
const COMMAND_PREFIX = 'focusa_pty';

export function createTauriPtyHandle(options: PtyHandleOptions = {}): PtyHandle {
  const invoke = options.invoke ?? (window as unknown as { __TAURI_INTERNALS__: { invoke: (c: string, a: Record<string, unknown>) => Promise<unknown> } }).__TAURI_INTERNALS__.invoke;
  const listeners = new Set<(event: PtyEvent) => void>();
  let generation = 0;
  let sequence = 0;
  let nativeSequence = 0;
  let attached = false;
  let identity: PtyAttachmentIdentity | undefined;
  let pollTimer: ReturnType<typeof setInterval> | undefined;
  let polling = false;

  function emit(event: PtyEvent): void {
    for (const listener of listeners) listener(event);
  }

  function localEvent(kind: 'attached' | 'resized' | 'interrupted' | 'detached' | 'closed' | 'restarted', extra: Partial<PtyEvent> = {}): void {
    if (!identity) return;
    sequence += 1;
    emit({ kind, ...extra, attachment_key: identity, work_surface_id: identity.work_surface_id, generation, sequence } as PtyEvent);
  }

  function failClosed(message: string): void {
    if (!identity) return;
    sequence += 1;
    emit({ kind: 'error', message, attachment_key: identity, work_surface_id: identity.work_surface_id, generation, sequence });
  }

  async function call(name: string, args: Record<string, unknown>): Promise<unknown> {
    try {
      return await invoke(`${COMMAND_PREFIX}_${name}`, args);
    } catch (error) {
      failClosed(error instanceof Error ? error.message : String(error));
      return undefined;
    }
  }

  function decodeNativeEvent(envelope: Record<string, unknown>): PtyEvent | undefined {
    const nativeKind = envelope.kind;
    const eventGeneration = Number(envelope.generation);
    const eventSequence = Number(envelope.sequence);
    const attachmentKey = envelope.attachment_key as PtyAttachmentIdentity;
    const workSurfaceId = String(envelope.work_surface_id ?? '');
    if (!attachmentKey || !workSurfaceId || !Number.isSafeInteger(eventGeneration) || !Number.isSafeInteger(eventSequence)) return;
    const kindName = typeof nativeKind === 'string'
      ? nativeKind
      : nativeKind && typeof nativeKind === 'object' ? Object.keys(nativeKind)[0] : '';
    const payload = nativeKind && typeof nativeKind === 'object'
      ? (nativeKind as Record<string, Record<string, unknown>>)[kindName]
      : undefined;
    const base = { attachment_key: attachmentKey, work_surface_id: workSurfaceId, generation: eventGeneration, sequence: eventSequence };
    if (kindName === 'attached') return { ...base, kind: 'attached', geometry: payload?.geometry as PtyTerminalGeometry };
    if (kindName === 'resized') return { ...base, kind: 'resized', geometry: payload?.geometry as PtyTerminalGeometry };
    if (kindName === 'output') return { ...base, kind: 'output', data: String(payload?.data ?? '') };
    if (kindName === 'error') return { ...base, kind: 'error', message: String(payload?.message ?? 'PTY error') };
    if (kindName === 'interrupted') return { ...base, kind: 'interrupted' };
    if (kindName === 'detached') return { ...base, kind: 'detached' };
    if (kindName === 'closed') return { ...base, kind: 'closed' };
    if (kindName === 'restarted') return { ...base, kind: 'restarted' };
  }

  async function pollNative(): Promise<void> {
    if (polling || !attached || !identity) return;
    polling = true;
    try {
      const result = await call('resync', {
        attachment_id: identity.attachment_id,
        work_surface_id: identity.work_surface_id,
        since_sequence: nativeSequence
      });
      const events = result && typeof result === 'object' && 'resync' in result
        ? (result as { resync?: { events?: Record<string, unknown>[] } }).resync?.events ?? []
        : [];
      for (const envelope of events) {
        const event = decodeNativeEvent(envelope);
        if (!event || event.sequence <= nativeSequence) continue;
        nativeSequence = event.sequence;
        generation = Math.max(generation, event.generation);
        sequence = Math.max(sequence, event.sequence);
        emit(event);
      }
    } finally {
      polling = false;
    }
  }

  function startPolling(): void {
    if (pollTimer) clearInterval(pollTimer);
    pollTimer = setInterval(() => void pollNative(), 50);
    (pollTimer as unknown as { unref?: () => void }).unref?.();
    void pollNative();
  }

  function stopPolling(): void {
    if (pollTimer) clearInterval(pollTimer);
    pollTimer = undefined;
  }

  function nativeSuccess(result: unknown): boolean {
    return Boolean(result && typeof result === 'object' && !('err' in result));
  }

  return {
    label: 'tauri-pty (native)',
    adapterKind: 'tauri',

    async attach(nextIdentity, geometry) {
      identity = nextIdentity;
      generation = 1;
      sequence = 0;
      nativeSequence = 0;
      const result = await call('attach', { identity: nextIdentity, geometry });
      if (nativeSuccess(result)) {
        attached = true;
        localEvent('attached', { geometry });
        startPolling();
      }
    },

    async write(data) {
      if (!attached || !identity) return false;
      const result = await call('input', {
        attachment_id: identity.attachment_id,
        work_surface_id: identity.work_surface_id,
        data,
        generation
      });
      return nativeSuccess(result);
    },

    async resize(geometry) {
      if (!attached || !identity) return false;
      const result = await call('resize', { attachment_id: identity.attachment_id, work_surface_id: identity.work_surface_id, geometry });
      if (nativeSuccess(result)) {
        localEvent('resized', { geometry });
        await pollNative();
      }
      return nativeSuccess(result);
    },

    async interrupt() {
      if (!attached || !identity) return false;
      const result = await call('interrupt', { attachment_id: identity.attachment_id, work_surface_id: identity.work_surface_id });
      if (nativeSuccess(result)) {
        localEvent('interrupted');
        await pollNative();
      }
      return nativeSuccess(result);
    },

    async detach() {
      if (!identity) return;
      await call('detach', { attachment_id: identity.attachment_id, work_surface_id: identity.work_surface_id });
      localEvent('detached');
      await pollNative();
      attached = false;
      stopPolling();
    },

    async close() {
      if (!identity) return;
      await call('close', { attachment_id: identity.attachment_id, work_surface_id: identity.work_surface_id });
      localEvent('closed');
      await pollNative();
      attached = false;
      stopPolling();
    },

    async restart() {
      if (!identity) return;
      const result = await call('restart', { attachment_id: identity.attachment_id, work_surface_id: identity.work_surface_id });
      if (nativeSuccess(result)) {
        generation += 1;
        localEvent('restarted');
        await pollNative();
      }
    },

    dispose() {
      stopPolling();
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
