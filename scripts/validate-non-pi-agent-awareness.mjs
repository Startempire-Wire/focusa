#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const files = [
  'docs/93-non-pi-agent-focusa-awareness-spec.md',
  'docs/current/NON_PI_AGENT_FOCUSA_USAGE.md',
  'docs/README.md',
  'README.md',
];
const failures = [];
for (const file of files) {
  const full = path.join(root, file);
  if (!fs.existsSync(full)) failures.push(`${file}: missing`);
}
const combined = files.filter((file) => fs.existsSync(path.join(root, file))).map((file) => fs.readFileSync(path.join(root, file), 'utf8')).join('\n');
for (const needle of [
  'OpenClaw',
  'oprnclaw',
  'Wirebot',
  'focusa_workpoint_resume',
  'focusa_workpoint_checkpoint',
  'focusa_tool_doctor',
  'focusa_evidence_capture',
  'focusa_predict_record',
  'focusa_predict_evaluate',
  'degraded',
  'Operator steering',
  '/v1/workpoint/resume',
]) {
  if (!combined.includes(needle)) failures.push(`missing required awareness term: ${needle}`);
}
if (failures.length) {
  console.error('Non-Pi agent awareness validation: failed');
  for (const failure of failures) console.error(`FAIL ${failure}`);
  process.exit(1);
}
console.log('Non-Pi agent awareness validation: passed');
console.log('targets=OpenClaw/oprnclaw,Wirebot,Claude Code,OpenCode,Letta');
