#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const root = process.cwd();
const files = {
  awareness: 'apps/pi-extension/src/awareness.ts',
  turns: 'apps/pi-extension/src/turns.ts',
  quickstart: 'docs/current/AGENT_AWARENESS_QUICKSTART.md',
  card: 'docs/current/FOCUSA_AGENT_UTILITY_CARD.md',
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
must('docsIndex', 'AGENT_AWARENESS_QUICKSTART.md');
must('docsIndex', 'FOCUSA_AGENT_UTILITY_CARD.md');
must('readme', 'Agent Awareness Quickstart');

if (failures.length) {
  console.error('Agent awareness validation: failed');
  for (const failure of failures) console.error(`FAIL ${failure}`);
  process.exit(1);
}
console.log('Agent awareness validation: passed');
console.log('surfaces=runtime_card,system_prompt,visible_startup_card,quickstart,utility_card_docs');
