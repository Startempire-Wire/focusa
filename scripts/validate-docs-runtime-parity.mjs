#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const workspaceManifest = fs.readFileSync(path.join(root, 'Cargo.toml'), 'utf8');
const currentVersion = workspaceManifest.match(/^version\s*=\s*"([^"]+)"/m)?.[1] || '0.9.13-dev';
const checks = [
  ['README.md', `v${currentVersion}`],
  ['README.md', 'Agent Awareness Quickstart'],
  ['README.md', 'Friendly Focusa Onboarding Q'],
  ['README.md', 'Tool Implementation-to-Spec Audit'],
  ['README.md', 'Model-Visible Awareness Surfaces'],
  ['README.md', 'Non-Pi Agent Focusa Usage'],
  ['docs/README.md', 'NON_PI_AGENT_FOCUSA_USAGE.md'],
  ['docs/README.md', '93-non-pi-agent-focusa-awareness-spec.md'],
  ['docs/current/API_REFERENCE_CURRENT.md', 'GET /v1/awareness/card'],
  ['docs/current/CLI_REFERENCE_CURRENT.md', 'awareness      Non-Pi agent awareness utility cards'],
  ['docs/current/CLI_REFERENCE_CURRENT.md', 'focusa awareness card'],
  ['docs/current/CLI_REFERENCE_CURRENT.md', 'focusa focus update --decision'],
  ['docs/current/CLI_REFERENCE_CURRENT.md', 'focusa workpoint checkpoint --project-root'],
  ['docs/current/CLI_REFERENCE_CURRENT.md', 'focusa lineage extract'],
  ['docs/current/CLI_REFERENCE_CURRENT.md', 'focusa state snapshot compare-latest'],
  ['docs/current/FOCUSA_FRIENDLY_ONBOARDING.md', 'Friendly Focusa Q'],
  ['docs/current/FOCUSA_FRIENDLY_ONBOARDING.md', 'project/architecture'],
  ['docs/current/FOCUSA_TOOL_CHOREOGRAPHY_MAP.md', 'project_identity'],
  ['docs/current/FOCUSA_MODEL_VISIBLE_AWARENESS.md', 'PROJECT_TRAJECTORY'],
  ['docs/current/FOCUSA_MODEL_VISIBLE_AWARENESS.md', 'PROJECT_INFRA'],
  ['docs/current/FOCUSA_TOOL_IMPLEMENTATION_SPEC_AUDIT.md', 'focusa focus update'],
  ['docs/current/NON_PI_AGENT_FOCUSA_USAGE.md', 'apps/focusa-awareness'],
  ['docs/current/NON_PI_AGENT_FOCUSA_USAGE.md', 'scripts/prove-openclaw-focusa-injection-live.mjs'],
  ['docs/93-non-pi-agent-focusa-awareness-spec.md', '/v1/awareness/card'],
  ['CHANGELOG.md', 'OpenClaw Focusa awareness plugin'],
  ['apps/focusa-awareness/openclaw.plugin.json', 'focusa-awareness'],
  ['apps/focusa-awareness/index.ts', '/v1/awareness/card'],
  ['scripts/validate-openclaw-focusa-awareness-config.mjs', 'focusa-awareness'],
  ['scripts/prove-openclaw-focusa-injection-live.mjs', 'focusa-awareness: injected card session='],
];
const failures = [];

function methodRouteInventory() {
  // Mirror scripts/generate-agent-route-classification.py reachability: only
  // modules wired into the running daemon are part of the served contract.
  // Uncompiled orphan files must not fabricate documentation requirements.
  const routesDir = path.join(root, 'crates/focusa-api/src/routes');
  const serverSource = fs.readFileSync(path.join(root, 'crates/focusa-api/src/server.rs'), 'utf8');
  const reachable = new Set(['server']);
  const queue = [...serverSource.matchAll(/routes::([a-zA-Z0-9_]+)::(?:router|routes)\(/g)].map((match) => match[1]);
  while (queue.length) {
    const moduleName = queue.pop();
    if (reachable.has(moduleName)) continue;
    reachable.add(moduleName);
    const filePath = path.join(routesDir, `${moduleName}.rs`);
    if (!fs.existsSync(filePath)) continue;
    const body = fs.readFileSync(filePath, 'utf8');
    for (const match of body.matchAll(/(?:super|crate::routes)::([a-zA-Z0-9_]+)::(?:router|routes)\(/g)) {
      queue.push(match[1]);
    }
  }
  const entries = [];
  for (const file of fs.readdirSync(routesDir).filter((name) => name.endsWith('.rs')).sort()) {
    const moduleName = file.replace(/\.rs$/, '');
    if (!reachable.has(moduleName)) continue;
    const text = fs.readFileSync(path.join(routesDir, file), 'utf8');
    for (const match of text.matchAll(/\.route\(\s*"([^"]+)"\s*,\s*(get|post|patch|delete|put)\(/g)) {
      const route = match[1];
      const method = match[2].toUpperCase();
      if (route.startsWith('/v1/')) entries.push({ file, method, route });
    }
  }
  return entries;
}

for (const [file, needle] of checks) {
  const full = path.join(root, file);
  if (!fs.existsSync(full)) {
    failures.push(`${file}: missing file`);
    continue;
  }
  const text = fs.readFileSync(full, 'utf8');
  if (!text.includes(needle)) failures.push(`${file}: missing ${needle}`);
}
const apiRefPath = path.join(root, 'docs/current/API_REFERENCE_CURRENT.md');
if (fs.existsSync(apiRefPath)) {
  const apiRef = fs.readFileSync(apiRefPath, 'utf8');
  for (const entry of methodRouteInventory()) {
    if (!apiRef.includes(`${entry.method} ${entry.route}`)) {
      failures.push(`docs/current/API_REFERENCE_CURRENT.md: missing registered route ${entry.method} ${entry.route} (${entry.file})`);
    }
  }
}

if (failures.length) {
  console.error('Docs/runtime parity validation: failed');
  for (const failure of failures) console.error(`FAIL ${failure}`);
  process.exit(1);
}
console.log('Docs/runtime parity validation: passed');
console.log('claims=Spec92/Spec93 awareness, CLI/API refs, route inventory parity, OpenClaw plugin, proof scripts');
