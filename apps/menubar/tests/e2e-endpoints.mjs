#!/usr/bin/env node
// E2E endpoint test for the menubar app against the live Focusa daemon.
//
// Uses only Node 20+ built-ins (node:test, node:assert, node:fetch).
// No new dependencies.
//
// Run: node tests/e2e-endpoints.mjs
//      or: npm test
//
// Verifies that the 33 endpoints the menubar polls (apps/menubar/src/)
// still resolve against the live daemon. Catches:
//   - the /v1/lineage/head → /v1/clt/nodes fix
//   - silent 5xx from dropped routes
//   - regressions after backend refactors
//
// Output: a one-line PASS/FAIL summary plus per-endpoint latency.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { predictionScopedPath, projectScopedPath } from '../src/lib/workLoopScope.js';

const __dirname = dirname(fileURLToPath(import.meta.url));
const DAEMON = process.env.FOCUSA_DAEMON_URL || 'http://127.0.0.1:8787';

// Extract the 33 endpoints the menubar actually polls.
// We read the source rather than hardcode so a future menubar edit
// doesn't silently drift from this test.
async function extractEndpoints() {
  const srcDir = join(__dirname, '..', 'src');
  // Match /v1/... in quoted or template strings, AND paths that
  // appear before a ${...} substitution in a template literal
  // (e.g. `/v1/sync/status/${peerId}`).
  const pattern = /['"`](\/v1\/[a-zA-Z0-9/_-]+)(?=\$\{|['"`]|\b)/g;
  const found = new Set();
  const { readdirSync, readFileSync: read } = await import('node:fs');
  for (const entry of readdirSync(srcDir, { recursive: true, withFileTypes: true })) {
    if (!entry.isFile()) continue;
    if (!/\.(ts|svelte|js)$/.test(entry.name)) continue;
    if (entry.name.endsWith('.d.ts')) continue;
    const path = join(entry.path || entry.parentPath, entry.name);
    let text;
    try { text = read(path, 'utf8'); } catch { continue; }
    let m;
    while ((m = pattern.exec(text)) !== null) found.add(m[1]);
  }
  return [...found].sort();
}

const ENDPOINTS = await extractEndpoints();
const SCOPED_ENDPOINTS = new Set([
  '/v1/focus/snapshots/recent',
  '/v1/metacognition/evaluations/recent',
  '/v1/metacognition/status',
  '/v1/predictions/recent',
  '/v1/predictions/stats',
  '/v1/work-loop/status',
  '/v1/work-loop/health',
  '/v1/work-loop/checkpoints',
]);
const REQUEST_SCOPE = await resolveRequestScope();

async function resolveRequestScope() {
  const explicitProjectRoot = process.env.FOCUSA_PROJECT_ROOT?.trim();
  const explicitContinuityId = process.env.FOCUSA_CONTINUITY_ID?.trim();

  try {
    const repositoryRoot = join(__dirname, '..', '..', '..');
    const identityQuery = `?${new URLSearchParams({
      cwd: repositoryRoot,
      ...(explicitProjectRoot ? { project_root: explicitProjectRoot } : {}),
    })}`;
    const [identityResponse, workpointResponse] = await Promise.all([
      fetch(`${DAEMON}/v1/project/identity${identityQuery}`),
      fetch(`${DAEMON}/v1/workpoint/current`),
    ]);
    if (!identityResponse.ok) return null;

    const identityBody = await identityResponse.json();
    const workpointBody = workpointResponse.ok ? await workpointResponse.json() : {};
    const identity = identityBody.project_identity ?? identityBody;
    const workpoint = workpointBody.workpoint ?? workpointBody.packet ?? workpointBody;
    const projectRoot =
      explicitProjectRoot ??
      identityBody.binding_decision?.selected_project_root ??
      identity.project_root ??
      workpointBody.project_root ??
      workpointBody.scope?.project_root ??
      workpoint.project_root ??
      workpoint.scope?.project_root;
    const continuityId =
      explicitContinuityId ??
      workpointBody.continuity_id ??
      workpointBody.scope?.continuity_id ??
      workpoint.continuity_id ??
      workpoint.scope?.continuity_id ??
      identity.continuity_id ??
      'menubar-e2e-route-probe';
    if (!projectRoot || !continuityId) return null;
    return {
      projectRoot: String(projectRoot),
      continuityId: String(continuityId),
      projectIdentity: { ...identity, project_root: String(projectRoot) },
    };
  } catch {
    return null;
  }
}

// Per-endpoint expected behavior. Anything not listed here defaults to
// "200 for GET, 405 for POST". 4xx is treated as success when the menubar
// intentionally calls an endpoint with missing/required params
// (e.g. /v1/sync/pull/{peer_id} called without peer_id).
const EXPECTED = {
  // 24 endpoints that should return 200 when called as the menubar calls them
  '/v1/bloatgaurd/domain/':      { expect: 404, note: 'needs {name} param' },
  '/v1/connect/room/':           { expect: 404, note: 'needs {room_id}/status path' },
  '/v1/connect/rooms':           { expect: 200 },
  '/v1/context-cognition':       { expect: 422, note: 'needs agent/cwd params' },
  '/v1/device/pair/start':       { expect: 405, note: 'POST endpoint, GET returns method-not-allowed' },
  '/v1/doctor':                  { expect: 200 },
  '/v1/events/recent':            { expect: 200 },
  '/v1/focus/frame/current':      { expect: 200 },
  '/v1/focus/snapshots/recent':   { expect: 200 },
  '/v1/health':                   { expect: 200 },
  '/v1/lineage/head':             { expect: 200, critical: true, note: 'was broken, fixed 2026-07-07' },
  '/v1/metacognition/evaluations/recent': { expect: 200, note: 'rehydrate hint — endpoint may 200 with rehydrate ref' },
  '/v1/metacognition/status':     { expect: 200 },
  '/v1/ontology/tool-contracts':  { expect: 200 },
  '/v1/predictions/recent':       { expect: 200 },
  '/v1/predictions/stats':        { expect: 200 },
  '/v1/project/identity':         { expect: 200 },
  '/v1/release/proof/status':      { expect: 200 },
  '/v1/silent-sessions':           { expect: [200, 404], note: 'current source route; 404 accepted only from pre-release live daemon' },
  '/v1/state/dump':               { expect: 200 },
  '/v1/sync/peers':               { expect: 200 },
  '/v1/sync/pull/':               { expect: 404, note: 'needs {peer_id} param' },
  '/v1/sync/status/':              { expect: 404, note: 'needs {peer_id} param' },
  '/v1/telemetry/cache-metadata/status': { expect: 200 },
  '/v1/telemetry/memory':          { expect: 200 },
  '/v1/telemetry/token-budget/status': { expect: 200 },
  '/v1/trajectory/view':          { expect: 200 },
  '/v1/work-loop/checkpoints':    { expect: [200, 409], note: '409 is typed no-canonical-work-item posture' },
  '/v1/work-loop/health':         { expect: [200, 409], note: '409 is typed no-canonical-work-item posture' },
  '/v1/work-loop/status':         { expect: [200, 409], note: '409 is typed no-canonical-work-item posture' },
  '/v1/workpoint/checkpoint':     { expect: 405, note: 'POST endpoint' },
  '/v1/workpoint/current':        { expect: 200 },
  '/v1/workpoint/evidence/link':  { expect: 405, note: 'POST endpoint' },
  '/v1/workpoint/resume':         { expect: 405, note: 'POST endpoint' },
};

let pass = 0, fail = 0, criticalFail = 0;
const results = [];

async function check(ep) {
  const scopeRequired = SCOPED_ENDPOINTS.has(ep);
  const requestPath = scopeRequired && REQUEST_SCOPE
    ? ep.startsWith('/v1/predictions/')
      ? predictionScopedPath(ep, REQUEST_SCOPE.projectIdentity, REQUEST_SCOPE.continuityId)
      : projectScopedPath(ep, REQUEST_SCOPE.projectRoot, REQUEST_SCOPE.continuityId)
    : ep;
  const url = `${DAEMON}${requestPath}`;
  const expected = EXPECTED[ep]?.expect ?? 200;
  const acceptable = Array.isArray(expected) ? expected : [expected];
  const isCritical = EXPECTED[ep]?.critical === true;
  const note = EXPECTED[ep]?.note ?? '';
  const t0 = performance.now();
  let actual, ok = false;
  try {
    if (scopeRequired && !REQUEST_SCOPE) {
      throw new Error('canonical project_root + continuity_id unavailable');
    }
    const r = await fetch(url, { method: 'GET' });
    actual = r.status;
    ok = acceptable.includes(actual);
  } catch (e) {
    actual = `ERR: ${e.message?.split('\n')[0] ?? e}`;
  }
  const dt = Math.round(performance.now() - t0);
  results.push({
    ep,
    expected: acceptable.join('/'),
    actual,
    ok,
    dt,
    note,
    critical: isCritical,
  });
  if (ok) pass++; else { fail++; if (isCritical) criticalFail++; }
}

await Promise.all(ENDPOINTS.map(check));

// Sort by status (fail first) then path
results.sort((a, b) => (a.ok === b.ok ? a.ep.localeCompare(b.ep) : a.ok ? 1 : -1));

const W = (s, n) => String(s).padEnd(n);
console.log('');
for (const r of results) {
  const tag = r.ok ? 'OK  ' : 'FAIL';
  const crit = r.critical ? ' [CRITICAL]' : '';
  const n = r.note ? ` — ${r.note}` : '';
  console.log(
    `  ${tag}  ${W(r.actual, 3)}  (${W(r.expected + ' expect', 8)})  ${W(r.dt + 'ms', 7)}  ${r.ep}${crit}${n}`,
  );
}

console.log('');
console.log(`  ${pass}/${ENDPOINTS.length} endpoints OK`);
if (fail) {
  console.log(`  ${fail} failed (${criticalFail} critical)`);
}

if (criticalFail > 0) {
  console.error('\n  CRITICAL: a fix-verified endpoint regressed. Bail.');
  process.exit(2);
}
if (fail > 0) {
  console.error('\n  Some endpoints failed expected status. Investigate.');
  process.exit(1);
}
console.log('\n  All endpoints pass. ✓\n');
