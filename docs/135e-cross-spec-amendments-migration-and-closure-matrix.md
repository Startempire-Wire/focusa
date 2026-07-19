# Spec 135E — Cross-Spec Amendments, Migration, Compatibility, and Closure Matrix

**Status:** draft, iterable, NOT FINAL — operator approval required  
**Owner:** Focusa / Verious Smith  
**Created:** 2026-07-17  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Closure relationship:** required companion; Spec 135 cannot close without Spec 135E.  
**Scope:** normative relationship between Spec 135 series and existing Focusa specs, including concurrency, multiplexed Instances/Sessions/Attachments, ontology primitives and governance, precedence, naming, amendment behavior, current/partial/planned truth, migration of existing projects, ontology state, snapshots, events, sessions, UI names, and clients, compatibility, deprecation, and whole-series closure.

---

## 0. One-line definition

Spec 135 does not replace Focusa’s existing concurrency, anti-singleton, ontology, ProjectIdentity, Trajectory, Workpoint, Context Cognition, Evidence, Spec Workbench, Receipts, provider-neutral closure, Mission Deck, project creation, menubar, or durable execution specs; it composes and amends them into one complete professional-workspace, Project Genesis, domain-semantic, and multiplexed Mission Canvas product path.

---

## 1. Amendment law

```text
Existing specs retain ownership of their primitives.
Spec 135 owns the integrated professional-workspace, C.R.I.S.T.,
and multiplexed Mission Canvas product path.
Where Spec 135 adds a required integration or surface, the originating spec
remains the primitive owner and Spec 135 remains the product-completion owner.
```

A cross-spec dependency cannot be excluded from Spec 135 closure merely because implementation occurs under another spec’s task tree.

---

## 2. Precedence law

When documents disagree:

1. current canonical runtime truth and reducer-backed authority win over UI projections;
2. explicit operator direction and latest approved spec amendment win over older design direction;
3. Specs 98 and 104 prohibit daemon-global singleton authority and govern typed project/workstream/attachment scope;
4. Specs 40 and 41 govern Instance/Session/Attachment multiplexing and concurrent proposal resolution;
5. Spec 133 governs durable Silent Sessions, runs, leases, and worktree isolation;
6. Spec 135 governs the complete professional-workspace and C.R.I.S.T. integration path;
7. the primitive-owning spec governs the internal semantics of that primitive;
8. Spec 135F governs the integrated domain-semantic registry, candidate/canonical graphs, domain packs, verification policies, slice policies, semantic subscriptions, and V1 compatibility projection;
9. Spec 135G governs Mission Canvas naming, Work Surfaces, multiplexed interaction, browser-context isolation, and session routing;
10. Spec 135A governs workspace projection and vertical UX;
11. Spec 135B governs C.R.I.S.T. Project Genesis;
12. Spec 135C governs UIAI rich-artifact, browser-origin, and live-refresh integration;
13. Spec 135D governs implementation ordering, no-deferral, framework reuse, multiplexing dependencies, and closure discipline;
14. Spec 135E governs cross-spec interpretation and migration.

No UI or adapter may override canonical Focusa state to resolve a disagreement.

---

## 3. Naming constitution

The Spec 135 series uses these names normatively:

```text
UIAI Engine Cockpit
  The only product or interface that uses the word Cockpit.

Focusa Mission Canvas
  The multiplexed interactive Focusa/Pi workspace projection.

Work Surface
  One tab, pane, split, or detached view inside a Mission Canvas.

Mission Deck
  The standalone guided Focusa PWA/experience.

Spec Workbench
  The Spec 120 adversarial specification environment.
```

Non-UIAI terms such as Focusa Cockpit, professional cockpit, runtime cockpit, Pi cockpit, Tauri cockpit, and cockpit view are deprecated and must migrate to Mission Canvas, Work Surface, Runtime View, or another precise non-Cockpit term.

Legacy code/test identifiers may retain compatibility aliases temporarily only under an explicit migration task. New implementation labels and the Spec 135 series may use Cockpit only as part of the exact name **UIAI Engine Cockpit**.

---

## 4. Cross-spec amendment matrix

| Existing spec/doc | Preserved ownership | Spec 135 amendment/integration requirement |
|---|---|---|
| [38 Thread Thesis](38-thread-thesis-spec.md) | cognitive workspace/thread thesis | Work Surfaces project thread/workstream identity without becoming a second thread model. |
| [39 Thread Lifecycle](39-thread-lifecycle-spec.md) | thread lifecycle and persistence | Mission Canvas suspension/rehydration must preserve thread/workstream lifecycle truth. |
| [40 Instance/Session/Attachment](40-instance-session-attachment-spec.md) | multiplexing engineers, runtime Instance/Session/Attachment identities and roles | Spec 135G makes these visible and routable through Work Surfaces and session inventory. |
| [41 Proposal Resolution Engine](41-proposal-resolution-engine.md) | timestamped async concurrency, pending proposals, deterministic resolution | Mission Canvas must show contention, origin session, evidence, outcome, and fork/compare actions. |
| [43 Multi-device Sync](43-multi-device-sync.md) | local-first bidirectional synchronization | Mission Canvas/project/session projections sync without turning device-local focused view into authority. |
| [45 Ontology Overview](45-ontology-overview.md) | additive ontology as semantic spine and reducer-owned truth boundary | Spec 135F operationalizes one shared semantic substrate beneath professional workspaces without creating a second runtime. |
| [46 Ontology Core Primitives](46-ontology-core-primitives.md) | ObjectType, Property, LinkType, ActionType, Status, ObjectSet, Constraint, Mission, provenance, verification, delta, and slice policy | Definitions become a core-owned versioned registry with generated contracts and exact V1 aliases. |
| [48 Ontology Links/Actions](48-ontology-links-actions.md) | typed relations and actions | Domain packs extend registered relations/actions; clients and workspace manifests may reference but not redefine them. |
| [49 Working Sets/Slices](49-working-sets-and-slices.md) | bounded working sets, membership classes, and slice bounds | Spec 135F adds a generalized slice-policy registry and domain-specific policies while preserving current software slice names. |
| [50 Classification/Reducer](50-ontology-classification-and-reducer.md) | proposal-only ambiguity and reducer canonicalization | Candidate and canonical graphs are separated; promotion requires registered verification/promotion policy. |
| [61 Domain-General Cognition](61-domain-general-cognition-core.md) | reusable Mission, Goal, Task, Decision, Constraint, Risk, Blocker, OpenLoop, WorkingSet, ActionIntent, Verification, Checkpoint, and EvidenceArtifact cognition | Becomes the mandatory shared pack beneath software, legal, markets, research, general, custom, and composite domain packs. |
| [70 Shared Status/Lifecycle](70-shared-interfaces-statuses-and-lifecycle.md) | shared statuses and lifecycle | Registry validation and generated clients consume one canonical status/lifecycle contract with versioned extensions. |
| [74 Identity Resolution](74-identity-and-reference-resolution.md) | canonical entities, aliases, candidates, resolution, and supersession | Domain-pack, session-origin, and legacy aliases use one resolution system; migration may not invent new canonical identity silently. |
| [77 Ontology Governance](77-ontology-governance-versioning-and-migration.md) | ontology versions, compatibility profiles, migration plans, deprecation, governance decisions | Spec 135F/135E require snapshot/event/domain-pack versions, minimum readers/writers, downgrade guards, migration evidence, and conformance checks. |
| [72 Agent Identity/Role](72-agent-identity-role-and-self-model-ontology.md) | AgentIdentity, RoleProfile, CapabilityProfile, PermissionProfile, Responsibility, HandoffBoundary | C.R.I.S.T. Role Composer must materialize a project-scoped RoleProfile while keeping permission separate. |
| [75 Projection/View](75-projection-and-view-semantics.md) | Projection, ViewProfile, ProjectionRule, ProjectionBoundary | Professional workspaces and Work Surfaces become versioned projections with switch/migration/fidelity verification. |
| [88 Workpoint Continuity](88-ontology-backed-workpoint-continuity.md) | canonical continuation and action authority | C.R.I.S.T. task activation and Work Rail rows bind to Workpoints; workspace/project profile/Work Surface never replaces Workpoint. |
| [98 Project-root CRDT](98-project-root-crdt-reconciliation-foundation-spec.md) | ProjectRootKey, WorkstreamKey, AttachmentKey, CRDT reconciliation, no silent merge | Mission Canvas and all clients must preserve root/workstream/attachment scope and visible conflict resolution. |
| [100 Context Cognition](100-context-cognition-spec.md) | bounded advisory context selection | Project Context corpus feeds Context Cognition; corpus is not dumped into prompts. |
| [104 Singleton Elimination](104-typed-scoped-runtime-and-singleton-elimination-spec.md) | zero authority-bearing singleton state across daemon/API/CLI/Pi/menubar/adapters | Focused Work Surface, selected tab, selected browser target, and active client view remain non-authoritative projections. |
| [107 Spec-First Lifecycle](107-spec-first-feature-lifecycle-and-claim-discipline-spec.md) | Idea → Spec → Tasks → Implementation → Proof → Closure | C.R.I.S.T. explicitly splits C.R.I.T. Task into governed Spec and Tasks stages. |
| [109 AX API](109-agent-first-api-redesign-ax-spec.md) | schemas, capabilities, preview/commit, idempotency, versioning, envelopes | Spec 135 APIs/events/types, including Mission Canvas/session/browser identity, must be generated and exposed through AX contracts; no placeholder success envelopes. |
| [111 Bootstrap](111-agent-context-bootstrap-and-delivery-spec.md) | bounded context delivery to agents | Active Workspace, Role, Genesis Spec, task, Workpoint, and explicit Attachment/session summaries become bounded preload inputs. |
| [116 Work-Item Closure](116-provider-neutral-work-item-closure-authority-spec.md) | provider-neutral work items and closure truth | Work Rail and C.R.I.S.T. task materialization consume Spec 116; required providers must have real adapters. |
| [117 Mission Deck](117-mission-deck-onboarding-recall-pwa-spec.md) | beginner/operator Mission Deck, next safe action, onboarding, Recall | Adds Quick Mission vs Full C.R.I.S.T. Genesis, Workspace selector, Mission Canvas launch/session overview, C.R.I.S.T. progress, and rich work/artifact views. |
| [117A Living Mission Field](117a-living-mission-field-pwa-spec.md) | living, non-admin PWA experience | Professional workspaces and multiplexed session views must retain living-field qualities and not become generic enterprise dashboards. |
| [119 Receipts](119-verifiable-agent-work-receipts-and-governed-execution-ledger-spec.md) | work receipts and execution ledger | Genesis, role approval, interview readiness, spec approval, task materialization, workspace change, session termination, browser-context cleanup, and completed work produce/reference Receipts. |
| [120 Spec Workbench](120-adversarial-spec-workbench-and-operator-approval-gates.md) | reality scanner, research, adversarial drafting, section gates, reconciliation, task plan | C.R.I.S.T. `S` and `T` invoke Spec 120; Project Genesis becomes a specialized template/handoff rather than a second spec system. |
| [121 Menubar](121-menubar-rearchitecture-spec.md) | typed/enveloped Svelte/Tauri data discipline | Shared generated contracts and design packages extend to workspace/Genesis/Receipt/Mission Canvas/session data. |
| [121A Menubar Living Field](121a-menubar-discipline-and-living-field-spec.md) | compact ambient menubar posture | Menubar shows peeks, session counts, progress, approvals, and conflicts; it does not become the full Mission Canvas. |
| [124 Project/First Mission](124-focusa-cli-redesign-project-dashboard-project-creation-scoped-authority-first-mission-command-hierarchy-and-launch-hardening-spec.md) | project creation, selection, templates, settings, Quick First Mission | Adds `project genesis/context/role/interview/spec/tasks` and `mission-canvas`; existing First Mission remains Quick Mission and selected project remains non-authoritative. |
| [125 Mandatory Trajectory](125-mandatory-trajectory-nonlazy-hlt-pi-receipt-ontology-interlock-spec.md) | mandatory HLT, trajectory history, Pi continuity | Project Genesis Spec must define/propose HLT/MLG/STG/Waypoints; generic/missing HLT remains loudly degraded and not silently accepted. |
| [130 Compaction Mission Packet](130-hlt-aware-compaction-mission-packet-and-bloatgaurd-context-firewall-spec.md) | bounded hot context, mission packet, context firewall | Workspace/Role/Genesis/session summaries enter compaction through bounded refs; source corpus, other sessions, and rich artifacts remain externalized. |
| [133 Durable Silent Sessions](133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md) | durable governed execution sessions, runs, leases, worktree isolation | Mission Canvas presents many Silent Sessions independently; close-view is distinct from pause/terminate and writer leases remain enforced. |
| [135G Multiplexed Mission Canvas](135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md) | Work Surface schema, session routing, browser-context isolation, restoration, proof | Required companion; prevents single-active-session UI from satisfying Spec 135. |
| UIAI Hand-in-Glove spec | browser/research/diagnostic proof execution | Spec 135C adds rich Workspace Artifact descriptors, browser context/target origin, targeted invalidation, and professional renderers without moving authority to UIAI. |

---

## 5. Spec 117 / Mission Deck amendment

Spec 117’s First Encounter flow is expanded:

```text
Daemon/project readiness
→ choose Quick Mission or Full C.R.I.S.T. Genesis
→ open Focusa Mission Canvas when concurrent work begins
```

### Quick Mission

Preserves:

```text
bind project
→ create Workpoint
→ attach proof
→ resume
```

### Full C.R.I.S.T. Genesis

Adds:

```text
context
→ role
→ interview
→ Project Genesis Spec
→ tasks
→ first Workpoint
→ workspace
→ Mission Canvas Work Surfaces
```

Mission Deck must show:

- C.R.I.S.T. progress;
- workspace selector;
- context/connector health;
- role approval state;
- interview readiness;
- Workbench approvals;
- task-plan status;
- active-session and waiting-input summary;
- Work Rail;
- rich artifacts;
- Open Mission Canvas action.

Spec 117’s next-safe-action law remains intact: each state exposes one primary next action.

---

## 6. Spec 120 / Workbench amendment

Spec 120 gains a Project Genesis entry template and C.R.I.S.T. handoff contract.

It must accept:

```yaml
project_genesis_input:
  project_root:
  continuity_id:
  workspace_profile_ref:
  context_pack_refs: []
  accepted_context_claim_refs: []
  role_profile_ref:
  interview_session_refs: []
  unresolved_questions: []
  contradictions: []
  requested_template: project_genesis
```

Spec 120 remains the sole adversarial spec engine.

Its task decomposition must include the Spec 135D no-deferral directive, Complete Feature Ledger mapping, and Spec 135G multiplexing requirements.

A Project Genesis Spec cannot be considered approved until mandatory HLT/Trajectory, evidence policy, authority, privacy, workspace, context, role, task-decomposition, and concurrency/isolation sections pass whole-spec reconciliation.

---

## 7. Spec 124 / project command amendment

Add canonical command families:

```text
focusa project genesis
focusa project profile
focusa project context
focusa project role
focusa project interview
focusa project spec
focusa project tasks
focusa workspace
focusa mission-canvas
focusa sessions
focusa attachments
```

`focusa first-mission` remains the Quick Mission path for immediate proof and evaluator value.

Project selection remains convenience only. Every canonical C.R.I.S.T. or session mutation requires explicit verified project root and continuity scope.

Project templates may recommend a workspace and starter interview domains, but they may not pre-approve context claims, role, spec, authority, shared browser contexts, or broadcast steering.

---

## 8. Spec 121 / shared UI amendment

The existing SvelteKit/Tauri application becomes a consumer of shared generated contracts and design/Mission Canvas packages.

Required shared layers:

```text
focusa-contracts
focusa-client
focusa-design-system
focusa-workspace-ui
focusa-mission-canvas-ui
```

The menubar remains compact under 121A:

- current workspace identity;
- latest Workpoint/proof;
- active/waiting/blocked session counts;
- C.R.I.S.T. progress/approval badge;
- latest Receipt;
- connector degradation warning;
- contention/approval warning;
- launch/open actions.

The full Project Genesis, Interview, Spec Workbench, Work Rail, multiplexed Mission Canvas, and rich artifact experience belongs in Pi/Mission Deck/UIAI Engine Cockpit rather than equal menubar tabs.

---

## 9. Current-versus-target truth register

### Implemented foundations

- project create/list/discover/use/settings;
- First Mission Quick route;
- ProjectIdentity;
- Workpoint/Trajectory/Evidence;
- SSE stream;
- Pi settings/profile-like controls;
- Beads adapter;
- Svelte/Tauri menubar stack;
- UIAI browser/search/screenshots/diagnostics/FPV/artifacts;
- typed scoped-state and singleton-elimination implementation evidence;
- durable Silent Session architecture and partial runtime surfaces.

### Partial foundations

- Mission Deck onboarding;
- Work-loop/task projections;
- provider-neutral task ecosystem;
- Receipts;
- native themes;
- UIAI-to-Pi rich rendering;
- project/profile PWA concepts;
- Instance/Session/Attachment and scoped runtime substrate without complete multiplexed Mission Canvas UX;
- reducer-backed ontology state/events/projections and Pi ontology context, but with route-local string catalogs, permissive verification, and no complete candidate/canonical or domain-pack substrate.

### Normative target / implementation gaps

- Spec 120 runtime Workbench;
- C.R.I.S.T. state/orchestrator;
- source adapter registry and OAuth connectors;
- context claims and impact assessment;
- Role Composer;
- persistent interview compendium;
- Workspace View Profile registry;
- Mission Canvas and Work Surface contracts;
- Pi dock/sidebar/session switcher contract;
- full Instance/Session/Attachment inventory and routing;
- proposal/contention UI;
- browser-context and browser-target isolation model;
- all provider adapters;
- Workspace Artifact bridge with session-origin identity;
- complete vertical workspaces;
- complete client parity;
- core-owned typed ontology registry;
- separate candidate and canonical semantic graphs;
- policy-backed verification and promotion ledger;
- versioned domain packs and generalized slice policies;
- ontology-derived Workpoint candidate projection;
- semantic delta subscriptions;
- V1 ontology/snapshot/event compatibility and downgrade protection.

No UI, docs, release note, or sales claim may collapse these categories.

---

## 10. Migration model

### 10.1 Existing projects

Existing projects remain valid and may start C.R.I.S.T. later.

Import sources:

- project marker;
- repository docs/code;
- Project Card;
- Trajectory/HLT;
- Workpoint;
- work items;
- Evidence/Receipts;
- project settings.

Inferred values remain candidate/advisory until operator review.

### 10.2 Existing Pi settings

Current `focusaPiBridge` profile/settings remain supported during migration.

Migration maps:

```text
current work-loop profile
→ operational policy reference

current theme
→ visual variant

new workspace selection
→ Workspace View Profile

current client-local session selection
→ focused Work Surface projection only
```

No old setting silently gains broader authority.

### 10.3 Existing TUI and generic UI names

Fixed tabs remain functional while dynamic registry and Mission Canvas support are introduced.

Migration path:

```text
existing Tab/view
→ registered panel or Work Surface renderer
→ Mission Canvas/profile selection
→ deprecation warning only after parity proof
```

Any non-UIAI use of Cockpit migrates to Mission Canvas, Work Surface, Runtime View, or another precise non-Cockpit term. UIAI Engine Cockpit keeps its established name.

### 10.4 Existing UIAI Pi results and browser sessions

Text-result behavior remains fallback-compatible.

New clients consume Workspace Artifact descriptors with project/workstream and session-origin refs. Old clients continue receiving compact text and stable refs.

Existing UIAI sessions lacking browser-context identity are labeled legacy/shared-context posture until migrated. They must not be silently presented as isolated.

### 10.5 Existing task providers

Beads remains operational. Other providers become visible only when real adapters pass health and integration proof.

### 10.6 Existing Instance/Session/Attachment and client-local singleton state

Existing scoped runtime identities remain authoritative. Client fields such as `activeProject`, `currentSession`, `activeWorkpoint`, `currentBrowser`, or selected tab must be classified as:

- scoped current state under a WorkstreamKey;
- keyboard-focused client projection;
- legacy authority-bearing singleton requiring removal.

Mission Canvas migration must not create a second session model or adopt a prior session from transcript/client memory.

### 10.7 Existing ontology state, snapshots, events, and clients

Current ontology objects, links, proposals, verifications, routes, action names, slice names, Pi context behavior, and persisted state remain available through an explicit V1 compatibility projection. Migration must:

- preserve the original serialized payload and stable references;
- classify ambiguous historical entries as legacy rather than silently upgrading trust;
- dual-read and, during the controlled transition, dual-project V1/V2 state;
- prove replay-equivalent V1 projections from archived snapshots and event logs;
- preserve unknown future semantic types and event envelopes without reinterpretation;
- fail closed or enter read-only compatibility mode when an authoritative event requires a newer reader;
- block incompatible old writers after V2 canonical state exists;
- retain old client bounded text/JSON behavior while exposing additive V2 contracts to capable clients.

No migration failure may be converted into an empty fresh state without explicit recovery and preserved backup.

---

## 11. Profile and schema versioning

Required:

```yaml
snapshot_schema_version:
stored_event_envelope_version:
minimum_reader_version:
minimum_writer_version:
ontology_registry_version:
shared_cognition_pack_version:
domain_pack_versions: []
mission_canvas_schema_version:
work_surface_schema_version:
instance_session_attachment_version:
browser_context_schema_version:
profile_version:
compatibility_version:
deprecated_field_ids: []
deprecated_panel_ids: []
deprecated_semantic_ids: []
deprecated_ui_names: []
migration_ref:
fallback_profile:
fallback_domain_pack:
```

Unknown incompatible profiles degrade to General workspace with an explicit migration warning. Unknown session kinds, attachment roles, browser isolation classes, or semantic definitions remain preserved unsupported references; they may not be coerced into a known type. A runtime below `minimum_writer_version` must not write the state store, even when it can render a bounded legacy projection.

Migration must preserve:

- project scope;
- canonical state;
- Instance/Session/Attachment identity;
- operator choices;
- workspace intent;
- Mission Canvas Work Surface refs and unread cursors;
- accessibility preferences;
- artifact/history references;
- browser-context isolation posture.

---

## 12. Public/private and export boundary

Spec 135 is public product architecture.

Public docs may contain:

- product architecture;
- workspace and Mission Canvas behavior;
- schemas;
- authority model;
- connector boundaries;
- browser isolation classes;
- UX and proof requirements.

Private project/customer data, raw connected content, OAuth credentials, browser cookies/storage, private transcripts, pricing/strategy details, and unredacted artifacts remain outside public projections.

External exports follow Spec 120/119 classification, redaction preview, approval, and Receipt requirements.

---

## 13. Whole-series closure matrix

| Required outcome | Governing spec | Closure proof |
|---|---|---|
| Canonical product contract | 135 | approved master + all companions linked |
| Domain-general ontology and semantic packs | 135F + 45–50 + 61 + 70 + 74 + 77 | registry/candidate/canonical/verification/slice/subscription runtime proof plus V1 replay equivalence |
| Multiplexed Mission Canvas and runtime attachments | 135G + 40 + 41 + 98 + 104 + 133 | multi-project/session/browser-isolation/contention/rehydration/writer-lease proof |
| Workspace projection and vertical UX | 135A | live cross-client workspace and Work Surface switch/split visual proof |
| Context ingestion and growth | 135B | real local/Google/Microsoft/mail/UIAI/task-provider sync evidence |
| Role and Interview | 135B | approved role revision + persistent interview resume evidence |
| Project Genesis Spec | 135B + 120 | adversarial section approvals + reconciliation + final approval |
| Tasks and first Workpoint | 135B + 116 + 120 | preview/approval/materialization/provider refs/Workpoint |
| UIAI artifacts and live refresh | 135C | screenshot/research/multi-context FPV evidence + targeted automatic rerender |
| No-deferral/build discipline | 135D | Complete Feature Ledger with no incomplete required entries |
| Compatibility/migration | 135E | existing project/client/session/name migration evidence |
| Receipts/closure truth | 119 + 116 | stable Receipt refs and provider reconciliation |
| Release claim | 107 + 135D | actual evidence across required clients, sessions, and platforms |

---

## 14. Acceptance criteria

Spec 135E is accepted when:

1. All affected specs are cross-linked in docs/indexes and implementation tasks.
2. Primitive ownership and integrated product ownership are explicit.
3. Quick Mission and Full C.R.I.S.T. Genesis coexist.
4. Spec 120 accepts the Project Genesis handoff/template, no-deferral decomposition law, and multiplexing requirements.
5. project, workspace, Mission Canvas, session, and attachment commands are specified without hidden authority.
6. shared UI-package boundaries are adopted.
7. current/partial/target capability truth is visible.
8. existing projects, settings, TUI, UIAI text clients, session identities, and Beads migrate without breakage.
9. profile/schema/session/name migration behavior is proven.
10. public/private export boundaries remain intact.
11. Whole-series closure is mechanically blocked by any incomplete companion requirement.
12. Spec 135F is cross-linked, decomposed, implemented, and proven as a required companion rather than a future ontology initiative.
13. Spec 135G is cross-linked, decomposed, implemented, and proven as a required companion rather than future UI work.
14. V1 ontology state, snapshots, events, routes, Pi behavior, clients, and Beads workflows migrate without destructive rewrites or hidden trust escalation.
15. Unknown-event preservation, minimum-reader/writer enforcement, downgrade-write protection, and failed-migration recovery are proven.
16. Focused Work Surface and selected client tab remain non-authoritative projections.
17. Existing UIAI sessions migrate with truthful browser-context isolation posture.
18. The word Cockpit is used only for UIAI Engine Cockpit in the Spec 135 series and new implementation labels.

---

## 15. Closure blockers

This spec cannot close while:

- an existing spec is implicitly replaced rather than amended;
- primitive ownership is ambiguous;
- Specs 40, 41, 98, 104, 133, or 135G disappear from the Complete Feature Ledger;
- cross-spec dependencies disappear from the Complete Feature Ledger;
- existing projects require destructive migration;
- selected-project, focused-Work-Surface, or selected-tab convenience state becomes hidden authority;
- old clients lose bounded fallback behavior;
- docs-only targets are described as implemented;
- public projections expose private context or browser storage;
- any companion spec remains incomplete;
- existing ontology state requires destructive in-place migration;
- an unknown future authoritative event is dropped, reinterpreted, or causes silent fresh-state fallback;
- an incompatible older daemon can write V2 canonical semantic state;
- domain-pack activation changes permission or Workpoint authority implicitly;
- UIAI browser-context isolation is guessed or falsely represented;
- non-UIAI surfaces retain Cockpit naming without a compatibility migration plan.
