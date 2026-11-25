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
let packet = null;
let rendered;
const deps = {
  exports: {}, LifecycleGenerationGuard,
  currentAttachmentKey: () => undefined,
  getAttachmentRuntime: () => ({ focusaAvailable: true }),
  currentProjectBindingDecision: () => ({ state: 'BOUND' }),
  getScopedWorkpointPacket: () => packet,
  truncateToWidth: (text, width) => text.slice(0, width),
  workRailSnapshotFromPacket: value => value,
  renderWorkRailWidget: (rail, width) => [`${rail.work_item_id} ${rail.next_slice}`.slice(0, width)],
  getSessionCwd: () => '/fixture',
  resolveInteractionMode: () => ({ mode: 'canvas-guided' }),
  scopedReceiptMatchesCurrentScope: receipt => receipt.matches,
  subscribeScopedStateChanges: listener => { listeners.add(listener); return () => listeners.delete(listener); },
};
const register = new Function(...Object.keys(deps), `${eraseTypeScriptSection(source.slice(source.indexOf('let refreshUnsubscribe:')))}\nreturn registerMissionCanvasWidget;`)(...Object.values(deps));
register({ on: (event, callback) => { handlers[event] = callback; } });
const ctx = name => ({ mode: 'tui', ui: { setWidget: (_id, factory) => {
  paints.push(name);
  rendered = factory?.(null, { fg: (_token, text) => text });
} } });
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
for (const [value, expected] of [
  [null, /Workstream not verified/],
  [{ canonical: false, work_item_id: 'foreign' }, /Workstream not verified/],
  [{ canonical: true, work_item_id: 'current-item', next_slice: 'review' }, /current-item review/],
]) {
  packet = value;
  handlers.session_start({}, ctx('render'));
  assert.match(rendered.render(80).join(' '), expected);
  assert.doesNotMatch(rendered.render(80).join(' '), /foreign/);
  handlers.session_shutdown({ reason: 'quit' });
}
assert.doesNotMatch(source, /\b(?:setTimeout|setInterval|focusaFetch|setActiveWorkpointPacket)\(/);
console.log('Mission Canvas widget: TUI-only subscriptions, scope and lifecycle fences passed');
