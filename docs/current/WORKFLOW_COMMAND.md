# focusa workflow command

Status: implemented MVP for `focusa-workflow-cmd`.

## Why

Evaluator gap: agents lacked a canonical scaffold for using Focusa productively. `focusa workflow` gives fast, accurate templates instead of ad-hoc command guessing.

## Commands

```bash
focusa workflow list
focusa workflow list --json
focusa workflow show long-refactor
focusa workflow apply feature-add --project-root <project-root> --continuity-id <continuity-id>
```

## Templates

The command ships six templates:

1. `long-refactor`
2. `multi-session-resume`
3. `incident-response`
4. `agent-handoff`
5. `feature-add`
6. `doc-update`

Each template includes:

- `when_to_use`
- `expected_outcome`
- 5–7 commands
- `recovery_hint`

## Acceptance proof

- Implementation: `crates/focusa-cli/src/commands/workflow.rs`
- CLI wiring: `crates/focusa-cli/src/main.rs`, `crates/focusa-cli/src/commands/mod.rs`
- Static guard: `tests/spec_workflow_cmd_static_test.sh`

## Safety boundary

`focusa workflow apply` prints a paste-ready sequence. It does not execute destructive operations; operators/agents remain responsible for proof and checkpointing.
