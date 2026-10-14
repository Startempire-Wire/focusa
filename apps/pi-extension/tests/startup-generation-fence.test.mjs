import assert from 'node:assert/strict';
import test from 'node:test';
import { LifecycleGenerationGuard } from '../src/lifecycle-guard.ts';
import { loadSourceSection } from './helpers/source-section-harness.mjs';

function deferred() {
  let resolve;
  const promise = new Promise(r => { resolve = r; });
  return { promise, resolve };
}

function healthFixture() {
  const guard = new LifecycleGenerationGuard();
  const entered = deferred(), reply = deferred();
  const runtime = {};
  const effects = { entryReads: 0, pressureRefreshes: 0 };
  const ctx = { cwd: '/tmp/startup-generation-fixture', ui: {}, sessionManager: {
    getSessionId: () => 'fixture-native-session', getBranch: () => [],
    getEntries: () => { effects.entryReads++; return []; },
  } };
  const run = loadSourceSection('session.ts',
    '    const lifecycleGeneration = healthLifecycle.begin();',
    '    let persistedBindingRoot = "";', {
      healthLifecycle: guard, getAttachmentRuntime: () => runtime,
      pi: { getFlag: () => false }, ctx, event: { reason: 'resume' },
      clearPublishedAttachmentEnvironment() {}, adoptPiProjectRoot: root => root,
      resetPiSessionScopedState() {}, rehydrateScopedStateChanges() {}, syncRuntimeFieldsToScopeStore() {},
      checkFocusa: async () => { entered.resolve(); await reply.promise; },
      refreshNativeSessionPressure: () => { effects.pressureRefreshes++; },
    }, 'undefined', true);
  return { guard, entered, reply, effects, run };
}

function bindingFixture() {
  const guard = new LifecycleGenerationGuard(), generation = guard.begin();
  const entered = deferred(), reply = deferred();
  const effects = { rootResolutions: 0 };
  const root = '/tmp/startup-generation-fixture';
  const run = loadSourceSection('session.ts',
    '    const bindingResolutionStartedAt = Date.now();',
    '    const previousBindingDecision = currentProjectBindingDecision(eventSessionId);', {
      healthLifecycleIsCurrent: () => guard.isCurrent(generation),
      getAttachmentRuntime: () => ({ projectBindingTelemetry: { startup_count: 0 } }),
      ctx: { cwd: root }, persistedBindingRoot: '',
      resolvePiProjectRootCandidate: () => ({ projectRoot: root, candidates: [], confidenceScore: 1, markers: [] }),
      focusaFetch: async () => { entered.resolve(); await reply.promise; return { binding_decision: { selected_project_root: root } }; },
      normalizeProjectRoot: root => String(root || ''),
      isProjectRootAuthoritySafe: candidate => candidate === root,
      setLastProjectRootResolution: () => { effects.rootResolutions++; },
    }, 'undefined', true);
  return { guard, entered, reply, effects, run };
}

for (const boundary of ['shutdown', 'replacement', 'current']) {
  for (const [name, fixture] of [['health', healthFixture], ['project binding', bindingFixture]]) {
    test(`actual awaited startup ${name} section fences ${boundary} before context effects`, async () => {
      const f = fixture();
      await f.entered.promise;
      if (boundary === 'shutdown') f.guard.end();
      if (boundary === 'replacement') f.guard.begin();
      f.reply.resolve();
      await f.run;
      const expected = boundary === 'current' ? 1 : 0;
      for (const [effect, count] of Object.entries(f.effects)) assert.equal(count, expected, effect);
    });
  }
}
