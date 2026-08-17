#!/usr/bin/env -S npx tsx
/** Generate Spec141 Agent Capability Descriptor V2 and cross-harness projections. */
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { registerTools } from "../apps/pi-extension/src/tools.ts";
import { registerAgentRuntimeTools } from "../apps/pi-extension/src/agent-runtime-tools.ts";
import {
  FOCUSA_TOOL_CONTRACTS,
  buildFocusaToolAffordanceCatalog,
  type FocusaToolAffordance,
  type FocusaToolContract,
} from "../apps/pi-extension/src/tool-contracts.ts";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const OUT = join(ROOT, "docs/contracts/spec141/generated-capability-v2");
const OPERATION_REGISTRY = JSON.parse(readFileSync(join(ROOT, "docs/contracts/spec135/generated-contract-v1/operation-registry.json"), "utf8"));
const OPERATION_BY_ROUTE = new Map(
  OPERATION_REGISTRY.operations.map((operation: any) => [
    `${operation.method} ${operation.path}`,
    operation,
  ]),
);
const CHECK = process.argv.includes("--check");
const WRITE = process.argv.includes("--write") || !CHECK;
const WORKSPACE_VERSION = (() => {
  const cargo = readFileSync(join(ROOT, "Cargo.toml"), "utf8");
  const workspacePackage = cargo.match(/\[workspace\.package\]([\s\S]*?)(?:\n\[|$)/)?.[1] ?? "";
  const version = workspacePackage.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  if (!version) throw new Error("workspace package version missing from Cargo.toml");
  return version;
})();

const OPERATOR_ALIGNMENT_CONTRACT = {
  schema: "focusa.operator_alignment.v1",
  authority: "operator_intent_constraints_and_confirmed_timeline_lead",
  requirements: [
    "refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps",
    "treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker",
    "consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation",
    "use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested",
    "never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range",
    "for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons",
    "use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation",
  ],
  project_reception: {
    candidate_scan_depth_min: 2,
    candidate_limit: 20,
    mutation_before_confirmation: false,
    guided_orientation_optional: true,
  },
};

interface RegisteredTool {
  name: string;
  label?: string;
  description?: string;
  promptSnippet?: string;
  promptGuidelines?: string[];
  parameters?: Record<string, unknown>;
  outputSchema?: Record<string, unknown>;
}

function stable(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(stable).join(",")}]`;
  if (value && typeof value === "object") {
    const record = value as Record<string, unknown>;
    return `{${Object.keys(record).sort().map((key) => `${JSON.stringify(key)}:${stable(record[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function digest(value: unknown): string {
  return `sha256:${createHash("sha256").update(stable(value)).digest("hex")}`;
}

function jsonSchema(value: unknown): Record<string, unknown> {
  return JSON.parse(JSON.stringify(value || { type: "object", properties: {}, additionalProperties: false }));
}

function routeKey(method: string, path: string): string {
  return `${method.toUpperCase()} ${path.split("?")[0]}`;
}

function policyProjection(contract: FocusaToolContract, route: { method: string; path: string }) {
  const canonical = OPERATION_BY_ROUTE.get(routeKey(route.method, route.path));
  const fallback = contract.operation_policy!;
  if (!canonical) {
    return {
      operation_id: null,
      method: route.method,
      path: route.path,
      ...fallback,
    };
  }
  return {
    operation_id: canonical.operation_id,
    method: canonical.method,
    path: canonical.path,
    operation_class: canonical.operation_class,
    capability_family: canonical.capability_family,
    commercial_treatment: canonical.commercial_treatment,
    policy_activation: canonical.policy_activation,
    required_feature: canonical.required_feature,
    limit_bucket: canonical.limit_bucket,
    recovery_allowance: canonical.recovery_allowance,
    source_owner: canonical.source_owner,
    policy_owner: canonical.policy_owner,
  };
}

function exampleFromSchema(schema: any, field = "value"): unknown {
  if (!schema || typeof schema !== "object") return null;
  if (schema.const !== undefined) return schema.const;
  if (Array.isArray(schema.enum) && schema.enum.length) return schema.enum[0];
  const variants = schema.anyOf || schema.oneOf;
  if (Array.isArray(variants) && variants.length) {
    const nonNull = variants.find((item: any) => item?.type !== "null") || variants[0];
    return exampleFromSchema(nonNull, field);
  }
  if (schema.default !== undefined) return schema.default;
  switch (schema.type) {
    case "object": {
      const result: Record<string, unknown> = {};
      for (const name of schema.required || []) {
        result[name] = exampleFromSchema(schema.properties?.[name] || {}, name);
      }
      return result;
    }
    case "array":
      return schema.minItems > 0 ? [exampleFromSchema(schema.items || {}, field)] : [];
    case "boolean":
      return false;
    case "integer":
    case "number":
      return schema.minimum ?? 0;
    case "string": {
      if (field.includes("project_root") || field.endsWith("path")) return "/tmp/focusa-project";
      if (field.includes("continuity")) return "continuity-demo";
      if (field.includes("url") || field === "origin") return "https://example.com";
      if (field.includes("query")) return "workpoint resume";
      if (field.includes("name")) return "focusa_workpoint_resume";
      const minimum = Math.max(1, Number(schema.minLength || 1));
      return "example".padEnd(minimum, "x");
    }
    default:
      return {};
  }
}

function strictObjects(schema: unknown, seen = new Set<unknown>()): void {
  if (!schema || typeof schema !== "object" || seen.has(schema)) return;
  seen.add(schema);
  const record = schema as Record<string, unknown>;
  if (record.type === "object" && record.additionalProperties === undefined) record.additionalProperties = false;
  for (const value of Object.values(record)) {
    if (Array.isArray(value)) value.forEach((entry) => strictObjects(entry, seen));
    else strictObjects(value, seen);
  }
}

function writeGenerated(name: string, value: unknown): void {
  const path = join(OUT, name);
  const body = name.endsWith(".md") ? String(value) : `${JSON.stringify(value, null, 2)}\n`;
  if (CHECK) {
    if (readFileSync(path, "utf8") !== body) throw new Error(`generated projection drift: ${relative(ROOT, path)}`);
  }
  if (WRITE) {
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, body);
  }
}

function annotations(contract: FocusaToolContract) {
  const profile = contract.side_effect_profile.toLowerCase();
  const readOnly = profile.startsWith("read") || profile === "none";
  const destructive = /delete|remove|revoke|kill|stop|rollback|restore/.test(`${contract.name} ${profile}`);
  const idempotent = readOnly || /idempoten|preview|verify|status|list|view|read|doctor|search|describe/.test(`${contract.name} ${contract.live_check}`);
  const openWorld = contract.family === "project_identity" || contract.name.includes("browser") || contract.name.includes("device");
  return { readOnlyHint: readOnly, destructiveHint: destructive, idempotentHint: idempotent, openWorldHint: openWorld };
}

function skillRefs(contract: FocusaToolContract): string[] {
  const refs = new Set(["skill:focusa"]);
  const byFamily: Record<string, string> = {
    workset: "skill:focusa-work-loop",
    callgraph: "skill:focusa-spec-implementation",
    credential: "skill:focusa-security-auth-licensing",
    cockpit: "skill:focusa-work-loop",
    background_job: "skill:focusa-silent-sessions",
    session_fanout: "skill:focusa-silent-sessions",
    workpoint: "skill:focusa-workpoint",
    work_loop: "skill:focusa-work-loop",
    focus_state: "skill:focusa-workpoint",
    metacognition: "skill:focusa-metacognition",
    tree_lineage: "skill:focusa-session-recovery",
    diagnostics_hygiene: "skill:focusa-troubleshooting",
    trajectory: "skill:focusa-workpoint",
    project_identity: "skill:focusa-project-scope",
    traversal: "skill:focusa-tool-discovery",
    session_transfer: "skill:focusa-session-recovery",
    awareness: "skill:focusa-agent-bootstrap",
    preload: "skill:focusa-agent-bootstrap",
    agent_runtime: "skill:focusa-spec-implementation",
  };
  if (byFamily[contract.family]) refs.add(byFamily[contract.family]);
  if (contract.name.includes("silent_sessions")) refs.add("skill:focusa-silent-sessions");
  if (contract.name.includes("browser")) refs.add("skill:focusa-browser-uiai");
  if (contract.name.includes("predict")) refs.add("skill:predictive-power");
  if (contract.name.includes("evidence") || contract.name.includes("outcome")) refs.add("skill:focusa-evidence-outcomes");
  if (contract.name.includes("bloatgaurd") || contract.name.includes("resource_mode")) refs.add("skill:focusa-resource-performance");
  if (contract.name.includes("context_cognition")) refs.add("skill:focusa-agent-bootstrap");
  if (contract.name.includes("device_pair")) refs.add("skill:focusa-security-auth-licensing");
  if (contract.name.includes("call_stack")) refs.add("skill:focusa-spec-implementation");
  if (contract.name.includes("instruction_integrity") || contract.name.includes("amendment")) {
    refs.add("skill:focusa-security-auth-licensing");
  }
  return [...refs];
}

const captured: RegisteredTool[] = [];
const pi = {
  registerTool(tool: RegisteredTool) { captured.push(tool); },
  on() {},
} as any;
registerTools(pi);
registerAgentRuntimeTools(pi);

const tools = new Map(captured.filter((tool) => tool.name.startsWith("focusa_")).map((tool) => [tool.name, tool]));
const affordances = new Map<string, FocusaToolAffordance>(buildFocusaToolAffordanceCatalog().map((item) => [item.name, item]));
const contracts = new Map(FOCUSA_TOOL_CONTRACTS.map((item) => [item.name, item]));

if (tools.size !== contracts.size) {
  const missingContracts = [...tools.keys()].filter((name) => !contracts.has(name));
  const staleContracts = [...contracts.keys()].filter((name) => !tools.has(name));
  throw new Error(
    `tool/contract count mismatch: ${tools.size}/${contracts.size}; missing=${missingContracts.join(",") || "none"}; stale=${staleContracts.join(",") || "none"}`
  );
}
for (const name of tools.keys()) if (!contracts.has(name)) throw new Error(`registered tool lacks contract: ${name}`);

const errorSchema = {
  type: "object",
  additionalProperties: false,
  required: ["failure_class", "retryable", "posture", "recovery"],
  properties: {
    failure_class: { type: ["string", "null"] },
    retryable: { type: "boolean" },
    posture: { type: "string", enum: ["safe_retry", "retry_with_idempotency_key", "check_side_effects_first", "do_not_retry_unchanged", "operator_required"] },
    field: { type: "string" },
    code: { type: "string" },
    message: { type: "string" },
    recovery: { type: "array", items: { type: "string" } },
  },
};

const descriptors = [...tools.values()].sort((a, b) => a.name.localeCompare(b.name)).map((tool) => {
  const contract = contracts.get(tool.name)!;
  const affordance = affordances.get(tool.name)!;
  const inputSchema = jsonSchema(tool.parameters);
  const outputSchema = jsonSchema(tool.outputSchema);
  strictObjects(inputSchema);
  strictObjects(outputSchema);
  const ann = annotations(contract);
  const routes = contract.api_routes.map((route) => {
    const [method, ...rest] = route.split(" ");
    return { method, path: rest.join(" ") };
  });
  const operation_policies = routes.map((route) => policyProjection(contract, route));
  const operation_policy = operation_policies[0] || {
    operation_id: null,
    method: null,
    path: null,
    ...contract.operation_policy!,
  };
  const descriptor = {
    schema: "focusa.agent_capability_descriptor.v2",
    operator_alignment: OPERATOR_ALIGNMENT_CONTRACT,
    capability_id: tool.name.replace(/^focusa_/, "focusa.").replaceAll("_", "."),
    tool_names: {
      pi: tool.name,
      mcp: tool.name.replace(/^focusa_/, "focusa.").replaceAll("_", "."),
      openai: tool.name,
      cli: contract.cli_commands,
      rest: routes,
    },
    version: "2.0.0",
    title: tool.label || contract.label,
    summary: contract.purpose,
    description: [tool.description || contract.purpose, `Use it when ${affordance.when_to_use[0] || contract.purpose}`, `It returns a typed Focusa result with bounded recovery and likely next capabilities.`].join(" "),
    family: contract.family,
    namespace: `focusa.${contract.family}`,
    operation_policy,
    operation_policies,
    operation_class: operation_policy.operation_class,
    capability_family: operation_policy.capability_family,
    commercial_treatment: operation_policy.commercial_treatment,
    policy_activation: operation_policy.policy_activation,
    required_feature: operation_policy.required_feature,
    limit_bucket: operation_policy.limit_bucket,
    recovery_allowance: operation_policy.recovery_allowance,
    source_owner: operation_policy.source_owner,
    policy_owner: operation_policy.policy_owner,
    availability: {
      requires_daemon: !contract.exemptions.includes("local_scratchpad_only") && !contract.exemptions.includes("pi_session_only"),
      supported_harnesses: ["pi", ...(contract.api_routes.length ? ["mcp", "openai", "rest"] : []), ...(contract.cli_commands.length ? ["cli"] : [])],
      parity_status: contract.parity_status,
      exemptions: contract.exemptions,
    },
    input_schema: inputSchema,
    output_schema: outputSchema,
    error_schema: errorSchema,
    result_envelope: "focusa.tool_result.v1",
    scope: contract.scope_requirement,
    authority: contract.authority_requirement,
    permissions: contract.api_routes.map((route) => ({ route, scope: contract.scope_requirement })),
    annotations: ann,
    side_effects: affordance.side_effects === "none" ? [] : [affordance.side_effects, contract.side_effect_profile],
    confirmation: { required: ann.destructiveHint, preview_supported: /preview|preflight|inspect|status|doctor/.test(tool.name) },
    idempotency: { supported: ann.idempotentHint, key_recommended: !ann.readOnlyHint && ann.idempotentHint },
    reversibility: { reversible: !ann.destructiveHint, compensation: ann.destructiveHint ? affordance.recovery : [] },
    cost_hint: { class: ann.openWorldHint ? "external_or_variable" : "local_bounded" },
    latency_hint: { class: /traverse|full|deep|browser/.test(tool.name) ? "cold_or_variable" : "hot_or_bounded" },
    token_budget: { discovery: "metadata_first", full_schema: "cold_load_on_selection" },
    examples: [{ description: affordance.when_to_use[0] || contract.purpose, arguments: exampleFromSchema(inputSchema), human_invocation: affordance.example, expected: affordance.expected_result }],
    anti_examples: affordance.when_not_to_use.map((description) => ({ description })),
    failure_classes: affordance.failure_classes,
    recovery: affordance.recovery,
    prerequisites: contract.scope_requirement.kind === "none" ? [] : ["verified project_root plus continuity_id when project-bound"],
    dependencies: affordance.likely_next_tools.map((name) => ({ capability: name, relation: "likely_next" })),
    likely_next_capabilities: affordance.likely_next_tools,
    skill_refs: skillRefs(contract),
    runbook_refs: [`runbook:${contract.family}`],
    docs_ref: contract.doc_path,
    spec_refs: contract.spec_path ? [contract.spec_path] : [],
    ontology: { action: contract.ontology_action, objects: contract.ontology_objects, core_surface: contract.core_surface },
    evidence_requirements: ann.readOnlyHint ? [] : ["stable evidence or receipt ref proving the mutation outcome"],
    deprecation: null,
    compatibility: { descriptor: "v2", result_envelope: "v1", minimum_focusa: "0.9.120-dev" },
    conformance_refs: [`spec141:${tool.name}`],
    prompt: { snippet: tool.promptSnippet || null, guidelines: tool.promptGuidelines || [] },
  };
  return { ...descriptor, descriptor_digest: digest(descriptor) };
});

const registryBase = {
  schema: "focusa.agent_capability_registry.v2",
  version: "2.0.0",
  source_authority: ["apps/pi-extension/src/tools.ts", "apps/pi-extension/src/tool-contracts.ts"],
  capability_count: descriptors.length,
  operator_alignment: OPERATOR_ALIGNMENT_CONTRACT,
  descriptors,
};
const registry = { ...registryBase, registry_digest: digest(registryBase) };

const mcp = {
  schema: "focusa.mcp_tool_projection.v2",
  registry_digest: registry.registry_digest,
  tools: descriptors.filter((d) => d.availability.supported_harnesses.includes("mcp")).map((d) => ({
    name: d.tool_names.mcp,
    title: d.title,
    description: d.description,
    inputSchema: d.input_schema,
    outputSchema: d.output_schema,
    annotations: d.annotations,
    _meta: { capability_id: d.capability_id, version: d.version, skill_refs: d.skill_refs, docs_ref: d.docs_ref, rest: d.tool_names.rest, authority: d.authority },
  })),
};

const openai = {
  schema: "focusa.openai_tool_projection.v2",
  registry_digest: registry.registry_digest,
  tools: descriptors.filter((d) => d.availability.supported_harnesses.includes("openai")).map((d) => ({
    type: "function",
    function: { name: d.tool_names.openai, description: d.description, strict: true, parameters: d.input_schema },
  })),
};

const piProjection = {
  schema: "focusa.pi_tool_projection.v2",
  registry_digest: registry.registry_digest,
  tools: descriptors.map((d) => ({ name: d.tool_names.pi, label: d.title, description: d.description, parameters: d.input_schema, outputSchema: d.output_schema, next_tools: d.likely_next_capabilities })),
};

const cli = {
  schema: "focusa.cli_command_projection.v2",
  registry_digest: registry.registry_digest,
  commands: descriptors.flatMap((d) => d.tool_names.cli.map((command) => ({ command, capability_id: d.capability_id, summary: d.summary, input_schema: d.input_schema, side_effects: d.side_effects, examples: d.examples, deprecation: d.deprecation }))),
};

const rest = {
  schema: "focusa.rest_agent_operation_projection.v2",
  registry_digest: registry.registry_digest,
  operations: descriptors.flatMap((d) => d.operation_policies.map((policy: any) => ({
    method: policy.method,
    path: policy.path,
    capability_id: d.capability_id,
    operation_id: d.tool_names.pi,
    canonical_operation_id: policy.operation_id,
    input_schema: d.input_schema,
    output_schema: d.output_schema,
    error_schema: d.error_schema,
    authority: d.authority,
    permissions: d.permissions,
    docs_ref: d.docs_ref,
    operation_class: policy.operation_class,
    capability_family: policy.capability_family,
    commercial_treatment: policy.commercial_treatment,
    policy_activation: policy.policy_activation,
    required_feature: policy.required_feature,
    limit_bucket: policy.limit_bucket,
    recovery_allowance: policy.recovery_allowance,
    source_owner: policy.source_owner,
    policy_owner: policy.policy_owner,
  }))),
};

const capabilitySkills = [...new Set(descriptors.flatMap((d) => d.skill_refs))].sort();
const skillManifests = readdirSync(join(ROOT, ".pi/skills"), { withFileTypes: true })
  .filter((entry) => entry.isDirectory() && existsSync(join(ROOT, ".pi/skills", entry.name, "SKILL.md")))
  .map((entry) => {
    const skillRef = `skill:${entry.name}`;
    const referencesDir = join(ROOT, ".pi/skills", entry.name, "references");
    const runbookRefs = existsSync(referencesDir)
      ? readdirSync(referencesDir)
          .filter((name) => name.endsWith(".md"))
          .sort()
          .map((name) => relative(ROOT, join(referencesDir, name)))
      : [];
    return {
      skill_ref: skillRef,
      manifest_ref: relative(ROOT, join(ROOT, ".pi/skills", entry.name, "SKILL.md")),
      packaged_manifest_ref: relative(ROOT, join(ROOT, "apps/pi-extension/skills", entry.name, "SKILL.md")),
      runbook_refs: runbookRefs,
    };
  })
  .sort((a, b) => a.skill_ref.localeCompare(b.skill_ref));
const runbookRefs = [...new Set(skillManifests.flatMap((skill) => skill.runbook_refs))].sort();
const piToolDocCount = readdirSync(join(ROOT, "docs/focusa-tools/tools")).filter((name) => /^focusa_.*\.md$/.test(name)).length;

const agentCardBase = {
  schema: "focusa.agent_card.v1",
  name: "Focusa",
  description: "Agent-first cognitive infrastructure with scoped Workpoints, Trajectory, evidence, recovery, browser interoperability, and cross-harness capability contracts.",
  version: WORKSPACE_VERSION,
  interfaces: ["pi", "mcp", "openai-functions", "cli", "rest"],
  capabilities: { streaming: true, durable_tasks: true, list_changed: true, progressive_discovery: true, structured_output: true },
  authentication: ["bearer", "device_pairing", "local_trusted"],
  operator_alignment: OPERATOR_ALIGNMENT_CONTRACT,
  registry_digest: registry.registry_digest,
  pi_tool_count: descriptors.length,
  pi_tool_docs_count: piToolDocCount,
  pi_tool_registry_path: "docs/contracts/spec141/generated-capability-v2/pi-tools.json",
  pi_tool_docs_path: "docs/focusa-tools/tools/",
  skill_count: skillManifests.length,
  skills: skillManifests.map((skill) => skill.skill_ref),
  skill_manifests: skillManifests,
  capability_skill_count: capabilitySkills.length,
  capability_skills: capabilitySkills,
  runbook_count: runbookRefs.length,
  runbook_refs: runbookRefs,
  capability_families: [...new Set(descriptors.map((d) => d.family))].sort(),
  extended_card_path: "/v1/agent/card",
};
const agentCard = { ...agentCardBase, card_digest: digest(agentCardBase) };

const markdown = [
  "# Spec141 Focusa Agent Capability Reference",
  "",
  `Registry digest: \`${registry.registry_digest}\``,
  "",
  "This file is generated. Use the descriptor registry for complete strict schemas and machine metadata.",
  "",
  "## Operator alignment contract",
  "",
  ...OPERATOR_ALIGNMENT_CONTRACT.requirements.map((requirement) => `- ${requirement}`),
  "",
  ...descriptors.flatMap((d) => [
    `## ${d.tool_names.pi}`,
    "",
    d.description,
    "",
    `- Capability: \`${d.capability_id}\``,
    `- Family: \`${d.family}\``,
    `- Side effects: ${d.side_effects.length ? d.side_effects.map((v) => `\`${v}\``).join(", ") : "none"}`,
    `- Skills: ${d.skill_refs.map((v) => `\`${v}\``).join(", ")}`,
    `- Dependencies/next: ${d.likely_next_capabilities.map((v) => `\`${v}\``).join(", ")}`,
    `- Documentation: \`${d.docs_ref}\``,
    "",
  ]),
].join("\n").trimEnd() + "\n";

writeGenerated("agent-capability-descriptors.json", registry);
writeGenerated("pi-tools.json", piProjection);
writeGenerated("mcp-tools.json", mcp);
writeGenerated("openai-tools.json", openai);
writeGenerated("cli-commands.json", cli);
writeGenerated("rest-agent-operations.json", rest);
writeGenerated("agent-card.json", agentCard);
writeGenerated("agent-capability-reference.md", markdown);

console.log(JSON.stringify({ status: "passed", mode: CHECK ? "check" : "write", capabilities: descriptors.length, registry_digest: registry.registry_digest, outputs: 8 }, null, 2));
