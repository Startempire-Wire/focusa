# Model Forgetting / Scope Override Incident and Attention Guard Spec — 2026-05-31

Status: implementation-backed guard spec; live integration proof pending
Owner: Focusa project
Incident class: retrieved-memory attention failure; project-scope override is one subtype
Related docs: [`WORKPOINT_SESSION_SCOPE_GUARD.md`](./WORKPOINT_SESSION_SCOPE_GUARD.md), [`DETOUR_SCOPE_ASSURANCE_AND_IMPLEMENTATION_STATUS_2026-05-29.md`](./DETOUR_SCOPE_ASSURANCE_AND_IMPLEMENTATION_STATUS_2026-05-29.md), [`../69-scope-failure-and-relevance-tracing.md`](../69-scope-failure-and-relevance-tracing.md), [`PROJECT_INTELLIGENCE_FLYWHEEL.md`](./PROJECT_INTELLIGENCE_FLYWHEEL.md), [`FOCUSA_MODEL_VISIBLE_AWARENESS.md`](./FOCUSA_MODEL_VISIBLE_AWARENESS.md)

Note: the file name is retained for continuity, but the parent failure class is broader than project scope.

## Executive summary

During a compacted Pi session, Focusa preserved a canonical Workpoint under the Focusa repo scope (`project_root=/home/wirebot/focusa`, continuity `focusa-cont-root-8a64612b-d338-4eca-9e27-bb0e9d11c7f8`). The visible symptom was cross-project action: the operator corrected that active work was the PTM remote project, yet the assistant inspected Focusa-local state before rebinding.

The parent failure was model forgetting / attention loss, not absence of memory. Focusa had a Workpoint, daemon state, Focus Slice context, telemetry paths, raw Pi session JSONL, and a Workpoint mission containing the warning. Those substrates were retrieved or retrievable, but they were not forced into model attention, checked for use, or preserved as a short visible anchor after compaction and tool-output flood.

The required guard is an anti-forgetting attention layer: retrieve preserved state + latest operator correction + pinned task/report summary + bounded session evidence, compute an attention/recall verdict, then allow or suppress action and force a concise recap when the model is likely to lose the thread. Project-scope arbitration remains a critical subtype of that broader layer.

## Historical accuracy / prior work boundary

This incident spec is an additive follow-up, not the first Focusa scope-protection design. Before this incident, the detour scope-assurance work had already implemented and verified broad-root quarantine, ProjectIdentity quorum, verified in-session identity preservation, `project_root + continuity_id` Workpoint authority, canonical/advisory transparency, and deterministic scope recovery. Those prior assurances are documented in [`DETOUR_SCOPE_ASSURANCE_AND_IMPLEMENTATION_STATUS_2026-05-29.md`](./DETOUR_SCOPE_ASSURANCE_AND_IMPLEMENTATION_STATUS_2026-05-29.md) and the closed `focusa-khm6` bead family.

The new failure was not that these substrate features were absent. The failure was that existing saved-scope and post-output safeguards did not force the model to attend to the latest operator correction/report summary before action. Therefore implementation must extend the existing Focus Slice, compaction, WorkpointResumePacketV2, current-ask/query-scope, Reflex, telemetry, and evidence-handle surfaces rather than building parallel scope or memory systems.

The follow-up decomposition for this incident is tracked by `focusa-yv8d`; its anti-duplication notes require reuse of existing substrates and explicitly classify prior scope-assurance features as historical prerequisites.

## Incident facts

- Preserved Workpoint scope: `/home/wirebot/focusa`.
- Operator-indicated active scope: PTM remote project, `/home/planmarr/plan-the-marriage`.
- Parent failure: a remembered correction existed but was not forced into model attention before action.
- Compaction packet correctly stated: Workpoint is canonical only when `project_root + continuity_id` match.
- The Workpoint mission itself contained the warning: “you are looking in the wrong place. This is the PTM remote project...”.
- The assistant still used the Focusa repo as the active action scope before rebinding to PTM.
- After a later compaction, a pre-compaction summary/report was effectively pushed out of the operator-visible window by tool output; Focusa preserved state, but the user could not easily recover the visible summary.
- Existing guard docs distinguish project root, continuity id, session id, and trajectory similarity, but they do not define a general attention/recall verdict that pins critical facts and verifies they are used.
- Same Pi session history contained prior PTM work and the explicit correction; the failure happened inside one long-running session, not because state lived in a different chat.
- Daemon/primitives recorded context and scope telemetry, but the active pre-action path did not require a contradiction or recall verdict before file/tool action.

## Evidence from current software and session review

Review date: 2026-05-31. Line numbers are local source positions from the review snapshot.

- `docs/current/FOCUSA_MODEL_VISIBLE_AWARENESS.md` states practical precedence is operator steering/current ask first, then identity prior, trajectory, and Workpoint. This is the intended policy.
- `apps/pi-extension/src/turns.ts:468` still injects Workpoint guidance as “authoritative continuation anchor unless the operator explicitly steers elsewhere,” but no structured scope-verdict block is emitted before the Workpoint section.
- `apps/pi-extension/src/turns.ts:664-668` orders Focus Slice sections as `CURRENT_ASK`, `QUERY_SCOPE`, trajectory, then Workpoint; this surfaces both signals but does not adjudicate contradictions between them.
- `apps/pi-extension/src/state.ts:396-420` classifies current ask as question/correction/instruction/meta and detects operator steering, but it does not extract project targets (`PTM`, `planmarr`, `/home/planmarr/plan-the-marriage`, remote host/domain hints).
- `apps/pi-extension/src/turns.ts:901-999` records current ask, query scope, and `steering_detected` telemetry; it does not set `action_authority=false` for conflicting project evidence.
- `apps/pi-extension/src/turns.ts:1091-1100` detects scope failure after assistant output; that is useful for measurement but too late to prevent wrong-project action.
- `apps/pi-extension/src/compaction.ts:141-166` refreshes a scoped Workpoint packet after compaction and accepts it when it matches the saved `project_root + continuity_id`; it does not compare packet scope to current-ask project hints.
- `apps/pi-extension/src/compaction.ts:276-309` formats WorkpointResumePacketV2 for prompt with `CANONICAL: true`, best-next tools, and authority boundary; it does not include current-ask action-authority status or a short anti-forgetting memory anchor.
- `apps/pi-extension/src/compaction.ts:465-476` emits a large Workpoint/learning-loop packet, but it does not guarantee a compact replayable summary remains visible after subsequent tool-output bursts.
- Pi session JSONL around the incident shows the contradiction in one file: current ask stored “wrong place / PTM remote project” while `projectRoot` stayed `/home/wirebot/focusa`, then compaction re-injected the canonical Focusa Workpoint.
- Earlier same-session JSONL contains PTM remote evidence (`/home/planmarr/plan-the-marriage`, PTM docs/specs/audits, HLT ledger, auth work), but no hot-path primitive scanned it on the conflict phrase.
- The observed “I cannot access because tool calls streamed it out of view” failure shows a second path: the model/operator can lose a just-produced report even when Focusa state remains healthy.

## Deep audit confirmation — 2026-05-31

A follow-up architecture pass confirmed the gap is cross-surface attention/action authority rather than a single missing memory store.

- Initial source search found no implemented `MEMORY_ANCHOR`, `AttentionRecallVerdict`, `visible_recap_required`, `latest_report_summary_ref`, `project_thread_observation`, `detect_semantic_project_scope_conflict`, `canonical_for_saved_scope`, or `action_authority_for_current_ask` markers outside this draft spec; the `focusa-yv8d` implementation now adds these markers across Pi Focus Slice/compaction, WorkpointResumePacketV2, report replay, project-switch ledger, Spec97 Reflex metadata, and telemetry regression gates.
- `apps/pi-extension/src/turns.ts:723-724` truncates the Focus Slice to the first four lines under pressure, which can preserve only header/projection/view/current ask and drop `QUERY_SCOPE`, `PROJECT_TRAJECTORY`, `WORKPOINT`, and tool affordances.
- `crates/focusa-core/src/expression/engine.rs` has reduced/pinned ASCC degradation, but Pi’s custom Focus Slice assembly bypasses that protection.
- `crates/focusa-api/src/routes/workpoint.rs` renders WorkpointResumePacketV2 with saved-scope canonicality and identity axes, but does not compute current-ask action authority.
- `crates/focusa-api/src/routes/project.rs` supports caller-supplied remote evidence, but the identity route does not yet extract project overrides from `current_ask`.
- Tool-output externalization exists, but no runtime policy forces a visible recap or report-summary replay after tool-output flood.

Implementation priority from this audit: first make a protected prefix/verdict survive degradation, then split Workpoint action authority, then add report replay, project-switch ledger, and semantic conflict primitive.


## Implementation status snapshot — 2026-05-31

The anti-forgetting guard is implemented in source and regression-tested; live integration proof is tracked separately by `focusa-yv8d.12`.

Implemented surfaces and proof handles:

- Protected attention prefix: `MEMORY_ANCHOR` and `ATTENTION_RECALL_VERDICT` in Focus Slice/compaction output; proof `tests/spec_attention_recall_anchor_static_test.sh`.
- Report replay: `latest_report_summary_ref` capture/persistence and visible recap after tool-output flood; proof `tests/spec_report_replay_static_test.sh` and `tests/spec_tool_output_flood_recap_static_test.sh`.
- Saved scope vs action authority: `canonical_for_saved_scope`, `matches_current_ask_scope`, `action_authority_for_current_ask`, and `scope_conflict_reason`; proof `tests/spec_project_scope_override_static_test.sh`.
- Scope arbitration: `CURRENT_ASK_SCOPE_VERDICT` before Workpoint continuation in Focus Slice/compaction; proof `tests/spec_scope_arbitration_block_static_test.sh`.
- Project-switch ledger: bounded same-session project observations surfaced without raw transcript scans; proof `tests/pi_session_project_switch_ledger_static_test.sh`.
- Semantic Reflex primitive: `detect_semantic_project_scope_conflict` outputs `CurrentScopeVerdict`; proof `tests/spec97_semantic_scope_conflict_primitive_static_test.sh`.
- Telemetry regression: `scope_conflict_detected` is query-distinguishable from generic `scope_mismatch`; proof `tests/scope_routing_regression_eval.sh`.

Failure reports for this class must cite evidence surfaces and rejected hypotheses. Agreement with an operator correction is not proof; the report must show which stored/retrieved/attended/action-authority layer failed or passed.

## Expected behavior

When Focusa retrieves or receives a critical fact that can change the next action, that fact must become a model-visible memory anchor and must be checked before tool/file action. Project correction is one case; a just-written report summary, operator correction, destructive-risk note, or current task invariant are the same class.

Expected route:

1. Extract critical facts from Workpoint, latest operator correction, compaction packet, and bounded session evidence.
2. Build a short `MEMORY_ANCHOR` before verbose packets: task, must-not-forget facts, latest report/spec summary handle, evidence handles, and next action.
3. Compute an `AttentionRecallVerdict` before Focus Slice finalization and before project-scoped tool calls.
4. If the verdict detects project conflict, mark the active packet as `canonical_for_saved_scope=true` but `action_authority_for_current_ask=false`.
5. If the verdict detects tool-output flood, compaction loss, or forgotten report risk, force a visible recap before continuing.
6. Route to `focusa_project_verify` / `focusa_project_identity` only for the project-conflict subtype.
7. Checkpoint or transfer into the correct project scope before doing file, API, or evidence work.
8. Surface concise operator-visible status, e.g. “Memory anchor: active report is X; no implementation yet; rebinding before action.”

## Root cause analysis

### 0. Retrieved memory was not converted into forced attention

Focusa stored and retrieved useful context, but the model still had to notice, prioritize, and use it. That is not a reliable memory architecture. Critical facts need an explicit attention checksum before action.

Examples:

- latest operator correction;
- active task invariant;
- just-produced report/spec summary;
- Workpoint mission warning;
- scope conflict;
- “do not implement yet” constraint.

### 0b. Tool-output flood hid the working summary

Large tool outputs and JSON packets can push the actual task/report summary out of the visible working window. Focusa needs recap checkpoints after tool bursts and a replay handle for the latest report summary.

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

### A0. Add an Attention/Recall Control Plane before memory injection and action

Focusa needs a small mandatory layer between state retrieval and model-visible action guidance. Its job is broader than project scope: determine whether critical retrieved facts have been pinned, whether action can proceed, and whether a visible recap is required.

Inputs:

- current operator ask after quoted Focusa context is stripped;
- latest operator correction and latest assistant report/spec summary;
- active Workpoint and WorkpointResumePacketV2;
- current `S.sessionCwd`, `S.currentAsk.projectRoot`, continuity id, and session id;
- Focusa daemon Workpoint/trajectory/project identity state;
- bounded Pi session JSONL/project-switch index;
- known project aliases, paths, domains, remotes, and HLT ledgers;
- tool-output pressure counters since the last visible recap.

Output:

```json
{
  "schema": "focusa.attention_recall_verdict.v1",
  "status": "attentive | attention_risk | conflict | unknown",
  "memory_anchor": {
    "task": "iterate anti-forgetting architecture spec",
    "must_not_forget": [
      "root failure is model forgetting / attention loss",
      "project-scope override is a subtype",
      "no implementation until architecture confidence is grounded"
    ],
    "latest_report_summary_ref": "docs/current/PROJECT_SCOPE_OVERRIDE_INCIDENT_AND_GUARD_SPEC_2026-05-31.md#executive-summary",
    "evidence_refs": ["workpoint:019e7e25-d784-72f1-af4d-fb882568adac"],
    "next_action": "iterate doc, not source implementation"
  },
  "scope_verdict": {
    "status": "aligned | conflict | override_candidate | unknown",
    "saved_scope": {"project_root": "/home/wirebot/focusa", "continuity_id": "..."},
    "current_ask_scope": {"project_alias": "PTM", "project_root": "/home/planmarr/plan-the-marriage", "confidence": "high"},
    "workpoint_canonical_for_saved_scope": true,
    "workpoint_action_authority": false
  },
  "attention_risks": ["tool_output_flood", "compaction_loss", "forgotten_report"],
  "action_allowed": false,
  "visible_recap_required": true,
  "required_next": ["recap_memory_anchor", "verify_current_project", "rebind_or_session_transfer"],
  "evidence_spans": ["current_ask:PTM remote", "session_jsonl:/home/planmarr/plan-the-marriage", "remote_hlt:docs/HLT_LEDGER.md"]
}
```

This layer must run before Focus Slice Workpoint rendering, compaction resume instructions, and project-scoped tool execution. It should also run after large tool-output bursts before the assistant finalizes or starts a new action.

### A0a. Add `MEMORY_ANCHOR` to Focus Slice and compaction summaries

The model-visible packet needs a tiny, high-salience block above verbose Workpoint JSON:

```text
MEMORY_ANCHOR:
  task: iterate anti-forgetting architecture spec
  must_not_forget:
    - root failure is model forgetting / attention loss
    - project scope is one subtype
    - no implementation yet
  latest_report: docs/current/PROJECT_SCOPE_OVERRIDE_INCIDENT_AND_GUARD_SPEC_2026-05-31.md
  next_action: edit docs only
```

This block should be short enough to remain visible even when tool definitions, Workpoint packets, or traversal output are large.

### A0b. Add tool-output flood recap and report replay

After a bounded number of tool calls or large output bytes, Pi should force a one- or two-line recap before continuing:

```text
Recap: auditing/iterating the anti-forgetting spec; no source implementation; latest finding is missing AttentionRecallVerdict + MemoryAnchor.
```

When an assistant produces a report/spec summary, Focusa should store a replayable summary handle and allow the model to restate it without relying on transcript visibility.

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

1. `tests/spec_attention_recall_anchor_static_test.sh`
   - Assert Focus Slice / compaction text includes `MEMORY_ANCHOR` before verbose Workpoint JSON.
   - Assert source exposes `AttentionRecallVerdict` or equivalent schema with `visible_recap_required`, `must_not_forget`, and `latest_report_summary_ref`.
   - Assert tool-output recap thresholds exist and are not only comments.

2. `tests/spec_report_replay_static_test.sh`
   - Assert report/spec summaries can be saved as bounded handles.
   - Assert a post-compaction prompt can recover the latest report summary without transcript-tail authority.

3. `tests/spec_project_scope_override_static_test.sh`
   - Assert Focus Slice / compaction text includes `CURRENT_ASK_SCOPE_VERDICT` or a scope sub-block inside `AttentionRecallVerdict`.
   - Assert Workpoint packet uses `canonical_for_saved_scope` separately from `action_authority_for_current_ask`.
   - Assert Focus Slice ordering puts attention/scope verdict before Workpoint continuation.

4. Extend `tests/scope_routing_regression_eval.sh`
   - Assert new telemetry events are accepted and queryable.
   - Assert `scope_conflict_detected` is distinguishable from generic `scope_mismatch`.

5. Add `tests/pi_session_project_switch_ledger_static_test.sh`
   - Assert Pi extension source contains a bounded project-switch/session-evidence substrate, not only `S.sessionCwd`.
   - Assert conflict phrases route through the ledger before Workpoint action authority is granted.

6. Add `tests/spec97_semantic_scope_conflict_primitive_static_test.sh`
   - Assert Reflex Primitive registry includes `detect_semantic_project_scope_conflict`.
   - Assert it outputs `CurrentScopeVerdict` and can suppress action authority without an API `scope_mismatch` first.

### Unit tests

1. After compaction, the model receives a Workpoint packet plus a `MEMORY_ANCHOR` saying “root failure is model forgetting / attention loss; project scope is a subtype.”
   - Expected: first assistant action/report preserves that framing and does not reduce the problem to project scope alone.

2. A just-produced report summary is followed by multiple large tool outputs.
   - Expected: `visible_recap_required=true`; assistant can restate the report summary from the summary handle without asking the operator to scroll.

3. Operator asks “what are you doing?” after tool-output flood.
   - Expected: assistant answers from `MEMORY_ANCHOR` and active Workpoint in one or two lines before any further tool calls.

4. Current ask: “you are looking in the wrong place; this is the PTM remote project” + saved Workpoint `/home/wirebot/focusa`.
   - Expected: `status=override_candidate` or `conflict`; `action_authority_for_current_ask=false`.

5. Current ask: “write an incident spec in the Focusa directory” + saved Workpoint `/home/wirebot/focusa`.
   - Expected: `status=aligned`; normal docs route allowed.

6. Current ask names a remote path `/home/planmarr/plan-the-marriage` while current cwd is `/root` or `/home/wirebot/focusa`.
   - Expected: project verify/rebind route first; no local Focusa file inspection as target action.

7. Same project, different continuity id.
   - Expected: existing continuity mismatch behavior still wins; no cross-session merge.

8. Long same-Pi-session replay containing Focusa and PTM events.
   - Input: saved Workpoint `/home/wirebot/focusa`, current ask “wrong place / PTM remote project,” session ledger has `/home/planmarr/plan-the-marriage` and `docs/HLT_LEDGER.md`.
   - Expected: PTM candidate outranks Focusa for current action; Focusa Workpoint remains canonical saved state but suppressed for action.

9. Operator asks to write a Focusa detour spec after the PTM incident.
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

- Focus Slice exposes a `MEMORY_ANCHOR` before verbose Workpoint/trajectory/tool-affordance payloads.
- Retrieved critical facts are considered usable only after `AttentionRecallVerdict` confirms they are pinned or forces a recap.
- A just-written report/spec summary remains replayable after compaction and tool-output flood without relying on transcript tail.
- Tool-output bursts trigger a concise visible recap before additional action when attention risk is high.
- A canonical packet is never treated as action-authoritative when current operator text declares a conflicting project.
- Focus Slice exposes a current-ask scope verdict before Workpoint instructions, either standalone or inside `AttentionRecallVerdict`.
- Workpoint resume rendering distinguishes saved-scope canonicality from current-ask action authority.
- Operator project corrections trigger bounded project verification/rebind before file/API work.
- Same-session project history is indexed as project-thread evidence, so “single Pi session” does not collapse multiple project scopes into one active target.
- Semantic project conflicts can suppress action before any API-level `scope_mismatch` occurs.
- Regression tests cover Focusa-local saved scope vs PTM remote current ask, plus generic post-compaction/tool-flood forgetting.
- Telemetry makes the event reviewable without raw transcript dependence.
- Docs explain the difference between “stored memory,” “retrieved memory,” “attended memory,” and “action authority.”
- Failure reports must cite evidence surfaces and rejected hypotheses; agreement with operator wording is not considered proof.

## Non-goals

- Do not weaken `project_root + continuity_id` as the saved Workpoint identity gate.
- Do not merge Focusa and PTM sessions because they share an operator or topic.
- Do not treat trajectory similarity as project authority.
- Do not store raw transcript text as durable evidence beyond bounded spans/summaries.

## Follow-up implementation beads

Suggested child beads:

1. Add `AttentionRecallVerdict` and `MEMORY_ANCHOR` generation in Pi Focus Slice and compaction output.
2. Add report-summary capture/replay handles for assistant-produced specs, audits, and final reports.
3. Add tool-output flood recap thresholds and visible recap enforcement.
4. Add current-ask project override detector in Pi extension as a scope subtype.
5. Add Workpoint action-authority fields to resume packet rendering.
6. Add scope arbitration block to Focus Slice and compaction output.
7. Build bounded Pi session project-switch ledger from JSONL/session entries.
8. Add semantic project-scope-conflict Reflex Primitive.
9. Add telemetry events and regression tests for attention loss, forgotten reports, and operator-declared project override.
10. Update Workpoint/project-identity/model-visible-awareness docs with stored vs retrieved vs attended memory and canonicality vs action-authority distinctions.

## Review discipline

This spec must remain falsifiable. Future updates should include:

- exact code/doc/session evidence reviewed;
- which hypothesis was rejected;
- what invariant would have prevented the incident;
- which test proves prevention before tool/file action;
- no claims that “Focusa would prevent this” without a passing pre-action regression.

## Design decision

Focusa should treat retrieved memory as action-usable only after it is pinned, checked, and either applied or recapped; canonical Workpoints are saved-state inputs, not sufficient action authority.
