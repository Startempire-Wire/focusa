import test from 'node:test';
import assert from 'node:assert/strict';
import { createOrientationPacket } from '../src/lib/orientation.mjs';
import { buildSafeSessionConfig, createPreflightedSession, preflightSafeSession, SessionCreateConflictError } from '../src/lib/session-create.mjs';

function packet(objective = 'Research the approved page') { return createOrientationPacket({
  objective, exclusions: ['Do not publish'], observation: { schema: 'focusa.browser_observation.v1', title: 'Example',
    url: 'https://example.com/', origin: 'https://example.com', captured_at: '2026-08-24T12:00:00Z' },
  project_root: '/approved/project', continuity_id: 'project-1', work_item_ref: 'focusa-1',
  role_profile_ref: 'role:researcher', agent_identity_ref: 'agent:browser-created',
}, () => new Date('2026-08-24T12:01:00Z')); }
function config(objective) { return buildSafeSessionConfig({ packet: packet(objective), display_name: 'Research Agent', provider: 'openai', model: 'gpt-5.6', auth_profile_ref: 'auth:default',
  workspace: { strategy: 'unsafe' }, destructive_actions_allowed: true }); }
function envelope(status, data, ok = true) { return { schema: 'focusa.silent_session_api_envelope.v1', ok, status, canonical: true, data }; }
function reply(status, body) { return { ok: status >= 200 && status < 300, status, json: async () => body }; }
async function approved(value, hash = 'hash-1') { return preflightSafeSession(value, { baseUrl: 'https://daemon.example', token: 'token', fetchImpl: async () => reply(200, envelope('preflight_ok', { validation: { valid: true, errors: [] }, redacted_config_hash: hash })) }); }
function store() { const records = new Map(); return { records, load: async (key) => records.get(key) ?? null,
  persist: async (record) => { records.set(record.idempotency_key, record); } }; }

 test('safe config preserves mission and bindings while fixing unsafe values', () => {
  const value = config();
  assert.match(value.identity.mission, /Research the approved page/);
  assert.equal(value.identity.project_root, '/approved/project'); assert.equal(value.identity.continuity_id, 'project-1');
  assert.equal(value.identity.work_item_ref, 'focusa-1'); assert.equal(value.identity.role_profile_ref, 'role:researcher');
  assert.deepEqual(value.harness, { kind: 'pi', adapter_version: '1', native_resume_policy: 'prefer' });
  assert.equal(value.model.selection_policy, 'exact'); assert.equal(value.model.fallback_policy, 'disabled'); assert.deepEqual(value.model.allowed_fallbacks, []);
  assert.equal(value.workspace.strategy, 'read_only_shared'); assert.equal(value.workspace.integration_policy, 'manual');
  assert.equal(value.resources.max_turns, 12); assert.equal(value.resources.max_wall_clock_seconds, 1800); assert.equal(value.resources.max_output_bytes, 16777216);
  assert.equal(value.governance.destructive_actions_allowed, false); assert.equal(value.governance.writer_lease_required, true);
});

test('preflight requires valid daemon hash before creation can proceed', async () => {
  const value = config();
  const result = await preflightSafeSession(value, { baseUrl: 'https://daemon.example', token: 'token', fetchImpl: async (url, options) => {
    assert.equal(url.pathname, '/v1/silent-sessions/preflight'); assert.equal(options.headers['x-focusa-permissions'], 'write:*');
    return reply(200, envelope('preflight_ok', { validation: { valid: true, errors: [] }, redacted_config_hash: 'abc123' }));
  } });
  assert.equal(result.config, value); assert.equal(result.redacted_config_hash, 'abc123'); assert.match(result.config_digest, /^[a-f0-9]{64}$/);
});

test('ambiguous create failure retries the same durably persisted key and yields one draft', async () => {
  const value = config(), idempotencyStore = store(); let calls = 0;
  const preflight = await approved(value);
  const result = await createPreflightedSession({ preflight, idempotency_key: 'create-1', idempotencyStore,
    requestOptions: { baseUrl: 'https://daemon.example', token: 'token', fetchImpl: async (_url, options) => {
      calls += 1; assert.ok(idempotencyStore.records.has('create-1'), 'key must persist before network create');
      const request = JSON.parse(options.body); assert.equal(request.idempotency_key, 'create-1');
      if (calls === 1) throw new TypeError('ambiguous timeout');
      return reply(200, envelope('replayed', { session: { id: 'session-1', lifecycle: 'draft' }, run_id: 'run-1', run_generation: 1, idempotent_replay: true }));
    } } });
  assert.equal(calls, 2); assert.equal(idempotencyStore.records.size, 1); assert.equal(result.session.id, 'session-1'); assert.equal(result.idempotent_replay, true);
});

test('changed payload reuse conflicts locally before network mutation', async () => {
  const idempotencyStore = store(); let calls = 0;
  const requestOptions = { baseUrl: 'https://daemon.example', token: 'token', fetchImpl: async () => { calls += 1; return reply(200, envelope('created', { session: { id: 'session-1' }, run: { id: 'run-1', generation: 1 } })); } };
  const first = await approved(config('First objective'), 'hash-1');
  const changed = await approved(config('Changed objective'), 'hash-2');
  await createPreflightedSession({ preflight: first, idempotency_key: 'same-key', idempotencyStore, requestOptions });
  await assert.rejects(createPreflightedSession({ preflight: changed, idempotency_key: 'same-key', idempotencyStore, requestOptions }), SessionCreateConflictError);
  assert.equal(calls, 1);
});

test('configuration changed after preflight fails before persistence or network', async () => {
  const mutable = structuredClone(config());
  const preflight = await approved(mutable);
  mutable.model.model = 'changed-after-review';
  let calls = 0;
  await assert.rejects(createPreflightedSession({ preflight, idempotency_key: 'mutated', idempotencyStore: store(),
    requestOptions: { baseUrl: 'https://daemon.example', token: 'token', fetchImpl: async () => { calls += 1; } } }), SessionCreateConflictError);
  assert.equal(calls, 0);
});

test('server idempotency conflict remains visible', async () => {
  const preflight = await approved(config());
  await assert.rejects(createPreflightedSession({ preflight, idempotency_key: 'conflict-key', idempotencyStore: store(),
    requestOptions: { baseUrl: 'https://daemon.example', token: 'token', fetchImpl: async () => reply(409, envelope('idempotency_conflict', null, false)) } }), SessionCreateConflictError);
});
