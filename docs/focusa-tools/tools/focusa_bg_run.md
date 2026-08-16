# `focusa_bg_run`

Run a terminal-blocking command as a first-class background job; the completion notification (with bounded output tail) arrives on the agent front terminal. The canonical TBQ dispatch primitive.

## When to use

- Capability family: `bg`; namespace: `focusa.bg`.
- Any terminal-blocking query (builds, tests, migrations, scans).

## Parameters and strict input schema

- `name` (required; string): job name shown in the completion notification.\n- `command` (required; string): full command line.\n- `cwd` (optional; string): working directory override.

## Output

Returns `focusa.tool_result.v1`; the completion envelope carries
`output_tail` (bounded 4KB) delivered to the front terminal via SSE.

## Anti-examples

- Polling the ledger in a loop (tail-is-sleep).
- Raw `setsid nohup … > log &` while the daemon is up.
