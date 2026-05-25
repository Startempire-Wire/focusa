# Strict Spec Gate CI Proof — 2026-05-25

Operator directive: use CI as the verification authority, not local-only gate results.

## Summary

Strict gate repairs are verified by GitHub Actions run `26400020443` on commit `44e3979`: `Spec Gates (strict)`, `Rust`, and `Menubar` all completed successfully.

## Repair sequence

- `bc66422` — isolated strict spec gate sessions between gate scripts in `scripts/ci/run-spec-gates.sh`.
- `ba1f676` — aligned `tests/continuous_pruning_test.sh` with token telemetry growth and async turn materialization.
- `44e3979` — fixed malformed JSON payload construction in `tests/pi_rpc_driver_contract_test.sh`.

## CI evidence

| GitHub Actions run | Commit | Result | Notes |
|---|---:|---|---|
| `26399309721` | `c18f42a` | failure | `channel_separation_test.sh` exposed active-session carryover from the prior gate. |
| `26399509648` | `bc66422` | failure | Session isolation fix worked; next failure moved to `continuous_pruning_test.sh`. |
| `26399809222` | `ba1f676` | failure | Continuous pruning gate passed; next failure moved to Pi RPC driver contract payload. |
| `26400020443` | `44e3979` | success | `Spec Gates (strict)`, `Rust`, and `Menubar` green. |

## Validation posture

GitHub Actions CI is the release authority for this slice. Local shell checks were used only as preflight before commit/push.
