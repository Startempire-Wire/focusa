# SPEC80 §20 Decomposition Lanes — 2026-04-21

Purpose: operationalize `docs/80-pi-tree-li-metacognition-tooling-spec.md` §20 into explicit BD lanes with ordering.

## 20.1 Functional gap lane mapping

| §20.1 row | Primary BD lane |
|---|---|
| Branch snapshot/restore API missing | `focusa-yro7.2.1.2` (B1.2 Snapshot/restore/diff contracts) + `focusa-yro7.4.2.2` |
| Metacognition API domain missing | `focusa-yro7.2.2.1` + `focusa-yro7.2.2.2` |
| CLI lineage parity gap | `focusa-yro7.3.2.1` + `focusa-yro7.3.2.2` |
| CLI metacognition parity gap | `focusa-yro7.3.3.1` + `focusa-yro7.3.3.2` |
| Export execution pipeline partial/stubbed | `focusa-yro7.3.1.1` + `focusa-yro7.3.1.2` |

## 20.2 Performance tuning lane mapping

| §20.2 path | Primary BD lane |
|---|---|
| Reflection + metacog latency budget | `focusa-yro7.4.3.1` |
| Snapshot/restore performance budget | `focusa-yro7.4.3.2` |
| CLI wrapper deterministic failover | `focusa-yro7.2.4.1` + `focusa-yro7.3.4.2` |
| Compaction serialization cost | `focusa-yro7.4.4.1` + `focusa-yro7.4.3.2` |

## 20.3 Full utilization proof mapping

| §20.3 criterion | Primary BD lane |
|---|---|
| Tree/lineage correctness | `focusa-yro7.4.1.1`, `focusa-yro7.4.1.2`, `focusa-yro7.4.2.1` |
| Tool-first metacognition loop | `focusa-yro7.2.2.1`, `focusa-yro7.2.2.2`, `focusa-yro7.5.3.1` |
| CLI/API parity | `focusa-yro7.3.2.*`, `focusa-yro7.3.3.*`, `focusa-yro7.3.4.*` |
| Outcome compounding evidence | `focusa-yro7.5.1.*`, `focusa-yro7.5.4.1` |
| Governance integrity | `focusa-yro7.6.3.1`, `focusa-yro7.6.3.2` |

## Required execution order (from §20.4)

1. Functional gaps first (Epic B/C lanes)
2. Performance gates second (Epic D lanes)
3. Utilization proof pack final (Epic E/F lanes)

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/evidence/SPEC80_LABEL_TAXONOMY_ENFORCEMENT_2026-04-21.md
- docs/evidence/SPEC80_CLAIM_VALIDATION_PROTOCOL_2026-04-21.md
