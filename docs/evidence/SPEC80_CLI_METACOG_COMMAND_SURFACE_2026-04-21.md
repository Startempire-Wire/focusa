# SPEC80 CLI Metacognition Command Surface Design — 2026-04-21

Purpose: implement `focusa-yro7.3.3.1` by defining the first-class `focusa metacognition ...` command surface for planned API/tool parity in Spec80 Epic C.

## Authority
- docs/80-pi-tree-li-metacognition-tooling-spec.md (§7, §15, §20.1)
- docs/evidence/SPEC80_METACOG_TOOL_CONTRACTS_2026-04-21.md

## Command domain

Top-level domain:
- `focusa metacognition <subcommand> [flags] --json`

Required subcommands:
1. `capture`
2. `retrieve`
3. `reflect`
4. `adjust`
5. `evaluate`

## Command contract sketch

| Subcommand | Required args | Output contract (JSON) | Planned API binding | Permission |
|---|---|---|---|---|
| `capture` | `--kind`, `--content`, `--rationale?`, `--confidence?`, `--strategy-class?` | `{ capture_id, stored, linked_turn_id }` | `POST /v1/metacognition/capture` | `metacognition:write` |
| `retrieve` | `--current-ask`, `--scope-tag ...`, `--k` | `{ candidates:[...], ranked_by, retrieval_budget }` | `POST /v1/metacognition/retrieve` | `metacognition:read` |
| `reflect` | `--turn-range`, `--failure-class ...` | `{ reflection_id, hypotheses:[...], strategy_updates:[...] }` | `POST /v1/metacognition/reflect` | `metacognition:write` |
| `adjust` | `--reflection-id`, `--selected-update ...` | `{ adjustment_id, next_step_policy, expected_deltas }` | `POST /v1/metacognition/adjust` | `metacognition:write` |
| `evaluate` | `--adjustment-id`, `--observed-metric <k=v> ...` | `{ evaluation_id, delta_scorecard, promote_learning }` | `POST /v1/metacognition/evaluate` | `metacognition:write` |

## CLI readiness semantics

- `--json` is mandatory for machine-stable parity tests.
- Non-JSON mode may render human summaries, but JSON schema remains authoritative.
- Until `/v1/metacognition/*` exists, each command should return deterministic planned-extension envelope:
  - `{ status: "not_implemented", command, planned_api_path, reason, label: "planned-extension" }`.

## Non-equivalence guard

- Existing `focusa reflect ...` commands and `/v1/reflect/*` routes are adjacent but not equivalent to the metacognition domain contract.
- C3 parity closure must reference dedicated `metacognition` domain wiring, not reuse `reflect` as substitute.

## Evidence citations

- docs/80-pi-tree-li-metacognition-tooling-spec.md (§7, §15, §20.1)
- docs/evidence/SPEC80_METACOG_TOOL_CONTRACTS_2026-04-21.md
- crates/focusa-cli/src/commands/reflection.rs
- crates/focusa-api/src/routes/reflection.rs
