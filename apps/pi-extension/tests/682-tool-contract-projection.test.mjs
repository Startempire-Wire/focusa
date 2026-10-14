import test from 'node:test';
import assert from 'node:assert/strict';
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';
import { loadPureSourceModule } from './helpers/source-section-harness.mjs';

const root = fileURLToPath(new URL('../../../', import.meta.url));
const legacyPath = new URL('../../../docs/current/focusa-tool-contracts.json', import.meta.url);
const runProducer = (cwd = root) => spawnSync(process.execPath,
  [join(root, 'scripts/validate-focusa-tool-contracts.mjs'), '--write-json'],
  { cwd, encoding: 'utf8', timeout: 30000, maxBuffer: 4 * 1024 * 1024 });

function runtimeContracts() {
  return loadPureSourceModule('tool-contracts.ts').FOCUSA_TOOL_CONTRACTS;
}

test('actual legacy registry producer retains canonical runtime operation policy', () => {
  const before = readFileSync(legacyPath);
  try {
    const result = runProducer();
    assert.equal(result.status, 0, result.stdout + result.stderr);
    const produced = JSON.parse(readFileSync(legacyPath, 'utf8')).contracts;
    const actual = runtimeContracts();
    assert.equal(produced.length, actual.length);
    for (const contract of actual) {
      const live = produced.find(item => item.name === contract.name);
      assert.deepEqual(live, contract, `embedded/live registry must match actual runtime contract ${contract.name}`);
    }
  } finally { writeFileSync(legacyPath, before); }
});

test('registry producer refuses missing canonical policy before writing a projection', () => {
  const fixture = mkdtempSync(join(tmpdir(), 'focusa-682-unknown-policy-'));
  for (const path of [
    'apps/pi-extension/src/tools.ts', 'apps/pi-extension/src/agent-runtime-tools.ts',
    'apps/pi-extension/src/sms-tools.ts', 'apps/pi-extension/src/tool-contracts.ts',
    'README.md', 'docs/focusa-tools/README.md', 'docs/current/focusa-tool-contracts.json',
  ]) {
    const target = join(fixture, path);
    mkdirSync(dirname(target), { recursive: true });
    copyFileSync(join(root, path), target);
  }
  const source = join(fixture, 'apps/pi-extension/src/tool-contracts.ts');
  const original = readFileSync(source, 'utf8');
  const changed = original.replace('operation_policy: operationPolicyForContract(contract),',
    'operation_policy: undefined,');
  assert.notEqual(changed, original, 'exact production policy owner anchor required');
  writeFileSync(source, changed);
  const projection = join(fixture, 'docs/current/focusa-tool-contracts.json');
  const before = readFileSync(projection);
  const result = runProducer(fixture);
  assert.notEqual(result.status, 0, 'unknown authority metadata must not be synthesized or ignored');
  assert.match(result.stdout + result.stderr, /canonical operation policy/i);
  assert.deepEqual(readFileSync(projection), before, 'failed generation must not replace existing registry');
});
