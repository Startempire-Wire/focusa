# Project Scope Override Incident and Guard Spec — 2026-05-31

Status: draft for iteration; updated after code/session evidence review
Owner: Focusa project
Incident class: cross-project scope confusion / operator-declared project override missed
Related docs: [`WORKPOINT_SESSION_SCOPE_GUARD.md`](./WORKPOINT_SESSION_SCOPE_GUARD.md), [`../69-scope-failure-and-relevance-tracing.md`](../69-scope-failure-and-relevance-tracing.md), [`PROJECT_INTELLIGENCE_FLYWHEEL.md`](./PROJECT_INTELLIGENCE_FLYWHEEL.md), [`FOCUSA_MODEL_VISIBLE_AWARENESS.md`](./FOCUSA_MODEL_VISIBLE_AWARENESS.md)

## Executive summary

During a compacted Pi session, Focusa preserved a canonical Workpoint under the Focusa repo scope (`project_root=/home/wirebot/focusa`, continuity `focusa-cont-root-8a64612b-d338-4eca-9e27-bb0e9d11c7f8`). The operator then explicitly corrected the scope: the active work was the PTM remote project, not the Focusa repo. The assistant still inspected Focusa-local state first, then only later verified the PTM remote project at `/home/planmarr/plan-the-marriage` on the remote host.

The failure was not that Focusa lacked memory. Focusa had a Workpoint, daemon state, Focus Slice context, telemetry paths, and the raw Pi session JSONL. The failure was that those substrates remained advisory/passive: no mandatory pre-action attention gate converted conflicting scope evidence into `action_authority=false`. Focusa protected the stored Workpoint boundary, but it did not decide whether that saved boundary was still the correct action target for the current ask.

## Incident facts

- Preserved Workpoint scope: `/home/wirebot/focusa`.
- Operator-indicated active scope: PTM remote project, `/home/planmarr/plan-the-marriage`.
- Compaction packet correctly stated: Workpoint is canonical only when `project_root + continuity_id` match.
- The Workpoint mission itself contained the warning: “you are looking in the wrong place. This is the PTM remote project...”.
- The assistant still used the Focusa repo as the active action scope before rebinding to PTM.
- Existing guard docs already distinguish project root, continuity id, session id, and trajectory similarity, but they do not yet define current-ask scope arbitration when operator steering conflicts with a canonical packet.
- Same Pi session history contained prior PTM work and the explicit correction; the failure happened inside one long-running session, not because state lived in a different chat.
- Daemon/primitives recorded context and scope telemetry, but the active pre-action path did not require a contradiction verdict before file/tool action.

## Evidence from current software and session review

Review date: 2026-05-31. Line numbers are local source positions from the review snapshot.

- `docs/current/FOCUSA_MODEL_VISIBLE_AWARENESS.md` states practical precedence is operator steering/current ask first, then identity prior, trajectory, and Workpoint. This is the intended policy.
- `apps/pi-extension/src/turns.ts:468` still injects Workpoint guidance as “authoritative continuation anchor unless the operator explicitly steers elsewhere,” but no structured scope-verdict block is emitted before the Workpoint section.
- `apps/pi-extension/src/turns.ts:664-668` orders Focus Slice sections as `CURRENT_ASK`, `QUERY_SCOPE`, trajectory, then Workpoint; this surfaces both signals but does not adjudicate contradictions between them.
- `apps/pi-extension/src/state.ts:396-420` classifies current ask as question/correction/instruction/meta and detects operator steering, but it does not extract project targets (`PTM`, `planmarr`, `/home/planmarr/plan-the-marriage`, remote host/domain hints).
- `apps/pi-extension/src/turns.ts:901-999` records current ask, query scope, and `steering_detected` telemetry; it does not set `action_authority=false` for conflicting project evidence.
- `apps/pi-extension/src/turns.ts:1091-1100` detects scope failure after assistant output; that is useful for measurement but too late to prevent wrong-project action.
- `apps/pi-extension/src/compaction.ts:141-166` refreshes a scoped Workpoint packet after compaction and accepts it when it matches the saved `project_root + continuity_id`; it does not compare packet scope to current-ask project hints.
- `apps/pi-extension/src/compaction.ts:276-309` formats WorkpointResumePacketV2 for prompt with `CANONICAL: true`, best-next tools, and authority boundary; it does not include current-ask action-authority status.
- Pi session JSONL around the incident shows the contradiction in one file: current ask stored “wrong place / PTM remote project” while `projectRoot` stayed `/home/wirebot/focusa`, then compaction re-injected the canonical Focusa Workpoint.
- Earlier same-session JSONL contains PTM remote evidence (`/home/planmarr/plan-the-marriage`, PTM docs/specs/audits, HLT ledger, auth work), but no hot-path primitive scanned it on the conflict phrase.

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

### 5. Same-session history was treated as a blob, not a project-switch ledger

The long Pi session held many project threads. Focusa can preserve that session, but preservation is not interpretation. A single Pi JSONL needs a bounded, queryable project-switch ledger that records when the active target moved between Focusa, PTM, ASAP, or other scopes.

Without that ledger, “same session” becomes misleading: the runtime can correctly preserve the session while still selecting the wrong project thread inside it.

### 6. Daemon state was observable but not authoritative at the perception/action boundary

The daemon and primitives can expose Workpoints, telemetry, predictions, metacog, and scope events. In this incident they did not fail by being absent; they failed by not being in the mandatory pre-action path.

Current behavior is closer to:

```text
retrieve preserved state -> inject context -> model notices or misses conflict
```

Needed behavior is:

```text
retrieve preserved state + current ask + session evidence -> compute contradiction verdict -> suppress or allow action
```

### 7. Scope failure detection is mostly post-turn measurement

Current scope-failure detection can emit `scope_verified`, `scope_contamination_detected`, and related telemetry after assistant output. That helps learn from mistakes, but it cannot reliably prevent the first wrong action in a turn.

The prevention point must move before Focus Slice finalization and before any tool call that reads/edits project files.

## Planned solution

### A0. Add an Attention Control Plane before memory injection

Focusa needs a small mandatory layer between state retrieval and model-visible action guidance.

Inputs:

- current operator ask after quoted Focusa context is stripped;
- active Workpoint and WorkpointResumePacketV2;
- current `S.sessionCwd`, `S.currentAsk.projectRoot`, continuity id, and session id;
- Focusa daemon Workpoint/trajectory/project identity state;
- bounded Pi session JSONL/project-switch index;
- known project aliases, paths, domains, remotes, and HLT ledgers.

Output:

```json
{
  "schema": "focusa.current_scope_verdict.v1",
  "status": "aligned | conflict | override_candidate | unknown",
  "saved_scope": {"project_root": "/home/wirebot/focusa", "continuity_id": "..."},
  "current_ask_scope": {"project_alias": "PTM", "project_root": "/home/planmarr/plan-the-marriage", "confidence": "high"},
  "contradiction": true,
  "workpoint_canonical_for_saved_scope": true,
  "workpoint_action_authority": false,
  "required_next": ["verify_current_project", "rebind_or_session_transfer", "checkpoint_correct_scope"],
  "evidence_spans": ["current_ask:PTM remote", "session_jsonl:/home/planmarr/plan-the-marriage", "remote_hlt:docs/HLT_LEDGER.md"]
}
```

This layer must run before Focus Slice Workpoint rendering, compaction resume instructions, and project-scoped tool execution.

### A1. Build a project-switch ledger from Pi session JSONL

Focusa should not cold-scan the full JSONL every turn. It should maintain a compact index:

```text
project_thread_observation:
  project_alias
  project_root
  remote_host/user/port when known
  evidence_ref
  first_seen_turn
  last_seen_turn
  recent_actions
  active_hlt_or_goal
  confidence
```

Triggers to update it:

- shell commands containing project paths/domains/remotes;
- Focusa project identity/session-transfer calls;
- HLT ledger reads/writes;
- Workpoint checkpoint/resume packets;
- operator text naming a project, client, domain, or remote target.

On conflict phrases, this ledger becomes hot-path evidence instead of raw transcript memory.

### A2. Promote semantic project conflicts into first-class Reflex Primitives

Existing `scope_mismatch` primitives are too API-result-oriented. Add a primitive for semantic conflict before tools fail:

```text
primitive_id: detect_semantic_project_scope_conflict
trigger: current ask names/negates a project that differs from active Workpoint scope
input: current ask, active Workpoint, project-switch ledger, ProjectIdentity candidates
output: CurrentScopeVerdict with action authority allowed/suppressed
```

This primitive should be usable by Pi, CLI, daemon, and any non-Pi adapter. It should not depend on the model noticing prose.

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
   - Assert Focus Slice ordering puts scope verdict before Workpoint continuation.

2. Extend `tests/scope_routing_regression_eval.sh`
   - Assert new telemetry events are accepted and queryable.
   - Assert `scope_conflict_detected` is distinguishable from generic `scope_mismatch`.

3. Add `tests/pi_session_project_switch_ledger_static_test.sh`
   - Assert Pi extension source contains a bounded project-switch/session-evidence substrate, not only `S.sessionCwd`.
   - Assert conflict phrases route through the ledger before Workpoint action authority is granted.

4. Add `tests/spec97_semantic_scope_conflict_primitive_static_test.sh`
   - Assert Reflex Primitive registry includes `detect_semantic_project_scope_conflict`.
   - Assert it outputs `CurrentScopeVerdict` and can suppress action authority without an API `scope_mismatch` first.

### Unit tests

1. Current ask: “you are looking in the wrong place; this is the PTM remote project” + saved Workpoint `/home/wirebot/focusa`.
   - Expected: `status=override_candidate` or `conflict`; `action_authority_for_current_ask=false`.

2. Current ask: “write an incident spec in the Focusa directory” + saved Workpoint `/home/wirebot/focusa`.
   - Expected: `status=aligned`; normal docs route allowed.

3. Current ask names a remote path `/home/planmarr/plan-the-marriage` while current cwd is `/root` or `/home/wirebot/focusa`.
   - Expected: project verify/rebind route first; no local Focusa file inspection as target action.

4. Same project, different continuity id.
   - Expected: existing continuity mismatch behavior still wins; no cross-session merge.

5. Long same-Pi-session replay containing Focusa and PTM events.
   - Input: saved Workpoint `/home/wirebot/focusa`, current ask “wrong place / PTM remote project,” session ledger has `/home/planmarr/plan-the-marriage` and `docs/HLT_LEDGER.md`.
   - Expected: PTM candidate outranks Focusa for current action; Focusa Workpoint remains canonical saved state but suppressed for action.

6. Operator asks to write a Focusa detour spec after the PTM incident.
   - Input: current ask explicitly says “in the Focusa directory”.
   - Expected: Focusa candidate regains current-action authority for doc work; PTM session remains preserved but not active.

### Integration proof

A live proof should demonstrate:

- canonical Focusa Workpoint can remain valid for Focusa scope;
- PTM correction suppresses Focusa action authority before any file/API action;
- project-switch ledger surfaces PTM evidence from the same Pi session without a full raw transcript scan;
- PTM project identity is verified from remote evidence;
- a new PTM-scoped Workpoint or session transfer becomes the action anchor;
- the operator-visible response explains the rebind in one or two lines.

## Acceptance criteria

- A canonical packet is never treated as action-authoritative when current operator text declares a conflicting project.
- Focus Slice exposes a current-ask scope verdict before Workpoint instructions.
- Workpoint resume rendering distinguishes saved-scope canonicality from current-ask action authority.
- Operator project corrections trigger bounded project verification/rebind before file/API work.
- Same-session project history is indexed as project-thread evidence, so “single Pi session” does not collapse multiple project scopes into one active target.
- Semantic project conflicts can suppress action before any API-level `scope_mismatch` occurs.
- Regression tests cover Focusa-local saved scope vs PTM remote current ask.
- Telemetry makes the event reviewable without raw transcript dependence.
- Docs explain the difference between “canonical Workpoint” and “right project for this ask.”
- Failure reports must cite evidence surfaces and rejected hypotheses; agreement with operator wording is not considered proof.

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
4. Build bounded Pi session project-switch ledger from JSONL/session entries.
5. Add semantic project-scope-conflict Reflex Primitive.
6. Add telemetry events and regression tests for operator-declared project override.
7. Update Workpoint/project-identity docs with the canonicality vs action-authority distinction.

## Review discipline

This spec must remain falsifiable. Future updates should include:

- exact code/doc/session evidence reviewed;
- which hypothesis was rejected;
- what invariant would have prevented the incident;
- which test proves prevention before tool/file action;
- no claims that “Focusa would prevent this” without a passing pre-action regression.

## Design decision

Focusa should treat canonical Workpoints as valid saved state, not unconditional action authority; current operator-declared project scope must be arbitrated before acting.
