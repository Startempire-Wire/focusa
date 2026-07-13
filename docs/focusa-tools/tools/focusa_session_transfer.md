# `focusa_session_transfer`

**Family:** `workpoint`  
**Label:** Session Transfer

## Purpose

Save, continue, inspect, or Spec130-roll over a long Focusa/Pi work session using explicit typed source/target scope. The wrapper composes project card, inferred Workpoint, Workpoint checkpoint/resume, trajectory view, and transfer payloads without deriving continuity from a project fingerprint.

## Parameters

- `action` — `save`, `continue`, `status`, or `rollover`.
- `source_scope` — explicit typed source scope/workstream object: `root_path` or `project_root`, optional `scope_kind`, `scope_id`, `canonical_name`, `fingerprint`, and required/known `continuity_id`.
- `target_scope` — explicit typed target scope/workstream object for handoff/rollover.
- `target_continuity_id` — explicit rotated target continuity id when the target root is the source root.
- `source_session_id` / `target_session_id` — native Pi session ids before/after transfer.
- `checkpoint_ref`, `workpoint_packet_ref`, `compaction_packet_ref` — packet/checkpoint anchors for Spec130 rollover handoff.
- `rollover_action` — `none`, `inspect`, `checkpoint`, `migrate`, `resume`, `commit`, or `rollback`.
- `project_root` / `continuity_id` — deprecated source-scope convenience fields; prefer `source_scope`.
- `current_ask`, `mission`, `next_action`, `write_preload`, `preload_target`, `preload_mode`, `receipt_preview`, `receipt_commit` — transfer/preload options.

## Expected result

Returns compact details containing `source_scope`, `target_scope`, source/target session ids, packet refs, rollover action, optional save checkpoint, target Workpoint resume packet, target trajectory view, and `operator_handoff` commands.

Continuity rules:

- `continuity_id` is workstream metadata under typed root scope.
- `target_continuity_id` must be explicit for rotating continuity.
- The Pi wrapper must not derive continuity from project fingerprint.

## Example

```text
focusa_session_transfer action="rollover" \
  source_scope='{"root_path":"/path/to/project","continuity_id":"cont-old"}' \
  target_continuity_id="cont-new" \
  source_session_id="pi-old" target_session_id="pi-new" \
  compaction_packet_ref="packet:123" rollover_action="checkpoint"
```

## Contract summary

- Side effects: `save` or `rollover_action="checkpoint"` may checkpoint a Workpoint; `continue`/`status` are read/compose operations.
- API routes composed: `POST /v1/project/session-transfer`, `GET /v1/project/card`, `POST /v1/workpoint/checkpoint`, `POST /v1/workpoint/resume`, `GET /v1/trajectory/view`.
- Scope contract: explicit source/target typed scopes; no continuity fingerprint fallback.
