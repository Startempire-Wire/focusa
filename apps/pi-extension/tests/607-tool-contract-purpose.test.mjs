import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const contracts = readFileSync(new URL("../src/tool-contracts.ts", import.meta.url), "utf8");
const scratchPurpose = "Write working notes to /tmp/pi-scratch/";

assert.equal(
  contracts.split(scratchPurpose).length - 1,
  1,
  "only focusa_scratch may carry the Scratchpad purpose text"
);

for (const [name, purposePattern] of [
  [
    "focusa_workset_projection",
    /deterministic membership, requirement-disposition, and settlement projection/,
  ],
  [
    "focusa_callgraph_observe",
    /CallGraph run's ledger row, dispatches, paths, and deterministic replay frontier/,
  ],
  ["focusa_credentials_verify", /Credential Authority.*without exposing secret values/],
  ["focusa_cockpit_projection", /Worksets, CallGraph frontiers, direction steers, and background jobs/],
  ["focusa_fast_forward", /deterministic fanout plan.*silent-session lanes/],
]) {
  const start = contracts.indexOf(`name: "${name}"`);
  assert.notEqual(start, -1, `missing contract for ${name}`);
  const next = contracts.indexOf("\n  {", start + 1);
  const block = contracts.slice(start, next === -1 ? contracts.length : next);
  assert.match(block, purposePattern, `${name} must describe its own capability`);
  assert.doesNotMatch(block, /pi-scratch|Scratchpad/, `${name} must not inherit Scratchpad guidance`);
}

console.log("Issue #607 tool-contract purpose isolation passed");
