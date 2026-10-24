import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { loadSourceSection } from './helpers/source-section-harness.mjs';
import { loadRegisteredTool } from './helpers/registered-tool-harness.mjs';

const source = process.env.FOCUSA_TEST_TOOLS_SOURCE
  ? readFileSync(process.env.FOCUSA_TEST_TOOLS_SOURCE, 'utf8') : undefined;
const now = Date.parse('2026-10-22T00:00:00Z');
const leaseFromStatus = loadSourceSection('state.ts', 'const WORK_LOOP_STATUS_SCHEMA',
  '// ── HTTP helper', {}, 'currentWorkLoopWriterLease');
const status = {
  schema: 'focusa.work_loop_status.v3', state: 'healthy',
  execution_partition: {
    writer_key: 'pi-test', fencing_token: 7, lease_freshness: 'current',
    lease_expires_at: new Date(now + 1000).toISOString(),
  },
};
assert.deepEqual(leaseFromStatus(status, 'pi-test', now), {
  writerId: 'pi-test', fencingToken: 7, expiresAt: new Date(now + 1000).toISOString(),
});
for (const patch of [
  { writer_key: 'foreign' }, { fencing_token: 0 }, { fencing_token: -1 },
  { fencing_token: true }, { fencing_token: '7' }, { fencing_token: null },
  { fencing_token: 1.5 }, { fencing_token: Number.MAX_SAFE_INTEGER + 1 },
  { lease_freshness: 'expired' }, { lease_expires_at: new Date(now).toISOString() },
  { lease_expires_at: 'invalid' },
]) {
  assert.equal(leaseFromStatus({ ...status, execution_partition: { ...status.execution_partition, ...patch } }, 'pi-test', now), null);
}
for (const invalid of [null, {}, { ...status, schema: 'unknown' }, { ...status, state: 'unknown' }]) {
  assert.equal(leaseFromStatus(invalid, 'pi-test', now), null);
}
const writerLeaseHeaders = loadSourceSection('tools.ts', '  function writerLeaseHeaders(',
  '  async function requiredWriterLeaseHeaders(', {}, 'writerLeaseHeaders');
const unverifiedWorkLoopWriterMessage = loadSourceSection('tools.ts', '  function unverifiedWorkLoopWriterMessage(',
  '  async function requiredWriterLeaseHeaders(', {}, 'unverifiedWorkLoopWriterMessage');
const compatibleWorkLoopStatusState = loadSourceSection('state.ts', 'const WORK_LOOP_STATUS_SCHEMA',
  '// ── HTTP helper', {}, 'compatibleWorkLoopStatusState');
function control(lease, response = { ok: true, status: 200, body: { ok: true } }) {
  const requests = [];
  const tool = loadRegisteredTool('focusa_work_loop_control', {
    preferredWriterId: async () => 'pi-test',
    currentWorkLoopLease: async () => lease,
    writerLeaseHeaders, unverifiedWorkLoopWriterMessage,
    focusaFetchDetailed: async (path, options) => { requests.push({ path, options }); return response; },
    explainWorkLoopResult: () => response.ok ? 'accepted' : 'rejected',
    compactApiEcho: value => value,
    workLoopLeases: new Map(), workLoopLeaseKey: () => 'verified-scope',
    rememberWorkLoopLease: () => null,
  }, [], source);
  return { tool, requests };
}
for (const action of ['pause', 'stop']) {
  const { tool, requests } = control(null);
  const result = await tool.execute('test', { action, reason: 'operator recovery' });
  assert.equal(requests.length, 1, `${action} must reach the daemon when no lease is available locally`);
  assert.equal(requests[0].path, `/work-loop/${action}`);
  assert.deepEqual(requests[0].options.headers, { 'x-focusa-writer-id': 'pi-test' });
  assert.equal(result.details.ok, true);
}
{
  const { tool, requests } = control(null);
  const result = await tool.execute('test', { action: 'resume' });
  assert.equal(result.details.ok, false);
  assert.equal(result.details.failure_class, 'writer_lease_unverified');
  assert.equal(result.details.status, 'degraded');
  assert.equal(result.details.request_sent, false);
  assert.match(result.content[0].text, /resume request not sent/);
  assert.doesNotMatch(result.content[0].text, /blocked|owned by another writer/);
  assert.equal(requests.length, 0, 'resume must not turn missing authority into recovery');
}
{
  const { tool, requests } = control(null, { ok: false, status: 409, body: { failure_class: 'writer_conflict' } });
  const result = await tool.execute('test', { action: 'stop' });
  assert.equal(requests.length, 1);
  assert.equal(result.details.ok, false, 'daemon refusal must remain a refusal');
}
{
  const lease = { writerId: 'pi-test', fencingToken: 7, expiresAt: status.execution_partition.lease_expires_at };
  const { tool, requests } = control(lease);
  await tool.execute('test', { action: 'stop' });
  assert.equal(requests[0].options.headers['x-focusa-fencing-token'], '7');
  assert.deepEqual(writerLeaseHeaders('pi-test', { ...lease, writerId: 'foreign' }), { 'x-focusa-writer-id': 'pi-test' });
}
{
  const tool = loadRegisteredTool('focusa_work_loop_writer_status', {
    focusaFetchDetailed: async () => ({ ok: true, status: 200, body: { ...status, status: 'idle', active_writer: null } }),
    compatibleWorkLoopStatusState,
    compactApiEcho: value => value,
  }, [], source);
  const result = await tool.execute('test', {});
  assert.deepEqual(result.details.preflight.recovery_controls, ['pause', 'stop']);
  assert.equal(result.details.preflight.writer_required_for.includes('control'), false);
  assert.equal(result.details.preflight.mutates, false);
  assert.equal(result.details.active_writer, 'none');
  assert.equal(result.details.writer_observation_verified, true);
}
for (const response of [
  { ok: false, status: 503, body: {} },
  { ok: true, status: 200, body: { ...status, status: 'idle' } },
  { ok: true, status: 200, body: { ...status, active_writer: '' } },
  { ok: true, status: 200, body: { schema: 'unsupported', active_writer: null } },
]) {
  const tool = loadRegisteredTool('focusa_work_loop_writer_status', {
    focusaFetchDetailed: async () => response,
    compatibleWorkLoopStatusState,
    compactApiEcho: value => value,
  }, [], source);
  const result = await tool.execute('test', {});
  assert.equal(result.details.active_writer, 'unverified');
  assert.equal(result.details.writer_observation_verified, false);
  assert.equal(result.details.ok, false);
}
{
  const { tool, requests } = control(null);
  const result = await tool.execute('test', { action: 'pause', preflight: true });
  assert.match(result.content[0].text, /route preview only/);
  assert.equal(result.details.request_sent, false);
  assert.equal(result.details.ownership_checked, false);
  assert.equal(result.details.budget_checked, false);
  assert.equal(result.details.execution_admission_checked, false);
  assert.equal(requests.length, 0);
}
console.log('PASS: actual registered pause/stop recovery, fenced resume, daemon refusal and shared lease projection');
