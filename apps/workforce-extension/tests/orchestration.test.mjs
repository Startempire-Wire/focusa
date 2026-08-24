import test from 'node:test';
import assert from 'node:assert/strict';
import { orchestrateAction, OrchestrationError, StaleTargetError } from '../src/lib/orchestration.mjs';

const target = Object.freeze({ session_id: 'session-1', run_id: 'run-1', generation: 1 });
function envelope(status, data = null, ok = true, extra = {}) { return { schema: 'focusa.silent_session_api_envelope.v1', ok, status, stale: false, failure_class: null, data, ...extra }; }
function reply(status, body) { return { ok: status >= 200 && status < 300, status, json: async () => body }; }
function status(lifecycle = 'draft') { return reply(200, envelope('status', { session: { id: target.session_id, lifecycle }, run: { id: target.run_id, generation: 1 }, temporal_context: {} })); }
function approval(action, expired = false) { return reply(201, envelope('approved', { approval: { schema: 'focusa.silent_session_approval_response.v1', status: 'approved',
  approval_id: `approval-${action}`, action, ...target, expires_at: expired ? '2020-01-01T00:00:00Z' : '2099-01-01T00:00:00Z',
  receipt_ref: `approval:approval-${action}`, action_idempotency_key: `action-${action}` } })); }
function mutation(action) { return reply(202, envelope(`${action}_requested`, { session: { id: target.session_id } })); }
function store() { const records = new Map(); return { records, load: async (key) => records.get(key) ?? null, persist: async (record) => records.set(record.idempotency_key, record) }; }
function options(replies, calls) { return { baseUrl: 'https://daemon.example', token: 'token', fetchImpl: async (url, init) => { calls.push({ url, init }); const next = replies.shift(); if (!next) throw new Error(`unexpected request ${url}`); return next; } }; }

 test('start refreshes, obtains exact approval, persists action key, mutates, then refreshes canonical state', async () => {
  const calls = [], idempotencyStore = store();
  const result = await orchestrateAction({ action: 'start', target, idempotency_key: 'start-1', idempotencyStore,
    requestOptions: options([status(), approval('start'), mutation('start'), status('validating')], calls) });
  assert.deepEqual(calls.map((call) => call.url.pathname), ['/v1/silent-sessions/session-1/status','/v1/silent-sessions/session-1/approvals','/v1/silent-sessions/session-1/start','/v1/silent-sessions/session-1/status']);
  const approvalBody = JSON.parse(calls[1].init.body), actionBody = JSON.parse(calls[2].init.body);
  assert.equal(approvalBody.action, 'start'); assert.equal(approvalBody.run_id, target.run_id); assert.equal(approvalBody.generation, 1);
  assert.equal(actionBody.approval_id, 'approval-start'); assert.equal(actionBody.idempotency_key, 'action-start');
  assert.ok(idempotencyStore.records.has('start-1:approval')); assert.ok(idempotencyStore.records.has('action-start')); assert.equal(result.canonical.session.lifecycle, 'validating');
});

test('pause and resume use exact targets without approval and never infer lifecycle', async () => {
  for (const action of ['pause','resume']) {
    const calls = [];
    const result = await orchestrateAction({ action, target, idempotency_key: `${action}-1`, idempotencyStore: store(),
      requestOptions: options([status(action === 'pause' ? 'running' : 'paused'), mutation(action), status(action === 'pause' ? 'pausing' : 'resuming')], calls) });
    assert.equal(calls.length, 3); assert.equal(calls.some((call) => call.url.pathname.endsWith('/approvals')), false);
    const body = JSON.parse(calls[1].init.body); assert.deepEqual(body, { run_id: 'run-1', generation: 1, idempotency_key: `${action}-1` });
    assert.equal(result.canonical.session.lifecycle, action === 'pause' ? 'pausing' : 'resuming');
  }
});

test('steer and cancel bind exact approval payload and target', async () => {
  for (const [action, payload] of [['steer',{ instruction: 'Check tests' }],['cancel',null]]) {
    const calls = [];
    await orchestrateAction({ action, target, payload, idempotency_key: `${action}-1`, idempotencyStore: store(),
      requestOptions: options([status(action === 'steer' ? 'running' : 'paused'), approval(action), mutation(action), status(action === 'steer' ? 'running' : 'cancelling')], calls) });
    const approvalBody = JSON.parse(calls[1].init.body), mutationBody = JSON.parse(calls[2].init.body);
    assert.deepEqual(approvalBody.payload, payload); assert.equal(approvalBody.action, action);
    if (action === 'steer') assert.equal(mutationBody.instruction, 'Check tests');
    assert.equal(mutationBody.approval_id, `approval-${action}`);
  }
});

test('stale initial target stops before approval or mutation', async () => {
  const calls = [];
  await assert.rejects(orchestrateAction({ action: 'start', target, idempotency_key: 'stale-1', idempotencyStore: store(),
    requestOptions: options([reply(409, envelope('stale_target', null, false, { stale: true, failure_class: 'stale_generation' }))], calls) }), StaleTargetError);
  assert.equal(calls.length, 1);
});

test('stale mutation refreshes once for reconfirmation and never auto-retries mutation', async () => {
  const calls = [];
  await assert.rejects(orchestrateAction({ action: 'cancel', target, idempotency_key: 'cancel-stale', idempotencyStore: store(),
    requestOptions: options([status('running'), approval('cancel'), reply(409, envelope('stale_target', null, false, { stale: true, failure_class: 'stale_generation' })),
      reply(409, envelope('stale_target', null, false, { stale: true, failure_class: 'stale_generation' }))], calls) }), StaleTargetError);
  assert.equal(calls.filter((call) => call.url.pathname.endsWith('/cancel')).length, 1);
  assert.equal(calls.length, 4);
});

test('expired approval requires reconfirmation before mutation', async () => {
  const calls = [];
  await assert.rejects(orchestrateAction({ action: 'start', target, idempotency_key: 'expired-1', idempotencyStore: store(),
    requestOptions: options([status(), approval('start', true)], calls) }), (error) => error instanceof OrchestrationError && error.kind === 'approval_expired');
  assert.equal(calls.length, 2);
});
