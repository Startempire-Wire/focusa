# Focusa Silent Sessions Runbook

## Observe first

1. Call `focusa_silent_sessions` with `action=list` or exact `session_id` plus `action=tail`.
2. Confirm current `run_id`, `generation`, status, capabilities, and receipt posture.
3. Reopen only through daemon-native identity; do not normalize to tmux names.

## Mutate safely

1. Preflight the intended mutation.
2. Supply exact session/run/generation plus daemon-issued `approval_id` and unique `idempotency_key`.
3. Use `send`, `interrupt`, `pause`, `resume`, `restart`, or `kill` only for the confirmed generation.
4. Re-read status/receipt, then capture and link evidence to the active Workpoint.

## Recovery

- Stale generation: list/reopen and use the returned current tuple.
- Duplicate/retry uncertainty: query receipt before retrying.
- Daemon degradation: `focusa_tool_doctor`; do not fall back to raw shell control.
- Blocked work: checkpoint and select the next ready item rather than abandoning state.

## Done condition

Daemon status and receipt prove the intended lifecycle transition, with no duplicate mutation and no orphaned run.
