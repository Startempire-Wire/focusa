# SPEC80 E3.1 — Form Schema + Ontology Alignment Spec

Date: 2026-04-21
Bead: `focusa-yro7.5.3.1`
Label: `documented-authority`

Purpose: lock implementation-facing schema for Appendix E practice+observation form, including ontology alignment fields.

## Canonical schema version

`practice_observation_v1`

## Required top-level fields

- `session_id`
- `clt_node_id`
- `work_item_id`
- `timestamp`
- `ontology_alignment`
- `practice`
- `observations`
- `adaptation`
- `outcome`
- `compound_candidate`

## Mandatory ontology fields

- `ontology_alignment.working_set_type`
- `ontology_alignment.membership_class`
- `ontology_alignment.status`
- `ontology_alignment.lifecycle_stage`
- `ontology_alignment.provenance_class`
- `ontology_alignment.verification_state`
- `ontology_alignment.reducer_event_refs[]`

## Branch integrity fields

- `observations.branch_context.forked`
- `observations.branch_context.from_clt_node`
- `observations.branch_context.restored_snapshot_id`

## Tool field ownership map

- capture -> `practice` + initial `observations`
- reflect -> enrich `observations` + `adaptation`
- plan_adjust -> `adaptation.strategy_changes`
- evaluate_outcome -> `outcome` + `compound_candidate`
- snapshot/restore -> `branch_context`

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§18.2, §18.4)
- docs/evidence/SPEC80_B2_1_CAPTURE_RETRIEVE_REFLECT_SCHEMAS_2026-04-21.md
