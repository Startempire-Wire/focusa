import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import test from 'node:test';
import plugin from './dist/index.js';

// Contract fixtures are extracted from the reviewed release sources, not
// invented registration receipts. These tests do not replace live host proof.
const versions = ['2026.10.7', '2026.11.3'];
test('package selects the built entry point rather than a cached source fallback', () => {
  const pkg = JSON.parse(readFileSync(new URL('./package.json', import.meta.url)));
  assert.equal(pkg.main, 'dist/index.js');
  assert.deepEqual(pkg.openclaw.extensions, ['./dist/index.js']);
});
for (const version of versions) {
  test(`one supported registration and one injection: ${version}`, async () => {
    const contract = JSON.parse(readFileSync(new URL(`./hooks-${version}.json`, import.meta.url)));
    const registrations = [];
    const api = {
      pluginConfig: { projectRoot: tmpdir(), continuityId: 'test-awareness-compat' },
      logger: { info() {}, warn() {} },
      on(name, handler) {
        assert.ok(contract.names.includes(name), `unsupported hook ${name}`);
        registrations.push({ name, handler });
      },
    };
    plugin.register(api);
    assert.equal(registrations.length, 1);
    assert.equal(registrations[0].name, 'before_prompt_build');
    const oldFetch = globalThis.fetch;
    let calls = 0;
    globalThis.fetch = async (url) => {
      calls++;
      const query = new URL(url).searchParams;
      assert.equal(query.get('project_root'), tmpdir());
      assert.equal(query.get('continuity_id'), 'test-awareness-compat');
      assert.equal(query.get('session_id'), 'fixture-session');
      for (const name of ['agent_id', 'operator_id', 'workspace_id']) assert.equal(query.has(name), false);
      return { ok: true, json: async () => ({ rendered_card: 'fixture awareness card' }) };
    };
    try {
      assert.deepEqual(await registrations[0].handler({}, { sessionKey: 'fixture-session' }),
        { prependContext: 'fixture awareness card' });
      assert.equal(calls, 1);
    } finally {
      globalThis.fetch = oldFetch;
    }
  });
}

test('the prompt hook preserves exact worktree selectors and rejects a foreign canonical reply', async (t) => {
  const root = tmpdir();
  let handler;
  plugin.register({
    pluginConfig: { projectRoot: root, continuityId: 'remote-stream', workingSubpathId: 'linked', remoteBindingId: 'enrolled' },
    logger: { info() {}, warn() {} },
    on(_name, callback) { handler = callback; },
  });
  const response = {
    rendered_card: 'verified linked worktree card', canonical: true,
    project_root: root, continuity_id: 'remote-stream', working_subpath_id: 'linked', remote_binding_id: 'enrolled',
  };
  t.mock.method(globalThis, 'fetch', async (raw, opts) => {
    const url = new URL(raw);
    assert.equal(url.searchParams.get('working_subpath_id'), 'linked');
    assert.equal(opts.headers['x-scope-remote-binding-id'], 'enrolled');
    assert.equal(opts.redirect, 'error');
    return { ok: true, json: async () => ({ ...response }) };
  });
  assert.deepEqual(await handler({}, { sessionKey: 'new-session' }), { prependContext: response.rendered_card });
  response.working_subpath_id = 'another-worktree';
  const foreign = await handler({}, { sessionKey: 'new-session' });
  assert.match(foreign.prependContext, /degraded/);
  assert.doesNotMatch(foreign.prependContext, /verified linked worktree card/);
  response.canonical = false;
  response.rendered_card = 'scope verification pending';
  assert.deepEqual(await handler({}, { sessionKey: 'new-session' }), { prependContext: response.rendered_card });
});

test('disabled plugin does not register hooks or tools', () => {
  let calls = 0;
  plugin.register({ pluginConfig: { enabled: false }, logger: { info() {}, warn() {} }, on() { calls++; }, registerTool() { calls++; } });
  assert.equal(calls, 0);
});

test('discovery can read the bounded catalog without project or session data', async (t) => {
  const tools = [];
  plugin.register({ pluginConfig: { focusaUrl: 'https://focusa.invalid' }, logger: { info() {}, warn() {} }, on() {}, registerTool(tool) { tools.push(tool); } });
  const manifest = JSON.parse(readFileSync(new URL('./openclaw.plugin.json', import.meta.url)));
  assert.deepEqual(tools.map(tool => tool.name), manifest.contracts.tools);
  const [tool] = tools;
  const mock = t.mock.method(globalThis, 'fetch', async (raw, opts) => {
    const url = new URL(raw);
    assert.equal(url.pathname, '/v1/agent/tools');
    assert.deepEqual(Object.fromEntries(url.searchParams), { limit: '8', query: 'workpoint' });
    assert.equal(opts.redirect, 'error');
    return { ok: true, json: async () => ({ total: 10, tools: Array.from({ length: 10 }, () => ({ capability_id: 'sample', summary: 'read-only', internal: 'not returned' })) }) };
  });
  assert.equal((await tool.execute('invalid', { query: 'x'.repeat(121) })).details.status, 'invalid_input');
  assert.equal(mock.mock.callCount(), 0);
  const result = (await tool.execute('search', { query: ' workpoint ' })).details;
  assert.equal(result.catalog_only, true);
  assert.equal(result.capabilities.length, 8);
  assert.deepEqual(result.capabilities[0], { id: 'sample', description: 'read-only' });
  mock.mock.mockImplementation(async () => ({ ok: true, json: async () => ({ degraded: true, tools: [] }) }));
  assert.equal((await tool.execute('degraded', {})).details.status, 'unavailable');
  mock.mock.mockImplementation(async () => { throw new Error('private error'); });
  assert.equal((await tool.execute('failure', {})).details.status, 'unavailable');
  const cancellation = AbortSignal.abort();
  const cancelled = (await tool.execute('cancelled', {}, cancellation)).details;
  assert.equal(cancelled.status, 'cancelled');
  assert.doesNotMatch(JSON.stringify(cancelled), /private error/);
});

test('recovery guidance qualifies the server envelope before returning advisory data', async (t) => {
  let tool;
  plugin.register({ pluginConfig: { focusaUrl: 'https://focusa.invalid' }, logger: { info() {}, warn() {} }, on() {}, registerTool(candidate) { if (candidate.name === 'focusa_reflex_primitives') tool = candidate; } });
  const body = { status: 'completed', read_only: true, advisory_only: true, items: [{ primitive_id: 'recover', family: 'recovery', failure_envelope: { private: true } }] };
  t.mock.method(globalThis, 'fetch', async raw => {
    const url = new URL(raw);
    assert.equal(url.pathname, '/v1/reflex/primitives');
    assert.deepEqual(Object.fromEntries(url.searchParams), { limit: '8', query: 'recovery' });
    return { ok: true, json: async () => body };
  });
  const result = (await tool.execute('read', { query: 'recovery' })).details;
  assert.equal(result.advisory_only, true);
  assert.deepEqual(result.items, [{ id: 'recover', family: 'recovery', trigger: '', recommended_tool: '' }]);
  body.status = 'blocked';
  assert.equal((await tool.execute('read', { query: 'recovery' })).details.status, 'unavailable');
});

test('unbound source configuration preserves fail-closed behavior', async () => {
  let handler;
  plugin.register({ pluginConfig: {}, logger: { info() {}, warn() {} }, on(_name, fn) { handler = fn; } });
  const result = await handler({}, { sessionKey: 'fixture-session' });
  assert.match(result.prependContext, /no scoped project context/);
  assert.match(result.prependContext, /Continue ordinary work/);
});

test('configured project lookup never substitutes a synthetic native session', async (t) => {
  let handler;
  plugin.register({
    pluginConfig: { projectRoot: tmpdir(), continuityId: 'test-awareness-compat' },
    logger: { info() {}, warn() {} }, on(_name, fn) { handler = fn; },
  });
  const fetch = t.mock.method(globalThis, 'fetch', async () => assert.fail('missing session must not fetch'));
  const result = await handler({}, {});
  assert.match(result.prependContext, /no scoped project context/);
  assert.equal(fetch.mock.callCount(), 0);
});

test('endpoint failure retains degraded fallback', async () => {
  let handler;
  plugin.register({
    pluginConfig: { projectRoot: tmpdir(), continuityId: 'test-awareness-compat' },
    logger: { info() {}, warn() {} }, on(_name, fn) { handler = fn; },
  });
  const oldFetch = globalThis.fetch;
  globalThis.fetch = async () => { throw new Error('fixture endpoint unavailable'); };
  try {
    const result = await handler({}, { sessionKey: 'fixture-session' });
    assert.match(result.prependContext, /Status: degraded/);
    assert.match(result.prependContext, /Focusa awareness read did not complete/);
    assert.doesNotMatch(result.prependContext, /fixture endpoint unavailable/);
  } finally { globalThis.fetch = oldFetch; }
});
