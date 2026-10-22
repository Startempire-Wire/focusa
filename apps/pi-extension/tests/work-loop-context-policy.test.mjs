import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { loadSourceSection } from './helpers/source-section-harness.mjs';
import { loadRegisteredTool } from './helpers/registered-tool-harness.mjs';

const source = process.env.FOCUSA_TEST_TOOLS_SOURCE
  ? readFileSync(process.env.FOCUSA_TEST_TOOLS_SOURCE, 'utf8') : undefined;
const writerId = 'pi-context-test';
const writerLeaseHeaders = loadSourceSection('tools.ts', '  function writerLeaseHeaders(',
  '  async function requiredWriterLeaseHeaders(', {}, 'writerLeaseHeaders');
function context(lease, response) {
  const calls = [];
  const deps = {
    localWriterId: writerId, currentWorkLoopLease: async () => lease,
    writerLeaseHeaders, getTurnCount: () => 1,
    explainWorkLoopResult: r => String(r.body?.status || r.status),
    compactApiEcho: body => body,
    focusaFetchDetailed: async (path, options) => { calls.push({ path, options }); return response; },
  };
  deps.requiredWriterLeaseHeaders = loadSourceSection('tools.ts',
    '  async function requiredWriterLeaseHeaders(', '  function firstBdReadyIdFromText(', deps,
    'requiredWriterLeaseHeaders');
  return { calls, tool: loadRegisteredTool('focusa_work_loop_context', deps, [], source) };
}
const accepted = { ok: true, status: 200, body: { status: 'accepted', writer_id: null } };
const empty = context(null, accepted);
const result = await empty.tool.execute('fixture', { current_ask: 'Verify scoped operator metadata' });
assert.equal(result.details.ok, true);
assert.equal(empty.calls.length, 1);
assert.equal(empty.calls[0].path, '/work-loop/context');
assert.equal(empty.calls[0].options.headers['x-focusa-writer-id'], writerId);
assert.equal(empty.calls[0].options.headers['x-focusa-fencing-token'], undefined);
for (const response of [
  { ok: false, status: 409, body: { error: 'existing writer claim rejects this caller' } },
  { ok: false, status: 503, body: { error: 'disk quota exceeded', status: 'failed' } },
]) {
  const refused = context(null, response);
  const result = await refused.tool.execute('fixture', { current_ask: 'Verify scoped operator metadata' });
  assert.equal(result.details.ok, false);
  assert.equal(result.details.status, response.status);
  assert.deepEqual(result.details.response, response.body);
}
const owned = context({ writerId, fencingToken: 7 }, accepted);
await owned.tool.execute('fixture', { current_ask: 'Verify scoped operator metadata' });
assert.equal(owned.calls[0].options.headers['x-focusa-fencing-token'], '7');
const invalid = context(null, accepted);
assert.equal((await invalid.tool.execute('fixture', { current_ask: '' })).details.ok, false);
assert.equal(invalid.calls.length, 0);
console.log('PASS: context metadata reaches daemon without acquiring ownership; existing ownership and storage refusals stay failures');
