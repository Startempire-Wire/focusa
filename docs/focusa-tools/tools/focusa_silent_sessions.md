# `focusa_silent_sessions`

Daemon-native Spec133 Silent Session facade for Pi. The tool is a thin API client; the daemon owns canonical state, authorization, lifecycle, model selection, process supervision, persistence, recovery, and completion truth.

## Actions

Native actions: `list`, `preflight`, `watch`, `pause`, `resume`, `config`, `receipt`, `capabilities`.

Legacy compatibility mappings:

| Legacy action      | Daemon behavior                |
| ------------------ | ------------------------------ |
| `start`            | exact session start route      |
| `reopen`, `health` | canonical session projection   |
| `tail`             | bounded cursor output page     |
| `send`             | authenticated foreground input |
| `interrupt`        | controlled interrupt           |
| `restart`          | new exact run generation       |
| `kill`             | authorized cancel              |

`session_name` remains only as an exact `session_id` alias. It is never normalized as a tmux name. `command` remains only as an input-text alias and is never executed by a shell.

## Exact-target rules

Mutations require:

- `session_id`;
- `run_id`;
- `generation`;
- durable `approval_id`;
- `idempotency_key`.

Legacy `approved` and `force` booleans are compatibility hints only and never grant authority.

## Output and authority

Results use the daemon envelope and report `parity: full`, `authority: daemon`, canonical status, side effects, evidence/receipt references, and recovery guidance. Observation is bounded and cursor-based.

## Removed ownership

The Pi extension does not:

- create or control tmux sessions;
- write `/tmp` registries or logs;
- compose shell launch commands;
- select or verify models;
- supervise process trees;
- claim canonical health or recovery;
- infer mutation authority.

## Recovery

Use `focusa_tool_doctor` when daemon access fails. Refresh the exact session/run generation after stale-target responses. For ambiguous mutation delivery, inspect canonical state before retrying with the same idempotency key.
