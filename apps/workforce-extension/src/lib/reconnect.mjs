import { MalformedSseEventError, parseSseStream } from './sse-parser.mjs';

const DELAYS_MS = Object.freeze([1000, 2000, 4000, 8000, 15000]);

export class StreamConsumerError extends Error {
  constructor(stage, cause) {
    super(`stream consumer failed during ${stage}`, { cause });
    this.name = 'StreamConsumerError';
    this.stage = stage;
  }
}

export class StreamAuthError extends Error {
  constructor(status) {
    super(`stream authorization failed (${status})`);
    this.name = 'StreamAuthError';
    this.status = status;
  }
}

export function reconnectDelay(attempt) {
  return DELAYS_MS[Math.min(Math.max(0, attempt), DELAYS_MS.length - 1)];
}

function streamUrl(baseUrl, cursor) {
  const url = new URL('/v1/events/stream', baseUrl);
  if (cursor) url.searchParams.set('cursor', cursor);
  return url.href;
}

export async function runReliableEventStream({
  baseUrl,
  token,
  initialCursor = null,
  fetchImpl = globalThis.fetch,
  onEvent,
  commitCursor,
  onState = () => {},
  sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms)),
  signal,
  maxReconnects = Number.POSITIVE_INFINITY,
}) {
  if (typeof fetchImpl !== 'function' || typeof onEvent !== 'function' || typeof commitCursor !== 'function') {
    throw new TypeError('fetchImpl, onEvent and commitCursor are required');
  }
  let cursor = initialCursor;
  let attempt = 0;

  while (!signal?.aborted) {
    onState(Object.freeze({ phase: cursor ? 'replaying' : 'live', cursor, attempt }));
    let acknowledgedOnConnection = false;
    try {
      const response = await fetchImpl(streamUrl(baseUrl, cursor), {
        method: 'GET',
        headers: { accept: 'text/event-stream', authorization: `Bearer ${token}` },
        signal,
      });
      if (response.status === 401 || response.status === 403) {
        onState(Object.freeze({ phase: 'unauthorized', cursor, attempt }));
        throw new StreamAuthError(response.status);
      }
      if (!response.ok || !response.body) throw new Error(`stream unavailable (${response.status})`);
      onState(Object.freeze({ phase: 'live', cursor, attempt }));
      for await (const event of parseSseStream(response.body)) {
        if (cursor && event.sequence <= Number(cursor)) continue;
        try { await onEvent(event); } catch (error) { throw new StreamConsumerError('render', error); }
        try { await commitCursor(event.cursor); } catch (error) { throw new StreamConsumerError('cursor_commit', error); }
        cursor = event.cursor;
        acknowledgedOnConnection = true;
        attempt = 0;
      }
    } catch (error) {
      if (error instanceof StreamAuthError || error instanceof StreamConsumerError || signal?.aborted) throw error;
      if (!(error instanceof MalformedSseEventError) && error?.name === 'AbortError') throw error;
    }
    if (signal?.aborted) break;
    if (!acknowledgedOnConnection) attempt += 1;
    if (attempt > maxReconnects) break;
    const delay_ms = reconnectDelay(Math.max(0, attempt - 1));
    onState(Object.freeze({ phase: 'reconnecting', cursor, attempt, delay_ms }));
    await sleep(delay_ms);
  }
  return Object.freeze({ cursor, stopped: signal?.aborted === true });
}
