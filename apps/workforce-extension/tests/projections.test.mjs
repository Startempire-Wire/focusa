import test from 'node:test';
import assert from 'node:assert/strict';
import { projectAudit, projectHealth, projectObservationFailure, projectRoster, projectWorkLoop } from '../src/lib/projections.mjs';

function session(index, lifecycle = 'running') { return {
  id: `session-${index}`, display_name: `Agent ${index}`, lifecycle, health: 'healthy', semantic_activity: 'working',
  current_run_generation: 1, updated_at: '2026-08-24T12:00:00Z', token: 'must-not-project', config: { secret: true },
}; }
function event(index) { return {
  schema: 'focusa.stream_event.v1', event_id: `event-${index}`, cursor: String(index), timestamp: '2026-08-24T12:00:00Z',
  event_type: 'session.changed', correlation_id: null, invalidate: ['roster'], payload: { token: 'must-not-project', raw: true },
}; }

test('health and work-loop projections select bounded canonical fields only', () => {
  const health = projectHealth({ schema: 'focusa.health.v1', ok: true, status: 'ok', version: '1.0.0', uptime_ms: 42, persistence: { token: 'secret' } });
  const loop = projectWorkLoop({ schema: 'focusa.work_loop_status.v3', state: 'healthy', status: 'running', enabled: true,
    project_status: 'active', current_task: { id: 'task-1', title: 'Ship', secret: 'hidden' }, raw: { token: 'secret' } });
  assert.equal(health.status, 'healthy'); assert.equal(loop.state, 'healthy'); assert.equal(loop.current_task.id, 'task-1');
  assert.doesNotMatch(JSON.stringify({ health, loop }), /secret|token|persistence|raw/);
});

test('unknown Work Loop state remains unknown', () => {
  const projection = projectWorkLoop({ schema: 'focusa.work_loop_status.v3', state: 'future_state', status: 'Future', enabled: true });
  assert.equal(projection.state, 'unknown');
});

test('roster contains daemon rows only, caps 200, and never infers unknown lifecycle', () => {
  const data = Array.from({ length: 201 }, (_, index) => session(index, index === 0 ? 'future_state' : 'running'));
  const projection = projectRoster({ schema: 'focusa.silent_session_api_envelope.v1', ok: true, data });
  assert.equal(projection.rows.length, 200); assert.equal(projection.source_count, 201); assert.equal(projection.truncated, true);
  assert.equal(projection.rows[0].id, data[0].id); assert.equal(projection.rows[0].lifecycle, 'unknown');
  assert.doesNotMatch(JSON.stringify(projection), /must-not-project|secret|config|token/);
});

test('audit keeps the latest 200 bounded rows and excludes raw payload', () => {
  const projection = projectAudit(Array.from({ length: 205 }, (_, index) => event(index + 1)));
  assert.equal(projection.length, 200); assert.equal(projection[0].cursor, '6'); assert.equal(projection.at(-1).cursor, '205');
  assert.doesNotMatch(JSON.stringify(projection), /must-not-project|payload|token|raw/);
});

test('failure projection preserves distinct capability states without messages or secrets', () => {
  for (const kind of ['unauthenticated','forbidden','unsupported','degraded']) {
    assert.deepEqual(projectObservationFailure({ kind, status: 403, message: 'token secret' }), { status: kind, http_status: 403 });
  }
});
