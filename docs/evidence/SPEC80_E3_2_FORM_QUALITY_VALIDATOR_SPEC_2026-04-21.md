# SPEC80 E3.2 — Form Quality Validator Spec

Date: 2026-04-21
Bead: `focusa-yro7.5.3.2`
Label: `documented-authority`

Purpose: define validation rules and failure outputs for Appendix E form ingestion.

## Required validation rules

1. `observations.signals[*].evidence_refs` non-empty.
2. `adaptation.decision` and `adaptation.rationale` both present.
3. `outcome.result=improved` requires >=1 observed metric.
4. `compound_candidate.promote=true` requires `learning_statement` and `applicability`.
5. If branch movement indicated, `clt_node_id` must match restored head context.
6. `ontology_alignment.status` and `ontology_alignment.provenance_class` required.
7. `ontology_alignment.reducer_event_refs` must include >=1 emitted reducer event name.

## Validator output schema

```json
{
  "valid": false,
  "errors": [
    {"code":"EVIDENCE_REFS_MISSING","path":"observations.signals[0].evidence_refs","message":"..."}
  ],
  "warnings": []
}
```

## Severity policy

- blocking errors: rules 1-7 failures.
- warnings: optional fields absent but non-critical.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§18.3)
- docs/evidence/SPEC80_E3_1_FORM_SCHEMA_ONTOLOGY_ALIGNMENT_SPEC_2026-04-21.md
