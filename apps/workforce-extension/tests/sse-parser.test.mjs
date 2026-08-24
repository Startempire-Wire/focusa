import test from 'node:test';
import assert from 'node:assert/strict';
import { createSseParser, MalformedSseEventError, validateStreamEvent } from '../src/lib/sse-parser.mjs';

function envelope(sequence, overrides = {}) {
  return {
    schema: 'focusa.stream_event.v1', event_id: `event-${sequence}`, sequence,
    cursor: String(sequence), timestamp: '2026-08-24T12:00:00Z', event_type: 'work.changed',
    schema_version: '1.0.0', scope: {}, source_state_revision: sequence,
    payload_ref: null, invalidate: [], correlation_id: null, causation_id: null,
    payload: { value: sequence }, ...overrides,
  };
}
function frame(event, newline = '\n') {
  return `id: ${event.cursor}${newline}event: focusa_event${newline}data: ${JSON.stringify(event)}${newline}${newline}`;
}

test('parses arbitrary UTF-8 chunk boundaries and CRLF frames', () => {
  const parser = createSseParser();
  const bytes = new TextEncoder().encode(frame(envelope(1), '\r\n') + frame(envelope(2)));
  const output = [];
  for (let index = 0; index < bytes.length; index += 3) output.push(...parser.push(bytes.slice(index, index + 3)));
  output.push(...parser.finish());
  assert.deepEqual(output.map((item) => item.cursor), ['1', '2']);
});

test('ignores comments and non-Focusa event frames', () => {
  const parser = createSseParser();
  assert.deepEqual(parser.push(': keep-alive\n\nevent: other\ndata: {}\n\n'), []);
});

test('rejects malformed envelopes and mismatched SSE ids', () => {
  assert.throws(() => validateStreamEvent(envelope(1, { cursor: '2' })), MalformedSseEventError);
  const parser = createSseParser();
  assert.throws(() => parser.push(`id: 9\nevent: focusa_event\ndata: ${JSON.stringify(envelope(1))}\n\n`), /does not match/);
});
