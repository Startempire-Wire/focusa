export class MalformedSseEventError extends Error {
  constructor(message) {
    super(message);
    this.name = 'MalformedSseEventError';
  }
}

function requiredString(value, field, max = 4096) {
  if (typeof value !== 'string' || !value || value.length > max) {
    throw new MalformedSseEventError(`${field} must be a bounded non-empty string`);
  }
  return value;
}

export function validateStreamEvent(value) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    throw new MalformedSseEventError('stream event must be an object');
  }
  if (value.schema !== 'focusa.stream_event.v1') {
    throw new MalformedSseEventError('stream event schema mismatch');
  }
  requiredString(value.event_id, 'event_id', 128);
  requiredString(value.cursor, 'cursor', 128);
  requiredString(value.timestamp, 'timestamp', 64);
  requiredString(value.event_type, 'event_type', 256);
  requiredString(value.schema_version, 'schema_version', 64);
  if (!Number.isSafeInteger(value.sequence) || value.sequence < 1) {
    throw new MalformedSseEventError('sequence must be a positive safe integer');
  }
  if (value.cursor !== String(value.sequence)) {
    throw new MalformedSseEventError('cursor must equal the durable sequence');
  }
  if (!value.scope || typeof value.scope !== 'object' || Array.isArray(value.scope)) {
    throw new MalformedSseEventError('scope must be an object');
  }
  if (!Array.isArray(value.invalidate)) {
    throw new MalformedSseEventError('invalidate must be an array');
  }
  if (!value.payload || typeof value.payload !== 'object' || Array.isArray(value.payload)) {
    throw new MalformedSseEventError('payload must be an object');
  }
  return Object.freeze({ ...value, scope: Object.freeze({ ...value.scope }), invalidate: Object.freeze([...value.invalidate]) });
}

function parseFrame(raw) {
  let eventName = 'message';
  let id = null;
  const data = [];
  for (const line of raw.replaceAll('\r\n', '\n').replaceAll('\r', '\n').split('\n')) {
    if (!line || line.startsWith(':')) continue;
    const separator = line.indexOf(':');
    const field = separator < 0 ? line : line.slice(0, separator);
    let value = separator < 0 ? '' : line.slice(separator + 1);
    if (value.startsWith(' ')) value = value.slice(1);
    if (field === 'event') eventName = value;
    else if (field === 'id') id = value;
    else if (field === 'data') data.push(value);
  }
  if (data.length === 0) return null;
  if (eventName !== 'focusa_event') return null;
  let parsed;
  try {
    parsed = JSON.parse(data.join('\n'));
  } catch {
    throw new MalformedSseEventError('focusa_event data is not valid JSON');
  }
  const envelope = validateStreamEvent(parsed);
  if (id !== null && id !== envelope.cursor) {
    throw new MalformedSseEventError('SSE id does not match envelope cursor');
  }
  return envelope;
}

function boundary(buffer) {
  const matches = [buffer.indexOf('\n\n'), buffer.indexOf('\r\n\r\n'), buffer.indexOf('\r\r')]
    .filter((index) => index >= 0);
  if (matches.length === 0) return null;
  const index = Math.min(...matches);
  const width = buffer.startsWith('\r\n\r\n', index) ? 4 : 2;
  return { index, width };
}

export function createSseParser() {
  const decoder = new TextDecoder();
  let buffer = '';
  return Object.freeze({
    push(chunk) {
      buffer += typeof chunk === 'string' ? chunk : decoder.decode(chunk, { stream: true });
      const events = [];
      for (let split = boundary(buffer); split; split = boundary(buffer)) {
        const raw = buffer.slice(0, split.index);
        buffer = buffer.slice(split.index + split.width);
        const event = parseFrame(raw);
        if (event) events.push(event);
      }
      return events;
    },
    finish() {
      buffer += decoder.decode();
      if (!buffer.trim()) return [];
      const event = parseFrame(buffer);
      buffer = '';
      return event ? [event] : [];
    },
  });
}

export async function* parseSseStream(body) {
  if (!body) throw new TypeError('SSE response body is required');
  const parser = createSseParser();
  const iterable = body[Symbol.asyncIterator]
    ? body
    : { async *[Symbol.asyncIterator]() { const reader = body.getReader(); try { while (true) { const item = await reader.read(); if (item.done) break; yield item.value; } } finally { reader.releaseLock(); } } };
  for await (const chunk of iterable) {
    for (const event of parser.push(chunk)) yield event;
  }
  for (const event of parser.finish()) yield event;
}
