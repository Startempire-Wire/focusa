import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const helperSource = readFileSync(
  fileURLToPath(new URL("../src/tool-discovery-visible.ts", import.meta.url)),
  "utf8"
);
const compiled = ts.transpileModule(helperSource, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
}).outputText;
const { modelVisibleDiscoveryPayload } = await import(
  `data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`
);

const descriptor = {
  schema: "focusa.tool_description.v2",
  descriptor: {
    name: "focusa_tool_search",
    family: "capability_discovery",
    side_effect_profile: "read_only",
    docs_ref: "docs/focusa-tools/tools/focusa_tool_search.md",
    input_schema: { type: "object", required: ["query"] },
    output_schema: { type: "object", required: ["results"] },
  },
};
const visible = modelVisibleDiscoveryPayload("tool describe", descriptor, () => "unused");
for (const expected of [
  "focusa_tool_search",
  "capability_discovery",
  "read_only",
  "input_schema",
  '"query"',
  "output_schema",
  '"results"',
]) {
  assert.match(visible, new RegExp(expected));
}

let stored = "";
const clipped = modelVisibleDiscoveryPayload(
  "tool bundle",
  { tools: [{ schema: "x".repeat(200) }] },
  (_kind, body) => { stored = body; return "local-discovery-1"; },
  40
);
assert.match(clipped, /\[HANDLE:text:local-discovery-1\]/);
assert.match(clipped, /\/focusa-rehydrate local-discovery-1/);
assert.ok(stored.length > 40);

const tools = readFileSync(
  fileURLToPath(new URL("../src/tools.ts", import.meta.url)),
  "utf8"
);
for (const label of ["tool search", "tool describe", "tool graph", "tool bundle", "Focusa Agent Card"]) {
  const escaped = label.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  assert.match(tools, new RegExp(`modelVisibleDiscoveryPayload\\([^)]*${escaped}`, "s"));
}
assert.match(tools, /tool graph blocked → unknown tool or family/);
assert.match(tools, /tool bundle blocked → unknown family/);
assert.match(tools, /registry_digest: `sha256:/);
assert.match(tools, /runtimeVersion = String\(health\?\.version/);
assert.doesNotMatch(tools, /version: "0\.9\.120-dev"/);
console.log("model-visible progressive discovery runtime contract passed");
