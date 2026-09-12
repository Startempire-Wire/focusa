#!/usr/bin/env node
// #600 regression: noun-phrase ownership statements must validate; matched
// pattern surfaced in rejection reasons for deterministic wording adjustment.
import { strict as assert } from "node:assert";
import { validateDecision } from "../src/tools.ts";

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
