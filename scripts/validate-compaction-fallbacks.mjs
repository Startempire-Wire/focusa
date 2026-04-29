#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const compaction = fs.readFileSync(path.join(root, 'apps/pi-extension/src/compaction.ts'), 'utf8');
const state = fs.readFileSync(path.join(root, 'apps/pi-extension/src/state.ts'), 'utf8');
const failures = [];

function requireIncludes(name, text, needle) {
  if (!text.includes(needle)) failures.push(`${name}: missing ${needle}`);
}

requireIncludes('compaction', compaction, 'buildCompactionFallbackSummary');
requireIncludes('compaction', compaction, 'Continuation anchored to mission');
requireIncludes('compaction', compaction, 'Resume scope is bound to project_root');
requireIncludes('compaction', compaction, 'No open questions recorded by Focusa or Workpoint.');
requireIncludes('compaction', compaction, 'Fallback summary hydrated from Workpoint, Focus State shadow, current ask, and session metadata.');
requireIncludes('state', state, "never emit bare 'none'");
requireIncludes('state', state, 'nearest related canonical source');
requireIncludes('state', state, 'Fallback Mission');
requireIncludes('state', state, 'Fallback Scope');

const forbidden = [
  'fs.intent || "none"',
  'fs.current_focus || fs.current_state || "none"',
  '|| "none"}`',
  '|| "none"}',
];
for (const needle of forbidden) {
  if (compaction.includes(needle)) failures.push(`compaction: forbidden fallback ${needle}`);
}

if (failures.length) {
  console.error('Compaction fallback validation: failed');
  for (const failure of failures) console.error(`FAIL ${failure}`);
  process.exit(1);
}
console.log('Compaction fallback validation: passed');
console.log('fallback_sources=workpoint,current_ask,frame,local_shadow,session_metadata');
