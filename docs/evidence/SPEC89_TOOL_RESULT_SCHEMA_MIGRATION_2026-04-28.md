# Spec89 Shared FocusaToolResult Schema and Migration Strategy — 2026-04-28

Active bead: `focusa-bcyd.1.5`.

## Schema artifact

Canonical Phase 0 schema draft: `docs/contracts/focusa-tool-result-schema-v1.json`.

## Required common fields

- `ok`: boolean tool-call success after transport and semantic validation.
- `status`: `accepted`, `completed`, `no_op`, `blocked`, `validation_rejected`, `degraded`, `offline`, or `error`.
- `canonical`: true only when result is grounded in Focusa canonical daemon/reducer state.
- `degraded`: true for local fallback, partial projections, timeout fallback, or non-canonical packet.
- `summary`: bounded visible text for Pi/user display.
- `retry`: `{safe, posture, reason}` with explicit retry semantics.
- `side_effects`: array of changed state classes, e.g. `focus_state_write`, `workpoint_checkpoint`, `snapshot_created`.
- `evidence_refs`: stable handles/docs/test refs, never giant raw blobs.
- `next_tools`: suggested Focusa tool calls that advance recovery or verification.
- `error`: structured field/code/message/allowed-values details when rejected or blocked.
- `raw`: compatibility slot for existing API body while migration proceeds.

## Family migration notes

| Family | Migration rule |
|---|---|
| scratchpad | `canonical=false`, `degraded=false`, side effect `local_scratch_write`; next tool usually `focusa_decide` or relevant Focus State write. |
| focus_state | distinguish `offline`, `no_active_frame`, `validation_rejected`, `write_failed`; critical failed writes include scratchpad evidence ref. |
| work_loop | include writer claim, active writer, required header, retry posture, and next tool such as `focusa_work_loop_status`. |
| workpoint | preserve `canonical`, `workpoint_id`, `idempotent_replay`, and `next_step_hint`; degraded fallback must never promote silently. |
| tree/snapshot/lineage | include snapshot/CLT refs, restore risk, truncation/perf warnings, and safe next diff/restore helpers. |
| metacognition | include artifact ids, quality warnings, promotion eligibility, metric hints, and doctor next steps. |
| lineage intelligence | include bounded/truncated status, extracted candidates, provenance, and promotion posture. |

## Backward compatibility

Phase 1 should keep existing `content[0].text` visible summaries while adding `details.tool_result_v1` or equivalent common details. Existing callers that read legacy `details.response` keep working through the `raw` compatibility slot.

## Acceptance linkage

This schema covers status/canonical/degraded/retry/evidence/next_tools/errors and has migration notes for every tool family.
