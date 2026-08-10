#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { parseJsonLikeTsLiteral } from './lib/json-like-ts.mjs';

const root = process.cwd();
const toolsPath = path.join(root, 'apps/pi-extension/src/tools.ts');
const agentRuntimeToolsPath = path.join(root, 'apps/pi-extension/src/agent-runtime-tools.ts');
const contractsPath = path.join(root, 'apps/pi-extension/src/tool-contracts.ts');
const readmePath = path.join(root, 'README.md');
const toolDocsIndexPath = path.join(root, 'docs/focusa-tools/README.md');
const registryJsonPath = path.join(root, 'docs/current/focusa-tool-contracts.json');
const choreographyJsonPath = path.join(root, 'docs/current/focusa-tool-choreography.json');
const writeJsonProjection = process.argv.includes('--write-json');

function read(file) {
  return fs.readFileSync(file, 'utf8');
}

function fail(message, detail = undefined) {
  failures.push({ message, detail });
}

const failures = [];
const toolsSrc = read(toolsPath);
const agentRuntimeToolsSrc = read(agentRuntimeToolsPath);
const contractsSrc = read(contractsPath);
const readme = read(readmePath);

const wrapperInstallIndex = toolsSrc.indexOf('pi.registerTool = ((tool: any) =>');
const wrapperInstallBlock = wrapperInstallIndex === -1 ? '' : toolsSrc.slice(wrapperInstallIndex, wrapperInstallIndex + 240);
if (wrapperInstallIndex === -1 || !wrapperInstallBlock.includes('withToolResultEnvelope(tool)') || !wrapperInstallBlock.includes('withAgentFirstSchemas(')) {
  fail('tool_result_v1 plus agent-first schema wrapper install not found in registerTools');
}

const toolMatches = [...toolsSrc.matchAll(/name: "(focusa_[^"]+)"/g)];
const agentRuntimeToolMatches = [...agentRuntimeToolsSrc.matchAll(/name: "(focusa_[^"]+)"/g)];
const dynamicPreloadMatch = toolsSrc.match(/const preloadReadTools:[^=]+ = (\[[\s\S]*?\n  \]);/);
const dynamicPreloadNames = dynamicPreloadMatch
  ? parseJsonLikeTsLiteral(dynamicPreloadMatch[1]).map(([name]) => name)
  : [];
const toolNames = [
  ...toolMatches.map((m) => m[1]),
  ...agentRuntimeToolMatches.map((m) => m[1]),
  ...dynamicPreloadNames,
];
for (const match of toolMatches) {
  if (wrapperInstallIndex !== -1 && match.index < wrapperInstallIndex) {
    fail('tool registered before tool_result_v1 wrapper install', match[1]);
  }
}
const uniqueToolNames = [...new Set(toolNames)];
if (toolNames.length !== uniqueToolNames.length) {
  fail('duplicate Pi tool registrations', toolNames.filter((name, idx) => toolNames.indexOf(name) !== idx));
}

const preloadMatch = contractsSrc.match(/const PRELOAD_TOOL_CONTRACTS: FocusaToolContract\[] = (\[[\s\S]*?\])\.map/);
const agentRuntimeMatch = contractsSrc.match(/const AGENT_RUNTIME_TOOL_CONTRACTS: FocusaToolContract\[] = (\[[\s\S]*?\])\.map/);
const jsonMatch = contractsSrc.match(/export const FOCUSA_TOOL_CONTRACTS: FocusaToolContract\[] = ([\s\S]*?)\n\](?:\s*as\s+[\w\[\].]+\)?\.map|\.map|\)|;)/);
if (!preloadMatch) {
  fail('could not parse PRELOAD_TOOL_CONTRACTS registry');
}
if (!agentRuntimeMatch) {
  fail('could not parse AGENT_RUNTIME_TOOL_CONTRACTS registry');
}
if (!jsonMatch) {
  fail('could not parse FOCUSA_TOOL_CONTRACTS registry');
}

let contracts = [];
if (preloadMatch && agentRuntimeMatch && jsonMatch) {
  const preloadRows = parseJsonLikeTsLiteral(preloadMatch[1]);
  const preloadContracts = preloadRows.map(([suffix, label, purpose, sideEffect, method]) => {
    const action = suffix.replace('_', '-');
    const write = sideEffect.startsWith('write');
    return {
      name: `focusa_preload_${suffix}`,
      family: 'preload',
      label,
      purpose,
      ontology_action: `preload.${suffix}`,
      ontology_objects: suffix.startsWith('receipt') ? ['AgentBootstrapReceipt'] : ['AgentBootstrapPacket'],
      api_routes: [`${method} /v1/preload/${action}`],
      cli_commands: [`focusa preload ${action}`],
      core_surface: 'Spec111 agent context bootstrap and delivery',
      doc_path: `docs/focusa-tools/tools/focusa_preload_${suffix}.md`,
      spec_path: 'docs/111-agent-context-bootstrap-and-delivery-spec.md',
      result_envelope: 'tool_result_v1',
      side_effect_profile: sideEffect,
      parity_status: 'full',
      exemptions: [],
      live_check: 'contract_static plus scoped preload route verification',
      scope_requirement: { kind: write ? 'write' : 'read', route_family: 'preload' },
      authority_requirement: write
        ? { kind: 'canonical', path: `/v1/preload/${action}` }
        : { kind: 'advisory_only' },
    };
  });
  const agentRuntimeRows = parseJsonLikeTsLiteral(agentRuntimeMatch[1]);
  const agentRuntimeContracts = agentRuntimeRows.map(([name, label, action, route, command, write]) => ({
    name: String(name),
    family: 'agent_runtime',
    label: String(label),
    purpose: `Operate the Spec 140 ${String(label).toLowerCase()} surface with typed scope and evidence.`,
    ontology_action: String(action),
    ontology_objects: ['ProjectAgentRuntimeConstitution', 'InstructionClaim', 'RuntimeArtifactProjection'],
    api_routes: route === 'local' ? [] : [String(route)],
    cli_commands: [String(command)],
    core_surface: 'Spec140 project-agent Runtime Constitution compiler and delivery',
    doc_path: `docs/focusa-tools/tools/${String(name)}.md`,
    spec_path: 'docs/140-project-agent-runtime-constitution-instruction-authority-system-prompt-and-cross-harness-compiler-spec.md',
    result_envelope: 'tool_result_v1',
    side_effect_profile: write ? 'confirmed_receipted_artifact_delivery' : 'read_or_preview_only',
    parity_status: route === 'local' ? 'pi_only' : 'full',
    exemptions: route === 'local' ? ['pi_only'] : [],
    live_check: 'contract_static plus typed /v1/agent-runtime route verification',
    scope_requirement: { kind: write ? 'write' : 'read', route_family: 'agent-runtime' },
    authority_requirement: write
      ? { kind: 'canonical', path: '/v1/agent-runtime/delivery/commit' }
      : { kind: 'advisory_only' },
  }));
  const baseContractsLiteral = jsonMatch[1].replace(/^[(]/, "")
    .replace(/\.\.\.PRELOAD_TOOL_CONTRACTS\s*,?/, '')
    .replace(/\.\.\.AGENT_RUNTIME_TOOL_CONTRACTS\s*,?/, '');
  const baseContracts = parseJsonLikeTsLiteral(`${baseContractsLiteral}\n]`);
  contracts = [...agentRuntimeContracts, ...preloadContracts, ...baseContracts];
}

const registryProjection = {
  schema: 'focusa.tool_contracts.v1',
  version: 'spec90.tool_contracts.v1',
  tool_count: contracts.length,
  contracts,
};
let registryJson = null;
if (writeJsonProjection) {
  fs.writeFileSync(registryJsonPath, `${JSON.stringify(registryProjection, null, 2)}\n`);
  registryJson = registryProjection;
} else if (fs.existsSync(registryJsonPath)) {
  registryJson = JSON.parse(read(registryJsonPath));
  if (JSON.stringify(registryJson.contracts) !== JSON.stringify(contracts) || registryJson.tool_count !== contracts.length) {
    fail('JSON registry drifted from TypeScript registry', registryJsonPath);
  }
} else {
  fail('missing JSON registry projection', registryJsonPath);
}

const nextToolsMatch = contractsSrc.match(new RegExp('const TOOL_NEXT_TOOLS: Record<string, string\\[]> = ([\\s\\S]*?)\\n\\};'));
let nextTools = {};
if (nextToolsMatch) {
  nextTools = parseJsonLikeTsLiteral(`${nextToolsMatch[1]}\n}`);
} else {
  fail('could not parse TOOL_NEXT_TOOLS map');
}

let choreographyJson = null;
if (fs.existsSync(choreographyJsonPath)) {
  choreographyJson = JSON.parse(read(choreographyJsonPath));
} else {
  fail('missing choreography JSON projection', choreographyJsonPath);
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

const validFamilies = new Set(['focus_state', 'workpoint', 'work_loop', 'metacognition', 'tree_lineage', 'diagnostics_hygiene', 'trajectory', 'project_identity', 'traversal', 'session_transfer', 'awareness', 'preload', 'agent_runtime']);
const validParity = new Set(['full', 'domain', 'pi_only', 'local_only', 'degraded_known', 'api_only']);
const validExemptions = new Set(['local_scratchpad_only', 'pi_session_only', 'doctor_composition_only', 'domain_cli_only', 'api_domain_only', 'pi_session_snapshot_only', 'pi_only', 'api_only']);

const routeInventory = new Set([...fs.readdirSync(path.join(root, 'crates/focusa-api/src/routes'))
  .filter((file) => file.endsWith('.rs'))
  .flatMap((file) => [...read(path.join(root, 'crates/focusa-api/src/routes', file)).matchAll(/\.route\(\s*"([^"]+)"/g)].map((m) => m[1]))]);

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
    const routePath = route.replace(/^(GET|POST|PATCH|PUT|DELETE)\s+/, '').split('?')[0];
    if (!routeInventory.has(routePath)) fail(`${prefix} API route not in route inventory`, route);
  }
}

if (!fs.existsSync(toolDocsIndexPath)) {
  fail('missing dedicated tool docs index', toolDocsIndexPath);
}
if (!readme.includes('docs/focusa-tools/README.md')) {
  fail('README missing dedicated tool docs index link', 'docs/focusa-tools/README.md');
}

if (!routeInventory.has('/v1/ontology/tool-contracts')) {
  fail('ontology tool-contracts projection route missing', '/v1/ontology/tool-contracts');
}
if (!routeInventory.has('/v1/ontology/tool-choreography')) {
  fail('ontology tool-choreography projection route missing', '/v1/ontology/tool-choreography');
}


if (Object.keys(nextTools).length) {
  const nextToolSources = Object.keys(nextTools);
  const missingNextToolSources = contractNames.filter((name) => !nextToolSources.includes(name));
  const extraNextToolSources = nextToolSources.filter((name) => !contractSet.has(name));
  if (missingNextToolSources.length) fail('tools missing per-tool next-tool routing', missingNextToolSources);
  if (extraNextToolSources.length) fail('per-tool next-tool routing has unknown sources', extraNextToolSources);
  for (const [source, targets] of Object.entries(nextTools)) {
    if (!Array.isArray(targets) || targets.length === 0) fail(`${source}: next tools must be non-empty array`);
    for (const target of targets || []) {
      if (!contractSet.has(target)) fail(`${source}: next tool target missing contract`, target);
    }
  }
}

if (writeJsonProjection && choreographyJson) {
  const edges = [];
  for (const [from, targets] of Object.entries(nextTools)) {
    for (const target of targets || []) edges.push({ from, to: target, weight: 1 });
  }
  choreographyJson = {
    ...choreographyJson,
    schema: 'focusa.tool_choreography.v1',
    contract_version: 'spec90.tool_contracts.v1',
    tool_count: contracts.length,
    edge_count: edges.length,
    per_tool_next_tools: nextTools,
    edges,
  };
  fs.writeFileSync(choreographyJsonPath, `${JSON.stringify(choreographyJson, null, 2)}\n`);
}

if (choreographyJson) {
  if (choreographyJson.schema !== 'focusa.tool_choreography.v1') fail('invalid choreography schema', choreographyJson.schema);
  if (!choreographyJson.dynamic_weight_policy) fail('missing choreography dynamic weight policy', choreographyJsonPath);
  if (!Array.isArray(choreographyJson.runtime_weight_adjustments)) fail('missing choreography runtime_weight_adjustments array', choreographyJsonPath);
  if (choreographyJson.tool_count !== contracts.length) fail('choreography tool_count mismatch', choreographyJson.tool_count);
  if (JSON.stringify(choreographyJson.per_tool_next_tools) !== JSON.stringify(nextTools)) {
    fail('choreography JSON drifted from TypeScript TOOL_NEXT_TOOLS', choreographyJsonPath);
  }
  const edges = Array.isArray(choreographyJson.edges) ? choreographyJson.edges : [];
  const expectedEdges = Object.values(nextTools).reduce((sum, targets) => sum + (Array.isArray(targets) ? targets.length : 0), 0);
  if (edges.length !== expectedEdges || choreographyJson.edge_count !== expectedEdges) {
    fail('choreography edge count mismatch', { expected: expectedEdges, actual: edges.length, declared: choreographyJson.edge_count });
  }
  for (const edge of edges) {
    if (!contractSet.has(edge.from) || !contractSet.has(edge.to)) fail('choreography edge references unknown tool', edge);
    if (typeof edge.weight !== 'number' || edge.weight <= 0) fail('choreography edge has invalid weight', edge);
  }
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
