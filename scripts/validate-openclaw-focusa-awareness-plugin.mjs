#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const pluginDir = path.join(root, 'apps/focusa-awareness');
const manifest = JSON.parse(fs.readFileSync(path.join(pluginDir, 'openclaw.plugin.json'), 'utf8'));
const source = fs.readFileSync(path.join(pluginDir, 'index.ts'), 'utf8');
const failures = [];
function must(label, cond) { if (!cond) failures.push(label); }

must('manifest id focusa-awareness', manifest.id === 'focusa-awareness');
must('manifest config schema exists', manifest.configSchema?.type === 'object');
for (const key of ['focusaUrl', 'adapterId', 'workspaceId', 'agentId', 'operatorId', 'projectRoot', 'timeoutMs']) {
  must(`manifest config key ${key}`, Boolean(manifest.configSchema.properties?.[key]));
}
for (const needle of [
  'api.on("before_agent_start"',
  '/v1/awareness/card',
  'adapter_id',
  'workspace_id',
  'agent_id',
  'operator_id',
  'cognition_degraded=true',
  'Operator steering always wins',
  'prependContext',
]) {
  must(`source contains ${needle}`, source.includes(needle));
}

if (failures.length) {
  console.error('OpenClaw Focusa awareness plugin validation: failed');
  for (const f of failures) console.error(`FAIL ${f}`);
  process.exit(1);
}
console.log('OpenClaw Focusa awareness plugin validation: passed');
console.log('plugin=apps/focusa-awareness route=/v1/awareness/card hook=before_agent_start');
