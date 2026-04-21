# SPEC80 C1.2 — Export Validation + Tests Plan

Date: 2026-04-21
Bead: `focusa-yro7.3.1.2`
Purpose: define the validation strategy for export execution parity, covering dataset correctness, dry-run behavior, and typed error paths.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§7, §20.1)
- docs/21-data-export-cli.md
- crates/focusa-cli/src/commands/export.rs

## Validation scope

Dataset families under test:
- `sft`
- `preference`
- `contrastive`
- `long-horizon`

Validation dimensions:
1. **Contract shape**: JSON envelopes include required fields and stable keys.
2. **Correctness**: eligibility filters and dataset-specific flags affect output selection as expected.
3. **Dry-run parity**: `--dry-run --explain --json` never mutates output files and reports deterministic metadata.
4. **Execution mode**: non-dry-run path returns typed success/error envelopes (no bare string failures).
5. **Error-path typing**: invalid input/runtime failures return `{status:"error", code, reason, dataset_type}`.

## Test matrix

| Test lane | Cases | Expected invariant |
|---|---|---|
| Dry-run contract | one case per dataset family | `status` present; `dataset_type` matches invocation; `dataset_flags`/filters serialized |
| Schema stability | repeated runs over same fixture | required keys unchanged across runs |
| Success-path execution (post-E2+) | per family once enabled | output file exists; `records_written > 0` when eligible |
| Input validation | malformed flags/time ranges/thresholds | typed error envelope with deterministic `code` |
| Budget/resource failure | synthetic I/O/reader failures | typed error envelope, no partial silent success |

## Planned test assets

1. `tests/spec80_export_execution_engine_plan_test.sh` (existing planning contract guard)
2. `tests/spec80_export_validation_plan_test.sh` (this bead; plan coverage assertions)
3. `tests/cli_contract_regression_test.sh` extension points:
   - add per-dataset dry-run contract assertions
   - add execution-mode typed error assertions once pipeline lands

## Acceptance gates for C1 closure

- Gate C1-A: all dataset families represented in validation matrix.
- Gate C1-B: dry-run and execution-mode contracts explicitly separated.
- Gate C1-C: typed error envelope requirements documented and regression-testable.
- Gate C1-D: migration path from current `not_implemented` to execution envelopes documented.

## Migration note (current -> target)

Current behavior uses `status: not_implemented` payloads and explicit pipeline-not-implemented reasons.
Target behavior keeps dry-run introspection stable while introducing execution success/error envelopes with typed codes.

## Evidence citations
- docs/80-pi-tree-li-metacognition-tooling-spec.md
- docs/21-data-export-cli.md
- crates/focusa-cli/src/commands/export.rs
- tests/cli_contract_regression_test.sh
- docs/evidence/SPEC80_EXPORT_EXECUTION_ENGINE_PLAN_2026-04-21.md
