# Spec 117A — Living Mission Field PWA Iteration

**Status:** draft, iterable, NOT FINAL — operator has not yet signed off.  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-08  
**Updated:** 2026-07-08 — integration pass after operator feedback: PWA must integrate the Omnigent comparison, chat-as-transcript-river, Focusa action cards around transcript turns, SaaS boundary, and living-field metaphor.  
**Scope:** Focusa PWA / Mission Deck direction, future `apps/deck/`, daemon-served `/deck`, chat/agent interaction posture, proof/receipt/action flows, SaaS connection, and visual product language.  
**Relationship to Spec 117:** this spec does **not** replace Spec 117. It refines Spec 117 with the 121A principle: keep the discipline, restore the soul.  
**Relationship to Spec 121A:** Spec 121A governs the menubar's living-field restoration. Spec 117A expands that philosophy into the PWA, where interaction can be richer without collapsing into a dashboard or chat clone.

---

## 0. One-line definition

Focusa Mission Deck PWA is an expanded living mission field where operators can observe, converse, steer, prove, publish, and coordinate AI-agent work through Focusa's typed runtime, without making chat, dashboards, or raw terminal control the center of the product.

---

## 1. Why this iteration exists

Spec 117 correctly introduced Mission Deck, guided onboarding, Recall, proof/no-proof education, PWA routes, and read-first safety.

But the PWA direction needs a stronger product soul and a stronger integration story.

```text
Focusa is not merely a dashboard.
Focusa is not merely a chat UI.
Focusa is not merely observation.
Focusa is not merely a remote terminal.
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
- human operator control;
- private-node canonical state;
- cloud/SaaS coordination without cloud becoming the canonical brain.

---

## 2. Normative basis

This spec inherits and reconciles:

- `docs/117-mission-deck-onboarding-recall-pwa-spec.md` — Mission Deck, first-run, guided walkthroughs, PWA safety, Recall labeling.
- `docs/121a-menubar-discipline-and-living-field-spec.md` — living field grammar, action rituals, discipline + soul posture.
- `docs/121-menubar-rearchitecture-spec.md` — typed envelopes, receipts, topology/license dependencies, runtime data discipline.
- `docs/11-menubar-ui-spec.md` — original focus cloud, background thought clouds, intuition pulses, ambient cognitive awareness.
- `docs/42-menubar-ux-improvements.md` — calm technology, ambient display, progressive disclosure.
- `docs/current/AUTHORITY_MODEL.md` — Context Authority, project/continuity/workpoint scope.
- `docs/current/GOLDEN_WORKFLOW.md` — canonical workflow and proof-backed continuation.
- `docs/53-focusa-device-pairing-spec.md` — local-first device trust, pairing, revocation, scopes.
- `docs/115-*` cloud/control-plane surfaces — cloud coordinates, private node decides.
- `docs/119-*` surfaces where receipts/governance ledger are defined.
- Omnigent comparison notes from operator iteration — multi-surface continuity, web/phone session access, session snapshot + live stream, agent room, approval cards, host registration, collaboration/fork patterns. Focusa may learn the interaction patterns without becoming an Omnigent-style meta-harness.

If these disagree:

```text
Daemon state is canonical.
Context Authority gates mutation.
Receipts prove side effects.
The private Focusa node owns project truth.
The PWA is the expanded living field.
Chat is a river inside the field, not the field itself.
Cloud coordinates; node decides.
```

---

## 3. Core product thesis

The PWA should be more powerful than the menubar.

The PWA should not be merely observational.

The PWA should not become a generic dashboard.

The PWA should not become a generic agent-chat clone.

The PWA should not default to browser shell control.

The correct product shape is:

```text
A living mission field with conversation, proof, recall, authority, receipts, cloud coordination, and agent action woven through it.
```

The operator must feel that Focusa is both:

```text
Typed enough to trust.
Alive enough to feel.
```

---

## 4. Required integration from current product discussion

This section records the things the PWA must integrate from the current iteration.

### 4.1 From the Omnigent comparison

Focusa should learn these patterns:

- same work accessible from terminal, browser, phone, native wrapper, and future SaaS;
- session snapshot + live event stream reconnect contract;
- host/node registration so work can run on trusted machines;
- mobile-friendly PWA for supervising and continuing work;
- agent room/sub-agent visibility;
- approval cards for risky actions;
- fork/handoff patterns;
- shareable but controlled session views;
- model/agent switching as explicit route, not hidden magic.

Focusa should **not** copy these as-is:

- chat-first product shape;
- generic meta-harness identity;
- browser terminal as default;
- raw agent session as the primary truth.

Focusa's adaptation:

```text
Omnigent shows agents working.
Focusa shows whether the mission is scoped, proven, safe, resumable, and ready for the next move.
```

### 4.2 From the chat correction

Chat must not be stripped.

The PWA must allow users to interact with:

- their Focusa environment;
- active agents;
- Workpoints;
- proof/evidence;
- Recall;
- SaaS/team/public receipt surfaces.

But chat becomes Transcript River, not the product center.

```text
Chat is the river.
Focusa is the map, weather, proof field, guardrail, and mission memory around it.
```

### 4.3 From the original menubar soul

The PWA must preserve the original cloud/focus metaphor in expanded form:

- Focus Cloud;
- Thought Clouds;
- Intuition Pulses;
- Evidence Particles;
- Drift Wind;
- Authority Ring;
- Topology Horizon.

The PWA may be denser and more interactive than the menubar, but it must still breathe.

### 4.4 From Spec 121 drift correction

Spec 121's discipline is needed, but its tab-first cockpit posture should not dominate the PWA.

The PWA can have many capabilities, but they must be framed as lenses over the living mission field.

### 4.5 From bootstrapped monetization/open ecosystem discussion

The PWA must support monetizable SaaS paths without turning Focusa into cloud memory by default.

Paid/team/cloud surfaces should enhance:

- node registry;
- cloud relay;
- public receipt hosting;
- shared team mission views;
- shared receipt ledger;
- collaboration links;
- policy packs;
- advanced Recall;
- publish/GTM proof workflows.

But canonical project truth remains local/private-node first unless the operator explicitly opts into a hosted/team model.

---

## 5. Primary metaphor

### 5.1 Living Mission Field

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
- next safe action;
- topology/cloud horizon;
- license/team context only where relevant.

### 5.2 Transcript River

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
- handoff packets;
- action intent cards;
- agent stream events.

Preferred framing:

```text
Chat is where conversation happens.
Focusa is where the mission lives.
```

### 5.3 Mission Clouds

Workpoints, Focus Frames, Recall results, and active mission objects appear as living clouds / nodes / constellations.

The PWA can use richer visuals than the menubar, but it should preserve calm motion, progressive disclosure, and organic hierarchy.

### 5.4 Evidence Anchors

Evidence appears as proof particles or anchors attached to the relevant Workpoint, claim, file, test, release step, or receipt.

Evidence should visibly stabilize a mission cloud.

### 5.5 Authority Lens

Context Authority appears as a lens or ring around any action-capable object.

Every mutation-capable card must show:

```text
authority: ok | ask | blocked | advisory | stale | proof_missing
```

### 5.6 Drift Weather

Drift is shown as pressure, wind, haze, or displacement rather than only red error text.

Drift should be visible before it becomes catastrophic.

### 5.7 Agent Streams

Agents appear as activity streams entering or orbiting the mission field.

Each agent stream should show:

- agent/harness;
- host/node;
- purpose: implement | review | explore | search | proof | handoff;
- active object;
- Workpoint binding;
- proof requirement;
- current status;
- last receipt/evidence;
- drift or blocked posture.

### 5.8 Receipt Glimmers and Ledger

Receipts appear as glimmers on the mission field and as durable rows in the Receipt Ledger.

A receipt should be rehydratable, linkable to evidence, and usable for public/team proof workflows when redaction allows.

---

## 6. PWA modes

The PWA has five primary modes. These are lenses over the same living field, not isolated product modules.

### 6.1 Observe

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
- Recall advisory context;
- topology horizon.

### 6.2 Converse

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

### 6.3 Steer

Purpose: safely influence mission direction.

Actions:

- create Workpoint candidate;
- checkpoint Workpoint;
- select active object;
- resolve drift;
- prepare handoff packet;
- route to an agent;
- propose next safe action;
- resume mission;
- fork a mission path;
- switch agent/model/harness with visible consequences.

All steering actions use action rituals (§11).

### 6.4 Prove

Purpose: attach, evaluate, and rehydrate proof.

Actions:

- link evidence;
- attach command output;
- attach screenshot/browser proof;
- attach test/API proof;
- rehydrate receipt;
- mark proof gap;
- verify no-proof/no-done;
- view proof meter;
- compare claim vs evidence.

### 6.5 Publish

Purpose: transform private proof into redacted public/team receipts.

Actions:

- prepare public card;
- preview redaction;
- publish receipt;
- share with team;
- export GTM proof;
- create buyer/evaluator demo artifact;
- generate a public project progress receipt.

Publish mode must always require explicit operator approval.

---

## 7. Layout direction

### 7.1 Default field layout

```text
┌────────────────────────────────────────────────────────────────────┐
│ Topology / node / relay horizon                                    │
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

### 7.2 Traditional panels are allowed as lenses

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
| Data Lattice | advanced raw runtime |

### 7.3 Raw dashboard fallback

An advanced operator may open a dense data view, but it is not the default product posture.

Name options:

```text
Data Lattice
Runtime Lattice
Advanced Runtime
```

Avoid making this the emotional center.

---

## 8. Specific UI primitives from this iteration

### 8.1 Turn Halos

Each transcript turn can have a small status halo:

```text
grounded
advisory
proof_missing
drift_risk
receipt_linked
blocked
```

### 8.2 Proof Shadows

Agent claims can show whether proof exists.

Example:

```text
Agent claim: install path fixed
Proof shadow: missing
Actions: attach proof | run doctor | mark unverified
```

### 8.3 Mission Lens

Toggle over transcript:

```text
Raw Transcript | Mission Lens
```

Mission Lens compresses transcript into:

- decisions;
- claims;
- evidence;
- open questions;
- next steps;
- drift risks;
- Workpoint changes;
- receipts.

### 8.4 Handoff Capsule

A portable packet for Pi, Claude Code, Codex, Cursor, OpenCode, UIAI Engine, or another harness.

```yaml
HandoffCapsule:
  mission:
  project_identity:
  continuity_id:
  workpoint:
  active_object:
  evidence_refs:
  do_not_drift:
  next_action:
  proof_required:
  target_agent:
  target_host:
```

### 8.5 Conversation-to-Workpoint Promotion

The operator can select a transcript span and say:

```text
Make this a Workpoint candidate.
```

Focusa responds with a candidate card and requires verification before canonical checkpoint.

### 8.6 Agent Room

A lens showing:

- active agents;
- host/node;
- agent purpose;
- Workpoint binding;
- current status;
- terminal/stream availability;
- proof requirements;
- inbox/results;
- handoff/fork/switch controls.

### 8.7 Public Receipt Preview

A publish lens that shows exactly what can leave the private node.

It must show:

- redacted fields;
- private fields omitted;
- evidence class;
- receipt id;
- public claim text;
- allowed destination;
- approval button.

---

## 9. Chat/conversation rules

### 9.1 Chat is allowed and important

The PWA must let the user interact with agents and the Focusa environment through chat-like input.

### 9.2 Chat cannot be naked transcript

The transcript must be annotated with Focusa state:

- Workpoint reference;
- evidence status;
- claim/proof relation;
- authority badge;
- drift warning;
- active object;
- receipt;
- next safe action;
- handoff capsule;
- action intent card;
- agent stream state.

### 9.3 Message targets

The composer must distinguish at least:

```text
Ask Agent     → send to selected agent/harness/session
Ask Focusa    → query Focusa daemon/state/Recall/Workpoint/proof
Command       → propose Focusa action intent
Handoff       → package Workpoint/context for agent continuation
Publish       → create receipt/public-card intent
```

### 9.4 Focusa annotations are first-class

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
Agent handoff prepared
Mission fork created
```

### 9.5 Focusa actions happen around the transcript

The PWA should support the operator's insight:

```text
Focusa actions happen all around the transcript.
```

This means the transcript is surrounded by:

- side-attached proof shadows;
- top-level mission context;
- inline action cards;
- hover/click rehydration;
- Workpoint promotion handles;
- agent handoff capsules;
- receipt glimmers;
- authority gates;
- next-safe-action prompts.

---

## 10. Multi-surface continuity model

The PWA should share state concepts with CLI, TUI, menubar, Pi tools, and SaaS.

### 10.1 Snapshot + live event stream

Preferred reconnect model:

```text
1. Open /v1/deck/events stream.
2. Fetch /v1/deck/mission-field snapshot.
3. Dedupe by event_id / receipt_id / workpoint_id.
4. Render current state.
5. Continue streaming field changes.
```

### 10.2 Same mission across surfaces

A mission may be viewed or continued from:

- CLI;
- TUI;
- menubar;
- PWA;
- phone PWA;
- future Focusa SaaS surface;
- Pi/agent tool calls.

All surfaces should speak in the same concepts:

```text
ProjectIdentity
Continuity ID
Workpoint
Evidence Ref
Receipt
Context Authority
Next Safe Action
Drift posture
```

### 10.3 Host/node registration

The PWA should understand nodes without becoming host-specific.

User-facing node states:

```text
local
private remote
cloud relay
degraded
unknown
```

Private infra labels like KH/OVH may appear in debug/agent-kb views only, not core product UI.

### 10.4 Collaboration/share/fork

Future collaboration should be expressed as Focusa-native mission actions:

- Share Mission View;
- Fork Mission Path;
- Handoff to Agent;
- Invite Reviewer;
- Publish Receipt;
- Team Proof Review.

Do not expose generic shared chat as the primary model.

---

## 11. Action ritual model

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

### 11.1 ActionIntentCard

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

### 11.2 No invisible mutation

No PWA click may silently mutate canonical Focusa state.

### 11.3 Degraded state is visible

If the action result is degraded or non-canonical, the UI must show that visibly and offer recovery or CLI fallback.

### 11.4 Approval cards

Risky actions use approval cards rather than generic confirmation modals.

The approval card should explain:

- what changes;
- why authority allows/blocks/asks;
- what proof is required;
- what receipt will exist;
- how to recover.

---

## 12. Core PWA packets

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
  surface_permissions:

TranscriptRiverPacket:
  session_id:
  items:
  focusa_annotations:
  claim_proof_links:
  workpoint_events:
  authority_events:
  action_cards:
  agent_streams:

ActionIntentPacket:
  action:
  target:
  preflight:
  side_effects:
  confirmation_required:
  cli_equivalent:
  expected_receipt:

ReceiptPacket:
  receipt_id:
  created_at:
  action:
  canonical:
  degraded:
  evidence_refs:
  rehydratable:
  next_tools:
  public_safe_projection:

AgentRoomPacket:
  agents:
    - agent_id:
      harness:
      host_node:
      purpose:
      status:
      workpoint_id:
      active_object:
      proof_required:
      latest_receipt:
      terminal_available:
      stream_available:
```

---

## 13. API route direction

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

Agent routes:

```text
GET  /v1/deck/agents
POST /v1/deck/agents/handoff/prepare
POST /v1/deck/agents/handoff/confirm
POST /v1/deck/agents/fork/prepare
POST /v1/deck/agents/fork/confirm
```

---

## 14. SaaS and local-first boundary

The PWA must integrate the SaaS layer without betraying local-first trust.

### 14.1 Cloud coordinates; node decides

Focusa Cloud / SaaS may provide:

- login/license;
- billing;
- node registry;
- cloud relay;
- shared receipt hosting;
- public proof pages;
- team views;
- collaboration links;
- upgrade flows;
- policy packs;
- marketplace/ecosystem primitives later.

The private Focusa node / daemon owns:

- canonical project state;
- Workpoint state;
- Evidence refs;
- Context Authority;
- local agent bridge;
- local/private transcripts unless explicitly published;
- mutation decisions.

### 14.2 Monetization posture

Free/local:

- local Mission Field;
- Workpoint view;
- proof/no-proof education;
- local transcript annotations;
- basic receipts;
- local PWA install;
- local pairing.

Paid/pro/team candidates:

- cloud relay;
- shared receipt ledger;
- public receipt hosting;
- team mission views;
- shared Workpoint review;
- advanced Recall;
- cross-device sync;
- managed node registry;
- publish workflows;
- policy packs;
- collaboration and multi-user work items.

Commercial state should appear when relevant, not dominate the core field.

### 14.3 Open ecosystem path

The PWA should be compatible with a future open `focusa-primitives` layer:

- Workpoint schema;
- EvidenceRef schema;
- MissionContractCard schema;
- RecallDeckCard schema;
- AuthorityBadge schema;
- ReceiptPacket schema;
- HandoffCapsule schema.

The product can spread primitives while monetizing hosted coordination, advanced proof, team workflows, and trusted Focusa runtime.

---

## 15. Terminal and raw shell posture

The PWA must not expose raw terminal/shell by default.

Allowed early:

- read-only command/result surfaces;
- CLI equivalent copy buttons;
- agent stream/status;
- proof command suggestions;
- safe action cards.

Future terminal bridge, if implemented, starts as:

```text
read-only terminal mirror
```

Write access requires a later approved spec with:

- paired device;
- short-lived token;
- explicit operator approval;
- scope boundary;
- Context Authority gate;
- visible audit event;
- revocation path.

---

## 16. Implementation phases

### Phase 0 — Sign-off and split from Spec 117

- Review 117A.
- Decide whether it becomes `117a` companion or a direct amendment to 117.
- Confirm that PWA is allowed to be interactive, not observation-only.
- Confirm the PWA must integrate Omnigent-derived interaction patterns without becoming a meta-harness.

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
- Add Turn Halos, Proof Shadows, Mission Lens, and Handoff Capsule.

### Phase 4 — Agent Room + multi-surface continuity

- Add Agent Room packet/lens.
- Add host/node awareness.
- Add handoff/fork preparation.
- Add snapshot + event-stream reconnect model.

### Phase 5 — Action rituals

- Implement action interpretation.
- Implement preflight card.
- Implement confirm path.
- Show receipts and field updates.

### Phase 6 — Proof and Publish

- Evidence anchors.
- Proof Lens.
- Receipt Ledger.
- Public receipt preview.
- Redaction review.

### Phase 7 — Cloud/team expansion

- Pairing.
- Cloud relay.
- Team mission view.
- Shared receipts.
- Node registry.
- License/upgrade flows surfaced contextually.

---

## 17. Acceptance criteria

117A is accepted when:

1. PWA is explicitly interactive, not observation-only.
2. Chat is included as Transcript River, not generic product center.
3. The PWA integrates Omnigent-derived multi-surface patterns in a Focusa-native way.
4. Living Mission Field metaphor governs default UI.
5. Workpoints, Evidence, Recall, Authority, Drift, Agents, Receipts, and SaaS state have living-field visual equivalents.
6. Every mutation follows the action ritual model.
7. Daemon remains canonical.
8. Cloud/paid/team surfaces enhance the product without becoming the product's soul.
9. Raw terminal/shell is absent by default unless a later approved spec gates it.
10. Public launch claims distinguish planned vs proven PWA features.
11. The PWA supports the principle: Focusa actions happen around the transcript.
12. Operator signs off before implementation is claimed.

---

## 18. Open operator questions

1. Should the PWA use `Mission Field`, `Living Mission Field`, or `Mission Deck` as visible product language?
2. Should the transcript river be collapsible by default or always partially visible?
3. Should the composer default to `Ask Focusa`, `Ask Agent`, or remember last mode?
4. Should `Command` be a visible composer mode or hidden behind natural-language interpretation?
5. Should Publish mode be available in local-first PWA before Focusa Cloud receipt hosting exists?
6. Should the first PWA release include visual motion immediately, or start with static living-field layout and animate later?
7. Should Agent Room launch with Pi-only support first, or support provider-neutral agent stream packets from the start?
8. Should the first SaaS-connected PWA path require Tailscale/private relay before public relay?

---

## 19. Summary

Spec 117 made Mission Deck useful.

Spec 117A makes Mission Deck feel like Focusa.

The PWA should be powerful enough to steer mature engineering work, but alive enough that the operator feels they are inside a mission environment, not staring at another dashboard.

```text
Focusa Mission Deck is the living field where agent conversations become durable, provable project work.
```
