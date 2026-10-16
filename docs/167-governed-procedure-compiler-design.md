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

## Core plan identity contract

`PROCEDURE_DIGEST_SCHEMA = focusa.procedure-plan.digest.v2` identifies
this encoding: SHA-256 over its UTF-8 bytes, one zero byte, then
`serde_json::to_vec(Procedure)`. Typed struct field order and step order
are part of the contract. The complete source includes schema, procedure
identity/version and every step's identity, kind, effect, explicit
confirmation, rollback reference and command. Output remains
`sha256:<lowercase hex>`.

Legacy concatenated-ID/command digests do not qualify v2 plans; callers
must recompile and review rather than reuse old approval/cache identity.
The procedure wire schema remains v1. Unsupported schemas remain rejected.
Consumers qualify received/persisted plans with `verify_plan(source, plan)`,
which compares the complete recompiled result, including safety/handoff and
rollback projections: retaining a correct digest cannot conceal altered
confirmation flags. Legacy plans still decode but fail v2 qualification.
This core correction does not implement dispatch, admit a run, or grant
execution authority. Runtime migration and cross-surface acceptance remain
required for the later slices below.

The independently generated test vector for procedure `digest-fixture`,
version 1, shell/local step `a`, confirmation false, null rollback and
command `run a` is
`sha256:5f068c6abb4a2d2f67231fcda8aa108e05cc31aa5da1b6922177fd0af6290cec`.

## Local compile/verify CLI contract

The source CLI exposes `procedure compile --definition <json-file>` and
`procedure verify --definition <json-file> --plan <preview-json-file>`.
These reuse the core compiler and verifier; they neither contact/start a
daemon nor dispatch work. Installed availability requires a release that
contains these commands; this document does not assert an installed binding.

With global `--json`, compilation returns `focusa.procedure_preview.v1`:
`schema`, `digest_schema`, `canonical:false`, `execution_authorized:false`,
and the typed `plan`. Verification consumes that same saved envelope,
requires its exact supported schemas and false authority claims, and emits
`focusa.procedure_verification.v1` with `verified:true` and the same
non-authority flags. Failure exits nonzero. Unsupported input fields,
blank identities/commands/rollback references and nonpositive versions fail
closed rather than silently dropping undeclared meaning.

This is integrity qualification, not resolution of arbitrary role, operation,
input/output or policy references. Native dispatch and all execution grants
remain outside these commands; content review cannot substitute for either.

## Slices (IR2+)

1. Core: Procedure/Step/Pack types + deterministic plan compiler +
   rollback ordering.
2. Ledger: procedure runs + step dispatches (callgraph_store pattern).
3. API/CLI: compile/preview/execute/resume-with-handoff-receipt.
4. Packs: digest-verified distribution via runtime_bundle.
