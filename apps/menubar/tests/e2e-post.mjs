#!/usr/bin/env node
// E2E POST test for the 4 POST endpoints that the menubar calls.
//
// Verifies that the POST endpoints the menubar uses (workpointActions.svelte.ts)
// accept POST and return a useful response — not 405.
//
// Uses only Node 20+ built-ins. No new deps.
//
// Run: node tests/e2e-post.mjs
//      or: npm run test:post
//
// The 4 POST endpoints:
//   /v1/workpoint/checkpoint      — menubar calls when "checkpoint work" button clicked
//   /v1/workpoint/resume          — menubar calls when "resume workpoint" button clicked
//   /v1/workpoint/evidence/link   — menubar calls when "link evidence" button clicked
//   /v1/device/pair/start         — menubar calls when "start device pairing" clicked
//
// Like the GET test, expected status is data-driven. POSTs to a checkin/out action
// need scope (project_root + continuity_id), so we read those from the daemon's
// /v1/project/identity response and pass them in.
//
// For tests that need a real workpoint, we read it from /v1/workpoint/current.
// If no workpoint exists (cold start), we use a sentinel — the daemon should
// return a structured 4xx, not crash.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const DAEMON = process.env.FOCUSA_DAEMON_URL || 'http://127.0.0.1:8787';

async function getJson(path, opts = {}) {
  const r = await fetch(`${DAEMON}${path}`, { method: 'GET', ...opts });
  if (!r.ok) throw new Error(`${path}: HTTP ${r.status}`);
  return r.json();
}

async function postJson(path, body) {
  return fetch(`${DAEMON}${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
}

async function scopeFromIdentity() {
  const id = await getJson('/v1/project/identity');
  const pi = id.project_identity ?? id;
  return {
    project_root: pi.project_root ?? pi.root ?? null,
    continuity_id: pi.continuity_id ?? null,
    session_id: pi.session_id ?? null,
    work_item_id: pi.work_item_id ?? null,
  };
}

async function currentWorkpointId() {
  const w = await getJson('/v1/workpoint/current');
  return w.workpoint_id ?? w.id ?? null;
}

let pass = 0, fail = 0;
const results = [];

async function check(label, fn, expect) {
  const t0 = performance.now();
  try {
    const actual = await fn();
    const ok = expect.status.includes(actual) ||
      (expect.statusRange && actual >= expect.statusRange[0] && actual <= expect.statusRange[1]);
    results.push({ label, expected: expect.label, actual, ok, dt: Math.round(performance.now() - t0) });
    if (ok) pass++;
    else fail++;
  } catch (e) {
    results.push({ label, expected: expect.label, actual: `ERR: ${e.message?.split('\n')[0] ?? e}`, ok: false, dt: Math.round(performance.now() - t0) });
    fail++;
  }
}

const SCOPE = await scopeFromIdentity();
const WORKPOINT_ID = await currentWorkpointId().catch(() => null);
const HAS_SCOPE = Boolean(SCOPE.project_root && SCOPE.continuity_id);

const PROBE_STATUSES = [200, 400, 404, 422];

await check('POST /v1/workpoint/checkpoint returns a structured response', async () => {
  // Try with full scope first. If continuity_id is missing, fall back to
  // an empty body and accept any structured 4xx as proof-of-life.
  const fullBody = {
    project_root: SCOPE.project_root,
    continuity_id: SCOPE.continuity_id ?? `probe-${Date.now()}`,
    session_id: SCOPE.session_id,
    workpoint_id: WORKPOINT_ID ?? `test-${Date.now()}`,
    mission: 'menubar-e2e-test',
    canonical: true,
    promote: false,
    checkpoint_reason: 'e2e_endpoint_test',
  };
  const r = await postJson('/v1/workpoint/checkpoint', fullBody);
  return r.status;
}, { status: PROBE_STATUSES, label: '200/400/404/422' });

await check('POST /v1/workpoint/resume returns a structured response', async () => {
  const body = {
    project_root: SCOPE.project_root,
    continuity_id: SCOPE.continuity_id ?? `probe-${Date.now()}`,
    session_id: SCOPE.session_id,
    workpoint_id: WORKPOINT_ID ?? `test-${Date.now()}`,
    mode: 'compact_prompt',
  };
  const r = await postJson('/v1/workpoint/resume', body);
  return r.status;
}, { status: PROBE_STATUSES, label: '200/400/404/422' });

await check('POST /v1/workpoint/evidence/link with empty body returns 422', async () => {
  const r = await postJson('/v1/workpoint/evidence/link', {});
  return r.status;
}, { status: [400, 422], label: '400/422 (missing required fields)' });

await check('POST /v1/device/pair/start returns a structured response', async () => {
  const body = {
    device_name: `menubar-e2e-${process.pid}`,
    platform: 'linux',
  };
  const r = await postJson('/v1/device/pair/start', body);
  return r.status;
}, { status: [200, 400, 422], label: '200/400/422' });

console.log('\n  POST endpoint smoke test\n');
const W = (s, n) => String(s).padEnd(n);
for (const r of results) {
  const tag = r.ok ? 'OK  ' : 'FAIL';
  console.log(`  ${tag}  ${W(r.actual, 22)}  ${W(r.expected, 14)}  ${W(r.dt + 'ms', 8)}  ${r.label}`);
}
console.log('');
console.log(`  ${pass}/${results.length} POST checks pass`);
if (HAS_SCOPE) {
  console.log(`  Scope used: ${SCOPE.project_root}`);
} else {
  console.log('  (no scope bound — 4 of 4 endpoints were skipped/probed via 400)');
}
if (WORKPOINT_ID) {
  console.log(`  Current workpoint: ${WORKPOINT_ID}`);
}

if (fail) {
  console.error(`\n  ${fail} failures`);
  process.exit(1);
}
console.log('\n  All POST checks pass. ✓\n');
