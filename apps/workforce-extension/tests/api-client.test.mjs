import test from 'node:test';
import assert from 'node:assert/strict';
import { fetchHealth, fetchRoster, fetchWorkLoop, ProjectionRequestError } from '../src/lib/api-client.mjs';

function reply(status, body) {
  return { ok: status >= 200 && status < 300, status, text: async () => typeof body === 'string' ? body : JSON.stringify(body) };
}

test('authenticated reads send bearer and read-only permission headers', async () => {
  const calls = [];
  const fixtures = {
    '/v1/health': { schema: 'focusa.health.v1', ok: true },
    '/v1/work-loop/status?summary_only=true': { schema: 'focusa.work_loop_status.v3' },
    '/v1/silent-sessions': { schema: 'focusa.silent_session_api_envelope.v1', ok: true, data: [] },
  };
  const fetchImpl = async (url, options) => { calls.push({ url: url.href, options }); return reply(200, fixtures[url.pathname + url.search]); };
  await fetchHealth({ baseUrl: 'https://daemon.example', token: 'secret', fetchImpl });
  await fetchWorkLoop({ baseUrl: 'https://daemon.example', token: 'secret', fetchImpl });
  await fetchRoster({ baseUrl: 'https://daemon.example', token: 'secret', fetchImpl });
  for (const call of calls) {
    assert.equal(call.options.headers.authorization, 'Bearer secret');
    assert.equal(call.options.headers['x-focusa-permissions'], 'read:*');
    assert.equal(call.options.method, 'GET');
  }
});

test('401, 403, unsupported, and degraded remain distinct', async () => {
  for (const [status, kind] of [[401,'unauthenticated'],[403,'forbidden'],[404,'unsupported'],[503,'degraded']]) {
    await assert.rejects(fetchHealth({ baseUrl: 'https://daemon.example', token: 'secret', fetchImpl: async () => reply(status, {}) }),
      (error) => error instanceof ProjectionRequestError && error.kind === kind && error.status === status);
  }
});

test('unknown or malformed schemas fail closed', async () => {
  await assert.rejects(fetchHealth({ baseUrl: 'https://daemon.example', token: 'secret', fetchImpl: async () => reply(200, { ok: true }) }),
    (error) => error.kind === 'unsupported');
  await assert.rejects(fetchHealth({ baseUrl: 'https://daemon.example', token: 'secret', fetchImpl: async () => reply(200, 'not-json') }),
    (error) => error.kind === 'invalid_envelope');
});
