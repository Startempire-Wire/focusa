#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { parseJsonLikeTsLiteral } from './lib/json-like-ts.mjs';

const root = process.cwd();
const read = (p) => fs.readFileSync(path.join(root, p), 'utf8');
const exists = (p) => fs.existsSync(path.join(root, p));

const toolsSrc = read('apps/pi-extension/src/tools.ts');
const agentRuntimeToolsSrc = read('apps/pi-extension/src/agent-runtime-tools.ts');
const contractsSrc = read('apps/pi-extension/src/tool-contracts.ts');
const registry = JSON.parse(read('docs/current/focusa-tool-contracts.json'));
const capabilityRegistry = JSON.parse(read('docs/contracts/spec141/generated-capability-v2/agent-capability-descriptors.json'));
const descriptorByTool = new Map(capabilityRegistry.descriptors.map((descriptor) => [descriptor.tool_names.pi, descriptor]));
const routeClassification = JSON.parse(read('docs/contracts/spec141/generated-capability-v2/route-classification.json'));
const failures = [];
const warnings = [];
const uplifts = [];

function addFailure(message, detail) { failures.push({ message, detail }); }
function addWarning(message, detail) { warnings.push({ message, detail }); }
function addUplift(message, detail) { uplifts.push({ message, detail }); }

const dynamicPreloadMatch = toolsSrc.match(/const preloadReadTools:[^=]+ = (\[[\s\S]*?\n  \]);/);
const dynamicPreloadNames = dynamicPreloadMatch ? parseJsonLikeTsLiteral(dynamicPreloadMatch[1]).map(([name]) => name) : [];
const toolNames = [
  ...[...toolsSrc.matchAll(/name: "(focusa_[^"]+)"/g)].map((m) => m[1]),
  ...[...agentRuntimeToolsSrc.matchAll(/name: "(focusa_[^"]+)"/g)].map((m) => m[1]),
  ...dynamicPreloadNames,
];
const uniqueToolNames = [...new Set(toolNames)];
const contractNames = registry.contracts.map((contract) => contract.name);
const contractSet = new Set(contractNames);
const toolSet = new Set(uniqueToolNames);

for (const name of uniqueToolNames) if (!contractSet.has(name)) addFailure('registered tool missing contract', name);
for (const name of contractNames) if (!toolSet.has(name)) addFailure('contract missing registered tool', name);

const contractValidator = await import('node:child_process').then(({ spawnSync }) => spawnSync('node', ['scripts/validate-focusa-tool-contracts.mjs'], { cwd: root, encoding: 'utf8' }));
if (contractValidator.status !== 0) addFailure('TypeScript contract registry differs from JSON projection', contractValidator.stderr || contractValidator.stdout);

const routeInventory = new Set(routeClassification.routes.map((route) => route.path));

function routePath(route) {
  return String(route).replace(/^(GET|POST|PATCH|PUT|DELETE)\s+/, '').split('?')[0];
}

function pascal(token) {
  return String(token).split('-').filter(Boolean).map((part) => part.charAt(0).toUpperCase() + part.slice(1)).join('');
}

function cliSourceFor(rootCommand) {
  const map = {
    focus: 'crates/focusa-cli/src/commands/focus.rs',
    workpoint: 'crates/focusa-cli/src/commands/workpoint.rs',
    metacognition: 'crates/focusa-cli/src/commands/metacognition.rs',
    predict: 'crates/focusa-cli/src/commands/predict.rs',
    project: 'crates/focusa-cli/src/commands/project.rs',
    trajectory: 'crates/focusa-cli/src/commands/trajectory.rs',
    traverse: 'crates/focusa-cli/src/commands/traverse.rs',
    resource: 'crates/focusa-cli/src/commands/resource.rs',
    lineage: 'crates/focusa-cli/src/commands/lineage.rs',
    clt: 'crates/focusa-cli/src/commands/clt.rs',
    state: 'crates/focusa-cli/src/commands/debug.rs',
    'context-cognition': 'crates/focusa-cli/src/commands/context_cognition.rs',
    'call-stack': 'crates/focusa-cli/src/commands/call_stack.rs',
    device: 'crates/focusa-cli/src/commands/device_pairing.rs',
    bloatgaurd: 'crates/focusa-cli/src/commands/bloatgaurd.rs',
    utility: 'crates/focusa-cli/src/commands/utility.rs',
    dxux: 'crates/focusa-cli/src/commands/dxux.rs',
    explain: 'crates/focusa-cli/src/commands/dxux.rs',
    awareness: 'crates/focusa-cli/src/commands/awareness.rs',
    bg: 'crates/focusa-cli/src/commands/bg.rs',
    silent: 'crates/focusa-cli/src/commands/silent.rs',
    hlt: 'crates/focusa-cli/src/commands/hlt.rs',
    'agent-runtime': 'crates/focusa-cli/src/commands/agent_runtime.rs',
    preload: 'crates/focusa-cli/src/commands/preload.rs',
    'daemon-routing': 'crates/focusa-cli/src/commands/daemon_routing.rs',
    help: 'crates/focusa-cli/src/commands/help.rs',
    ontology: 'crates/focusa-cli/src/commands/ontology.rs',
    temporal: 'crates/focusa-cli/src/commands/temporal.rs',
  };
  return map[rootCommand];
}

function checkCliCommand(toolName, command) {
  const tokens = String(command)
    .split(/\s+/)
    .filter(Boolean)
    .filter((token) => !token.startsWith('--') && !(token.startsWith('<') && token.endsWith('>')));
  if (!tokens.length) return;
  if (tokens[0] === 'tmux') return;
  if (tokens[0] !== 'focusa') {
    addWarning('non-focusa CLI command is not statically checked', { tool: toolName, command });
    return;
  }
  const rootCandidates = tokens[1].split('|');
  const rootCmd = rootCandidates[0];
  const cliPath = rootCandidates.length > 1 ? 'crates/focusa-cli/src/main.rs' : cliSourceFor(rootCmd);
  if (!cliPath || !exists(cliPath)) {
    addFailure('CLI command root has no implementation file', { tool: toolName, command, root: rootCmd });
    return;
  }
  if (tokens.length <= 2) {
    addWarning('CLI command points only to a command group, not a concrete subcommand', { tool: toolName, command });
    return;
  }
  const src = read(cliPath);
  for (const token of tokens.slice(2)) {
    const alternatives = token.split('|');
    for (const alternative of alternatives) {
      const variant = pascal(alternative);
      if (!src.includes(variant) && !src.includes(alternative)) {
        addFailure('CLI subcommand token not found in implementation source', { tool: toolName, command, token: alternative, expected_variant: variant, file: cliPath });
      }
    }
  }
}

const familyNextBlock = contractsSrc.match(/const FAMILY_NEXT_TOOLS: Record<FocusaToolFamily, string\[]> = \{([\s\S]*?)\n\};/);
const familyNextText = familyNextBlock?.[1] || '';
if (!familyNextText.includes('focus_state: ["focusa_project_identity", "focusa_trajectory_view", "focusa_workpoint_resume"]')) {
  addFailure('Focus State family next tools must route outward to project/trajectory/workpoint instead of note-tool loops');
}

const wrapperIndex = toolsSrc.indexOf('pi.registerTool = ((tool: any) =>');
if (wrapperIndex === -1) addFailure('tool_result_v1 wrapper installation missing');

for (const contract of registry.contracts) {
  const name = contract.name;
  const descriptor = descriptorByTool.get(name);
  const assignable = descriptor?.availability?.assignable === true;
  if (!descriptor) addFailure('tool capability descriptor missing', name);
  const first = Math.max(toolsSrc.indexOf(`name: "${name}"`), agentRuntimeToolsSrc.indexOf(`name: "${name}"`));
  if (first === -1 && !agentRuntimeToolsSrc.includes(`name: "${name}"`) && !dynamicPreloadNames.includes(name)) {
    addFailure('tool implementation block missing', name);
  } else {
    const primaryIndex = toolsSrc.indexOf(`name: "${name}"`);
    if (wrapperIndex !== -1 && primaryIndex !== -1 && primaryIndex < wrapperIndex) addFailure('tool registered before tool_result_v1 wrapper install', name);
    const implementationSrc = toolsSrc.includes(`name: "${name}"`) ? toolsSrc : agentRuntimeToolsSrc;
    const implementationFirst = implementationSrc.indexOf(`name: "${name}"`);
    const next = implementationSrc.indexOf('name: "focusa_', implementationFirst + 1);
    const block = implementationSrc.slice(implementationFirst, next === -1 ? implementationSrc.length : next);
    if (!dynamicPreloadNames.includes(name)) {
      if (!block.includes('description:')) addFailure('tool implementation missing description', name);
      if (!block.includes('parameters:')) addFailure('tool implementation missing parameters schema', name);
      if (!/async\s+execute|execute:\s*async/.test(block)) addFailure('tool implementation missing async execute', name);
    }
  }

  if (!contract.doc_path || !exists(contract.doc_path)) addFailure('contract doc missing', { tool: name, doc_path: contract.doc_path });
  else {
    const doc = read(contract.doc_path);
    for (const route of assignable ? (contract.api_routes || []) : []) if (!doc.includes(route)) addFailure('tool doc missing declared API route', { tool: name, route });
    if (!assignable && !doc.includes('capability is unavailable because its daemon router is not registered')) addFailure('unavailable tool doc missing explicit reason', name);
    for (const command of contract.cli_commands || []) if (!doc.includes(command)) addFailure('tool doc missing declared CLI command', { tool: name, command });
    if (!doc.includes(`Result envelope: \`${descriptor?.result_envelope}\``)) addFailure('tool doc missing result envelope contract summary', name);
  }

  for (const route of assignable ? (contract.api_routes || []) : []) {
    if (!routeInventory.has(routePath(route))) addFailure('declared API route absent from reachable Rust route inventory', { tool: name, route });
  }
  for (const command of contract.cli_commands || []) checkCliCommand(name, command);

  if (contract.parity_status === 'full' && (!contract.api_routes?.length || !contract.cli_commands?.length)) {
    addFailure('full parity contract missing API or CLI command', { tool: name, api_routes: contract.api_routes, cli_commands: contract.cli_commands });
  }
  if ((contract.family === 'workpoint' || contract.family === 'focus_state' || contract.family === 'tree_lineage' || contract.family === 'metacognition') && contract.cli_commands?.some((cmd) => /^focusa \w+$/.test(cmd))) {
    addWarning('contract CLI command is group-only and weak for model routing', { tool: name, cli_commands: contract.cli_commands });
  }
}

const onboarding = exists('docs/current/FOCUSA_FRIENDLY_ONBOARDING.md') ? read('docs/current/FOCUSA_FRIENDLY_ONBOARDING.md') : '';
const choreography = exists('docs/current/FOCUSA_TOOL_CHOREOGRAPHY_MAP.md') ? read('docs/current/FOCUSA_TOOL_CHOREOGRAPHY_MAP.md') : '';
for (const needle of ['Friendly Focusa Q', 'project_root', 'trajectory', 'Workpoint', 'evidence', 'predict', 'metacog']) {
  if (!onboarding.includes(needle)) addFailure('friendly onboarding missing required concept', needle);
}
for (const needle of ['project_identity', 'trajectory_view', 'workpoint_checkpoint', 'evidence_capture', 'predict_record', 'metacog']) {
  if (!choreography.includes(needle)) addFailure('tool choreography map missing route concept', needle);
}

if (!failures.length) {
  addUplift('No required implementation/spec gaps detected by static audit; continue improving route quality and live per-tool smoke coverage.');
}

const result = {
  status: failures.length ? 'failed' : warnings.length ? 'passed_with_warnings' : 'passed',
  tool_count: uniqueToolNames.length,
  contract_count: registry.contracts.length,
  api_route_count: routeInventory.size,
  failures,
  warnings,
  uplifts,
};

if (process.argv.includes('--json')) console.log(JSON.stringify(result, null, 2));
else {
  console.log(`Focusa tool implementation/spec audit: ${result.status}`);
  console.log(`tools=${result.tool_count} contracts=${result.contract_count} api_routes=${result.api_route_count}`);
  for (const failure of failures) console.error(`FAIL ${failure.message}${failure.detail ? ` ${JSON.stringify(failure.detail)}` : ''}`);
  for (const warning of warnings) console.warn(`WARN ${warning.message}${warning.detail ? ` ${JSON.stringify(warning.detail)}` : ''}`);
  for (const uplift of uplifts) console.log(`UPLIFT ${uplift.message}${uplift.detail ? ` ${JSON.stringify(uplift.detail)}` : ''}`);
}
process.exit(failures.length ? 1 : 0);
