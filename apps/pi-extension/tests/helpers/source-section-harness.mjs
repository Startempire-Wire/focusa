import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import * as nodeModule from 'node:module';

// Keep bounded-section execution in this existing owner. Native type erasure
// does not invoke a compiler or lower unsupported TypeScript value syntax.
export function eraseTypeScriptSection(source) {
  assert.equal(typeof nodeModule.stripTypeScriptTypes, 'function', 'Node native type erasure is required');
  const erased = nodeModule.stripTypeScriptTypes(source, { mode: 'strip' });
  const names = [];
  const script = erased.replace(/^([ \t]*)export\s+((?:(?:async\s+)?function\s*\*?\s*|class\s+|(?:const|let|var)\s+)([\w$]+))/gm,
    (_match, indent, declaration, name) => { names.push(name); return indent + declaration; });
  assert.ok(!/^\s*(?:import|export)\s/m.test(script), 'Unsupported module boundary in production section');
  return script + '\n' + [...new Set(names)].map(name => `exports.${name} = ${name};`).join('\n');
}

// Execute an exact production section with only explicit boundary adapters.
export function loadSourceSection(file, first, last, dependencies, resultExpression, asynchronous = false) {
  const source = readFileSync(new URL(`../../src/${file}`, import.meta.url), 'utf8');
  const start = source.indexOf(first), end = source.indexOf(last, start + first.length);
  assert.ok(start >= 0 && end > start, `Missing source section: ${file}:${first}`);
  const compiled = eraseTypeScriptSection(source.slice(start, end));
  const scope = { exports: {}, ...dependencies };
  const Constructor = asynchronous ? Object.getPrototypeOf(async function () {}).constructor : Function;
  return new Constructor(...Object.keys(scope), `${compiled}\nreturn ${resultExpression};`)(
    ...Object.values(scope)
  );
}

// Execute a self-contained production metadata module with the existing compiler.
export function loadPureSourceModule(file) {
  // Full-module CommonJS conversion remains an explicit compiler consumer;
  // bounded-section tests never load or execute this unrelated path.
  const ts = nodeModule.createRequire(import.meta.url)('typescript');
  const source = readFileSync(new URL(`../../src/${file}`, import.meta.url), 'utf8');
  const compiled = ts.transpileModule(source, {
    compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS },
  }).outputText;
  const exports = {};
  new Function('exports', compiled)(exports);
  return exports;
}
