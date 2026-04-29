#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const root = process.cwd();
const toolsPath = path.join(root, 'apps/pi-extension/src/tools.ts');
const contractsPath = path.join(root, 'apps/pi-extension/src/tool-contracts.ts');
const readmePath = path.join(root, 'README.md');
const registryJsonPath = path.join(root, 'docs/current/focusa-tool-contracts.json');

function read(file) {
  return fs.readFileSync(file, 'utf8');
}

function fail(message, detail = undefined) {
  failures.push({ message, detail });
}

const failures = [];
const toolsSrc = read(toolsPath);
const contractsSrc = read(contractsPath);
const readme = read(readmePath);

const toolNames = [...toolsSrc.matchAll(/name: "(focusa_[^"]+)"/g)].map((m) => m[1]);
const uniqueToolNames = [...new Set(toolNames)];
if (toolNames.length !== uniqueToolNames.length) {
  fail('duplicate Pi tool registrations', toolNames.filter((name, idx) => toolNames.indexOf(name) !== idx));
}

const jsonMatch = contractsSrc.match(/export const FOCUSA_TOOL_CONTRACTS: FocusaToolContract\[] = ([\s\S]*?)\n\];/);
if (!jsonMatch) {
  fail('could not parse FOCUSA_TOOL_CONTRACTS registry');
}

let contracts = [];
if (jsonMatch) {
  contracts = JSON.parse(`${jsonMatch[1]}\n]`);
}

let registryJson = null;
if (fs.existsSync(registryJsonPath)) {
  registryJson = JSON.parse(read(registryJsonPath));
  if (JSON.stringify(registryJson.contracts) !== JSON.stringify(contracts)) {
    fail('JSON registry drifted from TypeScript registry', registryJsonPath);
  }
} else {
  fail('missing JSON registry projection', registryJsonPath);
}

const contractNames = contracts.map((contract) => contract.name);
const uniqueContractNames = [...new Set(contractNames)];
if (contractNames.length !== uniqueContractNames.length) {
  fail('duplicate tool contract entries', contractNames.filter((name, idx) => contractNames.indexOf(name) !== idx));
}

const toolSet = new Set(uniqueToolNames);
const contractSet = new Set(contractNames);
const missingContracts = uniqueToolNames.filter((name) => !contractSet.has(name));
const extraContracts = contractNames.filter((name) => !toolSet.has(name));
if (missingContracts.length) fail('tools missing contracts', missingContracts);
if (extraContracts.length) fail('contracts without registered tools', extraContracts);

const validFamilies = new Set(['focus_state', 'workpoint', 'work_loop', 'metacognition', 'tree_lineage', 'diagnostics_hygiene']);
const validParity = new Set(['full', 'domain', 'pi_only', 'local_only', 'degraded_known']);
const validExemptions = new Set(['local_scratchpad_only', 'pi_session_only', 'doctor_orchestration_only', 'domain_cli_only', 'api_domain_only', 'approval_placeholder', 'pi_session_snapshot_only', 'pi_only']);

const routeInventory = new Set([...fs.readdirSync(path.join(root, 'crates/focusa-api/src/routes'))
  .filter((file) => file.endsWith('.rs'))
  .flatMap((file) => [...read(path.join(root, 'crates/focusa-api/src/routes', file)).matchAll(/\.route\("([^"]+)"/g)].map((m) => m[1]))]);

for (const contract of contracts) {
  const prefix = `${contract.name}:`;
  if (!validFamilies.has(contract.family)) fail(`${prefix} invalid family`, contract.family);
  if (!contract.label) fail(`${prefix} missing label`);
  if (!contract.purpose) fail(`${prefix} missing purpose`);
  if (!contract.ontology_action || !contract.ontology_action.includes('.')) fail(`${prefix} invalid ontology_action`, contract.ontology_action);
  if (!Array.isArray(contract.ontology_objects) || contract.ontology_objects.length === 0) fail(`${prefix} missing ontology_objects`);
  if (!contract.doc_path || !fs.existsSync(path.join(root, contract.doc_path))) fail(`${prefix} missing doc_path`, contract.doc_path);
  if (contract.result_envelope !== 'tool_result_v1') fail(`${prefix} invalid result_envelope`, contract.result_envelope);
  if (!validParity.has(contract.parity_status)) fail(`${prefix} invalid parity_status`, contract.parity_status);
  if (!Array.isArray(contract.exemptions)) fail(`${prefix} exemptions must be array`);
  for (const exemption of contract.exemptions || []) {
    if (!validExemptions.has(exemption)) fail(`${prefix} unknown exemption`, exemption);
  }
  if ((!contract.api_routes || contract.api_routes.length === 0) && (!contract.exemptions || contract.exemptions.length === 0)) {
    fail(`${prefix} missing API routes without exemption`);
  }
  if ((!contract.cli_commands || contract.cli_commands.length === 0) && (!contract.exemptions || contract.exemptions.length === 0)) {
    fail(`${prefix} missing CLI commands without exemption`);
  }
  for (const route of contract.api_routes || []) {
    const routePath = route.replace(/^(GET|POST|PATCH|PUT|DELETE)\s+/, '');
    if (!routeInventory.has(routePath)) fail(`${prefix} API route not in route inventory`, route);
  }
  if (!readme.includes(contract.doc_path)) fail(`${prefix} README missing tool doc link`, contract.doc_path);
}

if (!routeInventory.has('/v1/ontology/tool-contracts')) {
  fail('ontology tool-contracts projection route missing', '/v1/ontology/tool-contracts');
}

const byFamily = contracts.reduce((acc, contract) => {
  acc[contract.family] = (acc[contract.family] || 0) + 1;
  return acc;
}, {});

const result = {
  status: failures.length ? 'failed' : 'passed',
  tools: uniqueToolNames.length,
  contracts: contracts.length,
  by_family: byFamily,
  missing_contracts: missingContracts,
  extra_contracts: extraContracts,
  failures,
};

if (process.argv.includes('--json')) {
  console.log(JSON.stringify(result, null, 2));
} else {
  console.log(`Spec90 tool contracts: ${result.status}`);
  console.log(`tools=${result.tools} contracts=${result.contracts}`);
  console.log(`by_family=${JSON.stringify(result.by_family)}`);
  if (failures.length) {
    for (const f of failures) console.error(`FAIL ${f.message}${f.detail ? ` ${JSON.stringify(f.detail)}` : ''}`);
  }
}

process.exit(failures.length ? 1 : 0);
