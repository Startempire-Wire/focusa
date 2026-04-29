# Focusa Diagnostics and State Hygiene Tools

Use diagnostics and hygiene tools when Focusa state feels stale, degraded, duplicated, writer-conflicted, or confusing. Hygiene is proposal-first and non-destructive.

Each tool below includes what it does, when to use it, and one concrete example. Examples use Pi tool-call style; adapt parameter syntax to the active harness.

## `focusa_tool_doctor`

First stop for Focusa tool readiness: daemon health, Workpoint continuity, state symptoms, and likely repair action.

Example:

```text
focusa_tool_doctor scope="workpoint"
```

## `focusa_state_hygiene_doctor`

Read-only stale/duplicate Focus State diagnostic.

Example:

```text
focusa_state_hygiene_doctor
```

## `focusa_state_hygiene_plan`

Creates a proposal-style cleanup plan without mutation.

Example:

```text
focusa_state_hygiene_plan reason="old next steps may be stale after release"
```

## `focusa_state_hygiene_apply`

Approval-gated, non-destructive apply placeholder. Blocks without approved=true.

Example:

```text
focusa_state_hygiene_apply approved=false reason="review plan first"
```
