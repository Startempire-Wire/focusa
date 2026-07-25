# Spec 141 — Focusa Agent-First Tool, Skill, Runbook, and Documentation Release Gate

**Status:** operator-mandated release gate
**Date:** 2026-07-23
**Authority:** operator directive plus Specs 34, 89, 90, 104, 105, 111, 129, 135J, and 140
**Current release evidence:** `docs/evidence/141-focusa-agent-first-tool-surface-audit-20260724.{json,md}`

## 1. Purpose

Focusa is agent-first software. Every supported agent—from a weak model with little context to a strong long-horizon agent—must be able to:

1. discover what Focusa can do;
2. select the narrowest correct capability;
3. construct a valid call without guessing;
4. understand authority, scope, permissions, side effects, cost, and confirmation requirements;
5. interpret structured success, degraded, pending, blocked, and failure results;
6. recover through an exact bounded next action;
7. compose tools into dependency-correct workflows;
8. resume those workflows across compaction, process restart, harness switch, and machine boundary;
9. operate Focusa's companion browser/UIAI surfaces with the same contracts and evidence discipline;
10. acquire progressively deeper product knowledge without loading the entire software manual into every prompt.

This specification is not satisfied by having many tools, a large README, generic generated tool pages, or passing source-count checks. It requires complete machine-readable affordances, deep skills, executable runbooks, cross-harness interoperability, and behavioral evaluation.

## 2. Release rule

A tagged release is ineligible while any Spec 141 acceptance gate is red.

The gate covers:

- Pi tools;
- daemon REST routes;
- generated agent operation registry and OpenAPI;
- MCP;
- OpenAI-compatible function schemas;
- CLI human and JSON help;
- A2A-style agent capability discovery;
- UIAI Engine and WebMCP browser interoperability;
- Agent Skills and Focusa skills;
- internal agent knowledge;
- public repository documentation;
- behavioral conformance from weak-agent through strong-agent workflows;
- release notes and durable change history.

macOS validation authority resides outside the VPS environment and is not inferred from VPS work.

## 3. Current audited state

The canonical audit reports:

- 105 Pi tools;
- 105 TypeScript contracts;
- 105 per-tool Markdown pages;
- 81 generated agent operations/OpenAPI paths;
- 240 unique Axum route paths requiring eligibility classification;
- one MCP tool (`focusa.health`);
- 43 generic per-tool pages;
- 52 tool pages with examples;
- 23 tool pages with explicit parameter/input-schema language;
- zero tool pages with an explicit dependency/prerequisite/workflow section;
- one explicit `additionalProperties=false` Pi object schema;
- zero per-tool Pi output schemas;
- 20 operation-family `docs_ref` targets that do not exist;
- a drifted JSON projection despite source/contract name parity;
- 14 CLI machine-help entries for 17 top-level command families;
- no single cross-harness Agent Capability Manifest;
- no progressive Focusa tool search/describe/dependency-graph interface;
- no complete WebMCP/UIAI capability bridge;
- no weak-agent behavioral conformance suite.

These are release-gating defects, not documentation polish.

## 4. External benchmark findings and fit

### 4.1 High-fit patterns — adopt

| Pattern | Source | Focusa fit |
| --- | --- | --- |
| Deferred tool loading and tool search | Anthropic advanced tool use; OpenAI function calling | Essential with 105+ tools; reduces prompt bloat and improves selection |
| Namespaces and regex/search-based capability retrieval | Anthropic/OpenAI | Strong fit for Focusa families and domain packs |
| Strict JSON Schema and structured output | OpenAI strict mode; MCP `inputSchema`, `outputSchema`, `structuredContent` | Essential; current input strictness/output schemas are incomplete |
| Tool annotations | MCP `readOnlyHint`, `destructiveHint`, `idempotentHint`, `openWorldHint` | Direct fit with Focusa side-effect/authority/permission model |
| Paginated discovery and `listChanged` | MCP tools | Direct fit for runtime capability/version changes |
| Durable async tasks | MCP Tasks; A2A Tasks | Direct fit for Workpoints, Work Loop, Silent Sessions, and long browser work |
| Agent Card with skills/capabilities/auth/interfaces | A2A | Direct fit for cross-harness and cross-daemon discovery |
| Progressive skill disclosure | Agent Skills | Direct fit; metadata first, instructions on activation, resources on demand |
| Machine-readable examples and anti-examples | Anthropic tool guidance; MCP schemas | Essential for weak-agent reliability |
| Form/URL elicitation with accept/decline/cancel | MCP elicitation | Strong fit for operator approvals, auth, device pairing, and structured ambiguity resolution |
| Web-native structured tools | W3C Community Group WebMCP draft | Strong fit for Mission Canvas/UIAI browser workflows; avoids brittle DOM-only operation |
| GenAI/agent/tool telemetry semantics | OpenTelemetry GenAI conventions | Strong fit for traces, latency, errors, tool selection, and evaluation |
| `/llms.txt` and linked Markdown corpus | llms.txt proposal | Useful public discovery layer; must remain generated and freshness-checked |

### 4.2 Medium-fit patterns — govern before adoption

- programmatic tool calling inside a sandbox;
- client/server sampling;
- remote skill installation;
- agent-to-agent delegation across external vendors;
- dynamic UI tool registration from untrusted pages;
- URL-mode elicitation for sensitive data.

These require Focusa permissions, receipts, provenance, scope, and rollback boundaries.

### 4.3 Poor-fit patterns — reject

- loading every tool schema into every prompt;
- exposing every internal API route as an agent tool;
- prose-only permissions or recovery guidance;
- server-described safety annotations trusted without client verification;
- arbitrary code embedded in skill instructions;
- silent cross-project or cross-daemon authority adoption;
- autonomous browser mutation without evidence and operator-governed approval.

## 5. Canonical Agent Capability Descriptor V2

One source descriptor must generate every tool/operation projection.

Required fields:

```json
{
  "schema": "focusa.agent_capability_descriptor.v2",
  "capability_id": "focusa.workpoint.checkpoint",
  "tool_names": {
    "pi": "focusa_workpoint_checkpoint",
    "mcp": "focusa.workpoint.checkpoint",
    "openai": "focusa_workpoint_checkpoint",
    "cli": "focusa workpoint checkpoint"
  },
  "version": "1.0.0",
  "title": "Workpoint Checkpoint",
  "summary": "...",
  "description": "three-or-more-sentence operational description",
  "family": "workpoint",
  "namespace": "focusa.workpoint",
  "availability": {
    "requires_daemon": true,
    "supported_harnesses": ["pi", "mcp", "openai", "cli", "rest"],
    "required_capabilities": []
  },
  "input_schema": {},
  "output_schema": {},
  "error_schema": {},
  "result_envelope": "focusa.tool_result.v1",
  "scope": {},
  "authority": {},
  "permissions": [],
  "annotations": {
    "read_only": false,
    "destructive": false,
    "idempotent": true,
    "open_world": false
  },
  "side_effects": [],
  "confirmation": {},
  "idempotency": {},
  "reversibility": {},
  "cost_hint": {},
  "latency_hint": {},
  "token_budget": {},
  "examples": [],
  "anti_examples": [],
  "failure_classes": [],
  "recovery": [],
  "prerequisites": [],
  "dependencies": [],
  "likely_next_capabilities": [],
  "skill_refs": [],
  "runbook_refs": [],
  "docs_ref": "...",
  "spec_refs": [],
  "evidence_requirements": [],
  "deprecation": null,
  "compatibility": {},
  "conformance_refs": []
}
```

### 5.1 Generator rule

The descriptor generates:

- Pi TypeBox input schemas and strict result validators;
- MCP tools/list and tools/call projections;
- OpenAI strict function definitions;
- REST OpenAPI and JSON Schema;
- CLI JSON command inventory;
- A2A-style Agent Card skills/capabilities;
- per-tool agent documentation;
- skill dependency references;
- tool search index;
- tool dependency graph;
- conformance fixtures.

Hand-maintained duplicate totals, schemas, and route lists are prohibited.

## 6. Progressive discovery architecture

### 6.1 Bootstrap packet

The hot bootstrap exposes only:

- identity and health;
- `focusa_tool_search`;
- `focusa_tool_describe`;
- `focusa_tool_graph`;
- project identity/verify;
- Workpoint resume/checkpoint;
- trajectory view;
- tool doctor;
- bounded recovery guidance.

All other schemas are cold-loaded by search, family bundle, active Workpoint, or explicit operator request.

### 6.2 Required discovery surfaces

```text
GET  /v1/agent/card
GET  /v1/agent/tools?query=&family=&cursor=&limit=
GET  /v1/agent/tools/{capability_id}
GET  /v1/agent/tool-graph?anchor=&depth=
GET  /v1/agent/tool-bundles
GET  /v1/agent/tool-changes?since_digest=
POST /v1/agent/tool-search
```

Pi tools:

```text
focusa_tool_search
focusa_tool_describe
focusa_tool_graph
focusa_tool_bundle
focusa_agent_card
```

Every discovery response includes a registry digest, version, freshness, source authority, pagination, rehydrate references, and `listChanged` equivalent.

## 7. Invocation and result contract

### 7.1 Inputs

Every input object:

- uses strict JSON Schema;
- rejects unknown properties unless explicitly open;
- defines required, optional, default, enum, format, min/max, and conditional constraints;
- includes field-level descriptions and examples;
- distinguishes omitted, null, unknown, inferred, and explicit values;
- exposes scope and authority requirements before mutation;
- supports preview/preflight where applicable.

### 7.2 Outputs

Every capability has a strict output schema. `focusa.tool_result.v1` remains the common envelope, but `details` is typed per capability.

Required common semantics:

```text
status
canonical
degraded
failure_class
human_summary
machine_summary
side_effects
retry
recovery
next_tools
evidence_refs
receipt_refs
scope
authority
freshness
warnings
```

A human string without structured details is not a complete tool result.

### 7.3 Errors and recovery

Every declared failure class has:

- machine code;
- retryability;
- safe retry delay;
- whether input can be reused;
- exact recovery capability/command;
- whether state may have changed;
- rollback/compensation guidance;
- evidence to inspect;
- escalation boundary.

## 8. Cross-harness interoperability

### 8.1 MCP

MCP must project the curated capability registry, not one health tool.

Required:

- paginated `tools/list`;
- `notifications/tools/list_changed`;
- `inputSchema` and `outputSchema`;
- `structuredContent`;
- read/destructive/idempotent/open-world annotations;
- icons/title where useful;
- task-augmented execution for long-running capabilities;
- protocol errors versus tool execution errors;
- auth/scope capability negotiation.

### 8.2 OpenAI-compatible tools

Required:

- strict schemas;
- namespaced capability groups;
- deferred loading/tool search;
- validated call IDs and paired outputs;
- no parallel calls for authority-sensitive mutations unless explicitly safe;
- identical semantics to MCP/Pi projections.

### 8.3 CLI

`focusa help all --json` must be generated from Clap/capability authority and include every command/subcommand, arguments, defaults, examples, side effects, permissions, exit codes, deprecation, replacement, and related capability IDs.

### 8.4 REST/OpenAPI

Every Axum route is classified:

```text
agent_eligible
operator_only
internal
public_health
public_pairing
deprecated
```

Every agent-eligible route has an operation descriptor, materialized schemas, examples, errors, docs, and conformance tests. Every internal route is explicitly excluded with rationale.

### 8.5 Agent Card

Focusa publishes a versioned card containing:

- supported protocols/interfaces;
- capabilities and skill summaries;
- auth schemes;
- input/output modes;
- streaming/task support;
- compatibility versions;
- registry digest;
- extended-card route for authenticated detail;
- conformance and evidence refs.

## 9. Companion browser and UIAI interoperability

### 9.1 UIAI capability bridge

Focusa must ingest a UIAI capability manifest and expose browser operations through the same capability descriptor semantics.

Browser operations include:

- session lifecycle;
- page read/source/markdown;
- snapshot and stable refs;
- click/fill/select/press;
- diagnostics/console/network;
- screenshot/visual proof;
- async evaluation;
- browser context isolation;
- evidence capture and Workpoint linkage.

### 9.2 WebMCP

Focusa/UIAI should support a governed WebMCP adapter where available:

- discover page-registered tools;
- validate schemas and annotations;
- bind tools to the exact browser session/origin;
- require Focusa permission/confirmation for mutation;
- capture calls/results as evidence;
- fall back to accessibility/DOM automation when no WebMCP tool exists;
- never treat page annotations as trusted safety authority by themselves.

### 9.3 Browser workflow graph

Each browser-capable skill declares:

```text
source/read first
diagnostics on failure
snapshot refs before action
mutation confirmation class
evidence intake
recovery route
session cleanup
Workpoint attachment
```

## 10. Skills and runbook system

This phase follows the tool audit but is part of the same release gate.

### 10.1 Progressive skill layers

1. **metadata layer** — name, concise description, trigger phrases, compatibility, allowed tools;
2. **core instructions** — loaded only when activated;
3. **runbook resources** — loaded only for the selected workflow;
4. **deep references** — schemas/specs/evidence opened only when necessary.

### 10.2 Required skill domains

The final coverage map must evaluate at least:

- Focusa orientation and progressive discovery;
- project identity, scope, and cross-project safety;
- Workpoint and Trajectory operation;
- Focus State and cognitive slots;
- Work Loop, autonomy, and Silent Sessions;
- evidence, receipts, proof, and closure authority;
- compaction, rollover, resume, and session transfer;
- prediction, calibration, metacognition, and transfer learning;
- tool contracts and cross-harness interoperability;
- CLI/API/MCP/OpenAI/A2A use;
- UIAI/browser/WebMCP research and action;
- Mission Canvas, Work Rail, CRIST, and generated UI;
- ontology and domain packs;
- installation, repair, OTA, rollback, and uninstall;
- resources, Bloatgaurd, token budgets, and performance;
- permissions, security, auth, licensing, and device pairing;
- spec/call-stack/task implementation;
- diagnostics, doctor, recovery, and incident handling;
- release proof and changelogs;
- temporal authority, deadlines, and grounded forecasting;
- proposal settlement and outcome truth.

### 10.3 Runbook requirements

Every nontrivial workflow has:

- trigger and non-trigger examples;
- prerequisites;
- dependency graph;
- exact tool sequence;
- branch conditions;
- failure and recovery routes;
- authority/approval boundaries;
- evidence and done conditions;
- cross-harness variants;
- browser companion steps where relevant;
- minimal and deep modes;
- executable conformance fixture.

## 11. Internal agent documentation gate

Audit and reconcile:

- `AGENTS.md` surfaces;
- `.pi/skills/` and packaged skills;
- bootstrap prompts;
- preload profiles;
- utility/agent cards;
- Focusa tool docs;
- CLI/API docs;
- troubleshooting and failure playbooks;
- release/deployment guidance;
- spec indexes and manifests.

Rules:

- one canonical source per claim;
- generated totals and inventories;
- freshness/version metadata;
- no dead paths;
- no generic guidance where specific workflow knowledge exists;
- no VPS claim of macOS validation;
- machine-readable index plus human navigation.

## 12. Public documentation and recent-spec alignment

The rolling recent-spec set locked for this audit is:

```text
135b, 135c, 135d, 135e, 135f, 135g, 135h, 135i, 135j, 135k,
136, 137, 138, 139, 140
```

Public docs must explain implemented status and direction without claiming unfinished work as shipped.

Required public surfaces:

- top-level `README.md`;
- `docs/README.md`;
- `docs/llms.txt`;
- public architecture/feature overview;
- installation/update/recovery guide;
- tool/agent integration guide;
- MCP/OpenAI/CLI/API discovery guide;
- browser/UIAI integration guide;
- skills/runbooks index;
- compatibility and limitations;
- release notes and changelog history.

## 13. Behavioral evaluation

Static parity is necessary but insufficient.

### 13.1 Agent levels

Evaluate:

- weak/small model with only bootstrap metadata;
- medium model with tool search;
- strong model with deep skill access;
- non-Pi MCP client;
- OpenAI-compatible function client;
- CLI-only automation;
- browser/UIAI workflow agent.

### 13.2 Golden tasks

At minimum:

1. discover the right tool among 105+ capabilities;
2. recover from an invalid parameter without operator rescue;
3. distinguish read, preview, commit, destructive, and irreversible actions;
4. preserve project/continuity scope;
5. resume a Workpoint after compaction;
6. execute a multi-tool dependency chain;
7. diagnose daemon/tool failure;
8. perform UIAI read → action → diagnostics → evidence intake;
9. use the same capability through Pi, MCP, OpenAI, CLI, and REST;
10. avoid loading irrelevant schemas under token budget.

### 13.3 Metrics

```text
tool_selection_accuracy
first_call_validity
repair_success_rate
unsafe_call_rate
scope_violation_rate
workflow_completion_rate
recovery_turns
schema_tokens_loaded
total_prompt_tokens
latency
cross_harness_semantic_parity
evidence_completion_rate
```

## 14. Implementation order

### Phase A — Audit authority

Exact files:

- `scripts/audit-agent-first-tool-surfaces.py`
- `docs/evidence/141-focusa-agent-first-tool-surface-audit-20260724.json`
- `docs/evidence/141-focusa-agent-first-tool-surface-audit-20260724.md`
- `tests/spec141_agent_first_tool_audit_test.py`

### Phase B — Canonical descriptor and generated projections

Primary files:

- `apps/pi-extension/src/tool-contracts.ts`
- new canonical descriptor/generator under `packages/` or `crates/focusa-core/` after call-stack design;
- generated Pi/MCP/OpenAPI/CLI/docs artifacts;
- drift gates.

### Phase C — Discovery and interoperability

Primary surfaces:

- agent card;
- tool search/describe/graph/bundles/changes;
- MCP full projection;
- OpenAI strict projection;
- CLI exhaustive JSON help;
- REST route classification.

### Phase D — Browser/UIAI/WebMCP

Primary surfaces:

- capability manifest intake;
- browser operation descriptors;
- governed WebMCP adapter;
- evidence and Workpoint linkage;
- browser conformance tests.

### Phase E — Skills, runbooks, and internal docs

- complete coverage matrix;
- dependency DAG;
- missing skill implementation;
- packaged/root skill parity;
- executable runbooks;
- preload/bootstrap reconciliation.

### Phase F — Public docs and latest-spec reconciliation

- public surface update;
- rolling 15-spec matrix;
- shipped/planned status integrity;
- generated `llms.txt` and indexes.

### Phase G — Weak-to-strong agent acceptance

- conformance harness;
- golden workflows;
- token and safety budgets;
- cross-harness parity;
- final release-gate report.

## 15. Acceptance criteria

1. Audit reports zero critical/high findings.
2. One descriptor authority generates every projection.
3. Every agent-eligible capability has strict input/output/error schemas.
4. Every tool has specific examples, anti-examples, dependencies, recovery, skill refs, and docs.
5. MCP exposes the curated catalog with structured output and annotations.
6. OpenAI-compatible strict schemas and progressive loading pass.
7. CLI machine help covers every command/subcommand.
8. Every API route is classified and every agent route is fully contracted.
9. All operation docs/schema refs resolve.
10. Agent Card and compatibility negotiation pass.
11. UIAI/browser/WebMCP workflow contracts pass.
12. Skill coverage and dependency graph have no unexplained gaps.
13. Internal docs have no stale totals, dead paths, or contradictory authority.
14. Public docs accurately reflect implementation and recent-spec direction.
15. Weak-agent and cross-harness evaluations meet thresholds.
16. Prompt/tool schema loading remains inside Bloatgaurd budgets.
17. Every tagged release includes features, fixes, resolved issues, compare link, and complete commit audit.
18. CI runs the strict Spec 141 release gate.

## 16. Rollback and safety

- Generated projections are versioned and reproducible.
- Existing Pi tool names remain compatible during migration.
- Descriptor V1 remains readable until V2 parity is proven.
- MCP/OpenAI mutation tools remain opt-in until permission and receipt conformance passes.
- WebMCP page tools remain untrusted inputs to Focusa governance.
- Skill rewrites preserve prior versions until behavioral evaluation improves.
- Release automation changes retain the previous notes body as recoverable evidence.

## 17. Proof commands

```bash
python3 scripts/audit-agent-first-tool-surfaces.py \
  --json docs/evidence/141-focusa-agent-first-tool-surface-audit-20260724.json \
  --markdown docs/evidence/141-focusa-agent-first-tool-surface-audit-20260724.md
python3 tests/spec141_agent_first_tool_audit_test.py
node scripts/validate-focusa-tool-contracts.mjs
node scripts/audit-focusa-tool-implementation-spec-gaps.mjs --json
node scripts/audit-focusa-tool-suite-safe.mjs --json
python3 tests/spec104_tool_contract_static_audit.py
bash tests/pi_extension_final_toolset_audit_static_test.sh
bash tests/spec_mcp_jsonrpc_static_test.sh
bash tests/release_notes_workflow_static_test.sh
```

Final release proof additionally runs the behavioral conformance suite introduced by Phase G.
