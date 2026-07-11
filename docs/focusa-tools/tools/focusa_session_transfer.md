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
- `write_preload` — request preload write guidance; defaults `false` and never writes implicitly.
- `preload_target` — `cursor`, `claude`, `codex`, `pi`, `opencode`, or `generic`; defaults `cursor`.
- `preload_mode` — delivery mode; defaults `session_transfer`.
- `receipt_preview` — include a bounded receipt preview; defaults `true`.
- `receipt_commit` — explicitly commit the receipt; defaults `false`.

## Expected result

Returns a compact handoff with project root, continuity id, project-card run, inferred Workpoint candidate, optional save checkpoint, Workpoint resume packet, trajectory view, preload/receipt status, and `operator_handoff` commands for continuation, preload, and receipt preview. Continue without a prior save is degraded and recommends `focusa_preload_build`.

### Return shape (Pi wrapper details)

The wrapper composes three sub-calls; the response details object exposes each as a distinct field so the operator can tell them apart:

| Field | Source | Shape |
|---|---|---|
| `api_transfer` | `POST /v1/project/session-transfer` body | the raw envelope with `action`, `saved`, `transfer`, `latest_prior_save`, `storage` |
| `session_transfer_save_packet` | `apiBody.transfer` (when `action="save"`) | the **game-save** packet: `transfer_id`, `mission`, `next_action`, `inferred_workpoint_candidate`, `operator_handoff` |
| `workpoint_checkpoint_packet` | `POST /v1/workpoint/checkpoint` body (when `action="save"`) | the typed Workpoint: `workpoint_id`, `status`, `canonical`, `mission`, `next_slice` |
| `workpoint_resume_packet` | `POST /v1/workpoint/resume` body (when `action="continue"`) | the resumed Workpoint: `workpoint_id`, `rendered_summary`, `canonical`, `next_step_hint` |
| `trajectory` | `GET /v1/trajectory/view` body | trajectory view (advisory) |
| `project_card` | `GET /v1/project/card` body | project identity + inferred workpoint candidate + trajectory report card + crosswire health + success sequence |
| `operator_handoff` | from `apiBody.transfer.operator_handoff` or default | `command` (e.g. `cd <root> && pi`) + `first_tool` (the next session-transfer call) + `authority_boundary` |

## Example

Save:

```text
focusa_session_transfer action="save" project_root="/path/to/project" current_ask="Save current work for transfer"
```

Continue:

```text
focusa_session_transfer action="continue" project_root="/path/to/project"
```

## Failure recovery

`tool_result_v1.failure_class` is part of the recovery contract. On `project_identity_unverified`, run `focusa_project_verify` before retrying. On `daemon_unavailable`, run `focusa_tool_doctor` and retry. On `workpoint_checkpoint_blocked` during save, drop to `focusa_workpoint_resume` to capture current packet and retry. For `continue` actions, treat `inferred_workpoint_candidate` as advisory and prefer canonical `focusa_workpoint_resume` when a verified Workpoint is required.

When `apiBody.transfer?.operator_handoff?.first_tool` is present, it is the canonical next call. Re-running it with the same `project_root` + `continuity_id` should round-trip the save intact.

## Contract summary

- Family: Workpoint.
- Side effects: `save` may checkpoint a Workpoint; `continue` and `status` are read/compose operations.
- Result envelope: `tool_result_v1` with status, canonical/degraded posture, side effects, and next tools.
- API routes composed: `GET /v1/project/card`, `POST /v1/workpoint/checkpoint`, `POST /v1/workpoint/resume`, `GET /v1/trajectory/view`.
- API routes: `POST /v1/project/session-transfer` persists save packets to `project_session_transfers.jsonl` (append-only, scope-bounded by `(project_root, continuity_id)`, replay-friendly).

## Direct API (bypassing the Pi wrapper)

The underlying HTTP route is `POST /v1/project/session-transfer`. It accepts:

```json
{
  "action": "save|continue|status",
  "project_root": "/path/to/project",
  "continuity_id": "focusa-cont-...",
  "current_ask": "optional intent",
  "mission": "optional save mission",
  "next_action": "optional next action"
}
```

Response (save):

```json
{
  "schema": "focusa.project_session_transfer_response.v1",
  "action": "save",
  "saved": true,
  "transfer": {
    "transfer_id": "019ea...",
    "schema": "focusa.project_session_transfer.v1",
    "action": "save",
    "mission": "...",
    "next_action": "...",
    "inferred_workpoint_candidate": {...},
    "checkpoint_payload_hint": {...},
    "operator_handoff": {"command": "cd <root> && pi", "first_tool": "focusa_session_transfer action=\"continue\" ...", "authority_boundary": "project_root_plus_continuity_id"}
  },
  "storage": {"transfers_path": "/path/to/data/project_session_transfers.jsonl"}
}
```

Response (continue) returns the **latest prior save** in the `transfer` field. The status action returns `saved: false` when there is no prior save.
- CLI commands: `focusa project session-transfer`.
- Parity: `full`.
- Core surface: Focusa session transfer save/continue wrapper.
- Contract source: `docs/current/focusa-tool-contracts.json`.
