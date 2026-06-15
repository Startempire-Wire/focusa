# `focusa_claim_preclose_gate`

**Family:** `diagnostics_hygiene`  
**Label:** Completion Claim Gate

## Purpose

Check a close/final-report claim against acceptance criteria and evidence so partial, surrogate, local-only, or blocked proof cannot be presented as completion.

## When to use

- Before `bd close`.
- Before a final report claiming completion.
- Whenever proof is API-only, web-only, local-only, blocked, or not the same platform/runtime required by acceptance criteria.

## Expected result

A `tool_result_v1` envelope with `focusa.completion_claim_gate.v1` raw payload:

- `decision`: `allow` or `block`
- `evidence_class`: `actual`, `partial`, `blocked`, or `missing`
- `missing_required_evidence`
- `overclaim_risks`
- `recovery_commands`

## Contract summary

- API: `POST /v1/claim/preclose`.
- CLI: `focusa claim preclose`.
- Side effects: read-only advisory gate.
- Core: `focusa_core::claim_gate::CompletionClaimGateReport`.
