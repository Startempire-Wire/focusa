# Authority Model

Status: current operational authority map
Sources: Spec 106 vocabulary; Specs 158/164 current Workstream-rooted identity; Spec143 and issue #618 connected project flow

Focusa has many cognitive and operational surfaces. They do not compete. Each surface has an explicit authority role and output posture.

## Core rule

```text
Operator steering wins.
Canonical cognition requires verified ScopeRef/ProjectRootKey + WorkstreamId + ContinuityId and applicable AttachmentKey; project_root + continuity_id alone are insufficient.
Advisory/degraded/stale output is never canonical continuation truth.
```

## Authority table

| Surface | Authority role | Canonical when | Non-canonical states |
| --- | --- | --- | --- |
| Operator Ask | Current intent authority | The operator explicitly steers the current ask | stale when superseded by newer ask |
| Operator Steering | Final override authority | Explicit current operator instruction | not applicable |
| ProjectIdentity | Project boundary / scope authority | Verified `project_root` and canonical project identity match | degraded, blocked, mismatch |
| Workstream ID | Durable cognitive workspace owner | Exact resolved Workstream within the verified Scope | missing, mismatch, foreign |
| Continuity ID | Continuation lineage within a Workstream | Matches the resolved Workstream and applicable Workpoint/Trajectory lineage | missing, mismatch, stale |
| Attachment Key | Runtime-to-Workstream binding | Verified current attachment for the operation | missing, stale, foreign |
| Session ID | Temporal runtime metadata only | Never canonical task/scope authority | advisory metadata |
| HLT | Durable north-star trajectory authority within verified Workstream scope and lineage | Operator-defined or durably superseded for exact scope | missing, generic-placeholder, stale, foreign |
| MLG | Strategic milestone derived from HLT | Derived from canonical HLT for exact scope | inferred, stale, degraded |
| STG | Bounded current goal derived from HLT/MLG/current context | Derived from canonical HLT/MLG and current context for exact scope | inferred, stale, degraded |
| Waypoints | Proof-bearing progress markers | Attached to canonical trajectory scope | advisory, stale |
| Workpoint | Canonical immediate continuation contract | Canonical packet matches exact Workstream/lineage, attachment and current intent; effect admission remains separate | unavailable, stale, degraded, rejected_scope_mismatch |
| Evidence Ref | Proof authority | Stable evidence handle/ref linked to scoped object/workpoint | missing, unverified, private/redacted |
| Focus State | Bounded current cognitive state | Reducer-backed current frame for verified scope | stale, scope-mismatch, read-model-lag |
| Focus Stack | Nested attention structure | Active frame belongs to verified scope | stale, empty, scope-mismatch |
| Context Cognition | Advisory bounded context packet | Never task authority; scoped packet can be trusted only as advisory context | advisory, degraded, stale, mismatch |
| Context Authority | Mutation-boundary allow/block/ask gate | Current preflight verdict for exact action/scope | verify_first, planning_only, diagnosis_only, stale |
| Project Card | Advisory bootstrap/re-bootstrap intelligence card | Never direct continuation authority | advisory, low-confidence, cross-project |
| Call Stack Design | Advisory/evidence-linkable implementation blueprint | Evidence only when explicitly attached | advisory, stale, drifted |
| Metacognition | Reusable learning loop | Promoted only after evaluated outcome | advisory, unevaluated |
| Prediction | Forecast/calibration signal | Promoted only after evaluated outcome | advisory, unevaluated, wrong |
| Work-loop | Governed execution state, writer-controlled | Current writer owns exact project/workstream/loop scope | paused, blocked, writer-conflict, global-telemetry-only |

## Required posture labels

Any tool/API/UI result that includes an authority-bearing surface should expose one or more of:

```text
canonical
advisory
degraded
blocked
stale
```

## Canonical continuation chain

```text
Operator Ask / Steering
  → verified ProjectIdentity + Workstream ID + Continuity ID + applicable Attachment
  → canonical HLT / MLG / STG / Waypoints
  → canonical Workpoint
  → Evidence Ref proof
```

Context Cognition, Project Card, Metacognition, Prediction, and Call Stack Design are valuable supporting surfaces, but they remain advisory until connected to canonical scope, Workpoint, Trajectory, or evidence through explicit reducer-backed paths.

## Scope invariant

```text
project_root + continuity_id = lookup/lineage inputs, not complete authority
ScopeRef/ProjectRootKey + WorkstreamId + ContinuityId + applicable AttachmentKey = canonical binding
session_id = temporal metadata
transcript tail = never authority
```

Matching `project_root + continuity_id` never suffices for merging canonical cognition: the exact Workstream/lineage, attachment and approved operation must also agree. Prior-project or cross-project context may appear only as advisory context with visible warnings.

### Project/worktree binding candidates

All authority surfaces use the core ranked binding decision. Precedence is explicit root → active Git worktree → marked current/ancestor root → verified persisted-session root → bounded marked child under a parent-directory launch. The decision retains `canonical_parent_root` and `active_worktree_root` separately. Equal-ranked project roots are `ambiguous_project_binding`: API and CLI expose the candidates, Pi `focusa_project_identity`/`focusa_project_verify` do not confirm them, and resumed sessions do not restore canonical state until one root is explicit. A persisted root may rebind across worktrees only when both candidates share the same canonical Git parent.

## Journey and recovery

Use [the cohesive project flow](../agent/02-focusa-cohesive-project-flow.md). Bootstrap/Genesis establish or resume initialization; valid existing state is reused. Workpoint/frontier admission governs effects, while evidence-backed reconciliation and advancement keep the accepted outcome moving. Diagnose a failed operation and perform supported scoped recovery; preserve genuine boundaries and other writers rather than pause the whole mission by default. No documentation or selected folder can fabricate admission.

## Mutation boundary

Risky mutation requires Context Authority preflight before action. Risky mutation includes daemon restart, deploy, release publish, git push, destructive file operation, database migration, broad refactor, cross-project file edit, generated-code overwrite, secret/config change, live service action, and pairing/install/update ambiguity.

Allowed verdicts:

```text
allow
block
ask_operator
verify_first
diagnosis_only
planning_only
```
