# SPEC80 Focusa Ontology Authority Map — 2026-04-21

Purpose: complete `focusa-yro7.1.1.2` by mapping authoritative ontology docs (45-77 + integration docs) to SPEC80 clauses and decomposition lanes.

## Authority coverage map

| Focusa doc group | Authority focus | SPEC80 binding |
|---|---|---|
| `45-51` ontology overview/core/primitives/classification/expression | Canonical primitive laws, reducer authority, event minimums, slice expression policy | §4 ontology layers; §5 architecture decisions; §9 invariants; Appendix E ontology_alignment |
| `49` working sets/slices | bounded cognition sets, membership classes, refresh triggers | §4 layers 5/7/8/11/12; §8 outcome measurement context; Appendix E working_set_type + membership_class |
| `61,66,67,70,72-77` domain ontologies | object/link/action/status/lifecycle/governance families | §6 tool layer references; Appendix E `link_type_refs`, `action_type_refs`, status/provenance/verification fields |
| `58-65` visual/UI ontology family | visual model extraction, verification, implementation mapping | §4 layers 2/3/4/6; §12 latency/quality risks; future applicability in metacog retrieval contexts |
| `17` CLT lineage spec | lineage head/path integrity and compaction lineage behavior | §6.1 tree bridge tools; Appendix D replay tests; §20 full-utilization criterion #1 |
| `44` Pi×Focusa integration | `/fork` and `/tree` gaps, tool-first metacognition, bridge command model | §2 problem statement; §6 tools; §10 Gate C/Gate D; §20 matrices |
| `24` capabilities CLI | API↔CLI parity, no hidden writes, machine-readable operation | §7 CLI backlog; Appendix B binding matrix; §20.1 CLI parity gaps |

## Code-reality cross-check anchors

Implemented now (verified route presence):
- lineage APIs (`/v1/lineage/head|tree|node|path|children|summaries`)
- reflection APIs (`/v1/reflect/run|history|status`)

Planned/missing (must remain labeled, not implied):
- `/v1/focus/snapshots*`
- `/v1/metacognition/*`
- first-class CLI lineage/metacognition domains

## Decomposition obligations

1. Every BD spawned from SPEC80 must carry one label from §19.3 (`implemented-now`, `documented-authority`, `planned-extension`).
2. Every implemented-now claim must cite code path.
3. Every planned-extension claim must cite authoritative doc requirement and target endpoint/command.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/45-ontology-overview.md
- docs/46-ontology-core-primitives.md
- docs/49-working-sets-and-slices.md
- docs/50-ontology-classification-and-reducer.md
- docs/61-domain-general-cognition-core.md
- docs/66-affordance-and-execution-environment-ontology.md
- docs/67-query-scope-and-relevance-control.md
- docs/70-shared-interfaces-statuses-and-lifecycle.md
- docs/72-agent-identity-role-and-self-model-ontology.md
- docs/74-identity-and-reference-resolution.md
- docs/75-projection-and-view-semantics.md
- docs/76-retention-forgetting-and-decay-policy.md
- docs/77-ontology-governance-versioning-and-migration.md
- docs/17-context-lineage-tree.md
- docs/44-pi-focusa-integration-spec.md
- docs/24-capabilities-cli.md
