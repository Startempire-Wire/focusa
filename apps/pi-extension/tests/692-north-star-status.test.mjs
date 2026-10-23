import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { loadRegisteredTool } from './helpers/registered-tool-harness.mjs';
import { loadSourceSection } from './helpers/source-section-harness.mjs';

const root = '/tmp/synthetic-north-star-scope';
const continuity = 'north-star-a';
const normalizeProjectRoot = value => typeof value === 'string' ? value.trim().replace(/\/+$/, '') : '';
const owner = readFileSync(new URL('../src/north-star.ts', import.meta.url), 'utf8');
const canonicalDaemonNorthStarReadiness = owner.includes('export function canonicalDaemonNorthStarReadiness(')
  ? loadSourceSection('north-star.ts', 'export function canonicalDaemonNorthStarReadiness(', 'export type NorthStarSurfaceState',
      { normalizeProjectRoot }, 'canonicalDaemonNorthStarReadiness')
  : undefined;

async function run(body, ok = true, suppliedContinuity = continuity) {
  const calls = [];
  const state = { initialized: 0 };
  const tool = loadRegisteredTool('focusa_north_star_gate', {
    resolveFocusaToolProjectRoot: async () => root,
    getContinuityId: () => suppliedContinuity,
    ensureContinuityId: () => { state.initialized++; return 'unwanted-continuity'; },
    isProjectRootAuthoritySafe: value => value === root,
    canonicalDaemonNorthStarReadiness,
    focusaFetchDetailed: async path => { calls.push(path); return { ok, status: ok ? 200 : 503, body }; },
  });
  const result = await tool.execute({ trigger: 'isolated-source-fixture', projection: 'short' });
  return { result, calls, state };
}

function ready(status) {
  return {
    status: 'completed', canonical: true, authority: 'daemon_owned', advisory: false,
    project_root: root,
    projection: { schema: 'focusa.north_star_projection.v1', status: 'complete', project_root: root, continuity_id: continuity },
    workpoint_linkage: { status: 'ready', project_root: root, continuity_id: continuity, frontier_status: 'ready' },
    trajectory_integrity_guard: { status },
  };
}
for (const status of ['READY', 'ready']) {
  const { result, calls } = await run(ready(status));
  assert.equal(result.details.ok, true, `documented ${status} was falsely blocked`);
  assert.match(result.content[0].text, /NORTH STAR READY/);
  assert.equal(new URL(calls[0], 'http://fixture').searchParams.get('continuity_id'), continuity);
}
for (const body of [
  ready(undefined), ready('unknown'), ready('Ready'), ready('STALE'), ready('BLOCKED'),
  { ...ready('READY'), canonical: false },
  { ...ready('READY'), canonical: undefined },
  { ...ready('READY'), authority: 'advisory_external' },
  { ...ready('READY'), advisory: true },
  { ...ready('READY'), project_root: '/foreign' },
  { ...ready('READY'), projection: { ...ready('READY').projection, continuity_id: 'north-star-b' } },
  { ...ready('READY'), workpoint_linkage: { ...ready('READY').workpoint_linkage, project_root: '/foreign' } },
  { ...ready('READY'), projection: { ...ready('READY').projection, status: 'stale' } },
  { ...ready('READY'), projection: undefined, workpoint_linkage: undefined },
  { ...ready('READY'), status: 'blocked', code: 'NORTH_STAR_ADMISSION_BLOCKED' },
]) {
  const { result } = await run(body);
  assert.equal(result.details.ok, false, `unsupported or foreign projection promoted: ${JSON.stringify(body)}`);
  assert.doesNotMatch(result.content[0].text, /NORTH STAR READY/);
}
assert.equal((await run(ready('READY'), false)).result.details.ok, false);
const reconcile = await run({ ...ready('READY'), status: 'reconcile', code: 'WORKPOINT_FRONTIER_MISSING' });
assert.equal(reconcile.result.details.status, 'reconcile');
assert.match(reconcile.result.content[0].text, /before consequential mutation/);
const unscoped = await run(ready('READY'), true, '');
assert.equal(unscoped.result.details.ok, false);
assert.equal(unscoped.state.initialized, 0, 'read-only gate invented a continuity');
assert.equal(unscoped.calls.length, 0);
console.log('actual registered North Star projection: supported status versions, canonical provenance, exact scope, stale/unknown refusal and reconcile boundary passed');
