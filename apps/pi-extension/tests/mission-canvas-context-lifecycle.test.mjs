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
const display = { name: 'display' }, publisher = { name: 'publisher' };
let owner = display;
let checkout = 'A';
const deps = {
  exports: {}, LifecycleGenerationGuard,
  currentAttachmentKey: () => owner,
  runWithAttachmentRuntime: (key, run) => {
    const previous = owner; owner = key;
    try { return run(); } finally { owner = previous; }
  },
  getAttachmentRuntime: () => ({ focusaAvailable: true }),
  currentProjectBindingDecision: () => ({ state: 'BOUND' }),
  getScopedWorkpointPacket: () => owner === display && (!packet?.checkout || packet.checkout === checkout) ? packet
    : owner !== display ? { canonical: true, work_item_id: 'foreign', next_slice: 'do not show' } : null,
  truncateToWidth: (text, width) => text.slice(0, width),
  workRailSnapshotFromPacket: value => value,
  renderWorkRailWidget: (rail, width) => [`${rail.work_item_id} ${rail.next_slice}`.slice(0, width)],
  getSessionCwd: () => '/fixture',
  resolveInteractionMode: () => ({ mode: 'canvas-guided' }),
  scopedReceiptMatchesCurrentScope: receipt => owner === display && receipt.matches && (!receipt.checkout || receipt.checkout === checkout),
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
packet = { canonical: true, work_item_id: 'task-A', next_slice: 'review A', checkout: 'A' };
handlers.session_start({}, ctx('sibling-switch'));
const started = rendered;
assert.match(started.render(80).join(' '), /task-A/);
checkout = 'B';
assert.doesNotMatch(started.render(48).join(' '), /task-A/);
assert.match(started.render(48).join(' '), /Workstream not verified/);
owner = publisher;
const repaintCount = paints.length;
[...listeners][0]({ matches: true, checkout: 'A' });
assert.equal(paints.length, repaintCount);
packet = { canonical: true, work_item_id: 'task-B', next_slice: 'review B', checkout: 'B' };
[...listeners][0]({ matches: true, checkout: 'B' });
assert.equal(paints.length, repaintCount + 1);
for (const width of [48, 80, 120]) {
  const view = rendered.render(width).join(' ');
  assert.match(view, /task-B/);
  assert.doesNotMatch(view, /task-A|foreign/);
}
owner = display;
handlers.session_shutdown({ reason: 'quit' });
assert.deepEqual(started.render(80), []);
assert.doesNotMatch(source, /\b(?:setTimeout|setInterval|focusaFetch|setActiveWorkpointPacket)\(/);
console.log('Mission Canvas widget: TUI-only subscriptions, scope and lifecycle fences passed');
