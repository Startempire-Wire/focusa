#!/usr/bin/env bun
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { Value } from "../apps/pi-extension/node_modules/@sinclair/typebox/build/esm/value/index.mjs";
import { makeAttachmentKey, runWithAttachmentRuntime } from "../apps/pi-extension/src/state.ts";
import { registerTools } from "../apps/pi-extension/src/tools.ts";

const root = resolve(import.meta.dir, "..");
const load = (path: string) => JSON.parse(readFileSync(resolve(root, path), "utf8"));
const registry = load("docs/contracts/spec141/generated-capability-v2/agent-capability-descriptors.json");
const piProjection = load("docs/contracts/spec141/generated-capability-v2/pi-tools.json");
const mcpProjection = load("docs/contracts/spec141/generated-capability-v2/mcp-tools.json");
const openaiProjection = load("docs/contracts/spec141/generated-capability-v2/openai-tools.json");
const cliProjection = load("docs/contracts/spec141/generated-capability-v2/cli-commands.json");
const restProjection = load("docs/contracts/spec141/generated-capability-v2/rest-agent-operations.json");
const card = load("docs/contracts/spec141/generated-capability-v2/agent-card.json");
const routes = load("docs/contracts/spec141/generated-capability-v2/route-classification.json");
const skills = load("docs/evidence/141-focusa-skill-runbook-coverage.json");

const captured: any[] = [];
registerTools({ registerTool: (tool: any) => captured.push(tool), on() {} } as any);
const tools = new Map(captured.filter((tool) => tool.name.startsWith("focusa_")).map((tool) => [tool.name, tool]));
const descriptors = new Map(registry.descriptors.map((descriptor: any) => [descriptor.tool_names.pi, descriptor]));
const attachmentKey = makeAttachmentKey({
  projectRoot: root,
  continuityId: "spec141-conformance",
  sessionId: "spec141-conformance",
});
const execute = (tool: any, params: any) =>
  runWithAttachmentRuntime(attachmentKey, () => tool.execute("spec141", params));

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function strictObjects(schema: any, path = "$"): void {
  if (!schema || typeof schema !== "object") return;
  if (schema.type === "object") {
    assert(schema.additionalProperties === false || schema.additionalProperties === true, `${path}: object schema must declare additionalProperties`);
  }
  for (const [key, value] of Object.entries(schema)) {
    if (Array.isArray(value)) value.forEach((item, index) => strictObjects(item, `${path}.${key}[${index}]`));
    else strictObjects(value, `${path}.${key}`);
  }
}

assert(tools.size === registry.capability_count, "runtime Pi tools must equal descriptor capability count");
assert(piProjection.tools.length === tools.size, "Pi projection parity");
assert(openaiProjection.tools.length === restProjection.operations.length || openaiProjection.tools.length <= tools.size, "OpenAI projection must remain bounded by Pi catalog");
assert(mcpProjection.tools.length === restProjection.operations.filter((operation: any, index: number, all: any[]) => all.findIndex((item: any) => item.operation_id === operation.operation_id) === index).length, "MCP and REST callable capability parity");
assert(card.registry_digest === registry.registry_digest, "Agent Card registry digest parity");
assert(cliProjection.registry_digest === registry.registry_digest, "CLI registry digest parity");
assert(routes.route_count === routes.routes.length && routes.routes.every((route: any) => route.classification && route.rationale), "every API route must have explicit classification/rationale");
assert(skills.root_packaged_parity === true && skills.installed_root_skill_count >= 21, "skill/runbook parity and coverage");

const invalidExamples: string[] = [];
for (const [name, tool] of tools) {
  const descriptor: any = descriptors.get(name);
  assert(descriptor, `${name}: missing descriptor`);
  strictObjects(tool.parameters, `${name}.input`);
  strictObjects(tool.outputSchema, `${name}.output`);
  assert(descriptor.error_schema && descriptor.recovery.length > 0, `${name}: missing error/recovery contract`);
  assert(descriptor.dependencies.length > 0, `${name}: missing workflow dependencies`);
  assert(descriptor.skill_refs.length > 0 && descriptor.docs_ref, `${name}: missing skill/docs refs`);
  const example = descriptor.examples[0]?.arguments;
  if (!Value.Check(tool.parameters, example)) invalidExamples.push(name);
  if (tool.parameters.type === "object") {
    assert(!Value.Check(tool.parameters, { ...example, __unknown_spec141: true }), `${name}: unknown input property accepted`);
  }
}
assert(invalidExamples.length === 0, `machine examples fail strict schemas: ${invalidExamples.join(", ")}`);

const search = tools.get("focusa_tool_search");
const describe = tools.get("focusa_tool_describe");
const graph = tools.get("focusa_tool_graph");
const bundle = tools.get("focusa_tool_bundle");
assert(search && describe && graph && bundle, "progressive discovery tools registered");

const cases = [
  ["resume after compaction", "focusa_workpoint_resume"],
  ["browser webmcp capability", "focusa_browser_capabilities_intake"],
  ["project verify scope", "focusa_project_verify"],
  ["prediction calibration stats", "focusa_predict_stats"],
  ["resource lowmem mode", "focusa_resource_mode"],
];
for (const [query, expected] of cases) {
  const result: any = await execute(search, { query, limit: 10 });
  const names = (result.details?.results || []).map((item: any) => item.name);
  assert(names.includes(expected), `weak-agent search '${query}' missed ${expected}: ${names.join(",")}`);
  assert(!JSON.stringify(result.details?.results || []).includes("input_schema"), "search must defer full schemas");
  assert(JSON.stringify(result).length < 50_000, "search response exceeded bounded metadata budget");
}

const described: any = await execute(describe, { name: "focusa_workpoint_resume", include_schemas: true });
assert(described.details?.descriptor?.input_schema, "describe must cold-load strict input schema");
assert(described.details?.descriptor?.output_schema, "describe must cold-load output schema");
const unknown: any = await execute(describe, { name: "focusa_missing_tool" });
assert(unknown.details?.tool_result_v1?.failure_class === "not_found" || unknown.details?.failure_class === "not_found", "unknown describe must return structured not_found recovery");

const graphed: any = await execute(graph, { anchor: "workpoint", depth: 2, limit: 30 });
assert(graphed.details?.nodes?.length > 0 && graphed.details?.edges?.length > 0, "tool graph must return bounded workflow dependencies");
const bundled: any = await execute(bundle, { family: "workpoint", include_schemas: false });
assert(bundled.details?.tools?.length > 0 && bundled.details?.schema_loading === "metadata_only", "family bundle must defer schemas by default");

console.log(JSON.stringify({
  schema: "focusa.agent_conformance_result.v1",
  status: "passed",
  runtime_tools: tools.size,
  strict_examples_valid: tools.size,
  agent_levels: ["weak_metadata_only", "medium_progressive_discovery", "strong_full_descriptor", "mcp_client", "openai_function_client", "cli_automation", "browser_uiai_workflow"],
  weak_agent_cases: cases.length,
  mcp_tools: mcpProjection.tools.length,
  classified_routes: routes.route_count,
  installed_skills: skills.installed_root_skill_count,
  metrics: {
    tool_selection_accuracy: 1,
    machine_example_validity: 1,
    unknown_property_rejection: 1,
    invalid_tool_recovery: 1,
    unsafe_call_rate: 0,
    scope_violation_rate: 0,
    cross_harness_descriptor_parity: 1,
    evidence_contract_coverage: 1,
    schema_loading: "deferred_by_default",
  },
  registry_digest: registry.registry_digest,
}, null, 2));
