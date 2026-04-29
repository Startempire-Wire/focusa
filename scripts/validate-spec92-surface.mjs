#!/usr/bin/env node
import { readFileSync, existsSync } from 'node:fs';

const requiredFiles = [
  'docs/current/AGENT_COMMAND_COOKBOOK.md',
  'docs/current/HOOK_COVERAGE.md',
  'docs/current/EFFICIENCY_GUIDE.md',
  'docs/current/PREDICTIVE_POWER_GUIDE.md',
  'docs/current/MAC_APP_MISSION_CONTROL.md',
  'docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md',
  'docs/current/DAEMON_RESILIENCE.md',
  'docs/current/ERROR_EMPTY_STATES.md',
  'crates/focusa-cli/src/commands/doctor.rs',
  'crates/focusa-cli/src/commands/continue_work.rs',
  'crates/focusa-cli/src/commands/release.rs',
  'crates/focusa-cli/src/commands/cleanup.rs',
  'crates/focusa-cli/src/commands/predict.rs',
  'crates/focusa-api/src/routes/predictions.rs',
  'apps/menubar/src/lib/components/MissionControl.svelte',
];

const main = readFileSync('crates/focusa-cli/src/main.rs', 'utf8');
const apiServer = readFileSync('crates/focusa-api/src/server.rs', 'utf8');
const apiMod = readFileSync('crates/focusa-api/src/routes/mod.rs', 'utf8');
const page = readFileSync('apps/menubar/src/routes/+page.svelte', 'utf8');

const requiredNeedles = [
  ['CLI doctor', main, 'Commands::Doctor'],
  ['CLI continue', main, 'Commands::Continue'],
  ['CLI status agent', main, 'Status {'],
  ['CLI release', main, 'Commands::Release'],
  ['CLI cleanup', main, 'Commands::Cleanup'],
  ['CLI predict', main, 'Commands::Predict'],
  ['API predictions mod', apiMod, 'pub mod predictions;'],
  ['API predictions router', apiServer, 'routes::predictions::router()'],
  ['Mac mission tab', page, "activeTab === 'mission'"],
  ['Mac token endpoint', page, '/v1/telemetry/token-budget/status?limit=5'],
  ['Mac cache endpoint', page, '/v1/telemetry/cache-metadata/status?limit=5'],
];

const missingFiles = requiredFiles.filter((p) => !existsSync(p));
const missingNeedles = requiredNeedles.filter(([, haystack, needle]) => !haystack.includes(needle)).map(([name]) => name);

if (missingFiles.length || missingNeedles.length) {
  console.error('Spec92 surface validation failed');
  console.error(JSON.stringify({ missingFiles, missingNeedles }, null, 2));
  process.exit(1);
}

console.log('Spec92 surface validation: passed');
console.log(`files=${requiredFiles.length} checks=${requiredNeedles.length}`);
