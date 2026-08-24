import test from 'node:test';
import assert from 'node:assert/strict';
import { reconnectDelay, runReliableEventStream, StreamAuthError } from '../src/lib/reconnect.mjs';

function envelope(sequence, overrides = {}) {
  return {
    schema: 'focusa.stream_event.v1', event_id: `event-${sequence}`, sequence,
    cursor: String(sequence), timestamp: '2026-08-24T12:00:00Z', event_type: 'work.changed',
    schema_version: '1.0.0', scope: {}, source_state_revision: sequence,
    payload_ref: null, invalidate: [], correlation_id: null, causation_id: null,
    payload: { value: sequence }, ...overrides,
  };
}
function body(...events) {
  return { async *[Symbol.asyncIterator]() {
    for (const event of events) yield new TextEncoder().encode(`id: ${event.cursor}\nevent: focusa_event\ndata: ${JSON.stringify(event)}\n\n`);
  } };
}
function response(events, status = 200) { return { ok: status >= 200 && status < 300, status, body: events && body(...events) }; }

test('replays from acknowledged cursor, dedupes, and commits only after render', async () => {
  const controller = new AbortController();
  const calls = [], rendered = [], committed = [];
  const replies = [response([envelope(1), envelope(2)]), response([envelope(2), envelope(3)])];
  const result = await runReliableEventStream({
    baseUrl: 'https://daemon.example', token: 'secret', signal: controller.signal,
    fetchImpl: async (url, options) => { calls.push({ url, options }); return replies.shift(); },
    onEvent: async (event) => { rendered.push(event.cursor); if (event.cursor === '3') controller.abort(); },
    commitCursor: async (cursor) => { assert.equal(rendered.at(-1), cursor); committed.push(cursor); },
    sleep: async () => {},
  });
  assert.deepEqual(rendered, ['1', '2', '3']);
  assert.deepEqual(committed, ['1', '2', '3']);
  assert.match(calls[1].url, /cursor=2/);
  assert.equal(calls[0].options.headers.authorization, 'Bearer secret');
  assert.equal(result.cursor, '3');
});

test('malformed event never advances cursor and reconnects from last acknowledgement', async () => {
  const controller = new AbortController();
  const urls = [], committed = [];
  const malformed = { ...envelope(1), cursor: 'wrong' };
  const replies = [response([malformed]), response([envelope(1)])];
  await runReliableEventStream({
    baseUrl: 'https://daemon.example', token: 'secret', signal: controller.signal,
    fetchImpl: async (url) => { urls.push(url); return replies.shift(); },
    onEvent: async () => controller.abort(), commitCursor: async (cursor) => committed.push(cursor), sleep: async () => {},
  });
  assert.deepEqual(committed, ['1']);
  assert.equal(new URL(urls[1]).searchParams.has('cursor'), false);
});

test('consumer failure is terminal and never commits a cursor', async () => {
  let commits = 0;
  await assert.rejects(runReliableEventStream({
    baseUrl: 'https://daemon.example', token: 'secret', fetchImpl: async () => response([envelope(1)]),
    onEvent: async () => { throw new Error('render failed'); },
    commitCursor: async () => { commits += 1; }, sleep: async () => {},
  }), /stream consumer failed during render/);
  assert.equal(commits, 0);
});

test('401 and 403 stop without retry', async () => {
  for (const status of [401, 403]) {
    let calls = 0, sleeps = 0;
    await assert.rejects(runReliableEventStream({
      baseUrl: 'https://daemon.example', token: 'secret', fetchImpl: async () => { calls += 1; return response(null, status); },
      onEvent: async () => {}, commitCursor: async () => {}, sleep: async () => { sleeps += 1; },
    }), StreamAuthError);
    assert.equal(calls, 1); assert.equal(sleeps, 0);
  }
});

test('backoff is bounded at 15 seconds', () => {
  assert.deepEqual([0,1,2,3,4,5,20].map(reconnectDelay), [1000,2000,4000,8000,15000,15000,15000]);
});
