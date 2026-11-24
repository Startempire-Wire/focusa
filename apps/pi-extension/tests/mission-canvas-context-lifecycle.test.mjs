import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { eraseTypeScriptSection } from './helpers/source-section-harness.mjs';

const source = readFileSync(new URL('../src/mission-canvas-widget.ts', import.meta.url), 'utf8');
const lifecycle = readFileSync(new URL('../src/lifecycle-guard.ts', import.meta.url), 'utf8');
const LifecycleGenerationGuard = new Function(`${stripTypeScriptTypes(lifecycle.replace(/^export /gm, ''))}\nreturn LifecycleGenerationGuard;`)();
const handlers = {};
const listeners = new Set();
const paints = [];
const deps = {
  exports: {}, LifecycleGenerationGuard,
  currentAttachmentKey: () => undefined,
  getAttachmentRuntime: () => ({ focusaAvailable: true }),
  currentProjectBindingDecision: () => ({ state: 'BOUND' }),
  getSessionCwd: () => '/fixture',
  resolveInteractionMode: () => ({ mode: 'canvas-guided' }),
  scopedReceiptMatchesCurrentScope: receipt => receipt.matches,
  subscribeScopedStateChanges: listener => { listeners.add(listener); return () => listeners.delete(listener); },
};
const register = new Function(...Object.keys(deps), `${eraseTypeScriptSection(source.slice(source.indexOf('let refreshUnsubscribe:')))}\nreturn registerMissionCanvasWidget;`)(...Object.values(deps));
register({ on: (event, callback) => { handlers[event] = callback; } });
const ctx = name => ({ mode: 'tui', ui: { setWidget: () => paints.push(name) } });
handlers.session_start({}, { mode: 'rpc', hasUI: true });
assert.equal(listeners.size, 0);
for (const reason of ['reload', 'resume', 'new', 'fork']) {
  handlers.session_start({}, ctx('original'));
  const stale = [...listeners][0];
  handlers.session_shutdown({ reason });
  handlers.session_start({}, ctx('replacement'));
  assert.equal(listeners.size, 1);
  const count = paints.length;
  stale({ matches: true });
  const active = [...listeners][0];
  active({ matches: false });
  assert.equal(paints.length, count);
  active({ matches: true });
  assert.equal(paints.length, count + 1);
  handlers.session_shutdown({ reason: 'quit' });
  active({ matches: true });
  assert.equal(paints.length, count + 1);
  assert.equal(listeners.size, 0);
}
assert.doesNotMatch(source, /\b(?:setTimeout|setInterval|focusaFetch|setActiveWorkpointPacket)\(/);
console.log('Mission Canvas widget: TUI-only subscriptions, scope and lifecycle fences passed');
