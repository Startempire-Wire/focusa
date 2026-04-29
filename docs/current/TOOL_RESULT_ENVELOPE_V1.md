# Tool Result Envelope v1

Current Focusa Pi tools preserve visible summaries and attach structured result metadata when available.

## Purpose

Agents should not parse prose to decide whether a Focusa tool succeeded. They should inspect structured fields such as status, retry posture, canonical/degraded flags, evidence refs, side effects, and next-tool hints.

## Common fields

- `ok` — boolean success indication when available.
- `status` — current result state such as completed, accepted, pending, blocked, unavailable, validation_rejected, or degraded.
- `canonical` — true when Focusa says the result is authoritative.
- `degraded` — true when output is a fallback or partial result.
- `summary` — short human-readable result.
- `retry` — retry posture and guidance.
- `side_effects` — whether state was read, written, linked, checkpointed, or left unchanged.
- `evidence_refs` — stable proof refs associated with the result.
- `next_tools` — recommended next Focusa tools.
- `error` — structured error details when applicable.
- `raw` — compatibility copy of the underlying response.

## Agent usage rule

Use `status`, `canonical`, `degraded`, `retry`, and `next_tools` for recovery decisions. Treat `canonical=false`, `degraded=true`, `pending`, or `blocked` as a recovery state, not as a final success.
