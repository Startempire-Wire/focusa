import assert from 'node:assert/strict';
import { test } from 'node:test';
import { captureActiveTab, createOrientationPacket, renderOrientationMission, sanitizeBrowserObservation } from '../src/lib/orientation.mjs';

const now = () => new Date('2026-08-24T12:00:00Z');

test('active tab capture is explicit bounded metadata only', async () => {
  const calls = [];
  const observation = await captureActiveTab({ tabs: { query: async (query) => {
    calls.push(query); return [{ title: 'Focusa', url: 'https://user:pass@example.com/path?q=1#secret' }];
  } } }, now);
  assert.deepEqual(calls, [{ active: true, currentWindow: true }]);
  assert.equal(observation.url, 'https://example.com/path?q=1');
  assert.equal(observation.origin, 'https://example.com');
  assert.equal(observation.title, 'Focusa');
  assert.equal(Object.isFrozen(observation), true);
  for (const forbidden of ['body', 'cookies', 'forms', 'history', 'screenshot', 'selection']) assert.equal(forbidden in observation, false);
});

test('forbidden schemes and oversize metadata fail instead of truncating', () => {
  for (const url of ['chrome://settings', 'file:///tmp/x', 'data:text/plain,x', 'javascript:alert(1)']) {
    assert.throws(() => sanitizeBrowserObservation({ title: 'x', url, captured_at: now().toISOString() }));
  }
  assert.throws(() => sanitizeBrowserObservation({ title: 'x'.repeat(301), url: 'https://example.com', captured_at: now().toISOString() }));
});

test('orientation packet is immutable and mission projection exact', () => {
  const observation = sanitizeBrowserObservation({ title: 'Example', url: 'https://example.com/task#fragment', captured_at: now().toISOString() });
  const packet = createOrientationPacket({
    objective: 'Research the documented API.', exclusions: ['Do not publish', 'Do not purchase'], observation,
    project_root: '/work/focusa', continuity_id: 'focusa-main', work_item_ref: 'focusa-1', role_profile_ref: 'role:researcher',
  }, now);
  assert.equal(Object.isFrozen(packet), true);
  assert.equal(Object.isFrozen(packet.observation), true);
  assert.match(renderOrientationMission(packet), /^OBJECTIVE\nResearch the documented API\./);
  assert.match(renderOrientationMission(packet), /URL: https:\/\/example\.com\/task/);
  assert.match(renderOrientationMission(packet), /- Do not purchase$/);
  assert.throws(() => { packet.objective = 'changed'; }, TypeError);
});

test('all orientation bounds reject rather than silently clipping', () => {
  const observation = sanitizeBrowserObservation({ title: 'Example', url: 'https://example.com', captured_at: now().toISOString() });
  const base = { objective: 'Do work', exclusions: [], observation, project_root: '/p', continuity_id: 'c' };
  assert.throws(() => createOrientationPacket({ ...base, objective: 'x'.repeat(4001) }, now));
  assert.throws(() => createOrientationPacket({ ...base, exclusions: Array(11).fill('x') }, now));
  const maximumObservation = sanitizeBrowserObservation({ title: 't'.repeat(300), url: `https://example.com/${'p'.repeat(1900)}`, captured_at: now().toISOString() });
  assert.throws(() => createOrientationPacket({ ...base, observation: maximumObservation, objective: 'x'.repeat(4000), exclusions: Array(10).fill('y'.repeat(300)) }, now), /8192/);
});
