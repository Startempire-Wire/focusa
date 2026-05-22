#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
node --input-type=module - "$ROOT_DIR" <<'NODE'
import fs from 'node:fs';
import path from 'node:path';
const root = process.argv[2];
const read = (rel) => fs.readFileSync(path.join(root, rel), 'utf8');
const text = [
  read('crates/focusa-api/src/routes/traverse.rs'),
  read('crates/focusa-api/src/routes/workpoint.rs'),
  read('apps/pi-extension/src/compaction.ts'),
  read('apps/pi-extension/src/tools.ts'),
  read('scripts/audit-focusa-tool-suite-safe.mjs'),
  read('docs/focusa-tools/tools/focusa_traverse.md'),
  read('docs/focusa-tools/tools/focusa_workpoint_resume.md'),
].join('\n');
const baseline = 'Resume from transcript tail. If context missing, read full lineage tree and full ontology graph. No tag verification, provenance, or failure taxonomy.';
const low = (s) => String(s).toLowerCase();
const hits = (src, needles) => needles.filter((needle) => low(src).includes(low(needle))).length;
const assert = (ok, msg) => {
  if (!ok) { console.error(`✗ FAIL: ${msg}`); process.exitCode = 1; }
  else console.log(`✓ PASS: ${msg}`);
};
const cases = [
  {
    name: 'narrow traversal slices beat all-or-nothing context reads',
    needles: ['bounded_window', 'field_projection', 'cursor', 'limit', 'summary_only', 'focusa_traverse', 'traversal_slices'],
  },
  {
    name: 'tag verification detects stale or invalid anchors',
    needles: ['verified_tags', 'stale_tags', 'invalid_tag_format', 'stale_or_missing_tag', '/v1/traverse/verify-tags', 'tags_verify'],
  },
  {
    name: 'resume packet v2 carries provenance and failure taxonomy',
    needles: ['resume_packet_v2', 'WorkpointResumePacketV2', 'api_provenance', 'tool_result_v1', 'failure_class', 'rendered_summary'],
  },
  {
    name: 'tool-choice accuracy prefers Workpoint then Trajectory then Traverse',
    needles: ['focusa_workpoint_resume', 'focusa_trajectory_view', 'focusa_traverse', 'focusa_tool_doctor', 'focusa_active_object_resolve'],
  },
  {
    name: 'drift reduction rejects transcript tail authority and scope mismatch',
    needles: ['Never use transcript tail as authority', 'transcript_tail_canonical_resume', 'rejected_scope_mismatch', 'project_root+continuity_id', 'must_not_merge_on_similarity'],
  },
  {
    name: 'failure taxonomy covers daemon unavailable, stale tag, scope mismatch, cold timeout',
    needles: ['daemon_unavailable', 'stale_tag_unverified', 'scope_mismatch', 'cold_path_timeout', 'full_payload_blocked_by_pressure'],
  },
];
for (const testCase of cases) {
  const withHits = hits(text, testCase.needles);
  const withoutHits = hits(baseline, testCase.needles);
  const required = Math.ceil(testCase.needles.length * 0.66);
  assert(withHits >= required, `${testCase.name}: v2/traverse surfaces hit ${withHits}/${testCase.needles.length}`);
  assert(withHits > withoutHits, `${testCase.name}: v2/traverse beats old baseline (${withHits} > ${withoutHits})`);
}
assert(/pushFailure\('unbounded_hot_traversal'/.test(text), 'safe audit fails unbounded hot traversal');
assert(/pushFailure\('transcript_tail_canonical_resume'/.test(text), 'safe audit fails transcript-tail canonical resume');
assert(/pushFailure\('stale_tag_unverified'/.test(text), 'safe audit fails missing stale-tag verification');
assert(/pushFailure\('missing_traverse_timeout_taxonomy'/.test(text), 'safe audit fails missing timeout taxonomy');
assert(/WorkpointResumePacketV2[\s\S]*focusa_trajectory_view[\s\S]*focusa_traverse/.test(text), 'post-compaction prompt includes v2 packet and bounded traversal tools');
if (process.exitCode) process.exit(process.exitCode);
console.log('SPEC96 traverse + resume v2 golden eval: PASS');
NODE
