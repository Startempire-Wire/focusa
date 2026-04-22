# SPEC86 Shared-Status Lifecycle Parity Audit Matrix

**Date:** 2026-04-21  
**Spec:** `docs/86-shared-status-lifecycle-parity-spec.md`  
**Epic:** `focusa-9uow`  
**Task:** `focusa-9uow.1`

## Canonical sources checked
- `STATUS_VOCABULARY` in `crates/focusa-api/src/routes/ontology.rs`
- Lifecycle baseline in `docs/70-shared-interfaces-statuses-and-lifecycle.md`
- Runtime projection `GET /v1/ontology/primitives` -> `status_vocabulary`
- CLI projection `focusa --json ontology primitives` -> `status_vocabulary`

## Observed counts
| Surface | Count |
|---|---:|
| Canonical status vocabulary | 12 |
| Runtime API status vocabulary | 12 |
| CLI status vocabulary | 12 |

## Gap matrix
| Gap ID | Category | Gap | Severity |
|---|---|---|---|
| S-G1 | Drift guard | Existing tests validate subset statuses only | P1 |
| S-G2 | Lifecycle parity proof | No exact-set parity artifact for status vocabulary across canonical/API/CLI | P1 |

## Remediation mapping
- `focusa-9uow.2`: rely on explicit ontology primitives projection path on CLI.
- `focusa-9uow.3`: add exact-set status vocabulary parity runtime test + evidence.
