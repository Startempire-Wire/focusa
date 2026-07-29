import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const source = readFileSync(
  fileURLToPath(new URL("../src/agent-runtime-tools.ts", import.meta.url)),
  "utf8"
);
const index = readFileSync(fileURLToPath(new URL("../src/index.ts", import.meta.url)), "utf8");
const contracts = readFileSync(
  fileURLToPath(new URL("../src/tool-contracts.ts", import.meta.url)),
  "utf8"
);

for (const tool of [
  "focusa_agent_runtime_effective",
  "focusa_instruction_sources",
  "focusa_instruction_conflicts",
  "focusa_instruction_explain",
  "focusa_instruction_simulate",
  "focusa_runtime_constitution_preview",
  "focusa_prompt_variant_preview",
  "focusa_prompt_variant_diff",
  "focusa_agent_artifact_preview",
  "focusa_agent_artifact_delivery",
  "focusa_agent_artifact_verify",
  "focusa_agent_runtime_doctor",
]) {
  assert.match(source, new RegExp(`name: "${tool}"`), `missing ${tool}`);
  assert.match(contracts, new RegExp(`"${tool}"`), `missing contract for ${tool}`);
}
assert.match(source, /operator_confirmed: p\.confirmed/);
assert.match(source, /\/agent-runtime\/delivery\/commit/);
assert.match(source, /never writes files/i);
assert.match(index, /registerAgentRuntimeTools\(pi\)/);
console.log("Spec140 Pi agent-runtime tool surface passed");
