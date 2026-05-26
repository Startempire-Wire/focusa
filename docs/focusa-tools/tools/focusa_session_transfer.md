# `focusa_session_transfer`

**Family:** `workpoint`  
**Label:** Session Transfer

## Purpose

Save or continue a long Focusa/Pi work session like a game-save without forking the Pi session. The wrapper composes project card, inferred Workpoint, Workpoint checkpoint/resume, and trajectory view.

## When to use

- Before leaving a long session: `action="save"`.
- In a fresh Pi session: `action="continue"`.
- To inspect transfer readiness: `action="status"`.
- When project scope is verified but no canonical Workpoint exists; the wrapper uses `inferred_workpoint_candidate` instead of punting inference to the operator.

## Parameters

- `action` — `save`, `continue`, or `status`.
- `project_root` — project root to transfer; defaults to Pi cwd/session cwd.
- `current_ask` — current resume/save intent.
- `mission` — optional save mission; defaults to current ask or inferred Workpoint mission.
- `next_action` — optional exact next action for save.
- `continuity_id` — optional logical continuity id; defaults to project continuity.

## Expected result

Returns a compact handoff with project root, continuity id, project-card run, inferred Workpoint candidate, optional save checkpoint, Workpoint resume packet, trajectory view, and an `operator_handoff.first_tool` command for the next Pi session.

## Example

Save:

```text
focusa_session_transfer action="save" project_root="/path/to/project" current_ask="Save current work for transfer"
```

Continue:

```text
focusa_session_transfer action="continue" project_root="/path/to/project"
```

## Contract summary

- Family: Workpoint.
- Side effects: `save` may checkpoint a Workpoint; `continue` and `status` are read/compose operations.
- Result envelope: `tool_result_v1` with status, canonical/degraded posture, side effects, and next tools.
- API routes composed: `GET /v1/project/card`, `POST /v1/workpoint/checkpoint`, `POST /v1/workpoint/resume`, `GET /v1/trajectory/view`.
- API routes: `POST /v1/project/session-transfer` persists save packets to `project_session_transfers.jsonl`.
- CLI commands: `focusa project session-transfer`.
- Parity: `full`.
- Core surface: Focusa session transfer save/continue wrapper.
- Contract source: `docs/current/focusa-tool-contracts.json`.
