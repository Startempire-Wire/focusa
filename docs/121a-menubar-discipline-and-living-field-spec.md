# Spec 121A — Menubar Discipline and Living Field Restoration

**Status:** draft, iterable, NOT FINAL — operator has not yet signed off.  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-08  
**Scope:** `apps/menubar` experience layer, Focusa menubar surface strategy, and carry-forward principles for the Focusa PWA / Mission Deck surface.  
**Relationship to Spec 121:** this spec does **not** discard Spec 121. It reframes Spec 121 as the runtime/data-contract discipline layer and restores the original menubar soul as the visible experience layer.  
**Primary thesis:** keep the discipline, restore the soul.

---

## 0. One-line definition

The Focusa menubar may consume typed, envelope-normalized, receipt-bearing daemon surfaces, but it must render them as a calm living focus field — not as a miniature admin console, not as a tab-dense dashboard, and not as a clone of the future PWA.

---

## 1. Why this spec exists

Spec 121 correctly identifies engineering problems:

- runtime snapshots are too loosely typed;
- envelope handling has drifted into multiple implicit tracks;
- POST responses need receipts rather than disposable toasts;
- Workpoint, proof, topology, license, reminders, sync, and cloud surfaces need disciplined handling;
- the menubar must not invent data when the daemon is the source of truth.

But Spec 121 also creates product drift:

- it enumerates too many visible surfaces;
- it treats the menubar like a full Focusa console;
- it weakens the original cloud / focus / thought-field metaphor;
- it makes commercial and topology metadata too prominent;
- it risks replacing ambient cognitive awareness with a cramped engineering UI.

This spec preserves Spec 121's rigor while restoring the original design instinct:

```text
Focusa should feel alive.
Focusa should breathe.
Focusa should make invisible cognitive state visible without demanding attention.
```

---

## 2. Normative basis

This spec inherits and reconciles:

- `docs/11-menubar-ui-spec.md` — original ambient cognitive awareness, focus bubble, thought clouds, intuition pulses, awareness-not-control rules.
- `docs/42-menubar-ux-improvements.md` — calm technology, ambient display, progressive disclosure, organic cloud/bubble improvements.
- `docs/current/TAURI_MENUBAR_UP_TO_SPEED_SPEC.md` — runtime cockpit needs, Workpoint/Trajectory/Proof/Work Loop maturity.
- `docs/121-menubar-rearchitecture-spec.md` — typed runtime snapshot, envelope normalization, receipts, license/cloud/reminder dependencies.
- `docs/117-mission-deck-onboarding-recall-pwa-spec.md` — Mission Deck, PWA, guided onboarding, Recall, proof/no-proof education.

If these disagree:

```text
121 governs runtime discipline.
121A governs menubar experience posture.
117/121A jointly govern PWA direction.
The daemon remains source of truth.
The operator remains final authority.
```

---

## 3. Hard product law

```text
The menubar may consume every typed Focusa surface.
It must not visually expose every typed Focusa surface.
```

This is the main correction to Spec 121.

Spec 121's data model can be broad. The menubar's default visible UI must remain narrow, ambient, organic, and glanceable.

---

## 4. What remains from Spec 121

The following are preserved as engineering discipline:

1. **Typed runtime snapshot.** No broad `any` surfaces except tracked legacy escape hatches.
2. **Single envelope normalization.** One `normalize()` / equivalent path for envelope-wrapped and flat daemon responses.
3. **Receipts for write actions.** POSTs should produce a rehydratable receipt where available.
4. **Daemon source of truth.** Menubar reads and renders; it does not invent project truth.
5. **Spec-linked surfaces.** New surfaces need a normative basis.
6. **No toast as source of truth.** Toasts are transient feedback; receipts and daemon state are durable.
7. **No app-specific topology hacks.** User-facing product language must stay host-neutral.
8. **No code until sign-off.** This remains an iterative draft.

---

## 5. What changes from Spec 121

The following are changed by this iteration:

### 5.1 Tab-first cockpit is demoted

The menubar must not default to a dense row of many tabs.

Allowed:

```text
Living Field default
small peeks / drawers
advanced data view
settings
pairing wizard
```

Discouraged as default:

```text
focus / cockpit / trajectory / workpoint / proof / workloop / gate / sync / pair / settings / receipts / recall / cloud / license as equal visible tabs
```

### 5.2 Surfaces become living field objects

Spec 121 surfaces must be translated into visual metaphors before becoming UI.

### 5.3 Commercial state becomes contextual

License state must not dominate the header. License appears when:

- a locked action is attempted;
- Settings is opened;
- a paired cloud/team feature is being configured;
- the operator explicitly opens the license peek.

### 5.4 Topology becomes atmospheric, not host-specific

User-facing states:

```text
local
private remote
cloud relay
degraded
unknown
```

Forbidden in general product UI:

```text
KH
OVH
W4b letta path
W7 openclaw path
cPanel
LiteSpeed
Verious-specific VPS labels
```

Those may live in private agent knowledge or debug bundles, but not in the product-facing metaphor.

### 5.5 Actions are allowed, but as rituals

The menubar is no longer merely observational. Focusa is mature enough to allow safe actions.

But actions must feel deliberate:

```text
intent preview
Context Authority / scope posture
side-effect summary
CLI alternative
explicit confirmation
receipt after result
reversible or recoverable path when possible
```

A menubar action should feel like touching the living system carefully, not clicking admin buttons in a cramped panel.

---

## 6. Living Field visual grammar

### 6.1 Focus Cloud

The central object.

Represents:

- current Focus Frame;
- active Workpoint;
- current mission posture;
- canonical/degraded state;
- proof posture;
- safe-next-action readiness.

States:

| State | Visual idea |
|---|---|
| canonical + scoped | clear, stable, softly luminous |
| degraded | softened edge, partial haze |
| proof missing | incomplete ring / unanchored lower edge |
| blocked | compressed cloud, muted border |
| drift risk | slight lateral pull / wind line |
| work-loop active | slow inner pulse |

### 6.2 Thought Clouds

Orbiting context.

Represent:

- inactive Focus Frames;
- pinned candidates;
- Recall results;
- prior Workpoints;
- open questions;
- deferred constraints;
- archived context.

Distance, opacity, and sharpness communicate scope:

| Scope | Visual idea |
|---|---|
| current project + continuity | near and clear |
| same project, other continuity | near but softer |
| other project | far edge of field |
| global advisory | foggy / peripheral |
| stale / superseded | faded / drifting upward |

### 6.3 Evidence Particles

Small anchors that attach to a Focus Cloud or Workpoint Cloud.

Represent:

- test proof;
- file proof;
- screenshot proof;
- browser proof;
- API proof;
- release proof;
- receipt proof.

More verified evidence makes the cloud feel more grounded.

### 6.4 Receipt Glimmers

Receipts are not a tab by default.

A receipt appears as a brief glimmer attached to the affected cloud. Clicking it opens a Receipt Peek.

States:

| Receipt state | Visual idea |
|---|---|
| canonical true | anchored glimmer |
| canonical false | watch-tone shimmer |
| failure_class present | broken glimmer / muted red edge |
| rehydratable | small link thread to daemon source |

### 6.5 Authority Ring

A thin ring around the active cloud.

Represents:

- ok;
- advisory;
- blocked;
- stale;
- proof missing;
- global advisory.

The ring is the menubar equivalent of Context Authority. It should be understandable at a glance and explainable on hover/click.

### 6.6 Drift Wind

Subtle flow line showing off-scope pressure.

Represents:

- active object mismatch;
- project_root mismatch;
- stale context;
- task substitution risk;
- Workpoint drift.

Drift wind should not scream. It should make the system feel alive and gently corrective.

### 6.7 Reminder Pulses

Bottom-to-top pulses adapted from the original intuition-pulse metaphor.

Represent:

- pending reminder;
- surfaced candidate;
- weak signal;
- next safe action;
- proof gap;
- recovery hint.

Clicking a pulse opens the relevant peek.

### 6.8 Topology Horizon

Cloud/remote status becomes a horizon line or atmospheric layer behind the field.

States:

| Topology | Visual idea |
|---|---|
| local | clear background |
| private remote | thin horizon line |
| cloud relay | distant glow |
| degraded | haze / broken horizon |
| unknown | neutral fog |

### 6.9 Transcript Sparkline

The menubar should not host full chat. But it can show recent activity as a small river/sparkline:

- recent agent action;
- latest Focusa event;
- last operator action;
- last receipt;
- last proof change.

Full transcript belongs in the PWA / Mission Deck.

---

## 7. Surface translation table

| Spec 121 surface | 121A visible translation | Default visibility |
|---|---|---|
| ReceiptsPane | Receipt Glimmers + Receipt Peek | hidden until glimmer/click |
| License Tier Badge | License Horizon / Settings / locked-action explanation | contextual only |
| Cloud Indicator | Topology Horizon | ambient |
| RuntimeView | Advanced Data Lattice / Runtime Peek | not default |
| TopologyCard | Topology Horizon + Debug Peek | ambient + click |
| ReceiptsCard | Recent Receipt Peek | hidden until proof/receipt context |
| TrajectoryPeek | Mission Ladder Cloud / rung pulses | peek |
| WorkpointPeek | Active Focus Cloud / Workpoint Cloud | central |
| FirstRunWizard | Guided emergence flow | visible on first run |
| Recall tab | Thought Clouds / Recall Fog | PWA-first; menubar peek only |
| ProofPeek | Evidence Particles + Proof Peek | visible through particles |
| Work Loop | Heartbeat / inner pulse + Work Loop Peek | ambient + click |
| Gate | Authority Ring + Gate Peek | ambient + click |
| Pair | Pairing Bloom / First-run cloud bridge | first-run / settings |
| Settings | Settings | explicit only |

---

## 8. Menubar interaction model

### 8.1 Default state: Living Field

The default menubar view should answer:

```text
Is Focusa oriented?
Is the current Workpoint safe?
Is proof present or missing?
Is anything drifting?
Is the system healthy enough to continue?
```

It should not try to answer every possible operator question by default.

### 8.2 Peeks, not dashboards

Details open as peeks/drawers:

- Workpoint Peek;
- Proof Peek;
- Gate Peek;
- Topology Peek;
- Receipt Peek;
- Pairing Peek;
- Settings.

Peeks should be progressive disclosure, not permanent dense panels.

### 8.3 Action rituals

All write-capable actions follow:

```text
surface signal
→ operator clicks
→ intent preview
→ authority / scope check
→ explicit confirmation
→ daemon POST
→ receipt glimmer
→ receipt peek / CLI alternative
```

### 8.4 CLI alternative remains visible

Every menubar write action should include the equivalent CLI command when possible. This preserves Focusa's local-first, operator-respecting posture.

---

## 9. PWA carry-forward principle

This spec is written for the menubar, but it also corrects the PWA/Mission Deck direction.

The PWA should not become a generic dashboard or a chat clone.

The PWA should be:

```text
an expanded living mission field
```

The PWA can be more interactive than the menubar, but it should inherit the same living grammar:

| Menubar | PWA / Mission Deck |
|---|---|
| Focus Cloud | Active Mission Field center |
| Thought Clouds | Recall/Workpoint/Context constellation |
| Evidence Particles | Evidence Desk + proof anchors |
| Authority Ring | Context Authority Lens |
| Drift Wind | Drift Map / scope pressure |
| Reminder Pulses | Next Safe Action stream |
| Transcript Sparkline | Transcript River / Conversation Lens |
| Receipt Glimmer | Receipt Ledger + Public Receipt preview |
| Topology Horizon | Node / Cloud / Relay status layer |

The PWA may include chat, but chat is not the product center.

Preferred framing:

```text
Chat is the transcript river.
Focusa is the living mission field around it.
```

---

## 10. PWA modes implied by 121A

The PWA should support modes/lenses instead of one flat dashboard.

### 10.1 Observe

Read state, Workpoint, proof, recall, topology, health.

### 10.2 Converse

Interact with agent and Focusa via chat, with Focusa annotations around transcript turns.

### 10.3 Steer

Propose next action, checkpoint Workpoint, resolve drift, select active object, hand off to agent.

### 10.4 Prove

Attach evidence, rehydrate receipts, verify proof, distinguish no-proof/no-done.

### 10.5 Publish

Prepare redacted receipts, public proof cards, GTM demo artifacts, and team-visible updates.

Each mode should still feel connected to the living mission field.

---

## 11. Implementation phases

### Phase 0 — Operator sign-off

- Review 121A.
- Decide whether it supersedes 121's visible UX sections while preserving 121's runtime discipline.
- Mark 121 as runtime/data-contract layer, not final UX layer.

### Phase 1 — Surface inventory remap

For each existing and proposed menubar surface:

```text
raw surface
→ daemon source
→ normalized typed packet
→ living field metaphor
→ visible default / peek / advanced-only
```

### Phase 2 — Living Field shell

Implement or restore:

- Focus Cloud;
- Thought Clouds;
- Evidence Particles;
- Authority Ring;
- Drift Wind;
- Reminder Pulses;
- Topology Horizon.

### Phase 3 — Receipts as glimmers

- Preserve receipt discipline.
- Replace receipt-as-tab default with receipt glimmer + peek.
- Keep full receipt list in an advanced/recent proof area.

### Phase 4 — Action rituals

- Add gated menubar actions only where they fit the ritual model.
- No direct cramped admin buttons.
- Always show receipt and CLI alternative.

### Phase 5 — PWA spec iteration

Return to Spec 117 and create a PWA/Mission Deck iteration that applies 121A:

```text
Mission Deck = expanded living mission field
Transcript = river
Workpoints = clouds / anchors
Evidence = particles / desk
Recall = constellation / fog
Authority = lens / ring
Actions = gated rituals
Receipts = ledger / glimmers / publishable proof
```

---

## 12. Acceptance criteria

121A may be considered accepted when:

1. Spec 121's runtime discipline is preserved.
2. The original menubar cloud/focus/intuition metaphor is explicitly restored.
3. The menubar default is not tab-first or dashboard-first.
4. New surfaces are mapped to living field objects before implementation.
5. Menubar actions are allowed only through explicit action rituals.
6. Topology copy is host-neutral in product UI.
7. Commercial/license state is contextual, not dominant.
8. PWA carry-forward principles are recorded for the next Spec 117 iteration.
9. Operator signs off before any implementation work is claimed.

---

## 13. Open operator questions

1. Should the default menubar view remove visible tabs entirely and rely on peeks from the living field?
2. Should Receipts exist as a full advanced list, or only inside Proof/Receipt Peek?
3. Should license state be completely absent from the default header unless a locked action is attempted?
4. What is the right visual vocabulary for degraded state: haze, broken ring, dim cloud, or another metaphor?
5. Should PWA Spec 117A be created as `117a-living-mission-field-pwa-spec.md`, or should Spec 117 be amended directly?

---

## 14. Diff against Spec 121

| Area | Spec 121 | Spec 121A |
|---|---|---|
| Primary posture | menubar rearchitecture / runtime cockpit | disciplined runtime underneath living ambient field |
| Default UI | many surfaces/tabs/cards | central focus cloud + peeks |
| Receipts | new pane/tab | glimmer + peek + advanced history |
| License | header badge | contextual horizon/settings/locked-action reveal |
| Cloud topology | header cloud with host-specific states | topology horizon with host-neutral states |
| Recall | tab | thought clouds / PWA-first recall field |
| Actions | POSTs with receipts | deliberate action rituals with authority + CLI fallback |
| PWA relation | inherited from Spec 117 but not visually integrated | explicit carry-forward: expanded living mission field |

---

## 15. Summary

Spec 121 gave Focusa menubar discipline.

Spec 121A restores the soul.

The product should not choose between engineering maturity and living presence. Focusa needs both:

```text
Typed enough to trust.
Alive enough to feel.
```
