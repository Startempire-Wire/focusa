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

must('native before_agent_start hook', /\bapi\.on\s*\(\s*["']before_agent_start["']/.test(source));
for (const needle of ['current execution admission unverified', 'Configured lookup inputs (not verified binding)', 'Fallback context is advisory', 'independently admitted work continues']) {
  must(`degraded authority boundary: ${needle}`, source.includes(needle));
}

if (failures.length) {
  console.error('OpenClaw Focusa awareness plugin validation: failed');
  for (const f of failures) console.error(`FAIL ${f}`);
  process.exit(1);
}
console.log('OpenClaw Focusa awareness plugin validation: passed');
console.log('plugin=apps/focusa-awareness route=/v1/awareness/card hook=before_agent_start');
