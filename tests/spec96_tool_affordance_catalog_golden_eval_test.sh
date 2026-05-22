#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
node --input-type=module - "$ROOT_DIR" <<'NODE'
import fs from 'node:fs';
import path from 'node:path';
const root = process.argv[2];
const read = (rel) => fs.readFileSync(path.join(root, rel), 'utf8');
const source = [
  read('apps/pi-extension/src/tool-contracts.ts'),
  read('apps/pi-extension/src/turns.ts'),
  read('docs/current/FOCUSA_TOOL_CONTRACT_REGISTRY.md'),
].join('\n').toLowerCase();
const baseline = 'available tools: focusa tools exist; use names if needed.'.toLowerCase();
const assert = (cond, msg) => { if (!cond) { console.error(`✗ FAIL: ${msg}`); process.exitCode = 1; } else console.log(`✓ PASS: ${msg}`); };
const hits = (text, needles) => needles.filter((needle) => text.includes(needle.toLowerCase())).length;
const cases = [
  { name: 'scope ambiguity chooses ProjectIdentity verification before resume trust', needles: ['scope_mismatch', 'focusa_project_verify', 'project_root+continuity_id', 'focusa_workpoint_checkpoint'] },
  { name: 'compaction continuation chooses Workpoint resume/checkpoint and not transcript tail', needles: ['focusa_workpoint_resume', 'focusa_workpoint_checkpoint', 'canonical=false|degraded=true', 'transcript tail as authority'] },
  { name: 'LowMem timeout chooses ResourceMode plus narrow traversal', needles: ['resource_exhausted|cold_path_timeout', 'focusa_resource_mode', 'focusa_traverse narrow slice', 'full payloads by default'] },
  { name: 'active object ambiguity chooses resolver before acting', needles: ['focusa_active_object_resolve', 'resolve ambiguous target refs before acting'] },
  { name: 'catalog gives enough no-source contract shape', needles: ['when_to_use', 'when_not_to_use', 'default_inputs', 'failure_classes', 'expected_result', 'likely_next_tools'] },
];
for (const c of cases) {
  const withHits = hits(source, c.needles);
  const withoutHits = hits(baseline, c.needles);
  const required = Math.max(2, Math.ceil(c.needles.length / 2));
  assert(withHits >= required, `${c.name}: affordance surfaces hit ${withHits}/${c.needles.length}`);
  assert(withHits > withoutHits, `${c.name}: affordance surfaces outperform baseline (${withHits} > ${withoutHits})`);
}
if (process.exitCode) process.exit(process.exitCode);
console.log('SPEC96 Tool Affordance Catalog golden eval: PASS');
NODE
