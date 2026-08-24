#!/usr/bin/env bun
/** Generate specific per-tool agent documentation from Spec141 descriptors. */
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import process from "node:process";

const root = process.cwd();
const registryPath = resolve(root, "docs/contracts/spec141/generated-capability-v2/agent-capability-descriptors.json");
const registry = JSON.parse(readFileSync(registryPath, "utf8"));
const check = process.argv.includes("--check");
let drift = 0;

function markdownText(value: string): string {
  return value.replace(/https?:\/\/[A-Za-z0-9._~:/?#\[\]@!$&()*+,;=%-]+/g, (url) => `<${url}>`);
}

function inline(value: unknown): string {
  if (value === null || value === undefined) return "none";
  if (typeof value === "string") return value;
  return JSON.stringify(value);
}

const operationalNotes: Record<string, string[]> = {
  focusa_evidence_capture: [
    "Trajectory-aware evidence supplies proof alignment metadata for the active trajectory and its HLT, MLG, and STG context without expanding the evidence payload.",
  ],
  focusa_traverse: [
    "Evidence, ECS references, and trajectory projections can be inspected in HLT/STG-aligned bounded slices without requesting full payloads.",
  ],
  focusa_metacog_capture: [
    "Captures retain HLT/MLG/STG alignment within the active project_root + continuity_id scope for trajectory-context retrieval.",
  ],
  focusa_project_identity: [
    "binding_candidates rank active worktree, canonical parent, marker/Beads, resumed-session, and bounded parent-directory evidence; ambiguous_project_binding fails closed.",
  ],
  focusa_project_verify: [
    "Verification projects the same binding_candidates and refuses canonical authority when the candidate decision is ambiguous.",
  ],
};

for (const descriptor of registry.descriptors) {
  const required = new Set(descriptor.input_schema?.required || []);
  const properties = Object.entries(descriptor.input_schema?.properties || {});
  const parameterLines = properties.length
    ? properties.map(([name, schema]: [string, any]) => {
        const variants = schema.anyOf || schema.oneOf;
        const type = schema.type || (variants ? variants.map((item: any) => item.type || item.const).filter(Boolean).join(" | ") : "structured");
        const requirement = required.has(name) ? "required" : "optional";
        const constraints = [schema.minimum !== undefined ? `min=${schema.minimum}` : null, schema.maximum !== undefined ? `max=${schema.maximum}` : null, schema.default !== undefined ? `default=${inline(schema.default)}` : null].filter(Boolean).join(", ");
        return `- \`${name}\` (${requirement}; ${type}${constraints ? `; ${constraints}` : ""}): ${markdownText(schema.description || "See the strict descriptor schema.")}`;
      })
    : ["- No arguments."];
  const example = descriptor.examples?.[0] || {};
  const argumentsExample = typeof example.arguments === "string" ? { invocation: example.arguments } : (example.arguments || {});
  const antiExamples = descriptor.anti_examples?.length
    ? descriptor.anti_examples.map((item: any) => `- ${item.description || inline(item)}`)
    : ["- Do not use this tool when a narrower read-only or preview capability satisfies the task."];
  const recovery = descriptor.recovery?.length
    ? descriptor.recovery.map((item: string) => `- ${item}`)
    : ["- Inspect the structured failure class and follow its exact recovery/next-tool guidance."];
  const dependencies = descriptor.dependencies?.length
    ? descriptor.dependencies.map((item: any) => `- \`${item.capability}\` (${item.relation})`)
    : ["- No declared tool dependency; operator steering and current Workpoint still govern execution."];
  const routes = descriptor.tool_names?.rest?.length
    ? descriptor.tool_names.rest.map((route: any) => `\`${route.method} ${route.path}\``).join(", ")
    : "Pi-local only";
  const lines = [
    `# \`${descriptor.tool_names.pi}\``,
    "",
    descriptor.description,
    "",
    "## When to use",
    "",
    `- ${example.description || descriptor.summary}`,
    `- Capability family: \`${descriptor.family}\`; namespace: \`${descriptor.namespace}\`.`,
    `- Load this full contract after metadata search when exact invocation or recovery semantics are needed.`,
    ...(operationalNotes[descriptor.tool_names.pi] || []).map((note) => `- ${note}`),
    "",
    "## Parameters and strict input schema",
    "",
    ...parameterLines,
    "",
    `Unknown object properties are rejected. Canonical schema: \`agent-capability-descriptors.json#${descriptor.tool_names.pi}\`.`,
    "",
    "## Output",
    "",
    `Result envelope: \`${descriptor.result_envelope}\`.`,
    `Returns the typed envelope with status, canonical/degraded posture, side effects, evidence refs, retry posture, recovery, and likely-next tools.`,
    "",
    "## Example",
    "",
    "```json",
    JSON.stringify(argumentsExample, null, 2),
    "```",
    "",
    `Expected: ${example.expected || "a typed Focusa result matching the descriptor output schema"}`,
    "",
    "## Operator alignment",
    "",
    ...(descriptor.operator_alignment?.requirements || []).map((requirement: string) => `- ${requirement}`),
    "",
    "## Anti-examples",
    "",
    ...antiExamples,
    "",
    "## Authority, permissions, and side effects",
    "",
    `- Scope: \`${inline(descriptor.scope)}\``,
    `- Authority: \`${inline(descriptor.authority)}\``,
    `- Side effects: ${descriptor.side_effects?.length ? descriptor.side_effects.map((item: string) => `\`${item}\``).join(", ") : "none"}`,
    `- Read-only: \`${descriptor.annotations.readOnlyHint}\`; destructive: \`${descriptor.annotations.destructiveHint}\`; idempotent: \`${descriptor.annotations.idempotentHint}\`; open-world: \`${descriptor.annotations.openWorldHint}\`.`,
    `- Confirmation required: \`${descriptor.confirmation.required}\`; preview supported: \`${descriptor.confirmation.preview_supported}\`.`,
    "",
    "## Failure and recovery",
    "",
    `Declared failure classes: ${descriptor.failure_classes?.map((item: string) => `\`${item}\``).join(", ") || "none"}.`,
    "",
    ...recovery,
    "",
    "## Dependencies and workflow position",
    "",
    ...dependencies,
    "",
    `Prerequisites: ${descriptor.prerequisites?.length ? descriptor.prerequisites.join("; ") : "none"}.`,
    `Likely next: ${descriptor.likely_next_capabilities?.map((item: string) => `\`${item}\``).join(", ") || "none"}.`,
    "",
    "## Skills, protocols, and source authority",
    "",
    `- Skills: ${descriptor.skill_refs.map((item: string) => `\`${item}\``).join(", ")}`,
    `- Runbooks: ${descriptor.runbook_refs.map((item: string) => `\`${item}\``).join(", ")}`,
    `- Pi: \`${descriptor.tool_names.pi}\`; MCP: \`${descriptor.tool_names.mcp}\`; OpenAI: \`${descriptor.tool_names.openai}\`.`,
    `- CLI: ${descriptor.tool_names.cli.map((item: string) => `\`${item}\``).join(", ") || "none"}.`,
    `- REST: ${routes}.`,
    `- Assignable: \`${descriptor.availability.assignable}\`; parity: \`${descriptor.availability.parity_status}\`.`,
    ...(descriptor.availability.assignable ? [] : [`- This capability is unavailable because its daemon router is not registered. Declared unavailable routes: ${descriptor.availability.unavailable_route_refs.map((route: any) => `\`${route.method} ${route.path}\``).join(", ")}.`]),
    `- Specification: ${descriptor.spec_refs.map((item: string) => `\`${item}\``).join(", ") || "contract registry"}.`,
    `- Descriptor digest: \`${descriptor.descriptor_digest}\`.`,
  ];
  const body = `${lines.join("\n").trimEnd()}\n`;
  const outputPath = resolve(root, descriptor.docs_ref);
  const current = (() => { try { return readFileSync(outputPath, "utf8"); } catch { return null; } })();
  if (current !== body) {
    drift += 1;
    if (!check) {
      mkdirSync(dirname(outputPath), { recursive: true });
      writeFileSync(outputPath, body);
    }
  }
}

if (check && drift) {
  console.error(`Spec141 agent tool docs drift: ${drift} file(s)`);
  process.exit(1);
}
console.log(JSON.stringify({ status: "passed", mode: check ? "check" : "write", documents: registry.descriptors.length }));
