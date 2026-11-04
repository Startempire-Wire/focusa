import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import test from 'node:test';

const source = readFileSync(new URL('../src/tools.ts', import.meta.url), 'utf8');
const start = source.indexOf('  function summarizeWorkpointResponse(');
const end = source.indexOf('  function buildStateHygieneReport(', start);
assert.ok(start >= 0 && end > start, 'exercise the production summary implementation');
const summary = new Function('truncateForSummary', `${stripTypeScriptTypes(source.slice(start, end))}; return summarizeWorkpointResponse;`)((s, n) => String(s).slice(0, n));

test('old server read completion does not close an active Workpoint', () => {
  const text = summary({ status: 'completed', canonical: true, workpoint_id: 'wp-a', resume_packet: { status: 'active', next_slice: 'continue actual work' } });
  assert.match(text, /status=active id=wp-a canonical=true/);
  assert.match(text, /operation_status=completed/);
  assert.match(text, /next=continue actual work/);
});

test('new server lifecycle fields agree with its typed packet', () => {
  const text = summary({ status: 'active', workpoint_status: 'active', operation_status: 'completed', resume_packet: { status: 'active' } });
  assert.match(text, /^status=active /);
  assert.match(text, /operation_status=completed/);
});

test('a genuinely completed Workpoint remains completed', () => {
  assert.match(summary({ status: 'completed', resume_packet: { status: 'completed' } }), /^status=completed /);
});

test('a rejected response without a packet does not fabricate an active mission', () => {
  assert.match(summary({ status: 'blocked', canonical: false }), /^status=blocked /);
});

test('checkpoint acceptance is not fabricated into lifecycle completion', () => {
  assert.match(summary({ status: 'accepted', canonical: true }), /^status=accepted /);
});
