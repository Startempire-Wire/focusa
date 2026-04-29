#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const checks = [
  ['README.md', 'v0.9.11-dev'],
  ['README.md', 'Agent Awareness Quickstart'],
  ['README.md', 'Non-Pi Agent Focusa Usage'],
  ['docs/README.md', 'NON_PI_AGENT_FOCUSA_USAGE.md'],
  ['docs/README.md', '93-non-pi-agent-focusa-awareness-spec.md'],
  ['docs/current/API_REFERENCE_CURRENT.md', 'GET /v1/awareness/card'],
  ['docs/current/CLI_REFERENCE_CURRENT.md', 'awareness      Non-Pi agent awareness utility cards'],
  ['docs/current/CLI_REFERENCE_CURRENT.md', 'focusa awareness card'],
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
for (const [file, needle] of checks) {
  const full = path.join(root, file);
  if (!fs.existsSync(full)) {
    failures.push(`${file}: missing file`);
    continue;
  }
  const text = fs.readFileSync(full, 'utf8');
  if (!text.includes(needle)) failures.push(`${file}: missing ${needle}`);
}
if (failures.length) {
  console.error('Docs/runtime parity validation: failed');
  for (const failure of failures) console.error(`FAIL ${failure}`);
  process.exit(1);
}
console.log('Docs/runtime parity validation: passed');
console.log('claims=Spec92/Spec93 awareness, CLI/API refs, OpenClaw plugin, proof scripts');
