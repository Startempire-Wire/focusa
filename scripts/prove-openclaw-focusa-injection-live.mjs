#!/usr/bin/env node
import fs from 'node:fs';

const logPath = process.env.OPENCLAW_GATEWAY_LOG || '/home/wirebot/logs/openclaw-gateway.log';
const log = fs.readFileSync(logPath, 'utf8');
const tail = log.slice(-250_000);
const required = [
  'focusa-awareness: active url=http://127.0.0.1:8787 workspace=wirebot',
  '[hooks] running before_agent_start',
  'focusa-awareness: injected card session=',
];
const missing = required.filter((needle) => !tail.includes(needle));
if (missing.length) {
  console.error('OpenClaw Focusa injection proof: failed');
  for (const needle of missing) console.error(`FAIL missing log evidence: ${needle}`);
  process.exit(1);
}
const injected = tail.match(/focusa-awareness: injected card session=([^\n\r]+)/g) || [];
console.log('OpenClaw Focusa injection proof: passed');
console.log(`log=${logPath}`);
console.log(`injections=${injected.length}`);
console.log(`latest=${injected[injected.length - 1]}`);
