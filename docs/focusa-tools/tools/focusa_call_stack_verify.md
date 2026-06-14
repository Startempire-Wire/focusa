# `focusa_call_stack_verify`

**Family:** `workpoint`
**Label:** Call Stack Verify

## Purpose

Verify a saved Call Stack Design against bounded implementation surfaces and report drift before or during implementation. This is advisory only: it never mutates Focus State, Workpoints, Trajectory, code, or ledgers.

## When to use

- After `focusa_call_stack_design` created a design.
- Before reviewing/continuing implementation of a designed feature.
- When checking whether entry surface, handlers, services, adapters, storage, output envelope, evidence, or Workpoint/STG alignment drifted.

## Parameters

- `project_root` — project scope. Defaults to Pi session cwd.
- `continuity_id` — optional continuity scope filter.
- `design_id` — specific Call Stack Design id to verify.
- `entry_name` — entry name to verify when `design_id` is omitted; latest matching design is used.

## Expected result

Returns `tool_result_v1` with `ok`, `advisory=true`, `canonical=false`, plus:

- `design_id`
- `entry_surface`
- `entry_name`
- `drift_status` (`aligned`, `needs_review`, or `drifted`)
- `failures`, `warnings`
- `checks[]` with `id`, `status`, and bounded `message`
- `rehydrate_id`

## Drift checks

The verifier checks:

- requested `project_root + continuity_id` scope matches the design when supplied
- entry surface kind is supported
- entry surface string exists in bounded source search
- handlers are declared
- services are declared
- adapters are declared
- output envelope is `tool_result_v1`
- claimed evidence refs are flagged for follow-up verification
- Workpoint attachment intent is present or flagged as advisory review

## Failure recovery

`tool_result_v1.failure_class` is part of the recovery contract. Common values:

- `project_root_missing` — provide an explicit `project_root` and retry.
- `scope_mismatch` — project root is an agent runtime path; pick a real project folder.
- `call_stack_design_not_found` — call `focusa_call_stack_design` or pass the correct `design_id`/`entry_name`.
- `storage_unreadable` — inspect daemon logs and ledger permissions.
- `daemon_unavailable` — run `focusa_tool_doctor` and retry.

## Contract summary

- Family: `workpoint`
- Side effects: none
- Result envelope: `tool_result_v1`
- API route: `POST /v1/call-stack/verify`
- Core surface: Spec106/Spec103 advisory Call Stack drift checker
- Contract source: `docs/current/focusa-tool-contracts.json`

## Next tools

- `focusa_call_stack_design` — create/refine a design when drift is real.
- `focusa_workpoint_link_evidence` — attach verification evidence.
- `focusa_trajectory_assess` — reassess STG alignment after drift resolution.
