import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import ts from 'typescript';

// Execute an exact production section with only explicit boundary adapters.
export function loadSourceSection(file, first, last, dependencies, resultExpression, asynchronous = false) {
  const source = readFileSync(new URL(`../../src/${file}`, import.meta.url), 'utf8');
  const start = source.indexOf(first), end = source.indexOf(last, start + first.length);
  assert.ok(start >= 0 && end > start, `Missing source section: ${file}:${first}`);
  const compiled = ts.transpileModule(source.slice(start, end), {
    compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None },
  }).outputText;
  const scope = { exports: {}, ...dependencies };
  const Constructor = asynchronous ? Object.getPrototypeOf(async function () {}).constructor : Function;
  return new Constructor(...Object.keys(scope), `${compiled}\nreturn ${resultExpression};`)(
    ...Object.values(scope)
  );
}

// Execute a self-contained production metadata module with the existing compiler.
export function loadPureSourceModule(file) {
  const source = readFileSync(new URL(`../../src/${file}`, import.meta.url), 'utf8');
  const compiled = ts.transpileModule(source, {
    compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS },
  }).outputText;
  const exports = {};
  new Function('exports', compiled)(exports);
  return exports;
}
