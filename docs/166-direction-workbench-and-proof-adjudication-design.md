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
