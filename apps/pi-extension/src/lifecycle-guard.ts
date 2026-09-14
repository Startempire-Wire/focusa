/**
 * Generation fence for session-scoped refresh work (issues #301/#124/#310).
 *
 * Each Pi session lifecycle owns a generation id. Work scheduled under a
 * previous generation becomes a no-op the moment that lifecycle ends, so a
 * captured or delayed callback can never read a context that was invalidated
 * by session replacement, fork, switch, or reload.
 */
export class LifecycleGenerationGuard {
  private current: number | null = null;
  private next = 1;

  /** Begin a new lifecycle generation and return its id. */
  begin(): number {
    this.current = this.next;
    this.next += 1;
    return this.current;
  }

  /** End the active generation; every previously issued id becomes stale. */
  end(): void {
    this.current = null;
  }

  /** True when `generation` is the currently active generation. */
  isCurrent(generation: number): boolean {
    return this.current !== null && this.current === generation;
  }
}
