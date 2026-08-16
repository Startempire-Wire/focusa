# `focusa_bg_run_many`

Dispatch multiple independent terminal-blocking jobs in parallel; each delivers its own completion notification. The pipeline orchestration primitive.

## When to use

- Capability family: `bg`; namespace: `focusa.bg`.
- Independent long-running commands that can run in parallel.

## Parameters and strict input schema

- `jobs` (required; array): `{name, command, cwd?}` entries.

## Output

Returns `focusa.tool_result.v1`; the completion envelope carries
`output_tail` (bounded 4KB) delivered to the front terminal via SSE.

## Anti-examples

- Polling the ledger in a loop (tail-is-sleep).
- Raw `setsid nohup … > log &` while the daemon is up.
