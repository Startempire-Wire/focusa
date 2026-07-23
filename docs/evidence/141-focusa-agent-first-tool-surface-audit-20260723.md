# Spec141 Focusa Agent-First Tool Surface Audit

Generated: `2026-07-23T16:36:41.417691+00:00`

**Status:** `gaps_found`
**Release gate:** `fail`

## Metrics

- **pi_registered_tools:** `105`
- **tool_contracts:** `105`
- **per_tool_docs:** `105`
- **tool_families:** `{'awareness': 1, 'diagnostics_hygiene': 21, 'focus_state': 11, 'metacognition': 13, 'preload': 8, 'project_identity': 4, 'session_transfer': 6, 'trajectory': 14, 'traversal': 2, 'tree_lineage': 9, 'work_loop': 7, 'workpoint': 9}`
- **contract_json_validator_passed:** `True`
- **capability_descriptor_generator_passed:** `True`
- **capability_descriptors_v2:** `105`
- **capability_descriptors_with_strict_input:** `105`
- **capability_descriptors_with_output_schema:** `105`
- **agent_card_present:** `True`
- **generic_tool_docs:** `43`
- **docs_with_examples:** `52`
- **docs_with_explicit_input_schema:** `23`
- **docs_with_dependency_section:** `0`
- **pi_typebox_nodes:** `857`
- **pi_schema_description_tokens:** `482`
- **pi_strict_object_schema_markers:** `5`
- **pi_output_schema_markers:** `1`
- **tools_with_explicit_next_tool_graph:** `105`
- **mcp_exposed_tools:** `1`
- **mcp_tool_names:** `['focusa.health']`
- **cli_top_level_commands:** `81`
- **cli_machine_help_inventory_entries:** `14`
- **api_route_paths:** `444`
- **agent_operation_registry_entries:** `81`
- **agent_operation_openapi_paths:** `81`
- **operation_schema_refs:** `159`
- **materialized_openapi_schema_refs:** `159`
- **missing_operation_docs_refs:** `20`
- **tool_contracts_without_api_route:** `4`
- **tool_contracts_without_cli_command:** `15`

## Findings

### AF-TOOL-005 — CRITICAL — mcp

MCP exposes only a health probe rather than the curated agent operation catalog.

**Remediation:** Generate paginated MCP tools/list and tools/call from the canonical operation registry, including outputSchema, structuredContent, annotations, tasks, and listChanged.

```json
{
  "mcp_tools": [
    "focusa.health"
  ],
  "agent_operations": 81
}
```

### AF-TOOL-006 — HIGH — rest_openapi

The agent operation/OpenAPI registry covers only a subset of API routes and does not classify every remaining route as agent-eligible or internal.

**Remediation:** Classify every route; fully contract agent-eligible routes and explicitly mark internal/operator-only routes.

```json
{
  "api_routes": 444,
  "agent_operations": 81
}
```

### AF-TOOL-007 — CRITICAL — operation_docs

Every operation family docs_ref currently points to a missing document.

**Remediation:** Materialize and validate every operation docs_ref from the canonical descriptor.

```json
{
  "missing_count": 20,
  "refs": [
    "docs/focusa-api/routes/awareness.md",
    "docs/focusa-api/routes/bloatgaurd.md",
    "docs/focusa-api/routes/call_stack.md",
    "docs/focusa-api/routes/context_cognition.md",
    "docs/focusa-api/routes/device.md",
    "docs/focusa-api/routes/dxux.md",
    "docs/focusa-api/routes/evidence.md",
    "docs/focusa-api/routes/health.md",
    "docs/focusa-api/routes/license.md",
    "docs/focusa-api/routes/lineage.md",
    "docs/focusa-api/routes/metacognition.md",
    "docs/focusa-api/routes/predictions.md",
    "docs/focusa-api/routes/project.md",
    "docs/focusa-api/routes/resource.md",
    "docs/focusa-api/routes/state.md",
    "docs/focusa-api/routes/tool_doctor.md",
    "docs/focusa-api/routes/trajectory.md",
    "docs/focusa-api/routes/traverse.md",
    "docs/focusa-api/routes/work_loop.md",
    "docs/focusa-api/routes/workpoint.md"
  ]
}
```

### AF-TOOL-008 — CRITICAL — tool_docs

Per-tool documentation is structurally present but frequently generic and incomplete for deep agent operation.

**Remediation:** Generate specific parameter tables, positive/negative examples, failure recovery, prerequisites, dependency chains, and workflow position for every tool.

```json
{
  "docs": 105,
  "generic": 43,
  "with_examples": 52,
  "with_input_schema": 23,
  "with_dependencies": 0
}
```

### AF-TOOL-009 — HIGH — cli

Machine-readable CLI help is a curated subset rather than an exhaustive generated command schema.

**Remediation:** Generate JSON command schemas, flags, defaults, examples, effects, and migration metadata from Clap authority.

```json
{
  "top_level_commands": 81,
  "help_inventory": 14
}
```

### AF-TOOL-010 — HIGH — internal_docs

Canonical/current audit documents contain stale tool totals.

**Remediation:** Replace hand-maintained totals with generated values and freshness checks.

```json
{
  "current_tools": 105,
  "documents": [
    {
      "path": "docs/current/FOCUSA_TOOL_CONTRACT_REGISTRY.md",
      "stated_counts": [
        62
      ]
    },
    {
      "path": "docs/current/TOOL_RELIABILITY_AUDIT.md",
      "stated_counts": [
        59
      ]
    }
  ]
}
```

### AF-TOOL-011 — HIGH — progressive_discovery

Focusa lacks a dedicated search/describe/graph surface for cold-loading tool schemas and uses family-generic affordances for many tools.

**Remediation:** Add tool search, describe, dependency graph, namespaced bundles, digest/listChanged, and token-budgeted deferred schema loading.

```json
{
  "generic_affordances": 43,
  "total_tools": 105
}
```

### AF-TOOL-013 — HIGH — browser_interop

Browser interoperability is diagnostics-intake centric; Focusa lacks a machine-readable WebMCP/UIAI capability bridge and browser workflow dependency graph.

**Remediation:** Add UIAI/WebMCP capability discovery, session-isolated browser operation descriptors, evidence contracts, and browser-to-Workpoint workflow graphs.

```json
{
  "current_focusa_browser_tool": "focusa_browser_diagnostics_intake"
}
```

### AF-TOOL-014 — HIGH — agent_evaluation

Current tests validate contracts and surfaces but do not score weak-to-strong agents on tool selection, parameter completion, recovery, and cross-tool workflows.

**Remediation:** Add dumb-agent conformance fixtures, golden workflow tasks, invalid-call repair tests, token-cost budgets, and cross-harness behavioral parity evaluation.

```json
{
  "existing_static_audits": 7
}
```

## External benchmark sources

- https://www.anthropic.com/engineering/advanced-tool-use
- https://platform.claude.com/docs/en/agents-and-tools/tool-use/define-tools
- https://platform.openai.com/docs/guides/function-calling
- https://modelcontextprotocol.io/specification/2025-11-25/server/tools
- https://modelcontextprotocol.io/specification/2025-11-25/basic/utilities/tasks
- https://modelcontextprotocol.io/specification/2025-11-25/client/elicitation
- https://agentskills.io/specification
- https://a2a-protocol.org/latest/specification/
- https://webmachinelearning.github.io/webmcp/
- https://github.com/open-telemetry/semantic-conventions-genai
- https://llmstxt.org/
