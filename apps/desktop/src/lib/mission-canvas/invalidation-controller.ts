import {
  sameWorkstreamAuthorityContext,
  validateMissionCanvasContract
} from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';
import { authorityFromEvent } from './exact-scope';
import type { ProjectionLifecycleEvent, WorkstreamAuthorityContext } from './types';
import type { EventBatch } from './event-client';

export interface ProjectionRevision {
  projectionRevision: number;
  layoutRevision: number;
  /** The projection watermark is a separate generated cursor namespace. */
  durableEventCursor?: string;
  /** Exact generated authority for the projection being refreshed. */
  authority?: WorkstreamAuthorityContext;
}

export interface ProjectionCursor {
  kind: 'event' | 'cursor' | 'mission-canvas' | 'opaque-numeric';
  value: number;
}

export interface ProjectionEventClassification {
  refresh: boolean;
  reason: string;
  event?: ProjectionLifecycleEvent;
  cursor?: ProjectionCursor;
}

/**
 * Only events which can change the Core-resolved projection are allowed to
 * schedule a projection read.  Draft and Pi transcript events have their own
 * bounded consumers; refreshing the whole projection for those events would
 * turn routine activity into a shell refresh and could disturb a draft.
 */
export const PROJECTION_INVALIDATING_EVENT_KINDS: ReadonlySet<ProjectionLifecycleEvent['event_kind']> = new Set([
  'candidate_discovered',
  'contribution_eligible',
  'contribution_omitted',
  'contribution_merged',
  'projection_resolved',
  'layout_changed',
  'focus_changed',
  'profile_changed',
  'activity_mode_changed',
  'capability_changed',
  'projection_suspended',
  'projection_rehydrated',
  'migration_started',
  'migration_completed',
  'migration_failed'
]);

/**
 * Classify one generated canonical event at the projection boundary.
 *
 * The event client already performs stream ordering, but this second boundary
 * is intentional: an invalidation callback must not trust a hand-written
 * batch, stale watermark, or foreign subordinate authority to cause a read.
 * Unknown/ineligible events are ignored; they are never interpreted locally.
 */
export const event = Object.freeze({
  classify(candidate: unknown, current: ProjectionRevision, expectedScope?: WorkstreamAuthorityContext): ProjectionEventClassification {
    if (!current || typeof current !== 'object') return ignored('missing_projection_context');
    const scope = expectedScope ?? current.authority;
    if (!scope) return ignored('missing_authority');

    const scopeValidation = validateMissionCanvasContract('WorkstreamAuthorityContext', scope);
    if (!scopeValidation.valid) return ignored(`invalid_authority:${scopeValidation.errors.join(',')}`);

    if (current.authority !== undefined) {
      const currentAuthorityValidation = validateMissionCanvasContract(
        'WorkstreamAuthorityContext',
        current.authority
      );
      if (!currentAuthorityValidation.valid) {
        return ignored(`invalid_current_authority:${currentAuthorityValidation.errors.join(',')}`);
      }
      if (!sameWorkstreamAuthorityContext(current.authority, scope)) {
        return ignored('projection_scope_mismatch');
      }
    }

    if (!isNonNegativeSafeInteger(current.projectionRevision)) {
      return ignored('invalid_projection_revision');
    }
    if (!isNonNegativeSafeInteger(current.layoutRevision)) {
      return ignored('invalid_layout_revision');
    }

    const currentCursor = parseCursor(current.durableEventCursor);
    if (!currentCursor) return ignored('missing_or_invalid_projection_cursor');

    const structural = validateMissionCanvasContract('ProjectionLifecycleEvent', candidate);
    if (!structural.valid) return ignored(`invalid_event:${structural.errors.join(',')}`);

    const projectionEvent = candidate as ProjectionLifecycleEvent;
    const eventAuthority = authorityFromEvent(projectionEvent);
    const eventAuthorityValidation = validateMissionCanvasContract(
      'WorkstreamAuthorityContext',
      eventAuthority
    );
    if (!eventAuthorityValidation.valid) {
      return ignored(`invalid_event_authority:${eventAuthorityValidation.errors.join(',')}`, projectionEvent);
    }
    if (!sameWorkstreamAuthorityContext(eventAuthority, scope)) {
      return ignored('foreign_event_scope', projectionEvent);
    }

    const eventCursor = parseCursor(projectionEvent.event_cursor);
    if (!eventCursor) return ignored('invalid_event_cursor', projectionEvent);
    if (eventCursor.kind !== currentCursor.kind) {
      return ignored('event_cursor_namespace_mismatch', projectionEvent, eventCursor);
    }
    if (eventCursor.value <= currentCursor.value) {
      return ignored('stale_event_cursor', projectionEvent, eventCursor);
    }

    if (!PROJECTION_INVALIDATING_EVENT_KINDS.has(projectionEvent.event_kind)) {
      return ignored('event_not_projection_relevant', projectionEvent, eventCursor);
    }
    if (projectionEvent.projection_revision < current.projectionRevision) {
      return ignored('projection_revision_stale', projectionEvent, eventCursor);
    }
    if (projectionEvent.layout_revision < current.layoutRevision) {
      return ignored('layout_revision_stale', projectionEvent, eventCursor);
    }
    if (
      projectionEvent.projection_revision === current.projectionRevision
      && projectionEvent.layout_revision === current.layoutRevision
    ) {
      return ignored('projection_watermark_current', projectionEvent, eventCursor);
    }

    const reason = projectionEvent.projection_revision > current.projectionRevision
      ? 'projection_revision_advanced'
      : 'layout_revision_advanced';
    return { refresh: true, reason, event: projectionEvent, cursor: eventCursor };
  }
});

/** Named function for consumers that prefer a direct import over event.classify. */
export function classifyProjectionEvent(
  candidate: unknown,
  current: ProjectionRevision,
  expectedScope?: WorkstreamAuthorityContext
): ProjectionEventClassification {
  return event.classify(candidate, current, expectedScope);
}

export class MissionCanvasInvalidationController {
  #timer?: ReturnType<typeof setTimeout>;
  #pending = false;
  #refreshing = false;
  #disposed = false;
  #generation = 0;
  #boundScope?: WorkstreamAuthorityContext;
  #pendingEventIds = new Set<string>();
  #pendingCursor?: ProjectionCursor;

  constructor(
    private readonly reload: () => Promise<void> | void,
    private readonly coalesceMs = 32
  ) {}

  /**
   * Coalesce one or more relevant canonical events into one serialized refresh.
   * `enqueue` remains as a compatibility alias for existing callers.
   */
  coalesce(
    batch: EventBatch,
    current: ProjectionRevision,
    expectedScope?: WorkstreamAuthorityContext
  ): boolean {
    if (this.#disposed || !current || typeof current !== 'object') return false;

    const scope = expectedScope ?? current.authority;
    if (!scope) return false;
    if (this.#boundScope && !sameWorkstreamAuthorityContext(this.#boundScope, scope)) {
      return false;
    }
    if (!this.#boundScope) {
      try {
        this.#boundScope = freezeClone(scope);
      } catch {
        return false;
      }
    }

    const accepted = Array.isArray(batch?.accepted) ? batch.accepted : [];
    let invalidates = false;
    for (const candidate of accepted) {
      const classification = event.classify(candidate, current, this.#boundScope);
      if (!classification.refresh || !classification.event || !classification.cursor) continue;

      const eventId = classification.event.event_id;
      if (this.#pendingEventIds.has(eventId)) continue;
      if (this.#pendingCursor && !isAfter(classification.cursor, this.#pendingCursor)) continue;

      // The event stream is bounded by the generated operation.  Keep the
      // coalescer bounded too; once one valid event is present, another event
      // in the same window cannot justify another refresh.
      if (this.#pendingEventIds.size < 1024) this.#pendingEventIds.add(eventId);
      this.#pendingCursor = classification.cursor;
      invalidates = true;
    }
    if (!invalidates) return false;

    this.#pending = true;
    this.#schedule();
    return true;
  }

  enqueue(
    batch: EventBatch,
    current: ProjectionRevision,
    expectedScope?: WorkstreamAuthorityContext
  ): boolean {
    return this.coalesce(batch, current, expectedScope);
  }

  flush(): Promise<void> | void {
    this.#cancelTimer();
    if (this.#disposed || !this.#pending || this.#refreshing) return;
    return this.#runRefresh();
  }

  dispose(): void {
    this.#disposed = true;
    this.#generation += 1;
    this.#cancelTimer();
    this.#pending = false;
    this.#pendingEventIds.clear();
    this.#pendingCursor = undefined;
    this.#boundScope = undefined;
  }

  #schedule(): void {
    if (this.#disposed || this.#timer || this.#refreshing) return;
    const generation = this.#generation;
    this.#timer = setTimeout(() => {
      this.#timer = undefined;
      if (this.#disposed || generation !== this.#generation || !this.#pending) return;
      void this.#runRefresh().catch(() => {
        // Timer-triggered refreshes have no caller to receive a rejection. The
        // projection controller owns fail-closed error state; never leak an
        // unhandled rejection from the coalescer.
      });
    }, Math.max(0, Number.isFinite(this.coalesceMs) ? this.coalesceMs : 32));
  }

  async #runRefresh(): Promise<void> {
    if (this.#disposed || this.#refreshing || !this.#pending) return;
    this.#pending = false;
    this.#refreshing = true;
    this.#pendingEventIds.clear();
    this.#pendingCursor = undefined;
    try {
      await this.reload();
    } finally {
      this.#refreshing = false;
      if (this.#pending) this.#schedule();
    }
  }

  #cancelTimer(): void {
    if (this.#timer) clearTimeout(this.#timer);
    this.#timer = undefined;
  }
}

function ignored(
  reason: string,
  projectionEvent?: ProjectionLifecycleEvent,
  cursor?: ProjectionCursor
): ProjectionEventClassification {
  return { refresh: false, reason, event: projectionEvent, cursor };
}

function parseCursor(value: unknown): ProjectionCursor | undefined {
  if (typeof value !== 'string') return undefined;
  const normalized = value.trim();
  const prefixed = /^(event|cursor|mission-canvas):([0-9]+)$/.exec(normalized);
  const match = prefixed ?? /^([0-9]+)$/.exec(normalized);
  if (!match) return undefined;
  const kind = prefixed ? prefixed[1] as ProjectionCursor['kind'] : 'opaque-numeric';
  const valueNumber = Number(prefixed ? prefixed[2] : match[1]);
  if (!Number.isSafeInteger(valueNumber) || valueNumber < 0) return undefined;
  return { kind, value: valueNumber };
}

function isAfter(left: ProjectionCursor, right: ProjectionCursor): boolean {
  return left.kind === right.kind && left.value > right.value;
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
  if (!value || typeof value !== 'object' || seen.has(value as object)) return value;
  seen.add(value as object);
  for (const child of Object.values(value as Record<string, unknown>)) deepFreeze(child, seen);
  return Object.freeze(value);
}
