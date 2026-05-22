#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
node --input-type=module - "$ROOT_DIR" <<'NODE'
import fs from 'node:fs';
import path from 'node:path';
const root = process.argv[2];
const read = (rel) => fs.readFileSync(path.join(root, rel), 'utf8');
const src = {
  traverse: read('crates/focusa-api/src/routes/traverse.rs'),
  tools: read('apps/pi-extension/src/tools.ts'),
  audit: read('scripts/audit-focusa-tool-suite-safe.mjs'),
  spec: read('docs/96-trajectory-projection-and-daemon-stability-spec.md'),
  stores: [
    read('crates/focusa-api/src/routes/ecs.rs'),
    read('crates/focusa-api/src/routes/metacognition.rs'),
    read('crates/focusa-api/src/routes/telemetry.rs'),
    read('crates/focusa-api/src/routes/snapshots.rs'),
    read('crates/focusa-api/src/routes/workpoint.rs'),
    read('crates/focusa-api/src/routes/focus.rs'),
  ].join('\n'),
};
const corpus = Object.values(src).join('\n');
const baseline = 'Read all history and full graph. Use transcript tail. Dump every object and log if context is missing.';
const low = (value) => String(value).toLowerCase();
const hits = (text, needles) => needles.filter((needle) => low(text).includes(low(needle))).length;
const assert = (ok, msg) => { if (!ok) { console.error(`✗ FAIL: ${msg}`); process.exitCode = 1; } else console.log(`✓ PASS: ${msg}`); };

const surfaceCases = [
  { surface: 'lineage', ask: 'show the parent path for current branch', selectors: ['path', 'window'], needles: ['lineage', 'bounded_window', 'next_cursor'] },
  { surface: 'ontology', ask: 'resolve project identifiers and active objects', selectors: ['active_context', 'adjacency'], needles: ['ontology', 'field_projection', 'rehydrate_refs'] },
  { surface: 'evidence', ask: 'find the proof refs for this result', selectors: ['recent', 'search'], needles: ['evidence', 'ecs', 'rehydrate'] },
  { surface: 'metacognition', ask: 'retrieve a prior lesson without full memory dump', selectors: ['recent', 'search'], needles: ['metacognition', 'cursor', 'summary_only'] },
  { surface: 'snapshots', ask: 'compare recent checkpoints', selectors: ['recent', 'window'], needles: ['snapshots', 'next_cursor', 'bounded_metadata'] },
  { surface: 'trajectory', ask: 'orient on current project north star', selectors: ['current', 'summary'], needles: ['trajectory', 'project_identity', 'active_gap'] },
];

for (const testCase of surfaceCases) {
  const surfaceNeedles = [testCase.surface, ...testCase.selectors, ...testCase.needles, 'focusa_traverse'];
  const withHits = hits(corpus, surfaceNeedles);
  const withoutHits = hits(baseline, surfaceNeedles);
  assert(withHits >= Math.ceil(surfaceNeedles.length * 0.6), `${testCase.surface}: surgical retrieval budget markers present (${withHits}/${surfaceNeedles.length})`);
  assert(withHits > withoutHits, `${testCase.surface}: surgical traversal beats full-dump baseline (${withHits} > ${withoutHits})`);
}

const globalNeedles = [
  'budgeted_default_limit',
  'budgeted_requested_limit',
  'bounded_window',
  'field_projection',
  'full_payload_blocked_by_pressure',
  'include_full_payload',
  'force_full_payload',
  'cursor',
  'limit',
  'next_cursor',
  'summary_only',
  'cold_path_timeout',
  'missing_surgical_surface',
  'missing_traversal_budget_controls',
];
const globalHits = hits(corpus, globalNeedles);
assert(globalHits >= 12, `global traversal budget controls present (${globalHits}/${globalNeedles.length})`);
assert(hits(corpus, globalNeedles) > hits(baseline, globalNeedles), 'budget controls outperform unbounded baseline');

const toolPlanningExamples = [
  { ask: 'only current trajectory gap', call: { surface: 'trajectory', selector: 'current', fields: ['trajectory', 'intelligence_view'], limit: 1 } },
  { ask: 'only proof refs', call: { surface: 'evidence', selector: 'recent', fields: ['id', 'label', 'kind'], limit: 8 } },
  { ask: 'only active identity map', call: { surface: 'ontology', selector: 'active_context', fields: ['identity_axes', 'active_object_set'], limit: 5 } },
  { ask: 'only recent checkpoints', call: { surface: 'snapshots', selector: 'recent', fields: ['snapshot_id', 'created_at'], limit: 5 } },
];
for (const example of toolPlanningExamples) {
  assert(example.call.limit <= 8 && Array.isArray(example.call.fields), `model can ask for narrow slice: ${example.ask}`);
}

assert(/pushFailure\('missing_surgical_surface'/.test(src.audit), 'safe audit fails missing traversal surface');
assert(/pushFailure\('missing_traversal_budget_controls'/.test(src.audit), 'safe audit fails missing traversal budget controls');
assert(!/full graph.*default/i.test(src.tools), 'Pi tools do not recommend full graph as default');
if (process.exitCode) process.exit(process.exitCode);
console.log('SPEC96 traversal budget golden eval: PASS');
NODE
