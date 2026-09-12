#!/usr/bin/env node
// #600 regression: noun-phrase ownership statements must validate; matched
// pattern surfaced in rejection reasons for deterministic wording adjustment.
// Transpiles the self-contained decision-validation module so the test does
// not load the full tool graph (background-job-tools.test.mjs pattern).
import assert from "node:assert/strict";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import ts from "typescript";

const moduleSourcePath = fileURLToPath(new URL("../src/decision-validation.ts", import.meta.url));
const compiledModulePath = join(tmpdir(), `focusa-decision-validation-${process.pid}.mjs`);
const compiledModule = ts.transpileModule(readFileSync(moduleSourcePath, "utf8"), {
  compilerOptions: {
    module: ts.ModuleKind.ES2022,
    target: ts.ScriptTarget.ES2022,
  },
});
writeFileSync(compiledModulePath, compiledModule.outputText);
process.on("exit", () => rmSync(compiledModulePath, { force: true }));

const { validateDecision } = await import(pathToFileURL(compiledModulePath));

const ownership = [
  "Build agents own engineering-stage progression within the authorized outcome; advancing stages never expands authority.",
  "Engineering-stage progression belongs to the build agent, while the operator's authorized outcome and destination remain its boundary.",
];
for (const decision of ownership) {
  assert.equal(decision.length <= 160, true, "fixture must respect the 160-char limit");
  const result = validateDecision(decision);
  assert.equal(result.valid, true, `ownership statement rejected: ${JSON.stringify(result)}`);
}

const task = validateDecision("Fix all the activation bugs across the installer flow");
assert.equal(task.valid, false);
assert.match(task.reason, /matched: Fix all/);

const debug = validateDecision("Use cached error state for retries");
assert.equal(debug.valid, false);
assert.match(debug.reason, /debugging metadata/);

const stillBlocked = validateDecision("Build the release artifacts now");
assert.equal(stillBlocked.valid, false, "imperative Build must stay rejected");

console.log("#600 decision-validator regressions: PASS");