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

## Plan identity v2 (#689)

The existing pure compiler hashes UTF-8 `focusa.procedure-plan.digest.v2`, one
zero byte, then `serde_json::to_vec(Procedure)` bytes. Typed struct field order
and ordered steps are part of this encoding. Every source field participates,
including schema, procedure/version, step kind/effect, explicit confirmation,
rollback and command. Output remains `sha256:<lowercase hex>`.

Golden input: `digest-fixture`, version1, one shell/local step `a`, command
`run a`, explicit confirmation false, no rollback. Its digest is
`sha256:5f068c6abb4a2d2f67231fcda8aa108e05cc31aa5da1b6922177fd0af6290cec`.
Unsupported procedure schemas remain rejected. Legacy plans remain readable and
untouched; their old unframed digest cannot qualify a recompiled v2 plan. Review
and recompile rather than silently treating legacy identity as equivalent.
Producer/public typed-consumer and legacy-read compatibility tests do not prove
persisted-plan migration or live dispatch admission; those remain later slices.

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
