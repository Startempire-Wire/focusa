# SPEC80 F2.1 — Critical Path Mapping Spec

Date: 2026-04-21
Bead: `focusa-yro7.6.2.1`
Label: `documented-authority`

Purpose: define mandatory execution dependency path across API, CLI, replay, and compounding lanes.

## Critical path order

1. Governance + contract lock (Epic A/B)
2. CLI parity planning and export closure plan (Epic C)
3. Branch correctness + performance gate plans (Epic D)
4. Outcome measurement + Gate D reporting plans (Epic E)
5. Program closure proofs and sign-off (Epic F)

## Dependency rules

- No branch correctness rollout claims before D1-D4 evidence exists.
- No compounding readiness claims before E1-E4 evidence exists.
- No full-utilization claim before F4 criteria verifier packet is complete.

## Path report schema

```json
{
  "path_id": "spec80_critical_path_v1",
  "ordered_nodes": ["A","B","C","D","E","F"],
  "blocking_dependencies": [{"from":"D","on":"C"}],
  "ready_for_next": true
}
```

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§11, §20.4)
- docs/evidence/SPEC80_SECTION20_DECOMPOSITION_LANES_2026-04-21.md
