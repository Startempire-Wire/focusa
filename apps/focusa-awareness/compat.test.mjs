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

test('disabled plugin does not register a hook', () => {
  let calls = 0;
  plugin.register({ pluginConfig: { enabled: false }, logger: { info() {}, warn() {} }, on() { calls++; } });
  assert.equal(calls, 0);
});

test('unbound source configuration preserves fail-closed behavior', async () => {
  let handler;
  plugin.register({ pluginConfig: {}, logger: { info() {}, warn() {} }, on(_name, fn) { handler = fn; } });
  const result = await handler({}, { sessionKey: 'fixture-session' });
  assert.match(result.prependContext, /Status: degraded/);
  assert.match(result.prependContext, /project_root and continuity_id lookup inputs are missing/);
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
    assert.match(result.prependContext, /fixture endpoint unavailable/);
  } finally { globalThis.fetch = oldFetch; }
});
