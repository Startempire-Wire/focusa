# Focusa Work-loop Tools

Use work-loop tools for continuous execution state, writer ownership, preflight, and selecting next ready work without hijacking another writer.

Each tool below includes what it does, when to use it, and one concrete example. Examples use Pi tool-call style; adapt parameter syntax to the active harness.

## `focusa_work_loop_writer_status`

Read-only writer ownership and mutation preflight guidance. Use before any work-loop write when ownership is uncertain.

Example:

```text
focusa_work_loop_writer_status
```

## `focusa_work_loop_status`

Reads continuous work-loop state, budgets, active writer/task, and replay consumer status.

Example:

```text
focusa_work_loop_status
```

## `focusa_work_loop_control`

Controls work loop on/pause/resume/stop. Use preflight=true to avoid mutation and reveal route/writer first.

Example:

```text
focusa_work_loop_control action="pause" preflight=true reason="operator requested docs release"
```

## `focusa_work_loop_context`

Updates continuation decision context for loop decisions. Requires writer header in API path.

Example:

```text
focusa_work_loop_context current_ask="publish Focusa docs" ask_kind="instruction" scope_kind="mission_carryover" carryover_policy="allow_if_relevant"
```

## `focusa_work_loop_checkpoint`

Creates a manual loop checkpoint distinct from Workpoint checkpoint.

Example:

```text
focusa_work_loop_checkpoint summary="Docs release ready for validation."
```

## `focusa_work_loop_select_next`

Defers blocked work and selects next ready item.

Example:

```text
focusa_work_loop_select_next parent_work_item_id="focusa-xxxx"
```
