# `focusa_fast_forward`

Multiply parallel workloop-bound silent sessions (2x/4x/6x/8x…): deterministic FanoutPlan with an orchestrator lane (kind=agent, strong frontier refs) + worker lanes (kind=tool, weaker refs).

## When to use

- Capability family: `bg`; namespace: `focusa.bg`.
- Parallelizing a set of work items across session lanes.

## Parameters and strict input schema

- `multiplier` (required; number): 2, 4, 6, 8…\n- `work_items` (required; array of strings): work item refs.\n- `policy_max_turns_per_session` (optional; number): per-lane turn cap.

## Output

Returns `focusa.tool_result.v1`; the completion envelope carries
`output_tail` (bounded 4KB) delivered to the front terminal via SSE.

## Anti-examples

- Polling the ledger in a loop (tail-is-sleep).
- Raw `setsid nohup … > log &` while the daemon is up.
