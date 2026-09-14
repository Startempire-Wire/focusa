import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const read = (path) => readFileSync(new URL(path, import.meta.url), "utf8");
const contracts = read("../src/tool-contracts.ts");
const registry = JSON.parse(read("../../../docs/current/focusa-tool-contracts.json"));
const generatedReference = read(
  "../../../docs/contracts/spec141/generated-capability-v2/agent-capability-reference.md",
);
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

  const registryContract = registry.contracts.find((item) => item.name === name);
  assert.ok(registryContract, `missing canonical JSON contract for ${name}`);
  assert.match(registryContract.purpose, purposePattern);
  assert.doesNotMatch(registryContract.purpose, /pi-scratch|Scratchpad/);
  if (name === "focusa_workset_projection") {
    const route = "/v1/worksets/{workset_id}/projection";
    assert.deepEqual(registryContract.api_routes, [route]);
    assert.ok(block.includes(JSON.stringify(route)), "Pi contract must name the projection route");
    const routes = read("../../../crates/focusa-api/src/routes/worksets.rs");
    assert.ok(routes.includes(`.route("${route}", get(get_projection))`));
  }

  const heading = generatedReference.indexOf(`## ${name}`);
  assert.notEqual(heading, -1, `missing generated Markdown for ${name}`);
  const nextHeading = generatedReference.indexOf("\n## ", heading + 1);
  const generatedBlock = generatedReference.slice(
    heading,
    nextHeading === -1 ? generatedReference.length : nextHeading,
  );
  assert.match(generatedBlock, purposePattern);
  assert.doesNotMatch(generatedBlock, /pi-scratch|Scratchpad/);
}

console.log("Issue #607 source, canonical registry, and generated-doc purpose isolation passed");
