import { normalizeDaemonOrigin } from './validation.mjs';

const encoder = new TextEncoder();
function bounded(value, field, max) {
  if (typeof value !== 'string' || !value.trim() || encoder.encode(value.trim()).byteLength > max) {
    throw new TypeError(`${field} must contain 1..${max} bytes`);
  }
  return value.trim();
}
function timestamp(value, field, nullable = false) {
  if (nullable && value == null) return null;
  const date = new Date(value);
  if (!value || Number.isNaN(date.valueOf())) throw new TypeError(`${field} must be RFC3339`);
  return date.toISOString();
}

export const MAX_PUBLIC_WORK_BYTES = 16384;

// Public display artifact, never a connection, execution grant or live worker status.
export function validatePublicWorkSnapshot(input) {
  const fields = ['schema', 'visibility', 'project', 'mission', 'state', 'stage', 'next_action', 'checkpoint_at', 'published_at', 'stale'];
  if (!input || input.schema !== 'focusa.public_work_snapshot.v1' || input.visibility !== 'public' ||
      Object.keys(input).length !== fields.length || Object.keys(input).some(key => !fields.includes(key)) ||
      !['active', 'blocked', 'completed'].includes(input.state) || typeof input.stale !== 'boolean' ||
      typeof input.checkpoint_at !== 'string' || typeof input.published_at !== 'string') {
    throw new TypeError('public Work snapshot contract mismatch');
  }
  return Object.freeze({
    schema: input.schema, visibility: 'public',
    project: bounded(input.project, 'project', 100),
    mission: bounded(input.mission, 'mission', 480), state: input.state,
    stage: bounded(input.stage, 'stage', 120),
    next_action: bounded(input.next_action, 'next_action', 480),
    checkpoint_at: timestamp(input.checkpoint_at, 'checkpoint_at'),
    published_at: timestamp(input.published_at, 'published_at'), stale: input.stale,
  });
}

export function validateConnectionRecord(input) {
  if (!input || input.schema !== 'focusa.workforce_connection.v1') throw new TypeError('connection schema mismatch');
  const scopes = input.granted_scopes;
  if (!Array.isArray(scopes) || scopes.some((scope) => !['read', 'write'].includes(scope))) {
    throw new TypeError('granted_scopes must contain only read/write');
  }
  return Object.freeze({
    schema: 'focusa.workforce_connection.v1',
    connection_id: bounded(input.connection_id, 'connection_id', 128),
    label: bounded(input.label, 'label', 200),
    base_url: normalizeDaemonOrigin(input.base_url),
    device_id: bounded(input.device_id, 'device_id', 128),
    token: bounded(input.token, 'token', 4096),
    granted_scopes: Object.freeze([...new Set(scopes)].sort()),
    last_cursor: input.last_cursor == null ? null : bounded(input.last_cursor, 'last_cursor', 128),
    created_at: timestamp(input.created_at, 'created_at'),
    last_connected_at: timestamp(input.last_connected_at, 'last_connected_at', true),
  });
}

export function redactConnection(record) {
  const valid = validateConnectionRecord(record);
  return Object.freeze({ ...valid, token: '••••' });
}
