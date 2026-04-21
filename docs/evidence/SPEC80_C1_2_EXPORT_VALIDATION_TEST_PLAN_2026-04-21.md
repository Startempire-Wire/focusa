# SPEC80 C1.2 — Export Validation + Tests Plan

Date: 2026-04-21
Bead: `focusa-yro7.3.1.2`
Label: `documented-authority`

Purpose: define validation and test coverage required before export pipeline can be marked implemented.

## Test matrix by dataset type

| Dataset | Core assertions |
|---|---|
| SFT | honors `require_success`, `min_turns`; produces deterministic record order |
| Preference | enforces `min_delta`, `require_user_correction`; emits pair validity metadata |
| Contrastive | enforces `require_abandoned_branch`, `max_path_length`; outputs branch contrast pairs |
| Long-horizon | enforces `min_session_length`, `min_state_transitions`; outputs trajectory segments |

## Mode coverage

1. Dry-run mode
- no files written
- returns eligible/excluded counts
- returns exclusion reasons and sample schema preview
- stable JSON in `--json` mode

2. Write mode
- dataset file created at `--output`
- manifest emitted with thresholds/filters/version/timestamp
- stats include per-phase timings and record counts

## Negative-path coverage

- invalid dataset flags return `invalid_request` typed envelope.
- missing output path returns validation error.
- unsupported format returns validation error.
- permission-denied path returns typed denial (future capability policy integration).

## Determinism checks

- repeated run with same inputs and fixed data snapshot yields identical record_count and checksum.
- manifest filter payload equality across repeated runs.

## Regression harness placement

- CLI integration tests: `crates/focusa-cli/tests/export_*`
- API route tests: `crates/focusa-api/src/routes/training.rs` test module for `/v1/export/run`
- Core builder tests: `crates/focusa-core/src/training/export_*`

## Exit criteria

1. All four dataset types pass dry-run and write-mode suites.
2. Determinism checks pass on at least two repeated runs.
3. `GET /v1/export/status` can truthfully return `implemented: true`.

## Evidence citations
- docs/21-data-export-cli.md (§3-§6)
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§7, §10 Gate B)
- docs/evidence/SPEC80_C1_1_EXPORT_EXECUTION_ENGINE_PLAN_2026-04-21.md
