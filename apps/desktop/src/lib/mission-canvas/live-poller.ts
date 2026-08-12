// Bounded live-projection polling controller.
// Polls the daemon every interval_ms for a fresh projection.  When the projection
// revision changes, calls onProjectionChange.  Polling pauses when the browser tab
// is hidden (visibilitychange) and resumes when visible.
//
// Follows Spec 135J §3: the client may poll for fresh projections as a presentation
// preference; the daemon remains the canonical authority.

export interface LivePollOptions {
  /** Poll interval in ms.  Minimum 2000 (2s).  Default 5000. */
  intervalMs?: number;
  /** Maximum consecutive errors before polling stops.  Default 5. */
  maxErrors?: number;
  /** Called when the projection revision changes. */
  onProjectionChange: () => void | Promise<void>;
  /** Called when polling encounters an error. */
  onError?: (error: Error) => void;
  /** Called when polling stops due to max errors. */
  onMaxErrors?: () => void;
  /** Fetcher: returns the current projection revision.  Returns null on 404. */
  fetchRevision: () => Promise<number | null>;
  /** Current known revision. */
  currentRevision: () => number;
}

export interface LivePollController {
  start: () => void;
  stop: () => void;
  /** Force an immediate poll, bypassing the interval. */
  poll: () => Promise<void>;
  readonly running: boolean;
}

export function createLivePoller(options: LivePollOptions): LivePollController {
  const intervalMs = Math.max(2000, options.intervalMs ?? 5000);
  const maxErrors = options.maxErrors ?? 5;

  let timer: ReturnType<typeof setInterval> | null = null;
  let running = false;
  let consecutiveErrors = 0;
  let polling = false;

  function stop() {
    if (timer !== null) {
      clearInterval(timer);
      timer = null;
    }
    running = false;
  }

  async function poll() {
    if (polling) return;
    polling = true;
    try {
      const revision = await options.fetchRevision();
      if (revision !== null && revision !== options.currentRevision()) {
        await options.onProjectionChange();
      }
      consecutiveErrors = 0;
    } catch (error) {
      consecutiveErrors++;
      options.onError?.(error instanceof Error ? error : new Error(String(error)));
      if (consecutiveErrors >= maxErrors) {
        stop();
        options.onMaxErrors?.();
      }
    } finally {
      polling = false;
    }
  }

  function start() {
    if (running) return;
    running = true;
    consecutiveErrors = 0;
    // Poll immediately, then on interval
    poll();
    timer = setInterval(poll, intervalMs);
  }

  // Pause when tab hidden, resume when visible
  if (typeof document !== 'undefined') {
    document.addEventListener('visibilitychange', () => {
      if (document.visibilityState === 'hidden') {
        if (timer !== null && running) {
          clearInterval(timer);
          timer = null;
        }
      } else if (document.visibilityState === 'visible' && running) {
        poll();
        timer = setInterval(poll, intervalMs);
      }
    });
  }

  return { start, stop, poll, get running() { return running; } };
}
