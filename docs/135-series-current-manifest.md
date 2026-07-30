# Spec 135 Series — Current Authoritative Delivery Contract

**Status:** current, normative, frozen companion and delivery manifest  
**Parent:** [Spec 135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md)  
**Authority:** This document resolves product intent, host and renderer ownership, implementation ordering, testing, generated UI, cross-repository compatibility, decomposition, and closure conflicts across the Spec 135 series. When older companion wording, implementation code, tests, proof files, issue interpretations, or diagrams conflict with this contract, this contract governs.

Spec 135 and every companion below form one required implementation and closure set. The series is frozen at **135K**. No additional lettered companion is created for implementation clarification. Corrections are consolidated into this delivery contract, its machine-readable contracts, and the affected existing documents.

**Required current-reality audit:** [SPEC135_REALTIME_GENERATED_UI_SPEED_AND_CORE_INTEGRATION_AUDIT_2026-07-18.md](current/SPEC135_REALTIME_GENERATED_UI_SPEED_AND_CORE_INTEGRATION_AUDIT_2026-07-18.md)

## 0. Operator intent lock — Mission Canvas inside the Pi experience

This section is the first authority an agent must apply before changing Mission Canvas, Pi UI, generated UI, workspace profiles, Work Surfaces, or Spec 135 closure state.

### 0.1 Initial product vision

```text
Pi terminal interaction
        ⇅ one operator-controlled light switch
Focusa Mission Canvas rich professional GUI
```

The switch is invoked directly from Pi. It changes the active **presentation**, not the agent, project, session, Attachment, runtime, model stream, tools, Workpoint, Evidence, queues, or history.

```text
Canvas OFF
  Stock Pi remains the primary interaction surface.
  Focusa remains active and provides bounded terminal guidance.

Canvas ON
  Pi opens or focuses the full Focusa-owned rich Mission Canvas.
  The current Pi session becomes a pi_session Work Surface inside the Canvas.
  The same live agent, transcript, tools, canonical state, and queues continue.

Canvas OFF again
  The rich projection closes or unmounts.
  Keyboard focus returns to stock Pi.
  No runtime work is stopped or recreated.
```

“Directly in Pi” means Pi owns the command/shortcut/tool, exact session binding, status, lifecycle, and focus handoff. A portable graphical interface is rendered in a Focusa-owned local webview/window. Agents must not pretend that HTML/CSS, A2UI, Svelte, Lit, charts, CodeMirror, PDF.js, or rich document geometry can be rendered faithfully inside a terminal cell grid.

A box-drawing full-screen TUI may be useful as a terminal projection, but it is not the required rich GUI and must not be labeled as such.

### 0.2 Interaction mode and host renderer are separate typed axes

```text
Interaction mode
  canvas-guided
  terminal-guided
  headless

Host renderer
  focusa_pi_rich_window
  uiai_engine_cockpit
  mission_deck_web
  pi_terminal_projection
  native_tui
  menubar_peek
  headless_none
```

`canvas-guided` means that a Mission Canvas projection is active. It does not, by itself, mean terminal TUI, web, Tauri, UIAI Engine Cockpit, or any other renderer.

Required resolution:

```text
canvas-guided + Focusa-enhanced Pi
  → focusa_pi_rich_window

canvas-guided + terminal-only or compatibility environment
  → pi_terminal_projection, visibly labeled as a terminal fallback

canvas-guided + UIAI Engine Cockpit
  → rich Focusa projection hosted in the distinct UIAI-owned desktop product

terminal-guided
  → rich Canvas absent; stock Pi plus concise Focusa guidance

headless
  → no human UI calls, windows, prompts, notifications, or renderer activation
```

The machine-readable authority is:

`docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml`

Agents and tests must consume that contract rather than infer the model from prose or screenshots.

### 0.3 Required Focusa-enhanced Pi rich host

The primary rich Canvas host for the Focusa-enhanced Pi distribution is a **Focusa-owned Mission Canvas webview/window**, controlled from Pi and bound to the same live Pi session through typed scope and Attachment identity.

Required implementation shape:

```text
Pi command / shortcut / tool
→ exact ProjectRootKey + WorkstreamKey + Session + Attachment binding
→ start, reuse, focus, hide, or close Focusa Mission Canvas window
→ SvelteKit 2 / Svelte 5 shell
→ A2UI web_core + permanent Lit renderer for generated surfaces
→ Focusa Svelte Custom Elements
→ canonical Focusa API, Operation Registry, durable replay, and live tail
```

This is not a second product runtime. It does not fork Pi’s session manager, model stream, tools, transcript, or canonical Focusa state.

The current Pi transcript is one `pi_session` Work Surface. It does not become the entire Canvas shell.

### 0.4 Canonical Mission Canvas shell anatomy

All rich hosts preserve this ordered composition:

```text
┌ WORK SURFACES ─────────────────────────────────────────────────────────────┐
│ Overview · Pi · UIAI · Silent · Documents · Research · Evidence · custom │
├──────────────────────── FOCUSED WORK SURFACE ────────────┬── FOCUSA ─────┤
│ Pi transcript, browser projection, document, artifact,   │ Project/scope  │
│ research, generated C.R.I.S.T. surface, comparison,      │ active session │
│ Evidence, terminal, code diff, chart, redline, or result │ Workpoint      │
│                                                          │ next safe work │
│                                                          │ proof/authority│
│                                                          │ contention     │
├──────────────────────────────────────────────────────────┴────────────────┤
│ WORK RAIL · surface-local / project aggregate / labeled advisory         │
├───────────────────────────────────────────────────────────────────────────┤
│ STEERING QUEUE · explicit Attachment/session target                      │
├───────────────────────────────────────────────────────────────────────────┤
│ FOLLOW-UP QUEUE · explicit Attachment/session target                     │
├───────────────────────────────────────────────────────────────────────────┤
│ PROMPT EDITOR · focused Work Surface is the default recipient             │
└───────────────────────────────────────────────────────────────────────────┘
```

Optional host/profile regions may include a global scope bar, left activity rail, detached inspector, secondary pane, or compact launcher. They are not canonical conformance requirements and must not displace the ordered shell invariant.

A sidebar alone, status-card dashboard alone, Markdown transcript projection, or terminal shell does not satisfy the rich Mission Canvas requirement.

### 0.5 Workspace verticals

General, Software, Legal, Markets, Research, Custom, and composite profiles project the same canonical state. Switching profiles must recompose:

```text
layout geometry
panel composition and order
terminology
artifact renderer bindings
Evidence and verification emphasis
history projection
iconography
density
controls and next-action emphasis
```

Color-only switching is nonconformant. Hard-coded separate client applications per vertical are forbidden. Profiles resolve through shared registries and domain semantic bindings.

### 0.6 C.R.I.S.T. generated UI boundary

A2UI, `@a2ui/web_core/v0_9`, `@a2ui/lit/v0_9`, and Focusa Svelte Custom Elements render generated onboarding and C.R.I.S.T. interaction surfaces **inside Work Surfaces** in rich hosts.

A nontechnical generated UI path is required for every C.R.I.S.T. stage.

They do not automatically own the complete Mission Canvas shell, canonical runtime, permissions, workflow authority, or history.

A C.R.I.S.T. stage represented only as a transcript message, Markdown/JSON dump, static form, CLI selection, or decorative status panel is incomplete.

### 0.7 Toggle continuity invariants

The following survive Canvas on/off, restart, reconnect, and rehydration:

```text
ProjectRootKey and WorkstreamKey
Instance, Session, Attachment, and harness session reference
model stream and tool runtime
transcript and tool history
unsent Pi editor draft and unsent Canvas draft
Trajectory and Workpoint
task/provider state
open and focused Work Surfaces
Canvas layout and groups
steering and follow-up queues
Evidence and Receipts
authority, permissions, approvals, and contention
browser session/context/target identity
durable event cursor and history
```

Forbidden toggle effects:

```text
project or session restart
canonical state fork
Workpoint recreation
chat-tail reconstruction as authority
browser-context reassignment
loss of drafts or queues
repeated invitation to enable the Canvas
```

### 0.8 Product boundary

```text
Focusa Pi rich Mission Canvas
  Focusa professional workspace shell, Work Surfaces, Work Rail, queues,
  Focusa state, generated surfaces, vertical projection, and continuity.

UIAI Engine Cockpit
  Distinct UIAI-owned rich desktop product for browser execution, browser
  contexts/targets, FPV, Test Lab, Documents, diagnostics, and browser proof.
```

UIAI Engine Cockpit may host Focusa projections. It remains distinct. The Focusa Pi rich host must not be called a Cockpit.

### 0.9 Proof and closure firewall

Rich GUI proof requires runtime evidence of:

1. toggle invoked from Pi;
2. an actual Focusa-owned rich webview/window;
3. same Session and Attachment before, during, and after toggle;
4. live transcript/tool continuity and unsent-draft preservation;
5. real Work Surface switching, grouping, splits, suspension, and rehydration;
6. real vertical recomposition;
7. real generated C.R.I.S.T. interaction;
8. durable replay/reconnect;
9. responsive, accessibility, and visual proof through UIAI Engine Eval;
10. Evidence and Receipt references from the tested runtime.

The following cannot prove a rich GUI:

```text
source substring checks
handwritten pass JSON
static screenshot without runtime trace
Markdown representation of a split
process-local map presented as durable layout
box-drawing terminal shell
transcript-only C.R.I.S.T. stage
```

Until re-proven against this contract, claims for full rich Pi Mission Canvas, real rich split panes, full vertical workspace rendering, generated C.R.I.S.T. GUI inside the Canvas, and final Spec 135 acceptance remain reopened.

## 1. Required series

| Order | Spec | Required subject |
|---:|---|---|
| 1 | [135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md) | Master product, authority, workspace, C.R.I.S.T., and closure contract |
| 2 | [135A](135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md) | Workspace projection, Mission Canvas, Work Rail, themes, and vertical UX |
| 3 | [135B](135b-crist-project-genesis-context-role-interview-spec-tasks.md) | Context, Role, Interview, Spec, Tasks, and Project Genesis state |
| 4 | [135C](135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md) | UIAI artifacts, browser identity, FPV, and live refresh |
| 5 | [135D](135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md) | Complete implementation graph, framework reuse, performance, and no-deferral law |
| 6 | [135E](135e-cross-spec-amendments-migration-and-closure-matrix.md) | Cross-spec amendments, migration, compatibility, and closure |
| 7 | [135F](135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md) | Ontology core, semantic graphs, domain packs, verification, and reactive context |
| 8 | [135G](135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md) | Multiplexed Work Surfaces, sessions, attachments, browser isolation, and restoration |
| 9 | [135H](135h-cross-functional-alpha-grill-interview-and-implementation-acceleration-spec.md) | Grill Interview, Cross-Functional Alpha, decided OSS stack, and speed law |
| 10 | [135I](135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md) | Real-time generated C.R.I.S.T. UI, nontechnical onboarding, A2UI, and generated action surfaces |
| 11 | [135J](135j-core-api-operation-registry-durable-ui-stream-and-runtime-reuse-hardening-spec.md) | Core API Operation Registry, durable replayable stream, shared envelopes, and runtime reuse |
| 12 | [135K](135k-uxp-ufi-adaptive-generated-ui-friction-learning-and-nontechnical-usability-spec.md) | Canonical UXP/UFI reuse, adaptive generated UI, friction learning, and usability proof |

No companion is optional. Dependency ordering does not remove a requirement. This delivery contract governs conflicting implementation wording.

## 2. Final framework and ownership decisions

Agents must not reopen these decisions as option menus.

### 2.1 Browser execution and proof

UIAI Engine owns browser navigation, actions, DOM/accessibility snapshots, screenshots, viewport changes, contexts, authentication state, console/network diagnostics, responsive verification, visual comparison, browser recovery, Eval scenarios, and browser proof artifacts.

Focusa must not add Playwright Test, Library, CLI, MCP, fixtures, or configuration. All browser-facing end-to-end, visual, responsive, reconnect, and accessibility proof uses UIAI Engine Eval.

### 2.2 Generated UI

```text
A2UI v0.9.1
@a2ui/web_core/v0_9
@a2ui/lit/v0_9
Focusa Svelte Custom Elements in a trusted catalog
```

The maintained Lit renderer is the permanent web renderer for A2UI generated surfaces. Focusa does not build a second message processor, `SurfaceModel`, data-binding system, or complete custom Svelte A2UI renderer.

### 2.3 Streaming

```text
SQLite canonical event history
+ stable event IDs and sequence
+ cursor / Last-Event-ID replay
+ existing broadcast live tail
+ scoped invalidation
+ A2UI snapshots and deltas
```

AG-UI is an external compatibility adapter. It owns no canonical state, history, approval, tool authority, or persistence.

### 2.4 Model execution

```text
Focusa typed operation
→ daemon-owned Spec 133 governed execution session
→ Pi RPC AgentExecutionAdapter reference implementation
→ structured result
→ Focusa reducer
→ Evidence / Receipt
→ generated UI delta
```

Vercel WorkflowAgent, ToolLoopAgent, AI SDK UI, `@ai-sdk/svelte`, AI Gateway as a required service, or Vercel-owned durable workflow authority are forbidden Focusa runtime dependencies.

### 2.5 Contracts and generated clients

```text
Rust: Serde + Schemars + Utoipa
JSON Schema 2020-12
OpenAPI 3.0.3
TypeScript: openapi-typescript + openapi-fetch
UIAI Engine adapter generated outside Focusa core
A2UI catalog schemas and action bindings
```

Handwritten duplicate Focusa DTOs, operation registries, route catalogs, or action authority are forbidden when generation can represent the contract.

### 2.6 Shared UI and data stack

```text
SvelteKit 2 · Svelte 5 · Tailwind CSS 4
shadcn-svelte · Bits UI · Paneforge
TanStack Query · Table · Virtual
Svelte Flow · PDF.js · CodeMirror Merge · Apache ECharts
```

These libraries provide mechanics. Focusa owns domain projection, visual grammar, accessibility, and authority-bound actions.

## 3. Complete implementation graph and Cross-Functional Alpha

### Foundation Train

```text
F0 — compile this Delivery Contract and machine-readable host/renderer contract
F1 — feature ledger, delivery DAG, parity, framework, proof, and closure matrices
F2 — JSON Schema 2020-12 and OpenAPI 3.0.3
F3 — generated TypeScript clients and portable contract validation
F4 — Operation Registry, capability projection, and UI action bindings
F5 — shared ToolResult/error envelopes
F6 — durable SQLite replay plus broadcast live tail
F7 — capability/permission projection and compatibility handshake
F8 — Pi RPC AgentExecutionAdapter
F9 — A2UI web_core plus permanent Lit generated-surface renderer
F10 — Focusa Svelte Custom Elements
F11 — first UIAI Engine Eval scenario
F12 — one real Context operation through generated UI
F13 — Focusa Pi rich Mission Canvas window host and lifecycle bridge
F14 — Pi light-switch continuity traversal with real Work Surfaces
```

### Cross-Functional Alpha

```text
Alpha 1 — real Markdown/code and PDF Context ingestion and generated UI
Alpha 2 — Role approval, Grill Interview, close, and resume
Alpha 3 — Spec 120 cycle, provider-neutral plan, and Beads task
Alpha 4 — Workpoint, Work Rail, Evidence, closure, and Receipt
Alpha 5 — UIAI artifact, Evidence link, event invalidation, and live refresh
Alpha 6 — Pi and UIAI Work Surfaces, Attachment targeting, and browser isolation
Alpha 7 — General, Software, Legal/Markets as applicable, and Research projection over identical state
Alpha 8 — permanent nontechnical dogfood traversal
Alpha 9 — Pi terminal ⇄ rich Mission Canvas light-switch traversal over the same live session
```

Every slice crosses:

```text
requirement ID
→ greater Focusa primitive
→ schema and reducer/persistence
→ typed API and Operation Registry
→ generated clients/contracts
→ client renderer
→ real integration
→ UIAI Engine Eval when browser-facing
→ tests
→ Evidence
→ Receipt
```

Static UI, mock providers, placeholder results, transcript-only state, CLI-only completion, duplicate DTOs, manual route catalogs, and non-replayable streams do not satisfy a slice.

## 4. Greater Focusa primitive-submission rule

```text
general reusable Focusa primitive
→ reducer and canonical state
→ typed Focusa API
→ generated cross-language contracts
→ C.R.I.S.T. or workspace projection
→ client renderer
→ UIAI Engine Eval proof where applicable
→ Evidence
→ Receipt
```

Operation descriptors, permissions, preview/commit, replay, result envelopes, Evidence, Context, Role, Interviews, tasks, Workpoints, Instances/Sessions/Attachments, Work Surfaces, generated surface envelopes, UXP/UFI, connector health, and browser artifact references remain general primitives.

C.R.I.S.T.-specific code is limited to stage orchestration, readiness, language, interaction intent, and stage composition.

## 5. Machine-readable decomposition inputs

Before implementation or closure, validate:

```text
docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml
docs/contracts/spec135-complete-feature-ledger.v1.yaml
docs/contracts/spec135-delivery-dag.v1.yaml
docs/contracts/spec135-client-parity-matrix.v1.yaml
docs/contracts/spec135-framework-lock.v1.yaml
docs/contracts/spec135-proof-matrix.v1.yaml
```

Every requirement has a stable ID, owner, dependency, implementation task, client surface, renderer class, test, Eval scenario, Evidence requirement, Receipt requirement, migration requirement, and closure status.

Agents must not infer the delivery graph, renderer class, or GUI completion from prose, screenshots, source strings, or issue titles alone.

## 6. Required agent reading order

Before decomposition or code changes, read:

1. this Delivery Contract;
2. `docs/contracts/spec135-mission-canvas-host-renderer-contract.v1.yaml`;
3. [Spec 135 implementation acceleration directive](agent/spec135-implementation-acceleration-directive.md);
4. [Spec 135 real-time generated UI directive](agent/spec135-real-time-generated-ui-directive.md);
5. [Spec 135 UXP/UFI generated UI directive](agent/spec135-uxp-ufi-generated-ui-directive.md);
6. the affected existing 135A–135K documents;
7. the machine-readable delivery graph and current runtime evidence.

The implementation audit distinguishes code reality from normative target. Docs-only, enum-only, mock, terminal-only, and self-asserted proof must remain visibly partial.

## 7. Progressive product and merge laws

Every merge leaves a truthful working product. Unavailable capability is labeled unavailable, degraded, compatibility fallback, blocked, credentials required, dependency incomplete, or upgrade required.

Every implementation PR must:

1. reference stable requirement IDs;
2. update machine-readable ledgers;
3. name its interaction mode and host renderer separately;
4. submit reusable behavior to the greater Focusa primitive owner;
5. update generated contracts when contracts change;
6. include unit, contract, component, runtime, and UIAI Engine Eval proof as applicable;
7. link Evidence and Receipt references;
8. preserve exact project, workstream, Attachment, and browser-context scope;
9. avoid unapproved frameworks and duplicate runtimes;
10. leave capability and closure state truthful.

The current terminal `MissionCanvasShell`, Markdown vertical projections, process-local layout map, transcript C.R.I.S.T. stage, and static GUI-proof JSON may be retained only as partial terminal or scaffolding work. They cannot close the rich host requirements.

## 8. Permanent dogfood gate

```text
Onboarding
→ Context
→ Role
→ Grill Interview
→ Project Genesis Spec
→ Tasks
→ Workpoint
→ Evidence
→ Receipt
→ Pi terminal-guided interaction
→ Canvas ON from Pi
→ Focusa rich Mission Canvas with the same live session
→ vertical switch over identical state
→ real Work Surface switch/split/rehydration
→ UIAI artifact and browser-isolation proof
→ Canvas OFF back to the same Pi session
→ pause
→ restart
→ resume exact state
```

Every user-facing C.R.I.S.T. step is completed through generated plain-language UI. Browser portions are proven exclusively through UIAI Engine Eval.

## 9. Closure rule

```text
No companion is optional.
No accepted requirement is deferred through sequencing language.
The Pi light-switch rich Mission Canvas is part of the accepted product vision.
Terminal fallback is required but cannot impersonate the rich GUI.
Generated UI uses canonical operations, events, authority, Evidence, and Receipts.
UX adaptation never changes shell invariants, authority, or proof requirements.
Browser proof uses UIAI Engine Eval and never Playwright.
The full series closes only when every machine-readable ledger entry is verified.
```

## 10. Current reading order

```text
135
→ this Delivery Contract
→ mission-canvas-host-renderer machine contract
→ agent directives
→ 135A → 135B → 135C → 135D → 135E → 135F → 135G → 135H → 135I → 135J → 135K
→ machine-readable delivery graph
→ implementation and runtime proof
```
