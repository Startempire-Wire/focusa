#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { spawnSync } from 'node:child_process';

const root = process.cwd();
const baseUrl = process.env.FOCUSA_API_BASE_URL || 'http://127.0.0.1:8787';
const jsonMode = process.argv.includes('--json');
const safeFixturesMode = process.argv.includes('--safe-fixtures');
const failures = [];
const checkedEndpoints = [];
const fixtureChecks = [];

function readJson(file) {
  return JSON.parse(fs.readFileSync(path.join(root, file), 'utf8'));
}

function fail(message, detail) {
  failures.push({ message, detail });
}

async function getJson(endpoint) {
  const url = `${baseUrl}${endpoint}`;
  checkedEndpoints.push(endpoint);
  const res = await fetch(url, { signal: AbortSignal.timeout(5000) });
  if (!res.ok) throw new Error(`${endpoint} returned HTTP ${res.status}`);
  return await res.json();
}

const staticValidation = spawnSync(process.execPath, ['scripts/validate-focusa-tool-contracts.mjs', '--json'], {
  cwd: root,
  encoding: 'utf8',
});
if (staticValidation.status !== 0) {
  fail('static Spec90 validation failed', staticValidation.stderr || staticValidation.stdout);
}

const staticRegistry = readJson('docs/current/focusa-tool-contracts.json');
const staticContracts = staticRegistry.contracts || [];
let health = null;
let liveRegistry = null;
try {
  health = await getJson('/v1/health');
  if (!health?.ok) fail('daemon health returned non-ok', health);
} catch (err) {
  fail('daemon health request failed', err.message);
}
try {
  liveRegistry = await getJson('/v1/ontology/tool-contracts');
} catch (err) {
  fail('live ontology tool-contracts request failed', err.message);
}

const liveContracts = liveRegistry?.contracts || [];
const staticNames = [...staticContracts.map((contract) => contract.name)].sort();
const liveNames = [...liveContracts.map((contract) => contract.name)].sort();
const missingLiveContracts = staticNames.filter((name) => !liveNames.includes(name));
const extraLiveContracts = liveNames.filter((name) => !staticNames.includes(name));
function sortJson(value) {
  if (Array.isArray(value)) return value.map(sortJson);
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.keys(value).sort().map((key) => [key, sortJson(value[key])]));
  }
  return value;
}
const payloadEqual = JSON.stringify(sortJson(liveRegistry)) === JSON.stringify(sortJson(staticRegistry));

if (staticRegistry.version !== 'spec90.tool_contracts.v1') fail('unexpected static registry version', staticRegistry.version);
if (liveRegistry && liveRegistry.version !== staticRegistry.version) fail('live registry version mismatch', { static: staticRegistry.version, live: liveRegistry.version });
if (liveContracts.length !== staticContracts.length) fail('contract count mismatch', { static: staticContracts.length, live: liveContracts.length });
if (missingLiveContracts.length) fail('contracts missing from live registry', missingLiveContracts);
if (extraLiveContracts.length) fail('extra contracts in live registry', extraLiveContracts);
if (liveRegistry && !payloadEqual) fail('live registry payload differs from static JSON registry');

const apiRef = fs.readFileSync(path.join(root, 'docs/current/API_REFERENCE_CURRENT.md'), 'utf8');
if (!apiRef.includes('/v1/ontology/tool-contracts')) fail('API reference missing live tool-contracts route');

if (safeFixturesMode) {
  const safeFixtureEndpoints = [
    { family: 'workpoint', representative_tools: ['focusa_workpoint_resume'], endpoint: '/v1/workpoint/current' },
    { family: 'work_loop', representative_tools: ['focusa_work_loop_status'], endpoint: '/v1/work-loop/status' },
    { family: 'tree_lineage', representative_tools: ['focusa_tree_head', 'focusa_lineage_tree'], endpoint: '/v1/lineage/head' },
    { family: 'metacognition', representative_tools: ['focusa_metacog_recent_reflections'], endpoint: '/v1/metacognition/reflections/recent' },
    { family: 'focus_state', representative_tools: ['focusa_current_focus'], endpoint: '/v1/focus/frame/current' },
  ];
  for (const probe of safeFixtureEndpoints) {
    try {
      const body = await getJson(probe.endpoint);
      fixtureChecks.push({ ...probe, status: 'passed', response_kind: typeof body, mutates: false });
    } catch (err) {
      fixtureChecks.push({ ...probe, status: 'blocked', error: err.message, mutates: false });
      fail('safe fixture read-only probe failed', { endpoint: probe.endpoint, error: err.message });
    }
  }
}

const result = {
  status: failures.length ? 'failed' : 'passed',
  health: health ? { ok: health.ok, version: health.version } : null,
  static_version: staticRegistry.version,
  live_version: liveRegistry?.version || null,
  static_count: staticContracts.length,
  live_count: liveContracts.length,
  missing_live_contracts: missingLiveContracts,
  extra_live_contracts: extraLiveContracts,
  payload_equal: payloadEqual,
  checked_endpoints: checkedEndpoints,
  safe_fixtures_mode: safeFixturesMode,
  fixture_checks: fixtureChecks,
  failures,
};

if (jsonMode) {
  console.log(JSON.stringify(result, null, 2));
} else {
  console.log(`Spec91 live tool contract proof: ${result.status}`);
  console.log(`health=${result.health?.ok ? 'ok' : 'blocked'} version=${result.health?.version || 'unknown'}`);
  console.log(`static=${result.static_version} count=${result.static_count}`);
  console.log(`live=${result.live_version || 'none'} count=${result.live_count}`);
  console.log(`payload_equal=${result.payload_equal}`);
  console.log(`checked_endpoints=${result.checked_endpoints.join(',')}`);
  if (safeFixturesMode) console.log(`fixture_checks=${fixtureChecks.map((check) => `${check.family}:${check.status}`).join(',')}`);
  if (failures.length) {
    for (const failure of failures) console.error(`FAIL ${failure.message}${failure.detail ? ` ${JSON.stringify(failure.detail)}` : ''}`);
  }
}

process.exit(failures.length ? 1 : 0);
