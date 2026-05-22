# `focusa_resource_mode`

**Family:** `diagnostics-hygiene`  
**Label:** Focusa Resource Mode

## Purpose

Read or control Focusa ResourceMode, including activating or deactivating `LowMem` mode when resources are constrained.

## When to use

Use this tool when the model recognizes low resources, daemon hot paths risk timeouts, safe audit reports memory pressure, or the operator says “Activate LowMem mode” / “Deactivate LowMem mode”.

## When not to use

Do not use this tool to hide or disable Focusa tools. LowMem keeps tools callable with bounded summaries, degraded envelopes, omitted counts, and rehydrate refs.

## Example usage

```text
focusa_resource_mode action="status"
focusa_resource_mode action="activate_lowmem" reason="RSS above soft budget"
focusa_resource_mode action="deactivate_lowmem" reason="operator requested normal auto mode"
focusa_resource_mode action="set_mode" mode="normal"
```

## Expected result

The tool returns the current mode, forced/auto status, pressure reason, LowMem budget, deferred cold surfaces, pruning order, and `next_tools`. Pi results include `details.tool_result_v1` with `status`, `failure_class`, `canonical`, `degraded`, `side_effects`, and recovery posture.

## Recovery notes

- `activate_lowmem` sets a runtime LowMem override immediately.
- `deactivate_lowmem` clears the runtime override back to auto; auto detection may still choose LowMem if pressure remains.
- To force normal behavior, use `action="set_mode" mode="normal"`.
- If the route is unavailable, use `focusa_tool_doctor` and `/v1/health`.

## Related tools

- [`focusa_tool_doctor`](./focusa_tool_doctor.md)
- [`focusa_trajectory_view`](./focusa_trajectory_view.md)
- [`focusa_workpoint_resume`](./focusa_workpoint_resume.md)

## Source

Defined in `apps/pi-extension/src/tools.ts` and `POST /v1/resource/mode`.
