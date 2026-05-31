# Project Scope Override Incident and Guard Spec — 2026-05-31

Status: draft for iteration  
Owner: Focusa project  
Incident class: cross-project scope confusion / operator-declared project override missed  
Related docs: [`WORKPOINT_SESSION_SCOPE_GUARD.md`](./WORKPOINT_SESSION_SCOPE_GUARD.md), [`../69-scope-failure-and-relevance-tracing.md`](../69-scope-failure-and-relevance-tracing.md), [`PROJECT_INTELLIGENCE_FLYWHEEL.md`](./PROJECT_INTELLIGENCE_FLYWHEEL.md)

## Executive summary

During a compacted Pi session, Focusa preserved a canonical Workpoint under the Focusa repo scope (`project_root=/home/wirebot/focusa`, continuity `focusa-cont-root-8a64612b-d338-4eca-9e27-bb0e9d11c7f8`). The operator then explicitly corrected the scope: the active work was the PTM remote project, not the Focusa repo. The assistant still inspected Focusa-local state first, then only later verified the PTM remote project at `/home/planmarr/plan-the-marriage` on the remote host.

The failure was not that Focusa lacked a Workpoint. The failure was that the execution path treated `canonical=true` as sufficient for action even after the current ask contained a strong operator-declared project override. Focusa protected the stored Workpoint boundary, but it did not force a current-ask scope arbitration step before acting.

## Incident facts

- Preserved Workpoint scope: `/home/wirebot/focusa`.
- Operator-indicated active scope: PTM remote project, `/home/planmarr/plan-the-marriage`.
- Compaction packet correctly stated: Workpoint is canonical only when `project_root + continuity_id` match.
- The Workpoint mission itself contained the warning: “you are looking in the wrong place. This is the PTM remote project...”.
- The assistant still used the Focusa repo as the active action scope before rebinding to PTM.
- Existing guard docs already distinguish project root, continuity id, session id, and trajectory similarity, but they do not yet define current-ask scope arbitration when operator steering conflicts with a canonical packet.

## Expected behavior

When current operator text indicates the assistant is in the wrong project, Focusa should stop treating the current Workpoint as action-authoritative for the next step until scope is re-arbitrated.

Expected route:

1. Detect operator-declared project override in the current ask.
2. Compare it against the current Workpoint/project identity scope.
3. If conflict exists, mark the active packet as `canonical_for_saved_scope=true` but `action_authority_for_current_ask=false`.
4. Route to `focusa_project_verify` / `focusa_project_identity` with explicit local or remote project hints.
5. Checkpoint or transfer into the correct project scope before doing file, API, or evidence work.
6. Surface a concise operator-visible line: “Scope conflict detected: saved Workpoint is X; current ask indicates Y; rebinding before action.”

## Root cause analysis

### 1. Missing scope arbitration layer

Focusa had identity gates for Workpoint resume, but no separate gate that asks: “Does this canonical packet still answer the current operator-declared project?”

A packet can be canonical for its saved `project_root + continuity_id` and still be the wrong action anchor for a new or corrected operator ask.

### 2. Operator steering was advisory, not blocking

The system already treats operator steering as high priority in docs and tool guidance, but the runtime did not convert a project correction phrase into a blocking scope conflict.

Examples that should trigger blocking arbitration:

- “wrong place”
- “this is the PTM remote project”
- “use the remote server project”
- explicit project path/domain/host that differs from current Workpoint scope
- “not this repo/project”

### 3. Canonicality was overloaded

`canonical=true` currently means the Workpoint packet is a valid stored continuation packet for its scope. The assistant over-read that as “safe to act under this scope now.”

Needed distinction:

- `canonical_for_saved_scope`
- `matches_current_ask_scope`
- `action_authority_for_current_ask`

### 4. Remote project evidence was not elevated early enough

The PTM target was not only a semantic project name; it had durable remote evidence (`/home/planmarr/plan-the-marriage`, PTM docs, HLT ledger, auth files). Focusa should prefer explicit operator scope correction plus verifiable remote project evidence over a stale same-session local Focusa scope.

## Planned solution

### A. Add `CurrentAskScopeArbitration` to Pi Focus Slice generation

Add a small pre-action classifier in `apps/pi-extension/src/turns.ts` before Focus Slice assembly and before default Workpoint-resume guidance is rendered.

Proposed output block:

```text
CURRENT_ASK_SCOPE_VERDICT:
  status: aligned | override_candidate | conflict | unknown
  current_workpoint_project_root: /home/wirebot/focusa
  operator_indicated_project: PTM remote project
  operator_indicated_project_root: /home/planmarr/plan-the-marriage
  action_authority_for_current_ask: false
  required_next: focusa_project_verify -> focusa_project_identity -> focusa_workpoint_checkpoint/session_transfer
```

Rules:

- `aligned`: current ask has no competing project hint, or hints match current project identity.
- `override_candidate`: current ask names another project but lacks a verifiable root/host yet.
- `conflict`: current ask contains a project/path/domain/host that differs from current Workpoint scope.
- `unknown`: insufficient evidence; keep normal Workpoint route but warn if confidence is low.

### B. Split Workpoint authority labels

Extend Workpoint resume rendering with separate booleans:

```json
{
  "canonical_for_saved_scope": true,
  "matches_current_ask_scope": false,
  "action_authority_for_current_ask": false,
  "scope_conflict_reason": "operator_declared_different_project"
}
```

Do not change the stored Workpoint’s canonicality. The old packet remains valid for returning to Focusa work, but it is quarantined from action for the current PTM ask.

### C. Add operator-project override detector

Implement a bounded detector that extracts project hints from the current ask and recent operator correction text:

- explicit absolute paths (`/home/planmarr/plan-the-marriage`)
- domains / known project aliases (`PTM`, `Plan the Marriage`, `planmarr`)
- remote-host phrases (`remote project`, `remote server`, `client server`)
- correction phrases (`wrong place`, `not this`, `this is ...`)

Detector output should include confidence and evidence spans, not raw transcript blobs.

### D. Add rebind route

When `CurrentAskScopeArbitration.status=conflict`, the tool choreography should become:

```text
focusa_project_verify / focusa_project_identity
  -> focusa_session_transfer(action=continue) when an existing project transfer exists
  -> focusa_workpoint_checkpoint in the corrected project scope
  -> focusa_workpoint_resume for the corrected scope
```

If the corrected project is remote, accept remote evidence fields already supported by project identity tools (`remote_host`, `remote_user`, `remote_port`, `remote_repo_remote`, `remote_deploy_root`).

### E. Emit telemetry and learning hooks

Add events for review and regression evaluation:

- `project_scope_override_detected`
- `current_ask_scope_arbitrated`
- `scope_conflict_detected`
- `workpoint_action_authority_suppressed`
- `project_rebind_required`
- `project_rebind_verified`

These should feed the existing scope/relevance review path described in `docs/69-scope-failure-and-relevance-tracing.md`.

### F. Update operator-facing wording

When conflict is detected, the assistant should not continue silently. It should answer directly and act:

```text
Scope conflict: saved Workpoint is /home/wirebot/focusa; your current ask indicates PTM remote /home/planmarr/plan-the-marriage. Rebinding before action.
```

Avoid asking for permission unless the next operation is destructive or high-risk.

## Proposed tests

### Static tests

1. `tests/spec_project_scope_override_static_test.sh`
   - Assert Focus Slice / compaction text includes `CURRENT_ASK_SCOPE_VERDICT`.
   - Assert Workpoint packet uses `canonical_for_saved_scope` separately from `action_authority_for_current_ask`.

2. Extend `tests/scope_routing_regression_eval.sh`
   - Assert new telemetry events are accepted and queryable.
   - Assert `scope_conflict_detected` is distinguishable from generic `scope_mismatch`.

### Unit tests

1. Current ask: “you are looking in the wrong place; this is the PTM remote project” + saved Workpoint `/home/wirebot/focusa`.
   - Expected: `status=override_candidate` or `conflict`; `action_authority_for_current_ask=false`.

2. Current ask: “write an incident spec in the Focusa directory” + saved Workpoint `/home/wirebot/focusa`.
   - Expected: `status=aligned`; normal docs route allowed.

3. Current ask names a remote path `/home/planmarr/plan-the-marriage` while current cwd is `/root` or `/home/wirebot/focusa`.
   - Expected: project verify/rebind route first; no local Focusa file inspection as target action.

4. Same project, different continuity id.
   - Expected: existing continuity mismatch behavior still wins; no cross-session merge.

### Integration proof

A live proof should demonstrate:

- canonical Focusa Workpoint can remain valid for Focusa scope;
- PTM correction suppresses Focusa action authority;
- PTM project identity is verified from remote evidence;
- a new PTM-scoped Workpoint or session transfer becomes the action anchor;
- the operator-visible response explains the rebind in one or two lines.

## Acceptance criteria

- A canonical packet is never treated as action-authoritative when current operator text declares a conflicting project.
- Focus Slice exposes a current-ask scope verdict before Workpoint instructions.
- Workpoint resume rendering distinguishes saved-scope canonicality from current-ask action authority.
- Operator project corrections trigger bounded project verification/rebind before file/API work.
- Regression tests cover Focusa-local saved scope vs PTM remote current ask.
- Telemetry makes the event reviewable without raw transcript dependence.
- Docs explain the difference between “canonical Workpoint” and “right project for this ask.”

## Non-goals

- Do not weaken `project_root + continuity_id` as the saved Workpoint identity gate.
- Do not merge Focusa and PTM sessions because they share an operator or topic.
- Do not treat trajectory similarity as project authority.
- Do not store raw transcript text as durable evidence beyond bounded spans/summaries.

## Follow-up implementation beads

Suggested child beads:

1. Add current-ask project override detector in Pi extension.
2. Add Workpoint action-authority fields to resume packet rendering.
3. Add scope arbitration block to Focus Slice and compaction output.
4. Add telemetry events and regression tests for operator-declared project override.
5. Update Workpoint/project-identity docs with the canonicality vs action-authority distinction.

## Design decision

Focusa should treat canonical Workpoints as valid saved state, not unconditional action authority; current operator-declared project scope must be arbitrated before acting.
