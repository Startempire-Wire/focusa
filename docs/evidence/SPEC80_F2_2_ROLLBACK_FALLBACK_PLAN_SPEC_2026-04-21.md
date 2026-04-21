# SPEC80 F2.2 — Rollback and Fallback Plan Spec

Date: 2026-04-21
Bead: `focusa-yro7.6.2.2`
Label: `documented-authority`

Purpose: define rollback/fallback policy for failed phase gates.

## Rollback triggers

1. Gate C failure (branch correctness)
2. Gate D failure (compounding threshold/regression)
3. Gate E failure (governance integrity)

## Rollback actions

- Freeze promotion of planned-extension rollout claims.
- Revert to last verified phase-ready evidence baseline.
- Mark failed gate reason and bind to remediation bead.

## Fallback behavior

- Keep CLI stubs/honest not_implemented payloads for non-backed surfaces.
- Keep read-only lineage operations active where implemented.
- Disable promotion decisions until Gate D recovers.

## Rollback report schema

```json
{
  "rollback_id": "string",
  "trigger_gate": "C|D|E",
  "trigger_reason": "string",
  "rolled_back_to_phase": "0|1|2|3|4",
  "fallback_modes": [],
  "remediation_refs": []
}
```

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§10, §11)
- docs/evidence/SPEC80_C3_1_METACOGNITION_COMMAND_SURFACE_DESIGN_2026-04-21.md
