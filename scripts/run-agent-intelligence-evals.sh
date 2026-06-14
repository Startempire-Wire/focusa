#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CASES="${1:-$ROOT_DIR/tests/evals/agent_intelligence_cases.json}"

node --input-type=module - "$ROOT_DIR" "$CASES" <<'NODE'
import fs from 'node:fs';
import path from 'node:path';

const root = process.argv[2];
const casesPath = process.argv[3];
const fail = (msg) => { console.error(`✗ FAIL: ${msg}`); process.exit(1); };
const pass = (msg) => console.log(`✓ PASS: ${msg}`);

if (!fs.existsSync(casesPath)) fail(`cases file missing: ${casesPath}`);
const data = JSON.parse(fs.readFileSync(casesPath, 'utf8'));
if (data.schema !== 'focusa.agent_intelligence_evals.v1') fail('schema mismatch');
if (!Array.isArray(data.required_categories) || !Array.isArray(data.cases)) fail('required arrays missing');

const requiredFields = ['id','category','goal','input_signal','expected_behavior','required_refs','metric','score','threshold'];
const byCategory = new Map();
let scoreSum = 0;
for (const c of data.cases) {
  for (const field of requiredFields) {
    if (!(field in c)) fail(`case ${c.id || '<missing>'} missing ${field}`);
  }
  if (!Array.isArray(c.required_refs) || c.required_refs.length === 0) fail(`case ${c.id} missing required_refs`);
  if (typeof c.score !== 'number' || typeof c.threshold !== 'number') fail(`case ${c.id} score/threshold must be numeric`);
  if (c.score < c.threshold) fail(`case ${c.id} below threshold: ${c.score} < ${c.threshold}`);
  scoreSum += c.score;
  byCategory.set(c.category, (byCategory.get(c.category) || 0) + 1);
}
for (const category of data.required_categories) {
  if (!byCategory.has(category)) fail(`missing required category: ${category}`);
}
const aggregate = scoreSum / data.cases.length;
if (aggregate < data.aggregate_threshold) fail(`aggregate below threshold: ${aggregate.toFixed(3)} < ${data.aggregate_threshold}`);

for (const rel of [
  'docs/current/FOCUSA_AGENT_INTELLIGENCE_EVALS.md',
  'docs/current/GOLDEN_WORKFLOW.md',
  'docs/current/AUTHORITY_MODEL.md',
  'docs/current/AGENT_ADAPTER_CONTRACT.md',
]) {
  if (!fs.existsSync(path.join(root, rel))) fail(`required benchmark reference missing: ${rel}`);
}

pass(`agent intelligence eval cases pass: cases=${data.cases.length} categories=${data.required_categories.length} aggregate=${aggregate.toFixed(3)}`);
NODE
