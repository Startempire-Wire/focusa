#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { loadSkillsFromDir } from '/opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/dist/core/skills.js';

const checks = [
  '/root/.pi/skills',
  '/home/wirebot/focusa/apps/pi-extension/skills',
  '/root/apps/pi-extension/skills',
];
const failures = [];
for (const dir of checks) {
  if (!fs.existsSync(dir)) failures.push(`${dir}: missing`);
  const r = loadSkillsFromDir({ dir, source: 'user' });
  for (const d of r.diagnostics) failures.push(`${dir}: ${d.message || JSON.stringify(d)}`);
  if (dir === '/root/apps/pi-extension/skills' && r.skills.length !== 0) failures.push(`${dir}: stale compatibility dir must be empty to avoid duplicate skill collisions`);
}
const predictive = path.join('/home/wirebot/focusa/apps/pi-extension/skills/predictive-power/SKILL.md');
const text = fs.readFileSync(predictive, 'utf8');
for (const needle of ['---', 'name: predictive-power', 'description:', 'focusa_predict_record', 'focusa_predict_evaluate']) {
  if (!text.includes(needle)) failures.push(`predictive-power missing ${needle}`);
}
if (failures.length) {
  console.error('Skill hygiene validation: failed');
  for (const f of failures) console.error(`FAIL ${f}`);
  process.exit(1);
}
console.log('Skill hygiene validation: passed');
console.log('canonical=/root/.pi/skills; compatibility=/root/apps/pi-extension/skills empty');
