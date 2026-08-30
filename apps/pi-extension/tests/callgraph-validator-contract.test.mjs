import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const tools = readFileSync(
  fileURLToPath(new URL("../src/tools.ts", import.meta.url)),
  "utf8"
);
const moduleRegistry = readFileSync(
  fileURLToPath(new URL("../../../crates/focusa-api/src/routes/mod.rs", import.meta.url)),
  "utf8"
);
const server = readFileSync(
  fileURLToPath(new URL("../../../crates/focusa-api/src/server.rs", import.meta.url)),
  "utf8"
);
const route = readFileSync(
  fileURLToPath(new URL("../../../crates/focusa-api/src/routes/callgraph.rs", import.meta.url)),
  "utf8"
);
const generatedCapabilities = JSON.parse(readFileSync(
  fileURLToPath(new URL("../../../docs/contracts/spec141/generated-capability-v2/agent-capability-descriptors.json", import.meta.url)),
  "utf8"
));
const entitlement = readFileSync(
  fileURLToPath(new URL("../../../crates/focusa-api/src/middleware/entitlement.rs", import.meta.url)),
  "utf8"
);
const toolDoc = readFileSync(
  fileURLToPath(new URL("../../../docs/focusa-tools/tools/focusa_callgraph_validate.md", import.meta.url)),
  "utf8"
);

assert.match(moduleRegistry, /pub mod callgraph;/);
assert.match(server, /\.merge\(routes::callgraph::router\(\)\)/);
assert.match(route, /\.route\("\/v1\/callgraphs\/validate", post\(validate\)\)/);
assert.match(entitlement, /method == Method::POST && path == "\/v1\/callgraphs\/validate"/);

const toolStart = tools.indexOf('name: "focusa_callgraph_validate"');
const toolEnd = tools.indexOf("pi.registerTool({", toolStart + 1);
assert.ok(toolStart >= 0 && toolEnd > toolStart, "CallGraph validator tool block exists");
const validatorTool = tools.slice(toolStart, toolEnd);

assert.match(validatorTool, /`\$\{base\}\/callgraphs\/validate`/);
assert.doesNotMatch(validatorTool, /`\$\{base\}\/v1\/callgraphs\/validate`/);
assert.match(validatorTool, /timeoutBudgetForRoute\("\/callgraphs\/validate", "POST"\)/);
assert.match(validatorTool, /signal: controller\.signal/);
assert.match(validatorTool, /callgraph_validation_transport_failed/);
assert.match(validatorTool, /callgraph_validation_http_error/);
assert.match(validatorTool, /callgraph_validation_protocol_invalid/);
assert.match(validatorTool, /typeof body\?\.valid !== "boolean"/);
assert.match(validatorTool, /Array\.isArray\(body\?\.issues\)/);
assert.match(validatorTool, /body\.issues\.map/);

const descriptor = generatedCapabilities.descriptors.find(
  (candidate) => candidate.tool_names?.pi === "focusa_callgraph_validate"
);
assert.ok(descriptor, "generated CallGraph validator descriptor exists");
assert.match(descriptor.description, /validating a CallGraph definition deterministically/);
assert.doesNotMatch(descriptor.description, /scratch|working notes|focusa_decide/i);
assert.deepEqual(descriptor.tool_names.rest, [
  { method: "POST", path: "/v1/callgraphs/validate" },
]);
assert.match(toolDoc, /validating a CallGraph definition deterministically/);
assert.doesNotMatch(toolDoc, /scratch|working notes|focusa_decide/i);

console.log("CallGraph validator producer-consumer route contract passed");
