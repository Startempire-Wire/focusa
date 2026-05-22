#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
node --input-type=module - "$ROOT_DIR" <<'NODE'
import fs from 'node:fs';
import path from 'node:path';

const root = process.argv[2];
const read = (rel) => fs.readFileSync(path.join(root, rel), 'utf8');
const lower = (text) => String(text).toLowerCase();
const countHits = (text, needles) => needles.filter((needle) => lower(text).includes(lower(needle))).length;
const assert = (condition, message) => {
  if (!condition) {
    console.error(`✗ FAIL: ${message}`);
    process.exitCode = 1;
  } else {
    console.log(`✓ PASS: ${message}`);
  }
};

const sources = {
  pi: [
    read('apps/pi-extension/src/turns.ts'),
    read('apps/pi-extension/src/compaction.ts'),
    read('apps/pi-extension/src/awareness.ts'),
  ].join('\n'),
  cliApi: [
    read('docs/current/API_REFERENCE_CURRENT.md'),
    read('docs/current/CLI_REFERENCE_CURRENT.md'),
    read('docs/focusa-tools/tools/focusa_trajectory_view.md'),
    read('crates/focusa-cli/src/commands/awareness.rs'),
    read('crates/focusa-api/src/routes/trajectory.rs'),
  ].join('\n'),
  generic: [
    read('docs/current/NON_PI_AGENT_FOCUSA_USAGE.md'),
    read('crates/focusa-api/src/routes/awareness.rs'),
  ].join('\n'),
};

const baselines = {
  pi: 'Continue current task after compaction. Use latest transcript tail if needed.',
  cliApi: 'focusa workpoint resume --json; focusa doctor --json',
  generic: 'Focusa is available. Use doctor and workpoint resume when needed.',
};

const commonNeedles = [
  'focusa_trajectory_view',
  'project_root',
  'continuity_id',
  'high',
  'mid',
  'low',
  'active_gap',
  'evidence',
  'drift',
];

const cases = [
  {
    name: 'project mismatch demotes context instead of trusting carryover',
    needles: ['status=degraded', 'project mismatch', 'scope_mismatch', 'verify ProjectIdentity', 'cross_scope_workpoint_resume', 'mismatches'],
    surfaces: ['pi', 'cliApi'],
  },
  {
    name: 'compaction resume uses v2 packet plus trajectory before transcript tail',
    needles: ['WorkpointResumePacketV2', 'focusa_workpoint_resume', 'focusa_trajectory_view', 'Never use transcript tail', 'canonical=true', 'project_root+continuity_id'],
    surfaces: ['pi'],
  },
  {
    name: 'daemon degraded mode marks fallback and preserves trajectory recovery path',
    needles: ['degraded', 'cognition_degraded=true', '/v1/doctor', '/v1/trajectory/view', 'verify_first', 'focusa_tool_doctor'],
    surfaces: ['generic', 'pi', 'cliApi'],
  },
  {
    name: 'drift avoidance keeps same high-level trajectory sessions distinct',
    needles: ['must_not_merge_sessions', 'advisory only', 'same high-level', 'mid/low', 'project_root+continuity_id', 'session_id_is_temporal_metadata'],
    surfaces: ['pi', 'cliApi', 'generic'],
  },
  {
    name: 'assistance reduction gives specific next action instead of broad question asking',
    needles: ['context_sufficiency', 'recommended_action', 'active_gap', 'next_tools', 'best_next', 'operator_input', 'verify_first'],
    surfaces: ['pi', 'cliApi'],
  },
  {
    name: 'proof-based definition of done stays evidence-first',
    needles: ['proof/tests/API/file', 'capture/link evidence', 'evidence_refs', 'verification_hooks', 'Definition of Done', 'spec section'],
    surfaces: ['pi', 'generic', 'cliApi'],
  },
];

for (const [surface, text] of Object.entries(sources)) {
  const withScore = countHits(text, commonNeedles);
  const withoutScore = countHits(baselines[surface], commonNeedles);
  assert(withScore >= 6, `${surface}: trajectory-enriched prompt/source carries orientation markers (${withScore}/9)`);
  assert(withScore > withoutScore + 3, `${surface}: with-trajectory beats without-trajectory baseline (${withScore} > ${withoutScore}+3)`);
}

for (const testCase of cases) {
  const combined = testCase.surfaces.map((surface) => sources[surface]).join('\n');
  const baseline = testCase.surfaces.map((surface) => baselines[surface]).join('\n');
  const withHits = countHits(combined, testCase.needles);
  const withoutHits = countHits(baseline, testCase.needles);
  const required = Math.max(2, Math.ceil(testCase.needles.length / 2));
  assert(withHits >= required, `${testCase.name}: enriched surfaces hit ${withHits}/${testCase.needles.length}`);
  assert(withHits > withoutHits, `${testCase.name}: enriched surfaces outperform baseline (${withHits} > ${withoutHits})`);
}

const piCompaction = sources.pi;
assert(/focusa_workpoint_resume[\s\S]*focusa_trajectory_view[\s\S]*focusa_traverse/.test(piCompaction), 'Pi compaction guidance orders Workpoint -> Trajectory -> Traverse');
assert(!/full lineage tree[\s\S]{0,120}default read/i.test(piCompaction), 'Pi eval rejects full lineage as default resume context');

const genericCard = sources.generic;
assert(genericCard.includes('continuity_id') && genericCard.includes('session_id') && genericCard.includes('temporal metadata'), 'Generic awareness card separates continuity_id from temporal session_id');
assert(genericCard.includes('/v1/trajectory/view') && genericCard.includes('high/mid/low'), 'Generic awareness card injects trajectory orientation');

const api = sources.cliApi;
assert(api.includes('GET /v1/trajectory/view') && api.includes('focusa awareness card') && api.includes('--continuity-id'), 'CLI/API eval covers trajectory endpoint and awareness continuity flag');

if (process.exitCode) process.exit(process.exitCode);
console.log('SPEC96 trajectory multi-agent golden eval: PASS');
NODE
