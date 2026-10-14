# Compaction Fallbacks

Pi owns native compaction and tactical summaries. Focusa prepares and verifies
bounded, scoped supplementary context; it does not replace the native compactor
or start a competing turn.

## Policy

The complete current operator instruction and action-critical restrictions are
not fallback fields: retain their verified values/references or explicitly block.
A goal/title/old mission never reconstructs missing input or grants fresh action.
Current steering and exact scope checks remain authoritative.

At startup, input captured during binding takes priority over saved input,
including empty/meta steering. Complete saved input is restored only after the
existing identity and continuity owners have been adopted. Recovered action
permissions are revoked until fresh daemon validation; saved purpose, references
and evidence remain available for orientation. Input changes use the same
request-binding revocation owner, including changes to an empty request.
Late startup health or project-identity replies are discarded if the existing
lifecycle generation ended or was replaced; those replies cannot read session
entries or apply project-binding effects for the superseded startup.

Native recovery anchors identify their saved sidecar with the existing
`anchorRevision`, `semanticDigest` and exact session/project identity. A newer
state receives its own reference; missing, corrupt or mismatched references
never silently recover another generation.

Qualified ancillary presentation may use related sources, not random filler:

Presentation fallback order (never execution authority):

1. Focus State slot value.
2. Canonical WorkpointResumePacket fields:
   - `mission`
   - `next_slice`
   - `project_root`
   - `session_id`
   - `active_object_refs`
   - `verification_records`
   - `blockers`
3. Pi bridge local shadow:
   - recent decisions
   - constraints
   - failures
4. Current captured input or a clearly labeled frame goal/title as context only.
   Neither one substitutes for the other.
5. Session metadata such as current project root.
6. Only if no related canonical source exists: explicit explanatory sentence such as `No open questions recorded by Focusa or Workpoint.`

## Current-request and prepared-artifact fidelity

The normal prepare/native/verify lifecycle retains the complete captured operator
input—including literal continuation and read-only review—and its actual source
turn. Input above the existing private-sidecar byte ceiling is rejected rather
than silently shortened. Display and native-anchor previews never substitute for
the complete instruction.

Preparation captures the real reducer version under the existing state/write
fences. Local deltas are compared with the exact scoped canonical Focus State:
already committed values are acknowledged, pending values block preparation, and
compaction applies no deltas. An immediate transaction in the existing packet
store gives identical epoch/request retries one durable artifact; changed input
for an existing epoch conflicts. No second ledger or scheduler is introduced.

Verification compares artifact digest, complete current request, exact scope and
scoped cognition fields. It rejects stale source changes, foreign scope, missing
identity, legacy partial responses and a changed cached projection. Canonical
Active status alone does not establish a current operation/frontier. The caller
rechecks current request/session/scope before and after asynchronous responses;
a later failed or superseding prepare cannot revive an earlier handoff.

The production transport declares the existing required capability, propagates
parent cancellation and does not automatically replay mutations after unknown
completion. Legacy peers that cannot prove the extended identity remain
advisory-only. Quarantine preserves fresh steering without adopting saved action
permission. Required request/restriction/receipt/rehydration meaning must fit the
one pressure budget or explicitly block; optional presentation loses room first.

`verification_scope` names prepared-artifact/current-request/scoped-source proof.
It is not evidence that a model consumed or obeyed the context. Model use and
actual installed behavior have separate acceptance evidence.

## Non-goals

- Do not hallucinate decisions, constraints, artifacts, or test results.
- Do not fill slots with unrelated repo facts.
- Do not emit bare `none` for cognitive summary fields.

## Operator-visible observability

Compaction must never look like a frozen conversation. While native compaction is active, the Pi status surface reports phase, elapsed seconds, context pressure, and attempt number; long attempts emit a bounded visible heartbeat. Retry notices include the bounded primary error and retry delay. Terminal, coordinator, and resume-context failures are shown in the UI as well as durable telemetry or console logs. Timers are cleared on completion, failure, compact reset, session start, and shutdown.

Pi owns queued operator input and native continuation after manual or automatic compaction. Focusa queues its hidden resume projection with explicit `deliverAs:"nextTurn", triggerTurn:false`; it never uses the default steering mode and never starts a competing post-compaction turn. Because Pi's API returns `void`, Focusa records `unknown_completion`, performs no blind retry, and relies on the next-turn context path. The harness offers a bounded receipt signal: a same-session next agent turn starts after queuing. That event acknowledges the queue/turn transition, not exact semantic consumption or obedience by a model. Focusa acknowledges that event with a same-session-scoped `agent_start` handler (pure `compactionDeliveryAckEligible` decision; only the session that queued the delivery may ack it) that flips `unknown_completion`/`deferred_to_next_turn` to `delivered` and writes a durable `focusa-compaction-delivery-acknowledged` entry. Settled deliveries (`delivered`, `superseded_by_operator`, `failed`) are never re-acknowledged. Operator text submitted during compaction therefore remains authoritative and flows into Pi's native queue. Use admitted background jobs and completion receipts for blocking verification so steering remains available; do not replace receipts with polling loops.

## Guard

```bash
node scripts/validate-compaction-fallbacks.mjs
```

This static guard fails if legacy bare `none` summary fallbacks return or if Workpoint/current-ask/session fallback hooks disappear.

## Extension replacement and tool registration

Pi tears down extension instances on reload, new/resume, and fork. The retiring
compaction coordinator must release registration ownership and its request
callback so the replacement registers all tools and commands. Duplicate loads
without shutdown remain suppressed. An actual in-flight attempt retains its
exclusion until its own terminal callback settles; retired callbacks must not
use stale session contexts. `npm run test:reload-tools` exercises native SDK
reload and checks the complete active tool set, not daemon liveness alone.
