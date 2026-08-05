import type { MissionCanvasClient } from '../../../../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
import type { ExactScope, ProjectionLifecycleEvent } from './types';

export interface EventCursorStore {
  load(scope: ExactScope): string | undefined;
  persist(scope: ExactScope, cursor: string): void;
}

export interface EventBatch {
  accepted: ProjectionLifecycleEvent[];
  rejected: Array<{ event: ProjectionLifecycleEvent; reason: string }>;
  cursor?: string;
}

function sameScope(left: ExactScope, right: ExactScope): boolean {
  return left.project_root === right.project_root
    && left.continuity_id === right.continuity_id
    && left.attachment_id === right.attachment_id
    && left.session_id === right.session_id
    && (left.instance_id ?? null) === (right.instance_id ?? null)
    && (left.working_subpath_id ?? null) === (right.working_subpath_id ?? null);
}

export class MissionCanvasEventClient {
  #cursor?: string;
  #lastProjectionRevision = -1;
  #lastLayoutRevision = -1;
  #seenEventIds = new Set<string>();
  #running = false;
  #timer?: ReturnType<typeof setTimeout>;
  readonly #listeners = new Set<(batch: EventBatch) => void>();

  constructor(
    private readonly client: MissionCanvasClient,
    private readonly scope: ExactScope,
    private readonly cursorStore: EventCursorStore
  ) {
    this.#cursor = cursorStore.load(scope);
  }

  subscribe(listener: (batch: EventBatch) => void): () => void {
    this.#listeners.add(listener);
    return () => this.#listeners.delete(listener);
  }

  async poll(): Promise<EventBatch> {
    const events = await this.client.eventsStream({ scope: this.scope, after_cursor: this.#cursor });
    const accepted: ProjectionLifecycleEvent[] = [];
    const rejected: EventBatch['rejected'] = [];

    for (const event of events) {
      const reason = this.rejectionReason(event);
      if (reason) {
        rejected.push({ event, reason });
        continue;
      }
      this.#seenEventIds.add(event.event_id);
      this.#lastProjectionRevision = Math.max(this.#lastProjectionRevision, event.projection_revision);
      this.#lastLayoutRevision = Math.max(this.#lastLayoutRevision, event.layout_revision);
      this.#cursor = event.event_cursor;
      accepted.push(event);
    }

    if (this.#cursor && accepted.length > 0) this.cursorStore.persist(this.scope, this.#cursor);
    const batch = { accepted, rejected, cursor: this.#cursor };
    for (const listener of this.#listeners) listener(batch);
    return batch;
  }

  start(intervalMs = 750): void {
    if (this.#running) return;
    this.#running = true;
    const tick = async () => {
      if (!this.#running) return;
      try {
        await this.poll();
      } catch {
        // Projection state remains visible while the next bounded replay attempt waits.
      } finally {
        if (this.#running) this.#timer = setTimeout(tick, intervalMs);
      }
    };
    void tick();
  }

  stop(): void {
    this.#running = false;
    if (this.#timer) clearTimeout(this.#timer);
    this.#timer = undefined;
  }

  private rejectionReason(event: ProjectionLifecycleEvent): string | undefined {
    if (!sameScope(event.scope, this.scope)) return 'foreign_event_scope';
    if (this.#seenEventIds.has(event.event_id)) return 'duplicate_event';
    if (event.projection_revision < this.#lastProjectionRevision) return 'projection_revision_regressed';
    if (event.layout_revision < this.#lastLayoutRevision) return 'layout_revision_regressed';
    return undefined;
  }
}

export class SessionEventCursorStore implements EventCursorStore {
  readonly #prefix = 'focusa:mission-canvas:event-cursor:';

  load(scope: ExactScope): string | undefined {
    return globalThis.sessionStorage?.getItem(this.key(scope)) ?? undefined;
  }

  persist(scope: ExactScope, cursor: string): void {
    globalThis.sessionStorage?.setItem(this.key(scope), cursor);
  }

  private key(scope: ExactScope): string {
    return `${this.#prefix}${encodeURIComponent(JSON.stringify(scope))}`;
  }
}
