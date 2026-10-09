import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';
import ts from 'typescript';

// Exercise the actual renderer, not a copy of its classification logic.
const source = ts.createSourceFile('tools.ts', readFileSync(new URL('../src/tools.ts', import.meta.url), 'utf8'), ts.ScriptTarget.Latest, true);
let declaration;
function visit(node) {
  if (ts.isFunctionDeclaration(node) && node.name?.text === 'explainWorkLoopResult') declaration = node;
  ts.forEachChild(node, visit);
}
visit(source);
assert.ok(declaration, 'runtime error renderer must remain discoverable');
const compiled = ts.transpileModule(declaration.getText(source), {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None },
}).outputText;
const explain = vm.runInNewContext(`${compiled}\nexplainWorkLoopResult`, {
  safeErrorText: value => typeof value === 'string' ? value : JSON.stringify(value),
  formatWorkLoopScope: () => 'fixture-scope',
});

const conflict = body => ({ ok: false, status: 409, body });

test('missing-stage refusal reports the actual defect rather than invented scope loss', () => {
  const rendered = explain(conflict({
    code: 'NORTH_STAR_ADMISSION_BLOCKED', failure_class: 'north_star_admission_blocked',
    workpoint_linkage: { admission_gaps: ['lifecycle_stage_missing'] },
  }), 'generic conflict');
  assert.match(rendered, /missing its stage/);
  assert.doesNotMatch(rendered, /scope mismatch|unknown/);
});

test('an HTTP conflict alone does not establish mismatched scope', () => {
  assert.doesNotMatch(explain(conflict({ error: 'state changed' }), 'generic conflict'), /scope mismatch/);
  assert.match(explain(conflict({ failure_class: 'scope_mismatch' }), 'generic conflict'), /scope mismatch/);
});

test('recovery rejection explains its required safeguards', () => {
  const rendered = explain(conflict({ failure_class: 'lifecycle_repair_rejected' }), 'generic conflict');
  assert.match(rendered, /confirmation.*unchanged work.*evidence/);
  assert.doesNotMatch(rendered, /scope mismatch/);
});
