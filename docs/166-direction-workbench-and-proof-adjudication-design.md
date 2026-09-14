# Direction Workbench + proof adjudication design — #291 slice 0 (spec)

Status: design (IR0). Implementation slices follow this document.
Parent: #290/#291 (operator steering, proof adjudication, decision review).

## Problem

Operator steering, proof adjudication, and agent decision review are
scattered across surfaces (beads CLI, Mission Canvas, Pi slash commands,
Cockpit). Each reconstructs meaning from its own labels. #291 requires
ONE typed Direction Workbench where every steering/adjudication decision
is a typed, evidence-bound operation.

## Canonical operations (typed, idempotent)

| Operation | Payload | Evidence bound |
| --- | --- | --- |
| steer | {target_ref, direction, rationale, scope} | direction provenance ref |
| adjudicate | {claim_ref, verdict, adjudicator_ref} | completion_claim verdict (276) |
| review_decision | {decision_ref, outcome, feedback} | focusa_decide record |
| approve | {proposal_ref, approved_by} | operator authority ref |
| reject | {proposal_ref, reason, recovery} | rejection reasons |

Every operation lands as a FocusaEvent with a typed envelope + receipt.
The Workbench is a projection of the event ledger — never a second
store. Reuse: completion_authority (276), claim_gate (263),
capability_truth (279), background_jobs receipts (311-family).

## Typed ownership and legacy disposition — implementation gate

This design remains IR0; the existing SQL prototype and passing source tests do
not establish canonical operation authority or installed acceptance. The owning
foundations are [Spec 98](98-project-root-crdt-reconciliation-foundation-spec.md),
[Spec 104](104-typed-scoped-runtime-and-singleton-elimination-spec.md), and the
[capability permissions model](25-capability-permissions.md).

- Every new canonical operation/receipt requires a verified `WorkstreamKey`
  (`ScopeRef` plus `continuity_id`) and authenticated actor provenance. Reuse
  `scoped_state` types rather than introducing a second identity model.
- `Steer.scope` describes operation scope; its free text is **not** project or
  workstream ownership. Target labels, cwd, current selection and continuity
  alone cannot supply missing root authority.
- Request scope selection and capability permission are separate checks. Reuse
  the authentication, route-scope and entitlement owners; caller-supplied
  permission labels cannot substitute for verified grants.
- Bind the receipt to the canonical typed event/reducer path before treating
  its SQL projection as canonical authority. Do not create a parallel ledger
  or grant execution/release power through a steering receipt.
- Version the ownership-bearing envelope and prove cross-version consumption.
  Retain legacy receipt bytes as non-authoritative observations until an
  evidence-backed association is separately approved through the owning
  migration path. Never stamp old rows with the current caller's scope or
  silently rewrite history.
- Corrupt records remain explicit errors, not invented decisions or empty
  success. Missing ownership is a distinct blocked/quarantined disposition,
  not proof that the requested project has no steering.

Before scoped bounded queries or HTTP registration, acceptance must cover exact
root/workstream matching, foreign and incomplete identities, insufficient
permissions, legacy/unresolved ownership, replay/idempotency, corruption,
read-only observation and bounds applied **after scoped selection**. Cross-harness
and installed evidence remain required; source-only tests do not close #291.

## Workbench surface composition (#290/#284)

- Mission Canvas: direction lane bound to the active Workpoint.
- Pi: `focusa_direction steer/adjudicate/review` tools (strict schemas).
- Cockpit: the same operations rendered in the operator plane.
- Export: direction decisions in the CallGraph evidence envelope (289).

## Acceptance criteria

1. A steer operation appears on every surface with identical
   target_ref/evidence refs (no per-surface reconstruction).
2. Adjudication consumes the deterministic completion verdict (276);
   overrides must name the overridden atom + reason.
3. Every operation has a receipt; replay reproduces the ledger.
4. Review decisions reference the exact decision record; no free-text
   only paths.

## Slices (IR2+)

1. Core: DirectionOperation types + reducer arms (log + state where
   durable: steer sets work_loop direction context).
2. Ledger: direction_operations table + receipts (background_jobs
   pattern).
3. API: POST /v1/direction/{steer,adjudicate,review} + GET projection.
4. Pi tools + Mission Canvas lane (#290).
5. Export/evidence binding (#289 envelope).
