import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { stripTypeScriptTypes } from 'node:module';
import test from 'node:test';

const live = process.env.FOCUSA_LIVE_CHECKPOINT_CONTRACT === '1';
const sourceUrl = process.env.FOCUSA_CONTRACT_SOURCE
  ? new URL(`file://${process.env.FOCUSA_CONTRACT_SOURCE}`)
  : new URL('../src/tool-contracts.ts', import.meta.url);
let contracts;
let actual;

async function readActualContractsAndPolicy() {
  if (contracts) return;
  // Runtime type erasure only: no compiler, bundle, generated executable or install.
  const source = stripTypeScriptTypes(readFileSync(sourceUrl, 'utf8'), { mode: 'strip' });
  ({ FOCUSA_TOOL_CONTRACTS: contracts } = await import(`data:text/javascript,${encodeURIComponent(source)}`));
  const python = `import json,urllib.request,urllib.parse\nbase='http://127.0.0.1:8787/v1'\nout={}\nfor name in ['focusa_workpoint_checkpoint','focusa_workpoint_resume','focusa_project_identity']:\n req=urllib.request.Request(base+'/agent/tools?'+urllib.parse.urlencode({'query':name}),headers={'X-Focusa-Permissions':'ontology:read'})\n with urllib.request.urlopen(req,timeout=5)as response:body=json.load(response)\n out[name]=next(t for t in body['tools']if t['tool_names']['pi']==name)\nwith urllib.request.urlopen(base+'/health',timeout=5)as response:out['health']=json.load(response)\nprint(json.dumps(out))`;
  const quote = value => `'${value.replaceAll("'", "'\\''")}'`;
  actual = JSON.parse(execFileSync('ssh', [
    '-F', '/home/wirebot/.ssh/config', '-o', 'BatchMode=yes',
    '-o', 'ConnectTimeout=5', '-o', 'StrictHostKeyChecking=yes',
    'kh', `python3 -c ${quote(python)}`,
  ], { encoding: 'utf8', timeout: 25000, maxBuffer: 2 * 1024 * 1024 }));
}

test('production checkpoint contract matches real canonical write policy', { skip: !live }, async () => {
  await readActualContractsAndPolicy();
  const contract = contracts.find(c => c.name === 'focusa_workpoint_checkpoint');
  const policy = actual.focusa_workpoint_checkpoint.operation_policy;
  assert.equal(policy.method, 'POST');
  assert.equal(policy.path, '/v1/workpoint/checkpoint');
  assert.equal(policy.operation_class, 'value_mutation');
  assert.equal(contract.scope_requirement.kind, 'write', 'durable checkpoint is incorrectly described as a read');
  assert.equal(contract.authority_requirement.kind, 'canonical');
  assert.equal(contract.authority_requirement.path, 'daemon:/v1/workpoint/checkpoint');
  assert.match(contract.side_effect_profile, /write/);
  for (const key of ['operation_class', 'capability_family', 'commercial_treatment',
    'required_feature', 'limit_bucket', 'recovery_allowance', 'policy_owner']) {
    assert.deepEqual(contract.operation_policy[key], policy[key], `${key} disagrees with live enforcement`);
  }
  // Description provenance is distinct; neither catalog becomes execution authority.
  assert.equal(contract.operation_policy.source_owner, 'agent_capability_contracts');
  assert.equal(policy.source_owner, 'workpoint');
});

test('real resume and identity policies remain read projections', { skip: !live }, async () => {
  await readActualContractsAndPolicy();
  for (const name of ['focusa_workpoint_resume', 'focusa_project_identity']) {
    const contract = contracts.find(c => c.name === name);
    assert.equal(contract.scope_requirement.kind, 'read');
    assert.equal(contract.operation_policy.operation_class, 'read');
    assert.equal(actual[name].operation_policy.operation_class, 'read');
  }
});
