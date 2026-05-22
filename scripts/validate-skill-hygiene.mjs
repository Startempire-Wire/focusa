#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { execFileSync } from 'node:child_process';

async function importFirstExisting(candidates) {
  let lastError = null;
  for (const candidate of candidates.filter(Boolean)) {
    try {
      return await import(candidate.startsWith('/') ? pathToFileURL(candidate).href : candidate);
    } catch (err) {
      lastError = err;
    }
  }
  throw lastError || new Error('no skill loader candidates');
}

function globalNpmRoot() {
  try {
    return execFileSync('npm', ['root', '-g'], { encoding: 'utf8' }).trim();
  } catch {
    return '';
  }
}

const globalRoot = globalNpmRoot();
const packageCandidates = [
  process.env.PI_SKILLS_MODULE,
  '@mariozechner/pi-coding-agent/dist/core/skills.js',
  '@earendil-works/pi-coding-agent/dist/core/skills.js',
  globalRoot && path.join(globalRoot, '@mariozechner/pi-coding-agent/dist/core/skills.js'),
  globalRoot && path.join(globalRoot, '@earendil-works/pi-coding-agent/dist/core/skills.js'),
];
const { loadSkillsFromDir } = await importFirstExisting(packageCandidates);
const homeSkills = process.env.PI_SKILLS_DIR || (process.env.HOME ? path.join(process.env.HOME, '.pi/skills') : '');
const checks = [
  path.join(process.cwd(), 'apps/pi-extension/skills'),
  path.join(process.cwd(), '.pi/skills'),
  homeSkills,
].filter(Boolean).filter((dir, idx, arr) => arr.indexOf(dir) === idx);
const failures = [];
for (const dir of checks) {
  if (!fs.existsSync(dir)) {
    if (dir === path.join(process.cwd(), 'apps/pi-extension/skills')) failures.push(`${dir}: missing`);
    continue;
  }
  const r = loadSkillsFromDir({ dir, source: 'user' });
  for (const d of r.diagnostics) failures.push(`${dir}: ${d.message || JSON.stringify(d)}`);
}
const predictive = path.join(process.cwd(), 'apps/pi-extension/skills/predictive-power/SKILL.md');
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
console.log(`skill_dirs=${checks.join(',')}`);
