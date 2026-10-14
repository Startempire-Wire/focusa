// Actual production Pi prepare/verify functions → real authenticated test HTTP.
// No model, native compaction provider, production daemon or installation is used.
import assert from 'node:assert/strict';
import { loadPureTypeScript } from './helpers/registered-tool-harness.mjs';
import { loadSourceSection } from './helpers/source-section-harness.mjs';

const base = process.argv[2];
assert.ok(base?.startsWith('http://127.0.0.1:'));
const binding = loadPureTypeScript(new URL('../src/workpoint-request-binding.ts', import.meta.url));
const safety = loadPureTypeScript(new URL('../src/compaction-resume-safety.ts', import.meta.url));
const projection = loadPureTypeScript(new URL('../src/compaction-resume-projection.ts', import.meta.url));
const runtime = {
  currentAsk: { text: 'Read-only review; never deploy or change customer data.', sourceTurnId: 'synthetic-turn', updatedAt: 1 },
  sessionFrameKey: 'synthetic-session', sessionCwd: '/tmp/focusa-compaction-http-fixture',
  continuityId: 'synthetic-compaction-continuity', focusaAvailable: true, currentTier: 'normal',
  localDecisions: [], localConstraints: [], localFailures: [],
};
runtime.cfg = { focusaApiBaseUrl: `${base}/v1`, focusaToken: process.env.FOCUSA_AUTH_TOKEN, focusaApiTimeoutMs: 2500 };
const scope = () => ({ root_scope: { root_path: runtime.sessionCwd }, continuity_id: runtime.continuityId });
const transport = loadSourceSection('state.ts', 'export async function focusaFetch(', '// Fire-and-forget variant', {
  getAttachmentRuntime: () => runtime,
  currentAttachmentKey: () => ({ workstream: scope(), session_id: runtime.sessionFrameKey }),
  isProjectRootAuthoritySafe: root => root === runtime.sessionCwd,
}, 'focusaFetch');
const queued = [], calls = [];
const lifecycle = loadSourceSection('compaction.ts', 'type CompactionPrepareResult = {', 'export function registerCompaction', {
  ...binding, ...safety, ...projection, getAttachmentRuntime: () => runtime,
  currentCompactionScope: scope, getSessionFrameKey: () => runtime.sessionFrameKey,
  getTotalCompactions: () => 0, sanitizeFocusFailures: x => x || [], normalizeCompactionArtifacts: x => x || [],
  scheduleCompactionMemoryEvaluation() {}, persistState() {}, updateNorthStarCard() {},
  refreshTrajectoryClarityLifecycle: async () => {},
  queueCompactionResumeContext: (_ctx, text) => queued.push(text),
  renderResumeProjection: p => projection.renderCompactionResumeProjection(p, 'normal'),
  focusaFetch: async (route, options) => {
    calls.push(route);
    return transport(route, options);
  },
}, '{prepareCompactionEpoch, runPostCompactionVerification}');
const prepared = await lifecycle.prepareCompactionEpoch({ compactionEntry: { id: 'actual-node-consumer' } });
assert.ok(prepared, 'actual producer/caller contract must prepare');
assert.equal(prepared.resume_projection.current_ask.text, runtime.currentAsk.text);
await lifecycle.runPostCompactionVerification({ compactionEntry: { id: 'synthetic-native-completion' } }, { hasUI: false });
assert.deepEqual(calls, ['/compaction/prepare', '/compaction/verify']);
assert.equal(queued.length, 1);
assert.ok(queued[0].includes('never deploy or change customer data'));
assert.ok(queued[0].includes('steering only') || queued[0].includes('AUTHORITY: advisory'));
console.log('Actual authenticated API ↔ production Pi prepare/verify caller round-trip passed.');
