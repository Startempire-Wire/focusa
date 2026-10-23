#!/usr/bin/env bash
# Scenario definitions are not observations or proof of isolated arm execution.
# ARM identifies the requested arm only; no model/harness executor exists here yet.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CASES="${1:-$ROOT_DIR/tests/evals/agent_intelligence_cases.json}"
ARM="${ARM:-on}"

node --input-type=module - "$ROOT_DIR" "$CASES" "$ARM" <<'NODE'
import fs from 'node:fs';
import path from 'node:path';

const root = process.argv[2];
const casesPath = process.argv[3];
const arm = process.argv[4];
const fail = (msg) => { console.error(`FAIL: ${msg}`); process.exit(1); };
if (!['on', 'off', 'candidate'].includes(arm)) fail(`unsupported evaluation arm: ${arm}`);
if (!fs.existsSync(casesPath)) fail(`cases file missing: ${casesPath}`);
const data = JSON.parse(fs.readFileSync(casesPath, 'utf8'));
if (data.schema !== 'focusa.agent_intelligence_evals.v1') fail('schema mismatch');
if (!Array.isArray(data.required_categories) || data.required_categories.length === 0 ||
    !Array.isArray(data.cases) || data.cases.length === 0) fail('nonempty required arrays missing');
if (!Number.isFinite(data.aggregate_threshold) || data.aggregate_threshold < 0 || data.aggregate_threshold > 1)
  fail('aggregate threshold must be finite and between zero and one');
if ('score' in data || 'aggregate_score' in data) fail('scenario definitions cannot supply observed scores');

const contractsPath = path.join(root, 'docs/current/focusa-tool-contracts.json');
const toolNames = new Set(JSON.parse(fs.readFileSync(contractsPath, 'utf8')).contracts.map(c => c.name));
const requiredFields = ['id', 'category', 'goal', 'input_signal', 'expected_behavior', 'required_refs', 'metric', 'threshold'];
const byCategory = new Set();
const ids = new Set();
for (const c of data.cases) {
  for (const field of requiredFields) {
    if (!(field in c)) fail(`case ${c.id || '<missing>'} missing ${field}`);
  }
  if ('score' in c) fail(`case ${c.id} supplies an untrusted fixture score, not an observation`);
  if (!c.id || ids.has(c.id)) fail(`missing or duplicate case id: ${c.id}`);
  ids.add(c.id);
  if (!Number.isFinite(c.threshold) || c.threshold < 0 || c.threshold > 1)
    fail(`case ${c.id} threshold must be finite and between zero and one`);
  if (!Array.isArray(c.required_refs) || c.required_refs.length === 0) fail(`case ${c.id} missing required_refs`);
  for (const ref of c.required_refs) {
    if (typeof ref !== 'string' || !ref) fail(`case ${c.id} has an invalid required reference`);
    if (ref.startsWith('focusa_')) {
      if (!toolNames.has(ref)) fail(`case ${c.id} missing declared tool reference: ${ref}`);
    } else {
      const resolved = path.resolve(root, ref);
      if (!resolved.startsWith(root + path.sep) || !fs.existsSync(resolved) || !fs.statSync(resolved).isFile())
        fail(`case ${c.id} missing repository file reference: ${ref}`);
    }
  }
  byCategory.add(c.category);
}
for (const category of data.required_categories) {
  if (!byCategory.has(category)) fail(`missing required category: ${category}`);
}
for (const rel of [
  'docs/current/FOCUSA_AGENT_INTELLIGENCE_EVALS.md',
  'docs/current/GOLDEN_WORKFLOW.md',
  'docs/current/AUTHORITY_MODEL.md',
  'docs/current/AGENT_ADAPTER_CONTRACT.md',
]) {
  if (!fs.existsSync(path.join(root, rel))) fail(`required benchmark reference missing: ${rel}`);
}

// Definition/reference checks cannot establish execution, isolation, scores or safety.
console.error(`UNMEASURED: arm=${arm}; scenario definitions validated; no execution-backed ON/OFF/candidate observations, model/harness provenance or comparative metrics exist; benchmark acceptance unavailable`);
process.exit(78);
NODE
