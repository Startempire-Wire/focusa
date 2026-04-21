# SPEC80 Layer Compliance Review Checklist — 2026-04-21

Use this checklist to enforce `docs/80-pi-tree-li-metacognition-tooling-spec.md` §4 layer rule during BD decomposition and planning reviews.

## Checklist

1. Tool declares ontology layers touched (1..12) and rationale.
2. Tool declares classification label (`implemented-now|documented-authority|planned-extension`).
3. If label is `implemented-now`, code citation is present and valid.
4. If label is `planned-extension`, authoritative doc requirement + planned endpoint are cited.
5. Tool mutation scope is declared (`read-only|state-write|governance-write`).
6. For non-read-only tools, reducer event refs are declared.
7. Status/provenance/verification implications are mapped for outputs.
8. Branch/lineage implications are declared for any `/tree`-touching tool.
9. API path + CLI fallback mapping exists (or planned row exists in Appendix B).
10. Error model is typed and aligned with spec appendix contract.
11. Verification hook paths exist (tests/docs) or planned test beads are linked.
12. Performance budget impact annotation exists for reflection/snapshot heavy paths.

## Review outcome states

- PASS: all checks satisfied.
- PASS_WITH_PLANNED_GAPS: all mandatory checks pass, planned-extension items explicitly tracked as beads.
- FAIL: any missing mandatory check (1-7) or false implemented-now claim.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/evidence/SPEC80_TOOL_LAYER_DECLARATION_CONTRACT_2026-04-21.md
- docs/50-ontology-classification-and-reducer.md
