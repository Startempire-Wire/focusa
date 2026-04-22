# SPEC85 Relation-Type Parity Audit Matrix

**Date:** 2026-04-21  
**Spec:** `docs/85-relation-type-parity-spec.md`  
**Epic:** `focusa-wkw5`  
**Task:** `focusa-wkw5.1`

## Canonical sources checked
- `LINK_TYPES` in `crates/focusa-api/src/routes/ontology.rs`
- Baseline required links in `docs/48-ontology-links-actions.md`
- Runtime projection `GET /v1/ontology/primitives`
- CLI projection `focusa --json ontology primitives`

## Observed counts
| Surface | Count |
|---|---:|
| Canonical link types | 74 |
| Unique canonical link types | 74 |
| Spec48 required minimum links | 14 |
| Spec48 required links missing from canonical | 0 |

## Gap matrix
| Gap ID | Category | Gap | Severity |
|---|---|---|---|
| R-G1 | Drift guard | No exact-set relation parity runtime test was present | P1 |
| R-G2 | Evidence path | No dedicated relation parity evidence artifact was present | P1 |

## Remediation mapping
- `focusa-wkw5.2`: leverage CLI ontology parity surface added in spec84 scope.
- `focusa-wkw5.3`: add exact-set relation parity runtime test + evidence doc.
