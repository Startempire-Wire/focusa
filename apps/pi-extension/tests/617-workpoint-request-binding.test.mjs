import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync } from 'node:fs';
import ts from 'typescript';
import { loadPureTypeScript, loadRegisteredTool } from './helpers/registered-tool-harness.mjs';

const binding = loadPureTypeScript(new URL('../src/workpoint-request-binding.ts', import.meta.url));
const runtime = () => ({
  currentAsk: { text: 'continue', sourceTurnId: 'turn-1', updatedAt: 1 },
  sessionFrameKey: 'session-1', sessionCwd: '/tmp/focusa-binding-fixture', continuityId: 'continuity-1',
});
const reply = () => ({
  status: 'completed', canonical: true, degraded: false,
  action_authority_for_current_ask: true, matches_current_ask_scope: true,
  workpoint_id: 'fixture-workpoint',
  resume_packet: { canonical: true, project_root: '/tmp/focusa-binding-fixture', workpoint_id: 'fixture-workpoint' },
});

test('affirmative unchanged request binds without mutating the response', () => {
  const r = runtime();
  const captured = binding.captureResumeRequest(r, r.currentAsk.text);
  const response = reply();
  assert.equal(binding.evaluateResumeRequest(response, captured, r).accepted, true);
  const packet = { workpoint: { id: 'fixture-workpoint' } };
  const stamped = binding.stampResumeRequest(packet, captured);
  assert.equal(stamped.current_ask_binding, 'continue');
  assert.equal(stamped.workpoint.current_ask_binding, 'continue');
  assert.equal(packet.current_ask_binding, undefined);
  assert.equal(packet.workpoint.current_ask_binding, undefined);
});

for (const field of ['text', 'sourceTurnId', 'updatedAt', 'sessionFrameKey', 'sessionCwd', 'continuityId']) {
  test(`late response cannot bind after ${field} changes`, () => {
    const r = runtime();
    const captured = binding.captureResumeRequest(r, r.currentAsk.text);
    if (field in r.currentAsk) r.currentAsk[field] = field === 'updatedAt' ? 2 : 'changed';
    else r[field] = 'changed';
    assert.deepEqual(binding.evaluateResumeRequest(reply(), captured, r), {
      accepted: false, reason: 'resume_request_changed',
    });
  });
}

for (const alteration of [
  p => { delete p.action_authority_for_current_ask; },
  p => { delete p.matches_current_ask_scope; },
  p => { p.action_authority_for_current_ask = false; },
  p => { p.canonical = false; },
  p => { p.degraded = true; },
  p => { p.status = 'rejected_scope_mismatch'; },
  p => { p.current_ask_scope = { matches_current_ask_scope: false }; },
  p => { p.resume_packet_v2 = { action_authority_for_current_ask: false }; },
  p => { p.resume_packet_v2 = { current_ask_scope: { matches_current_ask_scope: false } }; },
]) {
  test(`unknown, denied or inconsistent authority stays unadoptable (${alteration})`, () => {
    const r = runtime(), p = reply();
    alteration(p);
    assert.equal(binding.evaluateResumeRequest(p, binding.captureResumeRequest(r, 'continue'), r).accepted, false);
  });
}

test('an explicitly different ask is advisory, not current-request authority', () => {
  const r = runtime();
  const result = binding.evaluateResumeRequest(reply(), binding.captureResumeRequest(r, 'another task'), r);
  assert.deepEqual(result, { accepted: false, reason: 'resume_evaluated_different_ask' });
  assert.doesNotMatch(JSON.stringify(result), /another task/);
});

function toolFixture(onFetch = () => {}, response = reply()) {
  const r = runtime();
  const adopted = [];
  let sent;
  const tool = loadRegisteredTool('focusa_workpoint_resume', {
    ...binding,
    getAttachmentRuntime: () => r,
    resolveFocusaToolProjectRoot: async () => r.sessionCwd,
    projectRootConfirmationGate: () => null,
    isProjectRootAuthoritySafe: () => true,
    ensureContinuityId: () => r.continuityId,
    buildFocusaSessionIdentity: async () => ({}),
    focusaFetchDetailed: async (_path, request) => {
      sent = JSON.parse(request.body);
      await onFetch(r);
      return { ok: true, status: 200, body: response };
    },
    scopeRecoveryContext: () => null,
    summarizeWorkpointResponse: () => 'fixture response',
    normalizeWorkpointResumePacketEnvelope: p => ({ ...p.resume_packet }),
    adoptWorkpointScopeForFrameRecovery: p => { adopted.push(p); return r.sessionCwd; },
    setActiveWorkpointSummary: () => {},
    persistState: () => {},
    compactApiEcho: value => value,
  });
  return { r, adopted, tool, sent: () => sent };
}

test('registered manual resume stamps the evaluated ask before scope adoption', async () => {
  const f = toolFixture();
  const result = await f.tool.execute('fixture-call', {});
  assert.equal(result.details.action_authority_for_current_ask, true);
  assert.equal(f.adopted.length, 1);
  assert.equal(f.adopted[0].current_ask_binding, f.sent().current_ask);
});

test('registered manual resume never adopts a late reply over a newer ask', async () => {
  const f = toolFixture(r => { r.currentAsk = { text: 'stop', sourceTurnId: 'turn-2', updatedAt: 2 }; });
  const result = await f.tool.execute('fixture-call', {});
  assert.equal(f.adopted.length, 0);
  assert.equal(result.details.action_authority_for_current_ask, false);
  assert.equal(result.details.scope_conflict_reason, 'resume_request_changed');
});

test('registered different-ask override never becomes current harness authority', async () => {
  const f = toolFixture();
  const result = await f.tool.execute('fixture-call', { current_ask: 'different task' });
  assert.equal(f.adopted.length, 0);
  assert.equal(result.details.canonical, false);
  assert.equal(result.details.scope_conflict_reason, 'resume_evaluated_different_ask');
});

test('registered legacy reply without affirmative scope flags is not adopted', async () => {
  const p = reply();
  delete p.matches_current_ask_scope;
  const f = toolFixture(() => {}, p);
  const result = await f.tool.execute('fixture-call', {});
  assert.equal(f.adopted.length, 0);
  assert.equal(result.details.action_authority_for_current_ask, false);
});

// Isolate the actual lifecycle function, not a reimplementation of its decisions.
function lifecycleFixture(onFetch = () => {}, response = reply()) {
  const source = readFileSync(new URL('../src/session.ts', import.meta.url), 'utf8');
  const start = source.indexOf('async function refreshSessionWorkpointPacket(');
  const end = source.indexOf('async function promptForConfirmedProjectRoot(', start);
  assert.ok(start >= 0 && end > start);
  const compiled = ts.transpileModule(source.slice(start, end), {
    compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None },
  }).outputText;
  const r = { ...runtime(), focusaAvailable: true };
  const writes = [], events = [];
  const deps = {
    ...binding,
    getAttachmentRuntime: () => r,
    isProjectRootAuthoritySafe: () => true,
    getSessionCwd: () => r.sessionCwd,
    ensureContinuityId: () => r.continuityId,
    focusaFetch: async () => { await onFetch(r); return response; },
    normalizeWorkpointResumePacketEnvelope: p => ({ ...p.resume_packet }),
    isWorkpointPacketScopedToCurrentSession: () => true,
    stampWorkpointPacketForCurrentPiSession: p => p,
    setActiveWorkpointPacket: p => writes.push(p),
    setActiveWorkpointSummary: () => {},
    focusaPost: (_path, event) => events.push(event),
  };
  const refresh = new Function(...Object.keys(deps), compiled + '\nreturn refreshSessionWorkpointPacket;')(...Object.values(deps));
  return { refresh, writes, events };
}

test('startup uses the same affirmative binding and stamps the evaluated ask', async () => {
  const f = lifecycleFixture();
  await f.refresh('fixture');
  assert.equal(f.writes.length, 1);
  assert.equal(f.writes[0].current_ask_binding, 'continue');
});

test('late startup reply cannot clear or replace a newer session packet', async () => {
  const f = lifecycleFixture(r => { r.sessionFrameKey = 'new-session'; });
  await f.refresh('fixture');
  assert.equal(f.writes.length, 0);
  assert.equal(f.events[0].payload.reason, 'resume_request_changed');
});

test('late rejected startup reply also preserves newer session state', async () => {
  const f = lifecycleFixture(r => { r.sessionFrameKey = 'new-session'; }, { status: 'rejected_scope_mismatch' });
  await f.refresh('fixture');
  assert.equal(f.writes.length, 0);
  assert.equal(f.events[0].payload.reason, 'resume_request_changed');
});

test('startup does not adopt legacy replies with unknown scope verdict', async () => {
  const p = reply();
  delete p.matches_current_ask_scope;
  const f = lifecycleFixture(() => {}, p);
  await f.refresh('fixture');
  assert.deepEqual(f.writes, [null]);
});

test('overlapping same-text requests adopt only the current turn response', async () => {
  let releaseOld;
  const oldReply = new Promise(resolve => { releaseOld = resolve; });
  let requests = 0;
  const f = toolFixture(async () => { if (++requests === 1) await oldReply; });
  const oldResult = f.tool.execute('old-call', {});
  await new Promise(resolve => setImmediate(resolve));
  f.r.currentAsk = { text: 'continue', sourceTurnId: 'turn-2', updatedAt: 2 };
  const currentResult = await f.tool.execute('current-call', {});
  releaseOld();
  const staleResult = await oldResult;
  assert.equal(currentResult.details.action_authority_for_current_ask, true);
  assert.equal(staleResult.details.scope_conflict_reason, 'resume_request_changed');
  assert.equal(f.adopted.length, 1);
});

test('1000 captured transitions reject obsolete replies without changing state', () => {
  const r = runtime(), captured = [];
  for (let i = 0; i < 1000; i++) {
    r.currentAsk = { text: 'continue', sourceTurnId: `turn-${i}`, updatedAt: i };
    captured.push(binding.captureResumeRequest(r, 'continue'));
  }
  const before = structuredClone(r);
  let accepted = 0;
  for (const request of captured.reverse()) {
    if (binding.evaluateResumeRequest(reply(), request, r).accepted) accepted++;
  }
  assert.equal(accepted, 1);
  assert.deepEqual(r, before);
});
