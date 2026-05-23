#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';

const root = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..');
const sourcePath = path.join(root, 'apps/pi-extension/src/compaction.ts');
const source = fs.readFileSync(sourcePath, 'utf8');
const match = source.match(/export function contextPressureWarningCopy[\s\S]*?\n}\n\nexport function contextTierLabel/);
if (!match) throw new Error('contextPressureWarningCopy export not found');
const fnSource = match[0]
  .replace(/\n\nexport function contextTierLabel[\s\S]*$/, '')
  .replace(/export function contextPressureWarningCopy\([^)]*\)/, 'function contextPressureWarningCopy(kind, pct, totalCompactions = 48)')
  .replace(/\): string/g, ')');
const sandbox = { Number, result: null };
vm.createContext(sandbox);
vm.runInContext(`${fnSource}\nresult = {\n  auto: contextPressureWarningCopy('auto_suggest', 93),\n  hard: contextPressureWarningCopy('hard_unconfirmed', 93),\n  handoff: contextPressureWarningCopy('handoff_unconfirmed', 93, 48),\n};`, sandbox);
const expected = {
  auto: '💡 Context at 93% — Focusa anchors are unconfirmed; checkpoint/resume Workpoint, /fork optional for UI isolation',
  hard: '⚠️ Context 93% — Focusa will try checkpointed compaction; scoped Workpoint anchor not yet confirmed',
  handoff: '💡 48 compactions with unconfirmed Workpoint anchor — resume/checkpoint Workpoint; handoff optional',
};
for (const [key, value] of Object.entries(expected)) {
  if (sandbox.result?.[key] !== value) {
    console.error(`✗ FAIL: ${key} warning mismatch`);
    console.error('expected:', value);
    console.error('actual:  ', sandbox.result?.[key]);
    process.exit(1);
  }
}
if (/Focusa continuity degraded|consider \/fork to preserve context quality|consider \/fork or \/new before fallback/.test(source)) {
  console.error('✗ FAIL: stale degrading warning text present');
  process.exit(1);
}
console.log('SPEC96 context-pressure warning runtime copy test: PASS');
