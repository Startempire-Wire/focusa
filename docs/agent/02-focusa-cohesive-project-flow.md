# Focusa cohesive agent project flow

This is the agent routing guide for one daemon-owned journey, not a new authority, scheduler, plan store or permission system. Its owning contracts are [Spec143](../143-focusa-master-release-cycle-trajectory-genesis-flow-implementation-spec.md), [Spec158 identity](../spec158/01-identity-ownership-and-reducer.md), [Spec164](../164-workstream-rooted-canonical-runtime-design.md) and [issue #618](https://github.com/Startempire-Wire/focusa/issues/618). Native tools/equivalent supported adapters execute those contracts; documentation does not activate capabilities.

## Stable destination, corrected route

Retain the accepted high-level outcome, current operator activity and permission ceiling. Goal-seeking means making evidence-backed corrections to the route, not replacing the mission at each error. Discussion, preparation, implementation and delivery have distinct grants. A process question does not automatically cancel an engineering assignment; explicit steering does control the affected activity.

The connected journey is:

**Verify binding → Bootstrap if needed → Genesis if needed → linked Ladder/specification/tasks → exact Workpoint/frontier → Prepare → Act → Reconcile → Advance.**

Bootstrap/Genesis are conditional entry/recovery operations, not rituals repeated every turn. Existing valid artifacts and evidence satisfy unchanged stages. Full/Medium/Short are projections of the same records, not separate plans or lifecycle stages.

## Entry and transitions

| Condition | Owning action | Required exit / handoff |
|---|---|---|
| New session, project/worktree switch or uncertain binding | Project identity/verification and supported attachment resolution | Exact ScopeRef/ProjectRootKey, WorkstreamId, ContinuityId and applicable AttachmentKey; session/work-surface IDs remain temporal/presentation metadata. |
| Missing local project baseline | Bootstrap preview, then authorized apply/repair | Verified project anatomy, task/Git choices and receipt; no inferred remote, stack or secret. |
| Missing or interrupted committed intent-to-first-action journey | Genesis status and start/resume/commit as applicable | Confirmed HLT, bound specification/acceptance, distinct MLG/STG/waypoints, reconciled tasks, first Workpoint and readiness receipt. Preserve existing history; takeover requires its own authority. |
| Existing initialized project | Scoped Trajectory view and Workpoint resume | Current instruction, ancestry, assessed gap, bounded action and current dependency-valid frontier agree; old readiness is not current execution admission. |
| Changed evidence, operator correction or newly discovered dependency | Reconcile/Refine through existing owners | Versioned affected requirements/graph/Workpoint, invalidated proofs identified, independent valid branches preserved. |
| Ready bounded action | Admit and execute through its actual owner | Verified scope, grants, stage, dependencies, budget and required evidence; accepted dispatch is not completion. |
| Receipt, failure or uncertainty | Reconcile actual effects and consumer proof | Updated observed state/gap, acceptance disposition and owned recovery/refinement work. |
| Unfinished outcome | Advance from updated dependencies | Next ready action, owned resolution step, or a precise genuine stop with supported resume trigger. |
| Whole accepted outcome proven | Settle through its owning authority | Required obligations and relevant in-flight/uncertain effects reconciled; no invented work after completion. |

## Ladder and discovered work

Orientation: **Project → HLT → MLG → STG → waypoint → assessed gap → Workpoint → eligible frontier**. Worksets own admitted membership/acceptance, CallGraphs own dependencies/readiness/recovery, and Workstreams own durable cognitive continuity. Evidence connects results back to observed state and acceptance.

A necessary correction does not need to have been listed as a waypoint in advance. Derive the smallest justified resolution action, link it upward to the accepted outcome and downward to targets, dependencies, authority, evidence and material rollback. Validate and commit the affected route before consequential execution. Cross-cutting work may have multiple typed links; do not duplicate the task or expand scope merely because something is related.

A missing frontier means the next action is not admitted. It calls for discovery and supported reconciliation, not automatic mission abandonment and not an unmanaged shell bypass. Replanning cannot self-issue credentials, takeover authority, budget or deployment permission.

## Request-local resume contract

Normally omit `current_ask` in `focusa_workpoint_resume`: the adapter forwards the current instruction. If supplied, it must exactly match the captured current operator ask; do not summarize it. `resume_evaluated_different_ask` can be an input/provenance mismatch even when the daemon reply is canonical. Inspect this before inferring a broken mission or missing consent.

A matching resume still does not establish effect admission: require the exact scope/attachment, linked operation, lifecycle stage and frontier. A newer packet from another Workstream, a selected folder, bootstrap-ready label or successful status read cannot substitute for these checks.

After compaction/reload/model or project transitions, read back canonical scoped state through the actual active native adapter. Use current steering rather than an old next-slice mentioning completed work. Changed files or another process's reload do not prove this agent refreshed.

## Recovery without surrender or bypass

1. Classify the actual response: pending observation, reporting failure, input mismatch, stale binding, missing dependency/proof, authority denial, conflict, resource pressure or uncertain effect.
2. Preserve the parent outcome and confirmed results. Identify only the affected operation and its dependents; keep independently admitted work advancing.
3. Read the strict installed contract and advertised scoped repair/rehydration route. Inspect exact bindings, records and receipts; never invent operations or silently use a different daemon.
4. Reconcile possible effects before replay. Use the operation's retry policy; otherwise at most one safe retry before selecting a supported alternative. Deterministic unchanged errors are not transient retries.
5. Perform authorized recovery, verify its exit evidence, then resume the interrupted action automatically. A successful view does not itself prove the repair.
6. Ask only for an irreducible choice/consent outside existing authority. If mandatory authority genuinely remains unavailable and no permitted recovery or independent action remains, preserve a scoped checkpoint and state the missing fact and resume condition. Do not manufacture a green gate.

For coordination conflicts, preserve the other owner and inspect the explicit options; no automatic takeover. For missing native integration, restore it through the approved path; shell diagnostics do not replace native authority. Pending work is observed, not dispatched again. Snapshot restore, rollback, credential changes and release actions are conditional consequential operations, never mandatory steps in a skill's capability list.

### Legacy lifecycle rejection

For `WORKPOINT_FRONTIER_MISSING`, inspect the expanded daemon response's `workpoint_linkage.admission_gaps`, linkage status and lifecycle stage before accepting a generic scope-mismatch summary as the diagnosis. A linked legacy record with only `lifecycle_stage_missing` differs from a foreign or unknown Workstream binding. Discover the installed, explicitly confirmed append-only lifecycle repair if advertised; preserve the original record and never execute its stale next-slice just because structural repair succeeds. Then reconcile the current instruction and proposed successor.

Generic `--lifecycle-action` help is not proof that a particular command implements repair. If the installed version lacks the required operation, use the approved trusted release/update path; never hand-replace a daemon or invent a stage flag. Keep independently permitted preparation advancing and retain the exact missing capability as an owned resolution requirement.

## Tool and machine-response clarity

An agent-facing response needs: operation status; exact affected scope/revisions; evidence and uncertainty; permitted recovery; exact next operation with known arguments or the precise missing input; recovery exit check; and interrupted action to resume. It must distinguish proposal/support/activation, readiness/admission/dispatch/completion, and reporting failure/mandatory authority failure. Use the owning versioned schemas and descriptors; this guide defines no independent response envelope.

Inspect a bounded expanded response when compact output omits the required fields. Do not infer missing facts from a shortened rendering. A tool catalog is discovery metadata, not a mandatory execution chain. Do not promise a repair operation until installed capability discovery confirms it.

## Coherent publication and acceptance

Update the canonical skill registry/generator, maintained guide/index and owning tool descriptors together. Regenerate project/packaged skills and machine/human tool projections through their existing writers; preserve authored skills and byte-identical mirrors. Discover inventory counts rather than hard-coding them. Validate freshness/parity and links. Approved installation and native refresh are separate from source generation.

Prove the reduced loop with focused producer and real consumer checks, supported cross-version/harness cases and installed evidence. Scenarios: initialized/new/interrupted Genesis; compaction/reload; changed ask versus paraphrased override; changed worktree/attachment; newly discovered in-scope dependency; one failed branch with independent work; lost reply and restart; genuine consent/budget boundary; and partial delivery versus whole-outcome settlement.

Measure valid-work rejection, unnecessary questions/stops, recovery success, unauthorized effects, false completion and cost/latency across named supported agents. Static prose/parity tests alone are not behavioral proof, and unrun scenarios remain explicitly unverified.
