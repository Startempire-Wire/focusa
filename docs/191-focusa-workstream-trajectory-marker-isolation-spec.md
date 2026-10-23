#191 — Workstream-qualified trajectory marker projections

## Outcome and scope

Repair issue695 inside the fixed seven-issue batch, retaining all19 Spec187 requirements. Existing Workstream trajectory records and semantic event ledgers remain authoritative; the project marker contains only derived compatibility projections. No identity, authority, lifecycle, operation, frontier, lease or attachment owner is replaced or weakened.

Source scope: API `project.rs`, the small `project_trajectory_marker.rs` boundary adapter, and `project_trajectory_workstream_tests.rs`. Existing atomic marker publication, backups, receipts and rollback remain at their current owners.

Confirmed `migrate`/`repair` operations acquire the existing project `BootstrapLock` before reading the shared marker and retain it through publication, ledger acknowledgement and any rollback. Contention refuses the operation without changing marker or ledger; no independent lock store or authority is introduced. `verify`, `preview` and unconfirmed mutations create no lock artifact. The held-transaction regression must fail on the predecessor, then pass with unchanged data during contention and both Workstream entries retained after separate accepted migrations. A missing body continuity uses the existing request `ScopeContext`; it never chooses another Workstream merely because its record is last in the project list. With multiple Workstreams and no qualifier in either location, the guard refuses canonical selection with `WORKSTREAM_SCOPE_REQUIRED`. Explicit body qualification and unqualified single-Workstream compatibility are retained.

## Versioned derived marker contract

The optional `workstream_trajectory_bindings` object has schema `focusa.project_workstream_bindings.v1` and an `entries` object keyed by the canonical trajectory record's continuity identifier within the already-verified Project. Each entry carries the exact `project_root`, `continuity_id`, and the unchanged existing `ProjectTrajectoryBindingRecord` / `TrajectoryIntegrityGuardRecord` representations. These are mirrors, never independent authority or caller-echoed proof.

- Reads select an exact scoped entry and reject malformed/unknown scoped schemas or conflicting root/continuity qualifiers. Missing scoped entries retain legacy comparison compatibility; a foreign singleton cannot supply the requested goal.
- Supported confirmed migration updates only the selected scoped projection, preserving other entries, shared identity, history and extension fields. Legacy top-level fields are populated only when both binding and guard are absent; existing or partial legacy slots remain unchanged, even when different Workstreams share a trajectory identifier.
- Stored-guard readiness uses the same scoped projection selector. Repeated verification/migration is a no-op when the canonical binding and its stored guard already agree, avoiding backup churn.
- Migration-event continuity is taken from the existing canonical record. No continuity or goal is invented, no ledger is rewritten, and semantic digests continue excluding guard/receipt attestations through the existing digest owner.

## Evidence and acceptance

Isolated baseline1741 reproduced the intended runtime assertion: a valid A scope entry was ignored in favor of B's singleton, yielding `MigrationRequired` instead of `Ready`; the production guard hash was unchanged. This is not a compile-error substitute.

Source checks must cover the focused reproduction, A/B migration and repetition, legacy compatibility, unknown schema/foreign qualifiers, cache tamper, missing canonical trajectory, extension preservation, formatting and strict lint. Run only on isolated OVH resources, serializing compilation within the verified memory cap and preserving terminal receipts.

Actual signed installed checkpoint and governed test-job receipts for otherwise-valid A and B are still required before issue acceptance. Source tests do not close issue695 or its production parent. No paid CI, partial release, client contact, live marker editing, database/lock deletion or unrelated service changes are authorized.

## Verified source checks — 2026-10-22

Final verification1743 passed52 project-module tests, explicitly requiring all five scoped isolation/migration/foreign-schema/tamper/shared-ID regressions to appear as passed. Formatting, strict API all-target Clippy with warnings denied and structural guards passed; total235.079seconds. Verified proof1744 reconciles the exact three source hashes.

The earlier broad `project_` filter passed78 existing checks but omitted the new workstream tests; it is not used as evidence that the focused regression became green. The final explicit module selection and required-name assertions close that source coverage gap. The unchanged-production baseline1741 remains the original expected-failure evidence; it was not redispatched.

Source verification is complete for this bounded repair; signed installed and accepted checkpoint/job outcomes remain pending.
