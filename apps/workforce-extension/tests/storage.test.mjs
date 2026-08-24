import assert from 'node:assert/strict';
import { test } from 'node:test';
import { redactConnection } from '../src/lib/contracts.mjs';
import { forgetConnection, listConnections, saveConnection } from '../src/lib/storage.mjs';

function mockChrome() {
  let state = {};
  const calls = [];
  return {
    calls,
    storage: {
      local: {
        get: async (key) => ({ [key]: state[key] }),
        set: async (value) => { calls.push(value); state = { ...state, ...structuredClone(value) }; },
      },
      sync: { set: async () => { throw new Error('sync storage forbidden'); } },
    },
  };
}
const record = (id='c-1') => ({
  schema: 'focusa.workforce_connection.v1', connection_id: id, label: 'KH', base_url: 'https://focusa.example/',
  device_id: 'device-1', token: 'secret-token', granted_scopes: ['write','read'], last_cursor: '42',
  created_at: '2026-08-24T12:00:00Z', last_connected_at: null,
});

test('connection token round-trips only through local storage', async () => {
  const chrome = mockChrome(); const saved = await saveConnection(record(), chrome);
  assert.equal(saved.token, 'secret-token'); assert.equal((await listConnections(chrome))[0].base_url, 'https://focusa.example');
  assert.equal('sync' in chrome.storage, true); assert.equal(chrome.calls.length, 1);
  assert.equal(JSON.stringify(redactConnection(saved)).includes('secret-token'), false);
  assert.equal(redactConnection(saved).token, '••••');
});

test('save replaces same id and forget removes token record', async () => {
  const chrome = mockChrome(); await saveConnection(record(), chrome);
  await saveConnection({ ...record(), label: 'KH renamed', token: 'rotated' }, chrome);
  const listed = await listConnections(chrome); assert.equal(listed.length, 1); assert.equal(listed[0].token, 'rotated');
  assert.equal(await forgetConnection('c-1', chrome), true); assert.deepEqual(await listConnections(chrome), []);
  assert.equal(await forgetConnection('missing', chrome), false);
});
