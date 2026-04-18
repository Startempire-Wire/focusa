# Doc 61 Truthful Core Cognition Substrate Frontier — 2026-04-16

Purpose:
- define the smallest **implementation** frontier for doc 61 that is truthful in runtime behavior
- prevent doc-61 closure from broad cognitive ontology prose without real consumers
- tie remaining work to executable slices under `focusa-5flh.*`

## Truthful baseline (already established)

Already grounded by existing decomposition artifacts:
- first real consumer path is Branch A routing in `apps/pi-extension/src/turns.ts` (`CURRENT_ASK`, `QUERY_SCOPE`, relevance/exclusion traces)
- doc-61 acceptance requires real consumers for primitives (`docs/BRANCH_ACCEPTANCE_CRITERIA_2026-04-13.md`)
- shared-substrate branch remains consumer/proof-gated (`docs/SHARED_SUBSTRATE_CONSUMER_PROOF_EXPANSION_2026-04-13.md`)

This frontier document defines what remains as code/test/evidence work.

## Remaining frontier slices (focusa-5flh children)

### S1 — Minimal canonical cognition primitives (`focusa-5flh.1`)
- Status: **extends existing substrate**
- Scope: keep only primitives required for decomposition, blocker handling, verification, and completion loops.
- Truth rule: reject speculative primitives that have no runtime consumer.
- Required evidence: primitive appears in canonical state/events **and** is consumed by a live route/loop path.

### S2 — BD traversal/decomposition binding (`focusa-5flh.2`)
- Status: **extends existing substrate**
- Scope: bind doc-61 primitives to actual work selection/handoff behavior in continuous execution paths.
- Truth rule: bead/decomposition structures must change runtime traversal behavior, not just documentation shape.
- Required evidence: daemon/route behavior shows primitive-driven selection or handoff decisions.

### S3 — Constraint/blocker policy control (`focusa-5flh.3`)
- Status: **extends existing substrate**
- Scope: ensure constraint/blocker primitives materially control continuation, fallback, and escalation.
- Truth rule: blocker classes must be policy-active, not passive labels.
- Required evidence: blocked/paused/escalated outcomes map to primitive states with observable status/checkpoint traces.

### S4 — End-to-end cognition-loop proof (`focusa-5flh.4`)
- Status: **new implementation surface (verification bundle)**
- Scope: integration tests proving decomposition → execution → verification → recovery → completion loop semantics.
- Truth rule: no doc-61 closure without runtime proof across the full loop, including blocker and recovery branches.
- Required evidence: executable tests with deterministic assertions against runtime APIs/events.

## Blocked vs executable now

Executable now:
- harden primitive definitions and route/daemon consumers that already exist
- add missing policy/test assertions that prove primitives alter behavior
- package proof surfaces so doc-61 closure is evidence-based

Blocked now:
- claiming doc-61 completion from vocabulary presence alone
- introducing broad cognition ontology surfaces without first consumer paths
- closing doc-61 while any S1-S4 slice lacks verified runtime evidence

## Closure standard for `focusa-5flh`

`focusa-5flh` can close only when all are true:
1. S1-S4 each have explicit implementation/test evidence (not narrative-only notes).
2. Every retained primitive has at least one named runtime consumer.
3. Integration proof covers blocker/recovery/completion, not only happy-path continuation.
4. Closure notes include a verified BD transition record and no unresolved verification blocker for S1-S4.
