#!/usr/bin/env node
import fs from 'node:fs';

const configPath = process.env.OPENCLAW_CONFIG_PATH || '/data/wirebot/users/verious/openclaw.json';
const data = JSON.parse(fs.readFileSync(configPath, 'utf8'));
const plugins = data.plugins || {};
const failures = [];
const pluginPath = '/home/wirebot/focusa/apps/openclaw-focusa-awareness';
if (!plugins.allow?.includes?.('focusa-awareness')) failures.push('plugins.allow missing focusa-awareness');
if (!plugins.load?.paths?.includes?.(pluginPath)) failures.push(`plugins.load.paths missing ${pluginPath}`);
const entry = plugins.entries?.['focusa-awareness'];
if (!entry?.enabled) failures.push('plugins.entries.focusa-awareness.enabled is not true');
for (const [key, expected] of Object.entries({ adapterId: 'openclaw', workspaceId: 'wirebot', agentId: 'wirebot', operatorId: 'verious.smith' })) {
  if (entry?.config?.[key] !== expected) failures.push(`focusa-awareness config ${key} != ${expected}`);
}
if (!String(entry?.config?.focusaUrl || '').startsWith('http://127.0.0.1:8787')) failures.push('focusaUrl must point at local Focusa daemon');
if (failures.length) {
  console.error('OpenClaw Focusa awareness config validation: failed');
  for (const failure of failures) console.error(`FAIL ${failure}`);
  process.exit(1);
}
console.log('OpenClaw Focusa awareness config validation: passed');
console.log(`config=${configPath} plugin=focusa-awareness path=${pluginPath}`);
