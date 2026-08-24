const MAX_ROWS = 200;
const encoder = new TextEncoder();
const LIFECYCLES = new Set(['draft','validating','queued','launching','initializing','running','waiting_input','blocked','pausing','paused','resuming','recovering','orphaned','completing','completed','failed','cancelling','cancelled']);
const HEALTH = new Set(['healthy','degraded','stale','unresponsive','process_exited','transport_lost','runner_lost','unknown']);
const ACTIVITY = new Set(['working','tool_running','thinking','waiting_for_operator','waiting_for_provider','waiting_for_dependency','idle_between_turns','verifying','checkpointing','integrating','unknown']);
const LOOP_STATES = new Set(['absent','unavailable','stale','unsupported','blocked','exhausted','zero','healthy']);

function bounded(value, max, fallback = null) {
  if (typeof value !== 'string' || !value) return fallback;
  if (encoder.encode(value).byteLength <= max) return value;
  let output = '';
  for (const character of value) {
    if (encoder.encode(`${output}${character}…`).byteLength > max) break;
    output += character;
  }
  return `${output}…`;
}
function known(value, values) { return typeof value === 'string' && values.has(value) ? value : 'unknown'; }

export function projectHealth(body) {
  if (body?.schema !== 'focusa.health.v1') throw new TypeError('health schema mismatch');
  return Object.freeze({
    schema: body.schema, status: body.ok === true && body.status === 'ok' ? 'healthy' : 'degraded',
    version: bounded(body.version, 64, 'unknown'), uptime_ms: Number.isSafeInteger(body.uptime_ms) ? body.uptime_ms : null,
  });
}

export function projectWorkLoop(body) {
  if (body?.schema !== 'focusa.work_loop_status.v3') throw new TypeError('work loop schema mismatch');
  const task = body.current_task;
  const current_task = task == null ? null : Object.freeze({
    id: bounded(typeof task === 'string' ? task : task.id, 128, 'unknown'),
    title: bounded(typeof task === 'object' ? task.title : null, 300),
  });
  return Object.freeze({
    schema: body.schema, state: known(body.state, LOOP_STATES), status: bounded(body.status, 64, 'unknown'),
    enabled: body.enabled === true, project_status: bounded(body.project_status, 64, 'unknown'), current_task,
    last_completed_task_id: bounded(body.last_completed_task_id, 128),
  });
}

export function projectRoster(envelope) {
  if (envelope?.schema !== 'focusa.silent_session_api_envelope.v1' || envelope.ok !== true || !Array.isArray(envelope.data)) {
    throw new TypeError('roster envelope mismatch');
  }
  const source = envelope.data.slice(0, MAX_ROWS);
  const rows = source.map((session) => Object.freeze({
    id: bounded(String(session?.id ?? ''), 128, 'unknown'),
    display_name: bounded(session?.display_name, 200, 'Unnamed session'),
    lifecycle: known(session?.lifecycle, LIFECYCLES), health: known(session?.health, HEALTH),
    semantic_activity: known(session?.semantic_activity, ACTIVITY),
    current_run_generation: Number.isSafeInteger(session?.current_run_generation) ? session.current_run_generation : null,
    updated_at: bounded(session?.updated_at, 64), projection: bounded(session?.projection, 32, 'canonical'),
  }));
  return Object.freeze({ rows: Object.freeze(rows), source_count: envelope.data.length, truncated: envelope.data.length > MAX_ROWS });
}

export function projectAudit(events) {
  if (!Array.isArray(events)) throw new TypeError('audit events must be an array');
  const selected = events.slice(-MAX_ROWS);
  return Object.freeze(selected.map((event) => {
    if (event?.schema !== 'focusa.stream_event.v1') throw new TypeError('audit event schema mismatch');
    return Object.freeze({
      event_id: bounded(event.event_id, 128, 'unknown'), cursor: bounded(event.cursor, 128, 'unknown'),
      timestamp: bounded(event.timestamp, 64), event_type: bounded(event.event_type, 256, 'unknown'),
      correlation_id: bounded(event.correlation_id, 128), invalidate: Object.freeze(
        Array.isArray(event.invalidate) ? event.invalidate.slice(0, 20).map((item) => bounded(String(item), 200, 'unknown')) : []
      ),
    });
  }));
}

export function projectObservationFailure(error) {
  const allowed = new Set(['unauthenticated','forbidden','unsupported','degraded','request_rejected','invalid_envelope']);
  return Object.freeze({ status: allowed.has(error?.kind) ? error.kind : 'degraded', http_status: error?.status ?? null });
}
