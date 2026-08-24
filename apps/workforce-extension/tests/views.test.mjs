import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { auditView, rosterView, statusView } from '../src/lib/views.mjs';

 test('status always carries text and a non-color state', () => {
  assert.deepEqual(statusView('paired'), { state: 'paired', label: 'Connected', tone: 'success' });
  assert.deepEqual(statusView('future'), { state: 'unknown', label: 'Unknown', tone: 'warning' });
  assert.match(statusView('degraded','network').label, /Degraded: network/);
});

test('roster view mirrors daemon rows and enables controls only for exact targets', () => {
  const projection = { rows: [{ id: 's1', display_name: 'One', lifecycle: 'running', health: 'healthy', semantic_activity: 'working' },
    { id: 's2', display_name: 'Two', lifecycle: 'unknown', health: 'degraded', semantic_activity: 'unknown' }] };
  const target = { session_id: 's1', run_id: 'r1', generation: 1 };
  const rows = rosterView(projection, new Map([['s1',target]]));
  assert.equal(rows.length, 2); assert.equal(rows[0].exact_target, target); assert.equal(rows[0].controls_enabled, true); assert.equal(rows[1].controls_enabled, false);
  assert.match(rows[1].status_text, /unknown/);
});

test('audit view excludes raw event payload', () => {
  const rows = auditView([{ event_id: 'e1', event_type: 'changed', timestamp: 'now', cursor: '1', invalidate: ['roster'], payload: { token: 'secret' } }]);
  assert.deepEqual(rows, [{ id: 'e1', primary: 'changed', secondary: 'now · cursor 1', invalidates: 'roster' }]);
  assert.doesNotMatch(JSON.stringify(rows), /secret|token|payload/);
});

test('panel has four labelled regions, keyboard-native controls, and live text status', async () => {
  const html = await readFile(new URL('../src/sidepanel.html', import.meta.url), 'utf8');
  const script = await readFile(new URL('../src/sidepanel.mjs', import.meta.url), 'utf8');
  for (const heading of ['Orientation','Creation','Workforce','Audit']) assert.match(html, new RegExp(`>${heading}<|>\\d\\. ${heading}<`));
  assert.match(html, /aria-live="polite"/); assert.match(html, /<button/g); assert.match(html, /<label for=/g);
  assert.doesNotMatch(html, /onclick=|tabindex="-[0-9]/);
  const pagehide = script.split("window.addEventListener('pagehide'",2)[1];
  assert.match(pagehide, /streamAbort\?\.abort/); assert.doesNotMatch(pagehide, /orchestrateAction|controlSession/);
  assert.doesNotMatch(script, /innerHTML|insertAdjacentHTML/);
});
