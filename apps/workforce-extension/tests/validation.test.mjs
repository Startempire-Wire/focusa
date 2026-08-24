import assert from 'node:assert/strict';
import { test } from 'node:test';
import { normalizeDaemonOrigin, originPermission, requestDaemonOriginPermission } from '../src/lib/validation.mjs';

test('daemon origins require HTTPS except exact loopback', () => {
  assert.equal(normalizeDaemonOrigin('https://focusa.example/'), 'https://focusa.example');
  assert.equal(normalizeDaemonOrigin('http://127.0.0.1:8787'), 'http://127.0.0.1:8787');
  assert.equal(normalizeDaemonOrigin('http://localhost:8787/'), 'http://localhost:8787');
  assert.equal(normalizeDaemonOrigin('http://[::1]:8787'), 'http://[::1]:8787');
  for (const value of ['http://focusa.example', 'ftp://focusa.example', 'https://user:pass@focusa.example', 'https://focusa.example/path', 'https://focusa.example/?token=x']) {
    assert.throws(() => normalizeDaemonOrigin(value));
  }
});

test('host permission is requested for exact origin from caller gesture', async () => {
  const calls=[]; const chrome={permissions:{request:async (request)=>{calls.push(request);return true;}}};
  assert.equal(originPermission('https://focusa.example'), 'https://focusa.example/*');
  assert.equal(await requestDaemonOriginPermission('https://focusa.example', chrome), true);
  assert.deepEqual(calls, [{ origins: ['https://focusa.example/*'] }]);
});
