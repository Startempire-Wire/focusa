# 192 — Fresh prepare checkpoint with an unbound saved predecessor

## Bounded outcome

Address the remaining issue660 checkpoint circularity within the existing seven-issue batch and all19 Spec187 requirements. This is a correction to the owning checkpoint/first-prepare path, not a new recovery platform, identity store or permission bypass.

Observed source mechanism: `workpoint.rs::checkpoint` selects any scoped Active/canonical saved record using `active_workpoint_for_scope`, then requires existing North Star mutation admission when that record exists. A canonical trajectory with no active Workpoint link therefore cannot replace an unbound saved record through this path. Existing lifecycle-stage repair deliberately requires an otherwise complete bound record and must remain narrow.

## Required boundaries

- Preserve the accepted canonical goal, immutable history, old record/proofs, project/continuity identity and other workstreams. A fresh checkpoint must not adopt the old mission or proof history.
- Reuse the existing first-checkpoint preparation owner only after the exact current canonical trajectory/ledger guard is verified and its missing link is genuinely `trajectory_workpoint_unbound`.
- Limit eligibility to an explicit fresh, fully scoped prepare checkpoint; bound, foreign, stale, ambiguous, unknown-goal or otherwise inadmissible contexts stay fail-closed. Existing lifecycle-transition and stage-repair rules remain intact.
- Do not touch writer claims/fencing tokens, take over a writer, fabricate identity, repair live client data, or turn caller-echoed identifiers into authority. If existing bootstrap authority cannot support this correction safely, preserve the refusal and identify the precise unresolved authority decision instead of adding an exception.
- No new API surface or authority capability is assumed. Code changes must stay at the existing checkpoint/predecessor and canonical trajectory-verification owners.

## Source proof and acceptance

Before implementation, produce a focused runtime regression for the unbound saved-predecessor case using isolated synthetic state and the actual checkpoint consumer. Do not count a compile failure as expected failure. Reuse existing test/state construction infrastructure rather than creating another service or controller.

Eligibility uses existing request fields only: explicit `operator_checkpoint`, fresh Workpoint/no supplied frame, explicit mission/next slice/work item/current turn, prepare stage, exact project-root target, and no submitted historical proofs. A candidate is not admission: the existing event-persistence owner verifies the goal and unchanged unbound linkage under its serialization lock and holds the existing writer-claims read guard through durable acknowledgement. A same-workstream live writer blocks recovery; other workstreams and expired claim records remain untouched. Existing writer-key bytes, fencing, normal bound-work admission, and stage-only repair stay owned by their existing implementations. There is no new API flag, identity, authority store or bootstrap capability.

The existing public materialization entry retains its signature and delegates to the single persistence implementation. Recovery failures are explicit (`WORKPOINT_PREPARE_SCOPE_REQUIRED`, `WORKPOINT_PREPARE_WRITER_ACTIVE`, `WORKPOINT_PREPARE_CONTEXT_CHANGED`); canonical goal verification retains its owning refusal payload.

The actual-consumer baseline1747 passed the normal first-checkpoint control and then reproduced `NORTH_STAR_ADMISSION_BLOCKED` / `trajectory_workpoint_unbound` with a saved predecessor, in137.925seconds with unchanged production checkpoint logic. QA1748 stopped on a test-vector mistake: `trajectory_id` is not the marker binding's `active_trajectory_id`; the corrected vector must alter the actual field. Neither a compile failure nor an ignored extension-field probe counts as a guard regression.

Then verify fresh prepare behavior, durable state/event consumption, unchanged history/proofs/leases and other scopes, all unsafe-case refusals including writer/linkage changes while waiting for the persistence lock, existing stage-only recovery, legacy compatibility, formatting and strict lint. Preserve exact source hashes and terminal receipts; run blocking work only through the approved isolated OVH route, serialized within the established memory cap.

Source QA1749 passed33 checkpoint tests and37 writer tests, including19 actual-consumer safety scenarios, formatting, strict API all-target Clippy and structural guards in229.590seconds. Exact five-file source hashes and the persisted-consumer results are recorded in proof1750. No installed or platform-release acceptance is implied.

The source-only task does not close issue660 or its production parent. Full signed installation and actual accepted scoped checkpoint/test-job proof remain required. No paid builds, partial release, quota change, client contact, lock/database deletion or unrelated service changes are authorized.
