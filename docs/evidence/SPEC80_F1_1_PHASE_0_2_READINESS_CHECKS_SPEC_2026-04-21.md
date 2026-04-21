# SPEC80 F1.1 — Phase 0-2 Readiness Checks Spec

Date: 2026-04-21
Bead: `focusa-yro7.6.1.1`
Label: `documented-authority`

Purpose: define readiness criteria for phases 0-2 before implementation rollout.

## Phase 0 — Design lock readiness

Required:
1. Tool contracts finalized (Epic B evidence set).
2. Label taxonomy + claim validation policy active (Epic A evidence set).
3. Spec matrix corrections applied for code reality.

## Phase 1 — CLI readiness

Required:
1. Export execution plan + validation plan complete (Epic C1).
2. Lineage/metacognition CLI surface and parity test plans complete (Epic C2/C3).
3. JSON schema registry + compatibility policy complete (Epic C4).

## Phase 2 — Tree/LI integration readiness

Required:
1. Branch replay scenario specs complete (Epic D1).
2. Conflict/idempotency specs complete (Epic D2).
3. Restore/compaction performance gate specs complete (Epic D3/D4).

## Readiness report schema

```json
{
  "phase": "0|1|2",
  "ready": true,
  "required_checks": [{"id":"string","pass":true,"evidence_refs":[]}],
  "blocking_items": []
}
```

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§11)
- docs/evidence/SPEC80_SECTION20_DECOMPOSITION_LANES_2026-04-21.md
