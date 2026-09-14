# Governed procedure compiler — #298 slice 0 (design)

Status: design (IR0). Multi-surface execution routing with human
handoffs and reusable workflow packs.

## Problem

Operational procedures (deploys, migrations, secret rotations) are
written ad hoc per surface. #298 requires ONE governed procedure
compiler: typed steps, effect classes, human-handoff gates, and
reusable packs.

## Canonical model

```
procedure := { procedure_id, version, steps: Step[], packs: PackRef[] }
step := {
  step_id, kind: shell|api|agent|human_handoff|approval,
  effect_class: none|local|external|destructive|financial|security,
  require_confirmation: bool, rollback_step: StepRef?,
  input: JsonSchema, output: JsonSchema
}
pack := { pack_id, procedures: ProcedureRef[], owner_ref, digest }
```

## Execution routing

- The compiler emits a deterministic execution plan: eligibility,
  confirmation gates (destructive/financial/security always confirm),
  rollback ordering (reverse execution of completed steps).
- Plans execute through the daemon; every step commits a dispatch row
  BEFORE execution (CallGraph #254 commit boundary reused).
- Human handoffs block the plan with a typed handoff receipt; the plan
  resumes only after the receipt lands.

## Reuse

- runtime_bundle (257) for pack distribution; bg jobs (311) for
  detached execution; direction operations (291) for steering;
  error envelope (261) for failures.

## Acceptance

1. A security-class step never executes without confirmation, on any
   surface.
2. Rollback replays in exact reverse order with receipts.
3. A pack runs identically on the CLI, Pi, and Cockpit (one compiler).
4. Procedure digests verify before execution.

## Slices (IR2+)

1. Core: Procedure/Step/Pack types + deterministic plan compiler +
   rollback ordering.
2. Ledger: procedure runs + step dispatches (callgraph_store pattern).
3. API/CLI: compile/preview/execute/resume-with-handoff-receipt.
4. Packs: digest-verified distribution via runtime_bundle.
