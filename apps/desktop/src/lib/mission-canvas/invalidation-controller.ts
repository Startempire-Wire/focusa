import type { EventBatch } from './event-client';

export interface ProjectionRevision {
  projectionRevision: number;
  layoutRevision: number;
}

export class MissionCanvasInvalidationController {
  #timer?: ReturnType<typeof setTimeout>;
  #pending = false;

  constructor(
    private readonly reload: () => Promise<void> | void,
    private readonly coalesceMs = 32
  ) {}

  enqueue(batch: EventBatch, current: ProjectionRevision): boolean {
    const invalidates = batch.accepted.some(
      (event) => event.projection_revision > current.projectionRevision
        || event.layout_revision > current.layoutRevision
    );
    if (!invalidates) return false;

    this.#pending = true;
    if (!this.#timer) {
      this.#timer = setTimeout(async () => {
        this.#timer = undefined;
        if (!this.#pending) return;
        this.#pending = false;
        await this.reload();
      }, this.coalesceMs);
    }
    return true;
  }

  flush(): Promise<void> | void {
    if (this.#timer) clearTimeout(this.#timer);
    this.#timer = undefined;
    if (!this.#pending) return;
    this.#pending = false;
    return this.reload();
  }

  dispose(): void {
    if (this.#timer) clearTimeout(this.#timer);
    this.#timer = undefined;
    this.#pending = false;
  }
}
