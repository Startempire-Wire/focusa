# SPEC84 Action-Type Parity Audit Matrix

**Date:** 2026-04-21  
**Spec:** `docs/84-action-type-parity-spec.md`  
**Epic:** `focusa-lpim`  
**Task:** `focusa-lpim.1`

## Canonical sources checked
- Ontology constants: `crates/focusa-api/src/routes/ontology.rs` (`ACTION_TYPES`)
- Ontology spec baseline: `docs/48-ontology-links-actions.md`
- Runtime projection: `GET /v1/ontology/primitives`
- CLI surface: `crates/focusa-cli/src/main.rs` command registry
- Existing tests: `tests/ontology_world_contract_test.sh`, `tests/ontology_affordance_schema_contract_test.sh`, `tests/doc70_shared_interfaces_lifecycle_contract_test.sh`

## Repro commands
```bash
python3 extract_action_matrix.py   # ad-hoc extraction (run in-shell snippet)
curl -sS http://127.0.0.1:8787/v1/ontology/primitives | jq '{action_types_count:(.action_types|length)}'
```

## Observed counts
| Surface | Count | Notes |
|---|---:|---|
| `ACTION_TYPES` constant | 92 | Canonical runtime list |
| Spec48 required minimum list | 10 | All present in canonical list |
| `/v1/ontology/primitives` action_types | 92 | Matches canonical count |
| Focusa CLI ontology command surface | 0 | No ontology command exposed in CLI |

## Gap matrix
| Gap ID | Category | Source of truth | Current exposure | Gap | Severity |
|---|---|---|---|---|---|
| A-G1 | CLI parity | `ACTION_TYPES` (92) | No CLI ontology primitives/world command | CLI cannot project action vocab for parity checks | P1 |
| A-G2 | Test depth | `ACTION_TYPES` (92) | Tests assert selected subsets only | Missing exact-set parity enforcement test | P1 |
| A-G3 | Drift guard | `ACTION_TYPES` | No generated manifest/snapshot gate | Runtime/API drift could pass unnoticed if subset tests remain green | P1 |

## Proposed remediation mapping
- `focusa-lpim.2`: add CLI ontology parity surface + shared source projection path.
- `focusa-lpim.3`: add exact-set parity contract tests and evidence artifact.

## Exit criteria for A1
- Canonical/source/exposure matrix documented.
- Concrete, scoped remediation items identified and linked to child tasks.
