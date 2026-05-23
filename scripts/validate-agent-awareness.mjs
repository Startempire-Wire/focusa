#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const files = {
  awareness: 'apps/pi-extension/src/awareness.ts',
  turns: 'apps/pi-extension/src/turns.ts',
  quickstart: 'docs/current/AGENT_AWARENESS_QUICKSTART.md',
  card: 'docs/current/FOCUSA_AGENT_UTILITY_CARD.md',
  onboarding: 'docs/current/FOCUSA_FRIENDLY_ONBOARDING.md',
  choreography: 'docs/current/FOCUSA_TOOL_CHOREOGRAPHY_MAP.md',
  docsIndex: 'docs/README.md',
  readme: 'README.md',
};
const text = Object.fromEntries(Object.entries(files).map(([k, p]) => [k, fs.readFileSync(path.join(root, p), 'utf8')]));
const failures = [];
function must(file, needle) {
  if (!text[file].includes(needle)) failures.push(`${files[file]} missing ${needle}`);
}

for (const needle of [
  'Focusa Utility Card',
  'focusa_tool_doctor',
  'focusa_workpoint_checkpoint',
  'focusa_workpoint_resume',
  'focusa_evidence_capture',
  'focusa_workpoint_link_evidence',
  'focusa_predict_record',
  'focusa_predict_evaluate',
  'focusa_metacog_',
  'focusa_work_loop_',
  'Operator steering always wins',
]) must('awareness', needle);

must('turns', 'buildFocusaUtilityCard("system")');
must('turns', 'customType: "focusa-utility-card"');
must('turns', 'S.seenFirstBeforeAgentStart');

for (const file of ['quickstart', 'card']) {
  for (const needle of ['Workpoint', 'doctor', 'evidence', 'prediction', 'compaction', 'operator']) must(file, needle);
}
for (const needle of ['Friendly Focusa Q', 'project_root', 'architecture', 'trajectory', 'Workpoint', 'evidence', 'predict', 'metacog']) must('onboarding', needle);
for (const needle of ['Route graph', 'project_identity', 'trajectory_view', 'workpoint_checkpoint', 'evidence_capture', 'predict_record', 'metacog']) must('choreography', needle);
must('docsIndex', 'AGENT_AWARENESS_QUICKSTART.md');
must('docsIndex', 'FOCUSA_AGENT_UTILITY_CARD.md');
must('docsIndex', 'FOCUSA_FRIENDLY_ONBOARDING.md');
must('docsIndex', 'FOCUSA_TOOL_CHOREOGRAPHY_MAP.md');
must('readme', 'Agent Awareness Quickstart');
must('readme', 'Friendly Focusa Onboarding Q');
must('readme', 'Focusa Tool Choreography Map');

if (failures.length) {
  console.error('Agent awareness validation: failed');
  for (const failure of failures) console.error(`FAIL ${failure}`);
  process.exit(1);
}
console.log('Agent awareness validation: passed');
console.log('surfaces=runtime_card,system_prompt,visible_startup_card,quickstart,utility_card_docs');
