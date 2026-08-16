# `focusa_bg_status`

Instant single-query snapshot of the background job ledger. Never use in a polling loop.

## When to use

- Capability family: `bg`; namespace: `focusa.bg`.
- At-a-glance job state; the completion notification is the primary delivery path.

## Parameters and strict input schema

- `job_id` (optional; string): omit to list recent jobs.

## Output

Returns `focusa.tool_result.v1`; the completion envelope carries
`output_tail` (bounded 4KB) delivered to the front terminal via SSE.

## Anti-examples

- Polling the ledger in a loop (tail-is-sleep).
- Raw `setsid nohup … > log &` while the daemon is up.
