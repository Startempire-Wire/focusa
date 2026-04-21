# SPEC80 C4.1 — JSON Schema Registry Plan

Date: 2026-04-21
Bead: `focusa-yro7.3.4.1`
Label: `documented-authority`

Purpose: define registry for machine-stable CLI/API JSON contracts consumed by tooling.

## Registry scope

1. Lineage command schemas
- head, tree, node, path, children, summaries

2. Export command schemas
- status, run dry-run result, run write result

3. Reflection/metacognition schemas
- reflect run/history/status
- metacognition stub payloads now, endpoint payloads when implemented

## Storage convention

- Directory: `docs/schemas/cli-api/`
- Naming: `<domain>.<command>.v<major>.schema.json`
- Example: `lineage.head.v1.schema.json`

## Governance rules

1. Any contract consumed by tools must have a registry schema.
2. `--json` output changes require schema diff + version decision.
3. CI/gate checks validate sample payloads against current schema.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§7, §15)
- docs/24-capabilities-cli.md
