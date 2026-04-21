# SPEC80 Tool-to-Layer Declaration Contract — 2026-04-21

Purpose: complete `focusa-yro7.1.2.1` by defining the mandatory declaration fields each planned tool spec must carry to satisfy the §4 layer rule in `docs/80-pi-tree-li-metacognition-tooling-spec.md`.

## Contract fields (required)

Every tool declaration MUST include all fields below:

1. `tool_id`
   - Canonical tool name.
2. `ontology_layers`
   - Non-empty list of numeric layer ids in range `1..12`.
3. `layer_semantics`
   - Per-layer rationale describing why each listed layer is touched.
4. `operation_kind`
   - One of: `read`, `mutate`, `interpret`, `composite`.
5. `authority_label`
   - One of §19.3 labels: `implemented-now` | `documented-authority` | `planned-extension`.
6. `authority_citations`
   - Code and/or authoritative spec citations supporting the label claim.
7. `gate_binding`
   - Acceptance/gate linkage (e.g., Gate B CLI readiness, Gate C anti-false-weaving).

## Validation rules

1. Reject declaration if `ontology_layers` is empty.
2. Reject declaration if any layer id is outside `1..12`.
3. Reject declaration if `authority_label=implemented-now` and no code citation is present (§19.3).
4. Reject declaration if `operation_kind` includes mutation/interpretation but `layer_semantics` is missing.
5. Reject declaration if `gate_binding` is absent.

## SPEC80 planned tool declarations

| tool_id | ontology_layers | operation_kind | authority_label | gate_binding |
|---|---:|---|---|---|
| `focusa_tree_head` | `8,12` | `read` | `documented-authority` | Gate C governance integrity |
| `focusa_tree_path` | `8,7,12` | `read` | `documented-authority` | Gate C governance integrity |
| `focusa_tree_snapshot_state` | `8,5,11` | `mutate` | `planned-extension` | Gate A branch correctness |
| `focusa_tree_restore_state` | `8,5,11,9` | `mutate` | `planned-extension` | Gate A branch correctness |
| `focusa_tree_diff_context` | `8,11,12` | `interpret` | `planned-extension` | Gate D metacog outcomes |
| `focusa_metacog_capture` | `6,11,12` | `mutate` | `planned-extension` | Gate D metacog outcomes |
| `focusa_metacog_retrieve` | `11,7,12` | `read` | `planned-extension` | Gate D metacog outcomes |
| `focusa_metacog_reflect` | `11,6,12` | `interpret` | `planned-extension` | Gate D metacog outcomes |
| `focusa_metacog_plan_adjust` | `11,9,5` | `mutate` | `planned-extension` | Gate D metacog outcomes |
| `focusa_metacog_evaluate_outcome` | `12,6,7,11` | `interpret` | `planned-extension` | Gate D metacog outcomes |

## Evidence citations

- docs/80-pi-tree-li-metacognition-tooling-spec.md (§4, §6, §19.3, §20)
- docs/evidence/SPEC80_FOCUSA_ONTOLOGY_AUTHORITY_MAP_2026-04-21.md
