# focusa recover command

Status: implemented MVP for `focusa-recover-cmd`.

## Why

Evaluator gap: when the daemon crashes or Pi loses Workpoint context, operators need one command that detects the state, proposes recovery, and surfaces a `recovery_hint` instead of leaving them to guess.

## Commands

```bash
focusa recover --dry-run --project-root <project-root> --continuity-id <continuity-id>
focusa recover --project-root <project-root> --continuity-id <continuity-id>
focusa --json recover --dry-run --project-root <project-root> --continuity-id <continuity-id>
```

## Behavior

- `focusa recover --dry-run` lists crashed state and proposed recovery without starting the daemon or mutating Workpoint state.
- If the daemon is unavailable, the envelope reports `daemon_unavailable_or_crashed`.
- Real recovery starts the daemon unless `--no-start-daemon` is set, reloads persisted state, then calls Workpoint resume for the last canonical Workpoint scoped by `project_root` + `continuity_id`.
- Output includes `proposed_recovery`, `workpoint_resume`, `recovery_hint`, and `next_tools`.

## Acceptance proof

- Implementation: `crates/focusa-cli/src/commands/recover.rs`
- CLI wiring: `crates/focusa-cli/src/main.rs`, `crates/focusa-cli/src/commands/mod.rs`
- Static guard: `tests/spec_recover_cmd_static_test.sh`

## Safety boundary

`project_root` is validated with the same unsafe-root guard as Workpoint commands. Broad roots such as `/root` are rejected before recovery actions.
