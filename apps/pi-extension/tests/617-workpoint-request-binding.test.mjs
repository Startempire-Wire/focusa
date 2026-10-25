import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync, mkdtempSync, readdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import { tmpdir } from 'node:os';
import ts from 'typescript';
import { loadPureTypeScript, loadRegisteredTool } from './helpers/registered-tool-harness.mjs';
import { loadSourceSection } from './helpers/source-section-harness.mjs';

const binding = loadPureTypeScript(new URL('../src/workpoint-request-binding.ts', import.meta.url));
function compactionRequestFixture({ onPrepare, onVerify, legacy = false } = {}) {
  const r = runtime();
  r.focusaAvailable = true; r.currentTier = 'normal';
  r.localDecisions = []; r.localConstraints = []; r.localFailures = [];
  const scope = () => ({ root_scope: { root_path: r.sessionCwd }, continuity_id: r.continuityId });
  const sent = []; const queued = [];
  const projection = loadPureTypeScript(new URL('../src/compaction-resume-projection.ts', import.meta.url));
  const safety = loadPureTypeScript(new URL('../src/compaction-resume-safety.ts', import.meta.url));
  const dependencies = {
    ...binding, getAttachmentRuntime: () => r, currentCompactionScope: scope,
    getSessionFrameKey: () => r.sessionFrameKey, getTotalCompactions: () => 0,
    sanitizeFocusFailures: (x) => x || [], scheduleCompactionMemoryEvaluation() {},
    normalizeCompactionArtifacts: (x) => x || [], queueCompactionResumeContext: (_ctx, text) => queued.push(text),
    refreshTrajectoryClarityLifecycle: async () => {}, persistState() {}, updateNorthStarCard() {},
    ...safety, ...projection, renderResumeProjection: (p) => projection.renderCompactionResumeProjection(p, 'normal'),
    focusaFetch: async (route, options) => {
      const body = JSON.parse(options.body); sent.push({ route, body });
      if (route.endsWith('/prepare')) {
        const packet = {
          schema_version: 'focusa.compaction_mission_packet.v1', status: 'degraded', packet_id: 'epoch-1',
          scope: { project_root: r.sessionCwd, continuity_id: r.continuityId, scope_status: 'verified' },
          current_ask: { text: body.current_ask, text_complete: !legacy, source_turn_id: body.epoch.source_turn_id },
        };
        const digest = `sha256:${createHash('sha256').update(JSON.stringify(packet)).digest('hex')}`;
        const result = { epoch_id: 'epoch-1', status: 'prepared', semantic_digest: digest, resume_projection: packet };
        await onPrepare?.(r, result, body);
        return result;
      }
      const result = { status: 'verified', required_fields_preserved: true, semantic_digest: body.expected_semantic_digest };
      onVerify?.(r, result, body);
      return result;
    },
  };
  const lifecycle = loadSourceSection('compaction.ts', 'type CompactionPrepareResult = {',
    'export function registerCompaction', dependencies,
    '{prepareCompactionEpoch, runPostCompactionVerification, epoch: () => activeCompactionEpoch}');
  return { r, sent, queued, ...lifecycle };
}

test('actual prepare caller sends full late restrictions and source-turn identity', async () => {
  const f = compactionRequestFixture();
  f.r.currentAsk.text = `${'Review only. '.repeat(600)}NEVER DEPLOY OR CHANGE CUSTOMER DATA.`;
  const result = await f.prepareCompactionEpoch({});
  assert.ok(result); assert.equal(f.sent[0].body.current_ask, f.r.currentAsk.text);
  assert.equal(f.sent[0].body.epoch.source_turn_id, 'turn-1');
  assert.equal(f.epoch().requestBinding.currentAsk, f.r.currentAsk.text);
});

test('literal continuation is preserved, not converted into an absent request', async () => {
  const f = compactionRequestFixture(); await f.prepareCompactionEpoch({});
  assert.equal(f.sent[0].body.current_ask, 'continue');
});

for (const failure of ['changed-request', 'partial-producer', 'producer-tail-loss']) {
  test(`prepare refuses ${failure} without adopting stale context`, async () => {
    const f = compactionRequestFixture({ legacy: failure === 'partial-producer', onPrepare: (r, result) => {
      if (failure === 'changed-request') r.currentAsk.sourceTurnId = 'new-turn';
      if (failure === 'producer-tail-loss') result.resume_projection.current_ask.text = 'clipped';
    } });
    assert.equal(await f.prepareCompactionEpoch({}), null); assert.equal(f.epoch(), null);
  });
}

for (const failure of ['none', 'digest-mismatch', 'changed-request', 'changed-scope', 'legacy-verify', 'prepared-tamper']) {
  test(`actual verify caller preserves steering and rejects ${failure} identity drift`, async () => {
    const f = compactionRequestFixture({ onVerify: (r, result) => {
      if (failure === 'digest-mismatch') result.semantic_digest = 'sha256:foreign';
      if (failure === 'changed-request') r.currentAsk.text = 'Stop. Review only.';
      if (failure === 'changed-scope') r.continuityId = 'foreign';
      if (failure === 'legacy-verify') delete result.semantic_digest;
      if (failure === 'prepared-tamper') f.epoch().prepare.resume_projection.current_ask.text = 'tampered';
    } });
    f.r.currentAsk.text = 'Read-only review; no deployment.';
    const prepared = await f.prepareCompactionEpoch({});
    await f.runPostCompactionVerification({ compactionEntry: { id: 'native-1' } }, { hasUI: false });
    assert.equal(f.sent[1].body.current_ask, 'Read-only review; no deployment.');
    assert.equal(f.sent[1].body.expected_semantic_digest, prepared.semantic_digest);
    if (failure === 'none') {
      assert.equal(f.queued.length, 1); assert.ok(f.queued[0].includes('Read-only review; no deployment.'));
    } else { assert.equal(f.queued.length, 0); assert.equal(f.r.compactResumeDeliveryState, 'failed'); }
  });
}

test('overlapping prepare replies cannot reactivate the superseded invocation', async () => {
  let releaseFirst, firstStarted, calls = 0;
  const entered = new Promise(resolve => { firstStarted = resolve; });
  const f = compactionRequestFixture({ onPrepare: async (_r, result) => {
    const n = ++calls;
    result.epoch_id = `epoch-${n}`;
    if (n === 1) {
      firstStarted(); await new Promise(resolve => { releaseFirst = resolve; });
    }
  } });
  const first = f.prepareCompactionEpoch({}); await entered;
  const second = await f.prepareCompactionEpoch({});
  assert.equal(second.epoch_id, 'epoch-2');
  releaseFirst(); assert.equal(await first, null);
  assert.equal(f.epoch().epochId, 'epoch-2');
});

test('a failed later prepare cannot leave an earlier epoch available for native completion', async () => {
  let count = 0;
  const f = compactionRequestFixture({ onPrepare: (_r, result) => {
    if (++count === 2) result.status = 'blocked';
  } });
  assert.ok(await f.prepareCompactionEpoch({}));
  assert.equal(await f.prepareCompactionEpoch({}), null);
  assert.equal(f.epoch(), null);
});

test('real transport does not replay unknown mutation completion and preserves parent cancellation', async () => {
  const r = runtime(); r.cfg = { focusaApiBaseUrl: 'http://127.0.0.1:1/v1', focusaApiTimeoutMs: 1000 };
  const attachment = () => ({ workstream: { root_scope: { root_path: r.sessionCwd }, continuity_id: r.continuityId }, session_id: r.sessionFrameKey });
  let calls = 0, receivedSignal;
  const transport = loadSourceSection('state.ts', 'export async function focusaFetch(', '// Fire-and-forget variant', {
    getAttachmentRuntime: () => r, currentAttachmentKey: attachment, isProjectRootAuthoritySafe: () => true,
    fetch: async (_url, opts) => {
      calls++; receivedSignal = opts.signal;
      throw new Error('synthetic unknown completion');
    },
  }, 'focusaFetch');
  assert.equal(await transport('/compaction/prepare', { method: 'POST', body: '{}' }), null);
  assert.equal(calls, 1, 'mutations are not blindly replayed');
  const controller = new AbortController(); controller.abort();
  assert.equal(await transport('/compaction/prepare', { method: 'POST', body: '{}', signal: controller.signal }), null);
  assert.equal(calls, 1, 'already-aborted parent dispatches no request');
  assert.ok(receivedSignal instanceof AbortSignal);
});

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
  const replacement = { ...r, currentAsk: { ...r.currentAsk } };
  assert.equal(binding.evaluateResumeRequest(response, captured, replacement).accepted, false);
  assert.equal(binding.evaluateResumeRequest(response,
    binding.captureResumeRequest(replacement, replacement.currentAsk.text), replacement).accepted, true);
  assert.deepEqual(JSON.parse(JSON.stringify(captured)), {
    evaluatedAsk: 'continue', currentAsk: 'continue', turnId: 'turn-1', askUpdatedAt: 1,
    sessionId: 'session-1', projectRoot: r.sessionCwd, continuityId: r.continuityId,
  });
});

test('actual session refresh fences a retained attachment lifecycle and permits fresh revalidation', async () => {
  const r = { ...runtime(), focusaAvailable: true };
  let current = true, expire = true, requests = 0, adopted = 0;
  const refresh = loadSourceSection('session.ts',
    'async function refreshSessionWorkpointPacket(', 'async function promptForConfirmedProjectRoot(', {
      ...binding, getAttachmentRuntime: () => r, getSessionCwd: () => r.sessionCwd,
      isProjectRootAuthoritySafe: () => true, ensureContinuityId: () => r.continuityId,
      focusaFetch: async () => { requests++; if (expire) current = false; return reply(); },
      setActiveWorkpointPacket: () => { adopted++; }, setActiveWorkpointSummary() {}, focusaPost() {},
      normalizeWorkpointResumePacketEnvelope: x => x, isWorkpointPacketScopedToCurrentSession: () => true,
      stampWorkpointPacketForCurrentPiSession: x => x,
    }, 'refreshSessionWorkpointPacket');
  await refresh('fixture', () => current);
  assert.equal(requests, 1);
  assert.equal(adopted, 0);
  current = true; expire = false;
  await refresh('fixture', () => current);
  assert.equal(requests, 2);
  assert.equal(adopted, 1);
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

function toolFixture(onFetch = () => {}, response = reply(), adoptionAllowed = true, transportOk = true) {
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
      return { ok: transportOk, status: transportOk ? 200 : 500, body: response };
    },
    scopeRecoveryContext: () => null,
    summarizeWorkpointResponse: () => 'fixture response',
    explainWorkLoopResult: () => 'fixture transport failure',
    normalizeWorkpointResumePacketEnvelope: p => ({ ...p.resume_packet }),
    adoptWorkpointScopeForFrameRecovery: p => {
      if (!adoptionAllowed) return null;
      adopted.push(p);
      return r.sessionCwd;
    },
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

test('local scope rejection overrides a positive response authority flag', async () => {
  const f = toolFixture(() => {}, reply(), false);
  const result = await f.tool.execute('fixture-call', {});
  assert.equal(f.adopted.length, 0);
  assert.equal(result.details.action_authority_for_current_ask, false);
  assert.equal(result.details.matches_current_ask_scope, false);
  assert.equal(result.details.scope_conflict_reason, 'resume_scope_adoption_rejected');
});

test('failed transport never grants authority from an optimistic response body', async () => {
  const f = toolFixture(() => {}, reply(), true, false);
  const result = await f.tool.execute('fixture-call', {});
  assert.equal(f.adopted.length, 0);
  assert.equal(result.details.action_authority_for_current_ask, false);
  assert.equal(result.details.matches_current_ask_scope, false);
  assert.equal(result.details.scope_conflict_reason, 'resume_transport_failed');
});

// Exercise the actual capture, recovery producer, native anchor and restoration
// code; disk integrity, version interoperability and live model use have their
// own independent proofs and are not implied by these bounded fixtures.
const require = createRequire(import.meta.url);
const persistenceExports = {};
new Function('exports', 'require', ts.transpileModule(
  readFileSync(new URL('../src/persistence.ts', import.meta.url), 'utf8'),
  { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS } }
).outputText)(persistenceExports, require);

const fullInstruction = suffix => `${'Keep this accepted purpose.\n'.repeat(60)}${suffix}`;

function captureInput(text) {
  const r = runtime();
  loadSourceSection('turns.ts', '    const storedAskText =', '    const activeWorkpoint =', {
    cleanedText: text, askKind: 'instruction', text, sourceTurnId: 'turn-2',
    getAttachmentRuntime: () => r, getTurnCount: () => 1,
    setCurrentTaskTurnStart: value => { r.currentTaskTurnStart = value; },
    estimateTokens: value => Math.ceil(value.length / 4),
    getSessionCwd: () => r.sessionCwd, getContinuityId: () => r.continuityId,
  }, 'undefined');
  return r;
}

function persistenceFixture(ask) {
  const anchors = [], writes = [];
  const r = {
    ...runtime(), currentAsk: ask, activeFrameTitle: 'fixture', activeFrameGoal: '',
    localDecisions: [], localConstraints: [], localFailures: [],
    lastFocusSnapshot: { decisions: [], constraints: [], failures: [], intent: '', currentFocus: '' },
    projectSwitchLedger: [], vitalInfoPrompted: {}, pendingLifecycleAdvisories: {},
    piSessionProjectRegistry: {}, projectBindingDecisions: {}, cataloguedDecisions: [], cataloguedFacts: [],
    persistRevision: 0, lastPersistAt: 0, pi: { appendEntry: (type, data) => anchors.push({ type, data }) },
  };
  const deps = {
    ...persistenceExports, getAttachmentRuntime: () => r,
    getScopedWorkpointPacket: () => null, getActiveWorkpointSummary: () => '',
    getLastProjectRootResolution: () => ({ projectRoot: r.sessionCwd }),
    normalizeProjectRoot: value => value, getLastTrajectoryClarity: () => null,
    getLastProjectIdentity: () => null, getLastProjectVerify: () => null,
    getLatestReportSummary: () => null, getTurnCount: () => 0, getTotalCompactions: () => 0,
    sanitizeFocusFailures: values => values, PROJECT_SWITCH_LEDGER_MAX_OBSERVATIONS: 6,
    focusaPost: () => {}, getFocusState: async () => ({}),
    writeRecoverySidecar: (state, digest, revision) => {
      writes.push({ state: structuredClone(state), digest, revision });
      return { key: 'a'.repeat(64), bytes: Buffer.byteLength(JSON.stringify(state)) };
    },
  };
  const methods = loadSourceSection('state.ts', 'const MAX_PERSIST_LIST_ITEMS =',
    '// ── Estimate tokens from bytes', deps, '({ buildPersistedRecoveryState, persistState })');
  return { r, anchors, writes, ...methods };
}

function restoreFixture(data, currentAsk = null) {
  const r = { ...runtime(), currentAsk };
  const restore = loadSourceSection('session.ts', 'function seedCurrentAskFromPersistedState(',
    'async function ensureActiveFrame(', {
      getAttachmentRuntime: () => r, stripQuotedFocusaContext: value => String(value),
      adoptPiProjectRoot: value => value, getSessionCwd: () => r.sessionCwd,
      getContinuityId: () => r.continuityId, classifyCurrentAsk: () => 'instruction',
      isNonTaskStatusLikeText: () => false,
    }, 'seedCurrentAskFromPersistedState');
  restore({ cwd: r.sessionCwd }, data);
  return r;
}

function exactAsk(text) {
  const r = runtime();
  return { ...r.currentAsk, text, textIsPreview: false,
    sessionId: r.sessionFrameKey, projectRoot: r.sessionCwd, continuityId: r.continuityId };
}

test('startup recovery retains a newer captured instruction and its original source turn', () => {
  const fresh = exactAsk(fullInstruction('NEW STEERING: inspect only, never deploy.'));
  const restored = restoreFixture({ currentAsk: exactAsk('old release instruction') }, fresh);
  assert.equal(restored.currentAsk, fresh);
});

test('startup recovery cannot revive old work after empty meta steering', () => {
  const fresh = { ...exactAsk(''), kind: 'meta', sourceTurnId: 'new-meta-turn' };
  assert.equal(restoreFixture({ currentAsk: exactAsk('old release instruction') }, fresh).currentAsk, fresh);
});

function startupRecoveryFixture(data, fresh = null) {
  const r = { ...runtime(), currentAsk: fresh, continuityId: '', turnCount: 0 };
  const order = [];
  const dependencies = {
    ...binding,
    getAttachmentRuntime: () => r,
    normalizeProjectRoot: value => String(value || '').trim(),
    isProjectRootAuthoritySafe: root => root === r.sessionCwd,
    setActiveWorkpointPacket: packet => { r.activeWorkpointPacket = packet; },
    setActiveWorkpointSummary: summary => { r.activeWorkpointSummary = summary; },
  };
  const stamp = loadSourceSection('state.ts', 'export function stampWorkpointPacketForCurrentPiSession(',
    'export function isWorkpointPacketScopedToCurrentSession(', dependencies, 'stampWorkpointPacketForCurrentPiSession');
  const adopt = loadSourceSection('state.ts', 'export function adoptPersistedContinuityForSession(',
    '// ── Build compact instructions', { ...dependencies, stampWorkpointPacketForCurrentPiSession: stamp },
    'adoptPersistedContinuityForSession');
  const seed = loadSourceSection('session.ts', 'function seedCurrentAskFromPersistedState(',
    'async function ensureActiveFrame(', {
      getAttachmentRuntime: () => r, stripQuotedFocusaContext: value => String(value),
      adoptPiProjectRoot: value => value, getSessionCwd: () => r.sessionCwd,
      getContinuityId: () => r.continuityId, classifyCurrentAsk: () => 'instruction',
      isNonTaskStatusLikeText: () => false,
    }, 'seedCurrentAskFromPersistedState');
  const setters = Object.fromEntries(['setLastProjectVerify', 'setLastProjectIdentity',
    'setLastProjectRootResolution', 'setLastTrajectoryClarity', 'setLatestReportSummary']
    .map(name => [name, () => order.push(name)]));
  loadSourceSection('session.ts', '        // §33.5 + §33.7: restore resumable session metadata',
    '        // Explicitly clear stale pollution', {
      ...setters, getAttachmentRuntime: () => r, d: data, ctx: { cwd: r.sessionCwd },
      eventSessionId: r.sessionFrameKey, selectedBindingRoot: r.sessionCwd, localBinding: {},
      setTurnCount: count => { r.turnCount = count; }, setTotalCompactions() {},
      seedCurrentAskFromPersistedState: (ctx, d) => { order.push('seed'); seed(ctx, d); },
      adoptPersistedContinuityForSession: (...args) => { order.push('adopt'); adopt(...args); },
    }, 'undefined');
  return { r, order };
}

function startupSavedState() {
  const r = runtime();
  return { sessionId: r.sessionFrameKey, continuityId: r.continuityId,
    currentAsk: exactAsk('Continue the source-only task; production remains unchanged.'),
    lastProjectIdentity: { project_root: r.sessionCwd },
    activeWorkpointPacket: { project_root: r.sessionCwd, continuity_id: r.continuityId,
      canonical: true, mission: 'specific saved task',
      action_authority_for_current_ask: true, matches_current_ask_scope: true,
      current_ask_scope: { action_authority_for_current_ask: true, matches_current_ask_scope: true } } };
}

test('startup executes existing identity and continuity recovery before complete request restoration', () => {
  const saved = startupSavedState(), { r, order } = startupRecoveryFixture(saved);
  assert.equal(r.currentAsk?.text, saved.currentAsk.text);
  assert.ok(order.indexOf('setLastProjectIdentity') < order.indexOf('seed'));
  assert.ok(order.indexOf('adopt') < order.indexOf('seed'));
});

test('startup does not turn saved action permission into a fresh grant', () => {
  const saved = startupSavedState();
  const { r } = startupRecoveryFixture(saved);
  assert.equal(r.activeWorkpointPacket.action_authority_for_current_ask, false);
  assert.equal(r.activeWorkpointPacket.matches_current_ask_scope, false);
  assert.equal(r.activeWorkpointPacket.current_ask_scope.action_authority_for_current_ask, false);
  assert.equal(saved.activeWorkpointPacket.action_authority_for_current_ask, true);
});

test('startup keeps fresh steering while recovering saved continuity', () => {
  const fresh = exactAsk('New instruction: inspect only.');
  assert.equal(startupRecoveryFixture(startupSavedState(), fresh).r.currentAsk, fresh);
});

test('a foreign saved session cannot seed a request or permission during startup recovery', () => {
  const saved = { ...startupSavedState(), sessionId: 'foreign-session' };
  const { r } = startupRecoveryFixture(saved);
  assert.equal(r.currentAsk, null);
  assert.equal(r.activeWorkpointPacket, null);
});

test('shared request revocation preserves context and never mutates the saved packet', () => {
  const original = startupSavedState().activeWorkpointPacket;
  const revoked = binding.revokeWorkpointRequestAuthority(original, 'fixture-revalidation-required');
  assert.equal(revoked.mission, original.mission);
  assert.equal(revoked.continuity_id, original.continuity_id);
  assert.equal(revoked.current_ask_scope.matches_current_ask_scope, false);
  assert.equal(revoked.current_ask_scope.scope_conflict_reason, 'fixture-revalidation-required');
  assert.equal(original.current_ask_scope.matches_current_ask_scope, true);
  assert.equal(binding.revokeWorkpointRequestAuthority(null, 'fixture'), null);
});

for (const newTaskText of ['Inspect only; never deploy.', '']) {
  test(`actual input owner revokes old request permission for ${newTaskText ? 'changed' : 'empty meta'} steering`, () => {
    const original = { ...startupSavedState().activeWorkpointPacket, current_ask_binding: 'old request' };
    let current = original;
    const writes = [], runtime = {};
    loadSourceSection('turns.ts', '    const activeWorkpoint = getActiveWorkpointPacket();',
      '    const projectRoot = adoptPiProjectRoot', {
        ...binding, newTaskText, getActiveWorkpointPacket: () => current,
        setActiveWorkpointPacket: packet => { current = packet; }, persistState: options => writes.push(options),
        _ctx: {}, sourceTurnId: 'changed-input', askKind: 'instruction', text: newTaskText,
        updateNorthStarCard() {}, observeProjectThreadHintsFromText() {},
        deriveQueryScope: () => ({ kind: 'fixture' }), isOperatorSteeringInput: () => true,
        getAttachmentRuntime: () => runtime,
      }, 'undefined');
    assert.equal(current.action_authority_for_current_ask, false);
    assert.equal(current.current_ask_scope.matches_current_ask_scope, false);
    assert.equal(current.current_ask_scope.scope_conflict_reason, 'operator_ask_changed_since_workpoint_binding');
    assert.equal(original.action_authority_for_current_ask, true);
    assert.deepEqual(writes, [{ immediate: true }]);
  });
}

test('input keeps complete multiline instructions while bounding only the task label', () => {
  const text = fullInstruction('EXCLUSION: do not deploy or change permissions.');
  assert.ok(text.length > 500);
  const r = captureInput(text);
  assert.equal(r.currentAsk.text, text);
  assert.equal(r.currentTaskLabel, text.slice(0, 500));
  assert.equal(r.currentTaskInputTokenEstimate, Math.ceil(text.length / 4));
});

test('changes beyond the old preview limit change request binding and persistence identity', () => {
  const before = fullInstruction('EXCLUSION: no deployment.');
  const after = fullInstruction('EXCLUSION: no credential changes.');
  const r = captureInput(before), captured = binding.captureResumeRequest(r, r.currentAsk.text);
  r.currentAsk.text = after;
  assert.equal(binding.evaluateResumeRequest(reply(), captured, r).accepted, false);
  const a = persistenceFixture(exactAsk(before)).buildPersistedRecoveryState();
  const b = persistenceFixture(exactAsk(after)).buildPersistedRecoveryState();
  assert.equal(a.currentAsk.text, before);
  assert.equal(a.currentAsk.textIsPreview, false);
  assert.notEqual(persistenceExports.semanticPersistenceDigest(a), persistenceExports.semanticPersistenceDigest(b));
});

test('actual persistence writes full request to the sidecar and preview-only native anchor', () => {
  const text = fullInstruction('EXCLUSION: retain the complete final restriction.');
  const f = persistenceFixture(exactAsk(text));
  f.persistState();
  assert.equal(f.writes.length, 1);
  assert.equal(f.writes[0].state.currentAsk.text, text);
  const anchor = f.anchors.find(value => value.type === 'focusa-state').data;
  assert.ok(anchor.currentAsk.text.length <= 320);
  assert.equal(anchor.currentAsk.textIsPreview, true);
  assert.ok(Buffer.byteLength(JSON.stringify(anchor)) <= persistenceExports.NATIVE_ANCHOR_MAX_BYTES);
  assert.equal(restoreFixture(f.writes[0].state).currentAsk.text, text);
  assert.equal(restoreFixture(anchor).currentAsk, null);
});

test('missing exact text does not reconstruct request authority from a frame goal', () => {
  assert.equal(restoreFixture({ frameGoal: 'old accepted purpose', frameTitle: 'Pi Task: old' }).currentAsk, null);
  const legacy = exactAsk('legacy bounded text');
  delete legacy.textIsPreview;
  assert.equal(restoreFixture({ currentAsk: legacy, frameGoal: 'old purpose', frameTitle: 'Pi Task: old' }).currentAsk, null);
});

// These fixtures live inside the admitted invocation-owned snapshot, which is
// retired recoverably by the owning runner rather than recursively deleted.
function withRealSidecar(callback) {
  const previous = process.env.FOCUSA_DATA_DIR;
  process.env.FOCUSA_DATA_DIR = mkdtempSync(join(tmpdir(), 'focusa-sidecar-regression-'));
  try { callback(process.env.FOCUSA_DATA_DIR); }
  finally {
    if (previous === undefined) delete process.env.FOCUSA_DATA_DIR;
    else process.env.FOCUSA_DATA_DIR = previous;
  }
}

function writeExactSidecar(text, revision) {
  const state = persistenceFixture(exactAsk(text)).buildPersistedRecoveryState();
  const digest = persistenceExports.semanticPersistenceDigest(state);
  const sidecar = persistenceExports.writeRecoverySidecar(state, digest, revision);
  return { state, anchor: { schema: persistenceExports.COMPACTION_PERSISTENCE_ANCHOR_SCHEMA,
    projectRoot: state.projectRoot, sessionId: state.sessionId,
    anchorRevision: revision, semanticDigest: digest, sidecarKey: sidecar.key } };
}

test('real atomic sidecar round-trip restores the complete late exclusion', () => {
  withRealSidecar(() => {
    const text = fullInstruction('EXCLUSION: no deployment, source grant only.');
    const { state, anchor } = writeExactSidecar(text, 1);
    const loaded = persistenceExports.loadPersistedRecoveryState(anchor);
    assert.deepEqual(loaded, JSON.parse(JSON.stringify(state)));
    assert.equal(restoreFixture(loaded).currentAsk.text, text);
    const { anchorRevision, ...unidentified } = anchor;
    assert.equal(persistenceExports.loadPersistedRecoveryState({ ...unidentified, revision: anchorRevision }), null);
  });
});

test('corrupt current sidecar cannot silently substitute an older accepted request', () => {
  withRealSidecar(root => {
    writeExactSidecar(fullInstruction('OLD permission: continue.'), 1);
    const { anchor } = writeExactSidecar(fullInstruction('CURRENT restriction: stop.'), 2);
    const directory = join(root, 'pi-session-state');
    const current = readdirSync(directory).find(name => name.includes('.r2.'));
    assert.ok(current, 'current generation exists');
    writeFileSync(join(directory, current), '{corrupt');
    assert.equal(persistenceExports.loadPersistedRecoveryState(anchor), null);
  });
});

for (const [field, value] of [
  ['sessionId', 'other-session'], ['projectRoot', '/tmp/other-project'], ['continuityId', 'other-continuity'],
]) {
  test(`complete recovery text is not restored into a different ${field}`, () => {
    const ask = { ...exactAsk(fullInstruction('restriction')), [field]: value };
    assert.equal(restoreFixture({ currentAsk: ask }).currentAsk, null);
  });
}
