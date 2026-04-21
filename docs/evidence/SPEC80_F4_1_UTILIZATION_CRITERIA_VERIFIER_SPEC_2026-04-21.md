# SPEC80 F4.1 — Utilization Criteria Verifier Spec

Date: 2026-04-21
Bead: `focusa-yro7.6.4.1`
Label: `documented-authority`

Purpose: define verifier that checks all §20.3 full-utilization criteria with traceable evidence.

## Criteria checks

1. Tree/lineage correctness dossier present and passing.
2. Tool-first metacognition loop dossier present.
3. CLI/API parity dossier present.
4. Outcome compounding dossier present with Gate D logic.
5. Governance integrity dossier present with audit compliance.

## Verifier output schema

```json
{
  "verifier_id": "spec80_full_utilization_v1",
  "criteria": [{"id":"c1","pass":true,"evidence_refs":[]}],
  "all_pass": true,
  "blocking_criteria": []
}
```

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§20.3)
- docs/evidence/SPEC80_UTILIZATION_PROOF_PACK_PLAN_2026-04-21.md
