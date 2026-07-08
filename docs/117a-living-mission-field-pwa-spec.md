# Spec 117A — Living Mission Field PWA Iteration

**Status:** draft, iterable, NOT FINAL — operator has not yet signed off.  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-08  
**Scope:** Focusa PWA / Mission Deck direction, future `apps/deck/`, daemon-served `/deck`, chat/agent interaction posture, proof/receipt/action flows, and visual product language.  
**Relationship to Spec 117:** this spec does **not** replace Spec 117. It refines Spec 117 with the 121A principle: keep the discipline, restore the soul.  
**Relationship to Spec 121A:** Spec 121A governs the menubar's living-field restoration. Spec 117A expands that philosophy into the PWA, where interaction can be richer without collapsing into a dashboard or chat clone.

---

## 0. One-line definition

Focusa Mission Deck PWA is an expanded living mission field where operators can observe, converse, steer, prove, and publish AI-agent work through Focusa's typed runtime, without making chat or dashboards the center of the product.

---

## 1. Why this iteration exists

Spec 117 correctly introduced Mission Deck, guided onboarding, Recall, proof/no-proof education, PWA routes, and read-first safety.

But the PWA direction needs a stronger product soul:

```text
Focusa is not merely a dashboard.
Focusa is not merely a chat UI.
Focusa is not merely observation.
Focusa is a living mission environment.
```

Focusa has matured. The PWA should allow the operator to interact with agents and Focusa state, but every interaction must still respect:

- Workpoint authority;
- Context Authority;
- Evidence discipline;
- receipt-backed side effects;
- local-first ownership;
- gated mutation;
- proof/no-proof separation;
- human operator control.

---

## 2. Normative basis

This spec inherits and reconciles:

- `docs/117-mission-deck-onboarding-recall-pwa-spec.md` — Mission Deck, first-run, guided walkthroughs, PWA safety, Recall labeling.
- `docs/121a-menubar-discipline-and-living-field-spec.md` — living field grammar, action rituals, discipline + soul posture.
- `docs/121-menubar-rearchitecture-spec.md` — typed envelopes, receipts, topology/license dependencies, runtime data discipline.
- `docs/current/AUTHORITY_MODEL.md` — Context Authority, project/continuity/workpoint scope.
- `docs/current/GOLDEN_WORKFLOW.md` — canonical workflow and proof-backed continuation.
- `docs/119-*` surfaces where receipts/governance ledger are defined.

If these disagree:

```text
Daemon state is canonical.
Context Authority gates mutation.
Receipts prove side effects.
PWA is the expanded living field.
Chat is a river inside the field, not the field itself.
```

---

## 3. Core product thesis

The PWA should be more powerful than the menubar.

The PWA should not be merely observational.

The PWA should not become a generic dashboard.

The PWA should not become a generic agent-chat clone.

The correct product shape is:

```text
A living mission field with conversation, proof, recall, authority, and agent action woven through it.
```

---

## 4. Primary metaphor

### 4.1 Living Mission Field

The default PWA home is a living mission field.

It visualizes:

- active mission;
- active Workpoint;
- trajectory ladder;
- proof posture;
- drift pressure;
- authority posture;
- active agents;
- Recall context;
- transcript river;
- receipts;
- next safe action.

### 4.2 Transcript River

Chat exists, but it is not central by default.

Chat is the transcript river flowing through the mission field.

It contains:

- user messages;
- agent replies;
- Focusa annotations;
- Workpoint checkpoints;
- Evidence links;
- receipt events;
- authority gates;
- drift warnings;
- handoff packets.

Preferred framing:

```text
Chat is where conversation happens.
Focusa is where the mission lives.
```

### 4.3 Mission Clouds

Workpoints, Focus Frames, Recall results, and active mission objects appear as living clouds / nodes / constellations.

The PWA can use richer visuals than the menubar, but it should preserve calm motion, progressive disclosure, and organic hierarchy.

### 4.4 Evidence Anchors

Evidence appears as proof particles or anchors attached to the relevant Workpoint, claim, file, test, release step, or receipt.

### 4.5 Authority Lens

Context Authority appears as a lens or ring around any action-capable object.

Every mutation-capable card must show:

```text
authority: ok | ask | blocked | advisory | stale | proof_missing
```

### 4.6 Drift Weather

Drift is shown as pressure, wind, haze, or displacement rather than only red error text.

Drift should be visible before it becomes catastrophic.

---

## 5. PWA modes

The PWA has five primary modes. These are lenses over the same living field, not isolated product modules.

### 5.1 Observe

Purpose: understand the current mission state.

Shows:

- ProjectIdentity;
- Continuity ID;
- active Workpoint;
- HLT / MLG / STG;
- evidence status;
- recent receipts;
- daemon/node health;
- work-loop posture;
- Recall advisory context.

### 5.2 Converse

Purpose: interact with Focusa and agents through a transcript river.

Composer modes:

```text
Ask Agent
Ask Focusa
Command
Handoff
Publish
```

The composer must make the target explicit before sending when ambiguity matters.

### 5.3 Steer

Purpose: safely influence mission direction.

Actions:

- create Workpoint candidate;
- checkpoint Workpoint;
- select active object;
- resolve drift;
- prepare handoff packet;
- route to an agent;
- propose next safe action;
- resume mission.

All steering actions use action rituals (§8).

### 5.4 Prove

Purpose: attach, evaluate, and rehydrate proof.

Actions:

- link evidence;
- attach command output;
- attach screenshot/browser proof;
- rehydrate receipt;
- mark proof gap;
- verify no-proof/no-done;
- view proof meter.

### 5.5 Publish

Purpose: transform private proof into redacted public/team receipts.

Actions:

- prepare public card;
- preview redaction;
- publish receipt;
- share with team;
- export GTM proof;
- create buyer/evaluator demo artifact.

Publish mode must always require explicit operator approval.

---

## 6. Layout direction

### 6.1 Default field layout

```text
┌────────────────────────────────────────────────────────────────────┐
│ Topology / license / node horizon                                  │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│   Recall fog          Evidence anchors           Agent streams      │
│       ○                     • • •                    ↓             │
│                                                                    │
│                 ACTIVE MISSION FIELD / WORKPOINT CLOUD             │
│                                                                    │
│   Drift weather       Authority lens          Next safe action      │
│                                                                    │
├──────────────────────── Transcript River ──────────────────────────┤
│ User / Agent / Focusa annotated conversation                        │
└────────────────────────────────────────────────────────────────────┘
```

### 6.2 Traditional panels are allowed as lenses

The PWA may include panes for efficiency, but they should be framed as lenses:

| Lens | Traditional equivalent |
|---|---|
| Mission Field | dashboard home |
| Transcript River | chat |
| Proof Lens | evidence panel |
| Recall Lens | search/results |
| Agent Room | active agents/workers |
| Action Lens | command palette |
| Publish Lens | receipts/public cards |
| Node Lens | cloud/topology/settings |

### 6.3 Raw dashboard fallback

An advanced operator may open a dense data view, but it is not the default product posture.

Name options:

```text
Data Lattice
Runtime Lattice
Advanced Runtime
```

Avoid making this the emotional center.

---

## 7. Chat/conversation rules

### 7.1 Chat is allowed and important

The PWA must let the user interact with agents and the Focusa environment through chat-like input.

### 7.2 Chat cannot be naked transcript

The transcript must be annotated with Focusa state:

- Workpoint reference;
- evidence status;
- claim/proof relation;
- authority badge;
- drift warning;
- active object;
- receipt;
- next safe action;
- handoff capsule.

### 7.3 Message targets

The composer must distinguish at least:

```text
Ask Agent     → send to selected agent/harness/session
Ask Focusa    → query Focusa daemon/state/Recall/Workpoint/proof
Command       → propose Focusa action intent
Handoff       → package Workpoint/context for agent continuation
Publish       → create receipt/public-card intent
```

### 7.4 Focusa annotations are first-class

A Focusa event may appear between transcript turns, but it should not look like a normal chat message.

Examples:

```text
Workpoint checkpointed
Evidence linked
Authority blocked mutation
Recall result promoted to candidate
Receipt rehydrated
Proof gap detected
Drift warning resolved
```

---

## 8. Action ritual model

Every mutation-capable PWA action follows this sequence:

```text
operator intent
→ Focusa interpretation
→ scope / authority / proof preflight
→ visible side-effect preview
→ optional edit
→ explicit confirmation
→ daemon POST
→ receipt returned
→ field updates
```

### 8.1 ActionIntentCard

Every action preview should include:

```yaml
ActionIntentCard:
  action:
  target:
  project_root:
  continuity_id:
  active_workpoint:
  authority_posture:
  proof_required:
  side_effects:
  blocked_actions:
  cli_equivalent:
  expected_receipt:
```

### 8.2 No invisible mutation

No PWA click may silently mutate canonical Focusa state.

### 8.3 Degraded state is visible

If the action result is degraded or non-canonical, the UI must show that visibly and offer recovery or CLI fallback.

---

## 9. Core PWA packets

The PWA should consume daemon packets rather than reconstruct product truth in Svelte.

Suggested packets:

```yaml
MissionFieldPacket:
  project_identity:
  continuity_id:
  active_workpoint:
  trajectory_ladder:
  proof_meter:
  authority_posture:
  drift_weather:
  active_agents:
  recall_clouds:
  next_safe_action:
  recent_receipts:
  topology_horizon:
  license_context:

TranscriptRiverPacket:
  session_id:
  items:
  focusa_annotations:
  claim_proof_links:
  workpoint_events:
  authority_events:

ActionIntentPacket:
  action:
  target:
  preflight:
  side_effects:
  confirmation_required:
  cli_equivalent:

ReceiptPacket:
  receipt_id:
  created_at:
  action:
  canonical:
  degraded:
  evidence_refs:
  rehydratable:
  next_tools:
```

---

## 10. API route direction

Spec 117 already listed initial Deck routes. 117A refines them around living-field packets.

Read routes:

```text
GET /v1/deck/home
GET /v1/deck/mission-field
GET /v1/deck/transcript-river
GET /v1/deck/recall-clouds
GET /v1/deck/proof-lens
GET /v1/deck/agent-room
GET /v1/deck/topology-horizon
GET /v1/deck/events
```

Action-intent routes:

```text
POST /v1/deck/actions/interpret
POST /v1/deck/actions/preflight
POST /v1/deck/actions/confirm
```

Specific action routes may still exist, but the PWA should prefer the intent/preflight/confirm ritual path where possible.

Publish routes:

```text
POST /v1/deck/publish/prepare
POST /v1/deck/publish/preview-redaction
POST /v1/deck/publish/confirm
```

---

## 11. Safety and monetization posture

The PWA can expose monetizable value without making cloud canonical.

Free/local:

- local Mission Field;
- Workpoint view;
- proof/no-proof education;
- local transcript annotations;
- basic receipts;
- local PWA install.

Paid/pro/team candidates:

- cloud relay;
- shared receipt ledger;
- public receipt hosting;
- team mission views;
- advanced Recall;
- cross-device sync;
- managed node registry;
- publish workflows;
- policy packs;
- collaboration and multi-user work items.

Commercial state should appear when relevant, not dominate the core field.

---

## 12. Implementation phases

### Phase 0 — Sign-off and split from Spec 117

- Review 117A.
- Decide whether it becomes `117a` companion or a direct amendment to 117.
- Confirm that PWA is allowed to be interactive, not observation-only.

### Phase 1 — Mission Field packet

- Define `MissionFieldPacket`.
- Add API route or adapter that composes current existing daemon state into one read packet.
- Keep daemon canonical; PWA does not infer truth.

### Phase 2 — PWA shell

- Create `apps/deck/` or equivalent.
- Serve `/deck` locally.
- Add manifest and service worker.
- Implement basic living mission field, not dashboard-first layout.

### Phase 3 — Transcript River

- Add chat/conversation surface.
- Add Focusa annotations.
- Add composer modes.
- Allow agent and Focusa interaction without centering generic chat.

### Phase 4 — Action rituals

- Implement action interpretation.
- Implement preflight card.
- Implement confirm path.
- Show receipts and field updates.

### Phase 5 — Proof and Publish

- Evidence anchors.
- Proof Lens.
- Receipt Ledger.
- Public receipt preview.
- Redaction review.

### Phase 6 — Cloud/team expansion

- Pairing.
- Cloud relay.
- Team mission view.
- Shared receipts.
- Node registry.

---

## 13. Acceptance criteria

117A is accepted when:

1. PWA is explicitly interactive, not observation-only.
2. Chat is included as Transcript River, not generic product center.
3. Living Mission Field metaphor governs default UI.
4. Workpoints, Evidence, Recall, Authority, Drift, Agents, and Receipts have living-field visual equivalents.
5. Every mutation follows the action ritual model.
6. Daemon remains canonical.
7. Cloud/paid/team surfaces enhance the product without becoming the product's soul.
8. Raw terminal/shell is absent by default unless a later approved spec gates it.
9. Public launch claims distinguish planned vs proven PWA features.
10. Operator signs off before implementation is claimed.

---

## 14. Open operator questions

1. Should the PWA use `Mission Field` or `Living Mission Field` as the visible product language?
2. Should the transcript river be collapsible by default or always partially visible?
3. Should the composer default to `Ask Focusa` or `Ask Agent`?
4. Should `Command` be a visible composer mode or hidden behind natural-language interpretation?
5. Should Publish mode be available in local-first PWA before Focusa Cloud receipt hosting exists?
6. Should the first PWA release include visual motion immediately, or start with static living-field layout and animate later?

---

## 15. Summary

Spec 117 made Mission Deck useful.

Spec 117A makes Mission Deck feel like Focusa.

The PWA should be powerful enough to steer mature engineering work, but alive enough that the operator feels they are inside a mission environment, not staring at another dashboard.

```text
Focusa Mission Deck is the living field where agent conversations become durable, provable project work.
```
