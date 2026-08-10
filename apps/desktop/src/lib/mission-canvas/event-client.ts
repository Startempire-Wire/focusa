import type { MissionCanvasClient } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
import {
  validateMissionCanvasContract,
  type ValidationResult
} from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
import { authorityFromEvent, sameWorkstreamAuthority as sameScope, workstreamAuthorityStorageKey } from './exact-scope';
import type { ProjectionLifecycleEvent, WorkstreamAuthorityContext } from './types';

export interface EventCursorStore {
  load(scope: WorkstreamAuthorityContext): string | undefined;
  persist(scope: WorkstreamAuthorityContext, cursor: string): void;
}

export interface EventBatch {
  accepted: ProjectionLifecycleEvent[];
  rejected: Array<{ event: ProjectionLifecycleEvent; reason: string }>;
  cursor?: string;
}

/**
 * Event transport identity is always validated by the generated contract.  The
 * event client may compare identity and stream ordering, but it never derives a
 * Workstream from a tab, CWD, latest record, or presentation state.
 */
export const eventScope = {
  validate(value: unknown): ValidationResult {
    return validateMissionCanvasContract('WorkstreamAuthorityContext', value);
  },

  validateEvent(value: unknown): ValidationResult {
    return validateMissionCanvasContract('ProjectionLifecycleEvent', value);
  }
};

export class MissionCanvasEventClient {
  #scope: WorkstreamAuthorityContext;
  #initializationError?: string;
  #cursor?: string;
  #lastProjectionRevision = -1;
  #lastLayoutRevision = -1;
  #lastEventSequence?: number;
  #seenEventIds = new Set<string>();
  #running = false;
  #timer?: ReturnType<typeof setTimeout>;
  #failureCount = 0;
  readonly #listeners = new Set<(batch: EventBatch) => void>();

  constructor(
    private readonly client: MissionCanvasClient,
    scope: WorkstreamAuthorityContext,
    private readonly cursorStore: EventCursorStore
  ) {
    try {
      this.#scope = freezeClone(scope);
    } catch {
      // Keep construction non-throwing so a malformed handoff can become a
      // recoverable, fail-closed poll error instead of taking down the shell.
      this.#scope = scope;
      this.#initializationError = 'invalid_workstream_scope:uncloneable';
      return;
    }

    const scopeValidation = eventScope.validate(this.#scope);
    if (!scopeValidation.valid) {
      this.#initializationError = `invalid_workstream_scope:${scopeValidation.errors.join(',')}`;
      return;
    }

    let persistedCursor: unknown;
    try {
      persistedCursor = cursorStore.load(this.#scope);
    } catch {
      this.#initializationError = 'event_cursor_load_failed';
      return;
    }

    if (persistedCursor !== undefined && typeof persistedCursor !== 'string') {
      this.#initializationError = 'invalid_persisted_cursor:type';
      return;
    }

    const sequence = parseEventCursor(persistedCursor);
    if (persistedCursor !== undefined && sequence === undefined) {
      this.#initializationError = 'invalid_persisted_cursor';
      return;
    }

    this.#cursor = persistedCursor;
    this.#lastEventSequence = sequence;
  }

  subscribe(listener: (batch: EventBatch) => void): () => void {
    this.#listeners.add(listener);
    return () => this.#listeners.delete(listener);
  }

  async poll(): Promise<EventBatch> {
    this.assertReady();
    if (typeof (this.client as { eventsStream?: unknown }).eventsStream !== 'function') {
      throw new Error('operation_unavailable:focusa.mission_canvas.events.stream');
    }

    const input = this.#cursor === undefined
      ? { ...this.#scope }
      : { ...this.#scope, after_cursor: this.#cursor };
    const events = await this.client.eventsStream(input);
    if (!Array.isArray(events)) throw new Error('invalid_event_stream_response:expected_array');

    const accepted: ProjectionLifecycleEvent[] = [];
    const rejected: EventBatch['rejected'] = [];
    const seenEventIds = new Set(this.#seenEventIds);
    let nextSequence = this.#lastEventSequence;
    let nextProjectionRevision = this.#lastProjectionRevision;
    let nextLayoutRevision = this.#lastLayoutRevision;

    for (const candidate of events) {
      const reason = this.rejectionReason(candidate, {
        lastEventSequence: nextSequence,
        lastProjectionRevision: nextProjectionRevision,
        lastLayoutRevision: nextLayoutRevision,
        seenEventIds
      });
      if (reason) {
        rejected.push({ event: candidate as ProjectionLifecycleEvent, reason });
        continue;
      }

      const event = candidate as ProjectionLifecycleEvent;
      const sequence = parseEventCursor(event.event_cursor);
      // rejectionReason has already checked this. Keeping the guard adjacent
      // to state advancement makes a future validator weakening fail closed.
      if (sequence === undefined) {
        rejected.push({ event, reason: 'invalid_event_cursor' });
        continue;
      }
      seenEventIds.add(event.event_id);
      nextSequence = sequence;
      nextProjectionRevision = event.projection_revision;
      nextLayoutRevision = event.layout_revision;
      accepted.push(event);
    }

    const nextCursor = accepted.at(-1)?.event_cursor ?? this.#cursor;
    if (nextCursor !== undefined && nextCursor !== this.#cursor) {
      try {
        // Persist before committing in-memory stream state. A failed durable
        // write must leave the old cursor available for a safe replay retry.
        this.cursorStore.persist(this.#scope, nextCursor);
      } catch {
        throw new Error('event_cursor_persist_failed');
      }
    }

    this.#seenEventIds = seenEventIds;
    this.#lastEventSequence = nextSequence;
    this.#lastProjectionRevision = nextProjectionRevision;
    this.#lastLayoutRevision = nextLayoutRevision;
    this.#cursor = nextCursor;

    const batch = { accepted, rejected, cursor: this.#cursor };
    for (const listener of this.#listeners) listener(batch);
    return batch;
  }

  start(intervalMs = 750): void {
    if (this.#running) return;
    this.#running = true;
    const tick = async () => {
      if (!this.#running) return;
      let delay = intervalMs;
      try {
        await this.poll();
        this.#failureCount = 0;
      } catch {
        this.#failureCount += 1;
        delay = Math.min(intervalMs * 2 ** Math.min(this.#failureCount, 5), 30_000);
      } finally {
        if (this.#running) this.#timer = setTimeout(tick, delay);
      }
    };
    void tick();
  }

  stop(): void {
    this.#running = false;
    if (this.#timer) clearTimeout(this.#timer);
    this.#timer = undefined;
    this.#failureCount = 0;
  }

  private assertReady(): void {
    if (this.#initializationError) throw new Error(this.#initializationError);
  }

  private rejectionReason(
    candidate: unknown,
    state: {
      lastEventSequence: number | undefined;
      lastProjectionRevision: number;
      lastLayoutRevision: number;
      seenEventIds: Set<string>;
    }
  ): string | undefined {
    const eventValidation = eventScope.validateEvent(candidate);
    if (!eventValidation.valid) return `invalid_event:${eventValidation.errors.join(',')}`;

    const event = candidate as ProjectionLifecycleEvent;
    if (typeof event.event_id !== 'string' || event.event_id.trim().length === 0) return 'invalid_event_id';
    if (typeof event.event_kind !== 'string' || event.event_kind.trim().length === 0) return 'invalid_event_kind';
    if (!isNonNegativeSafeInteger(event.projection_revision)) return 'invalid_projection_revision';
    if (!isNonNegativeSafeInteger(event.layout_revision)) return 'invalid_layout_revision';
    if (typeof event.payload_ref !== 'string' || event.payload_ref.trim().length === 0) return 'invalid_payload_ref';
    if (typeof event.occurred_at !== 'string' || event.occurred_at.trim().length === 0) return 'invalid_occurred_at';
    if (!Array.isArray(event.evidence_refs) || !Array.isArray(event.receipt_refs)) return 'invalid_event_refs';

    const eventAuthority = authorityFromEvent(event);
    const authorityValidation = eventScope.validate(eventAuthority);
    if (!authorityValidation.valid) return `invalid_event_scope:${authorityValidation.errors.join(',')}`;
    if (!sameScope(eventAuthority, this.#scope)) return 'foreign_event_scope';
    if (state.seenEventIds.has(event.event_id)) return 'duplicate_event';

    const sequence = parseEventCursor(event.event_cursor);
    if (sequence === undefined || sequence <= 0) return 'invalid_event_cursor';
    if (state.lastEventSequence !== undefined && sequence === state.lastEventSequence) return 'duplicate_cursor';
    if (state.lastEventSequence !== undefined && sequence < state.lastEventSequence) return 'event_cursor_regressed';
    if (event.projection_revision < state.lastProjectionRevision) return 'projection_revision_regressed';
    if (event.layout_revision < state.lastLayoutRevision) return 'layout_revision_regressed';
    return undefined;
  }
}

function parseEventCursor(cursor: string | undefined): number | undefined {
  if (typeof cursor !== 'string') return undefined;
  const raw = cursor.trim();
  const sequence = raw.startsWith('event:')
    ? raw.slice('event:'.length)
    : raw.startsWith('cursor:')
      ? raw.slice('cursor:'.length)
      : raw;
  if (!/^[0-9]+$/.test(sequence)) return undefined;
  const parsed = Number(sequence);
  return Number.isSafeInteger(parsed) ? parsed : undefined;
}

function isNonNegativeSafeInteger(value: unknown): value is number {
  return typeof value === 'number' && Number.isSafeInteger(value) && value >= 0;
}

function freezeClone<T>(value: T): T {
  let clone: T;
  if (typeof globalThis.structuredClone === 'function') {
    try {
      clone = globalThis.structuredClone(value);
    } catch {
      // Svelte 5 $state proxies are not structured-cloneable.
      clone = JSON.parse(JSON.stringify(value)) as T;
    }
  } else {
    clone = JSON.parse(JSON.stringify(value)) as T;
  }
  return deepFreeze(clone);
}

function deepFreeze<T>(value: T, seen = new WeakSet<object>()): T {
  if (typeof value !== 'object' || value === null || seen.has(value)) return value;
  seen.add(value);
  for (const child of Object.values(value as Record<string, unknown>)) deepFreeze(child, seen);
  return Object.freeze(value);
}

export class LocalEventCursorStore implements EventCursorStore {
  readonly #prefix = 'focusa:mission-canvas:event-cursor:';

  load(scope: WorkstreamAuthorityContext): string | undefined {
    try {
      return globalThis.localStorage?.getItem(this.key(scope)) ?? undefined;
    } catch {
      return undefined;
    }
  }

  persist(scope: WorkstreamAuthorityContext, cursor: string): void {
    try {
      globalThis.localStorage?.setItem(this.key(scope), cursor);
    } catch {
      // Replay remains correct from the server-selected cursor when persistence is unavailable.
    }
  }

  private key(scope: WorkstreamAuthorityContext): string {
    return `${this.#prefix}${workstreamAuthorityStorageKey(scope)}`;
  }
}
