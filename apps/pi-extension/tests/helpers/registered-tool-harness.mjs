import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';
import { Type } from '@sinclair/typebox';

const source = readFileSync(new URL('../../src/tools.ts', import.meta.url), 'utf8');
export function loadPureTypeScript(url) {
  const code = ts.transpileModule(readFileSync(url, 'utf8'), {
    compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS },
  }).outputText;
  const exports = {};
  new Function('exports', code)(exports);
  return exports;
}

export function loadRegisteredTool(name, dependencies, helperSections = []) {
  const anchor = source.indexOf(`name: "${name}"`);
  const start = source.lastIndexOf('pi.registerTool({', anchor);
  const end = source.indexOf('pi.registerTool({', anchor);
  assert.ok(anchor >= 0 && start >= 0 && end > anchor, `Missing bounded registration: ${name}`);
  const helpers = helperSections.map(([first, last]) => {
    const i = source.indexOf(first), j = source.indexOf(last, i + first.length);
    assert.ok(i >= 0 && j > i, `Missing helper section: ${first}`);
    return source.slice(i, j);
  }).join('\n');
  const code = ts.transpileModule(helpers + source.slice(start, end), {
    compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None },
  }).outputText;
  let tool;
  const deps = { ...dependencies, Type, pi: { registerTool(value) { tool = value; } } };
  new Function(...Object.keys(deps), code)(...Object.values(deps));
  assert.equal(tool.name, name);
  return tool;
}
