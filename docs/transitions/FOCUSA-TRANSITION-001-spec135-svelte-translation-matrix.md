# FOCUSA-TRANSITION-001 — Spec 135 Svelte Translation Matrix

**Contract:** `CARDINAL-135-SVELTE-001`

**Source authority:** unchanged Spec 135 master and 135A–135K

**Destination:** Focusa Desktop → Mission Canvas → Svelte GUI tab

**Separate surface:** Focusa Desktop → Agent TUI → authentic PTY-backed Pi terminal

## Cardinal interpretation

The Spec 135 series was written when Mission Canvas was planned as a Pi TUI overlay. Its requirements remain authoritative and are not reduced by the host transition. Every planned model, widget, command, interaction, state, workflow, proof obligation, and acceptance behavior is translated into the Svelte-hosted Mission Canvas or an approved generated renderer hosted by it.

Pi-overlay wording identifies source behavior. It does not authorize a terminal-only implementation, a fixed Svelte dashboard, fixture-only behavior, or omission. Visual handoffs define adaptive visual grammar and populated recomposition examples; they do not define permanent inventory.

Focusa Core continues to own contribution eligibility, layout resolution, identity, authority, operation binding, persistence, event replay, and recomposition. Svelte renders canonical `ResolvedWorkspaceProjection` output. A2UI/Lit and trusted Focusa Custom Elements remain the generated-UI execution path.

## Series reevaluation

| Source | Unchanged functional obligation | Svelte translation obligation |
|---|---|---|
| Spec 135 master | Mission Canvas, professional workspaces, C.R.I.S.T., activities, profiles, contextual actions, evidence, continuity | Implement the complete adaptive workspace in the Mission Canvas GUI tab; no route-defined collection of fixed dashboards |
| 135A | Workspace projection, Pi sidebar, Work Rail, vertical UX, no dead chrome, responsive composition | Translate overlay regions into projection-backed Svelte navigation, contextual Work Rail contributions, responsive recursive layouts, and absent-when-ineligible DOM |
| 135B | C.R.I.S.T. Project Genesis, Context/Role/Interview/Spec/Tasks, state transitions and task formation | Render governed generated C.R.I.S.T. workflows in Svelte-hosted A2UI/Lit surfaces with canonical draft, operation, evidence, and lifecycle bindings |
| 135C | UIAI rich artifacts, live refresh, research bridge, source provenance | Render exact UIAI artifact and evidence references as Work Surfaces; UIAI Engine retains browser execution and visual-proof authority |
| 135D | Complete implementation order, framework reuse, performance, and no deferral | Preserve dependency order and reuse existing Core, generated contracts, A2UI/Lit, trusted elements, and UIAI; the host change cannot defer required functionality |
| 135E | Cross-spec amendments, migration, and closure | Apply Spec 158 identity/transition overlays without weakening Spec 135 behavior; closure requires Svelte-destination evidence |
| 135F | Domain-general ontology, semantic graph, domain packs, reactive context | Let domain/vertical packs supply semantics and eligible contributions; activity, profile, context, capability, and evidence changes dynamically recompose the GUI |
| 135G | Multiplexed Work Surfaces, session attachments, browser-context isolation, steering and follow-up | Translate Work Surface strips, focus, lifecycle, exact-recipient routing, queues, prompt routing, session inventory, and attachment isolation into authority-bound Svelte contributions |
| 135H | Cross-functional alpha findings, usability risks, implementation acceleration | Carry every accepted finding and closure criterion into Desktop implementation and evidence; do not substitute shell polish for functional completion |
| 135I | Real-time generated C.R.I.S.T. UI, nontechnical onboarding, Core API integration | Stream canonical generated snapshots/deltas into approved renderers, preserve understandable guided UX, and bind every action to current generated operations |
| 135J | Core API operation registry, durable UI stream, runtime reuse hardening | Use generated clients/validators, exact operation bindings, replay/resume, stale-revision rejection, idempotency, and shared runtime semantics in Desktop |
| 135K | UXP/UFI adaptive generated UI, friction learning, nontechnical usability | Preserve adaptive generated composition and evidence-backed friction learning without turning learned behavior into client-local authority or fixed layout rules |

## Original Pi-overlay implementation inventory

| Source implementation surface | Translation destination |
|---|---|
| `apps/pi-extension/src/mission-canvas-model.ts` | Generated Desktop transport types, exact-scope projection controller, contribution registry, and Svelte runtime models |
| `apps/pi-extension/src/mission-canvas-widget.ts` | `MissionCanvasFrame.svelte`, `MissionCanvasRenderer.svelte`, recursive `ProjectionLayoutRenderer.svelte`, and trusted contribution components |
| `apps/pi-extension/src/work-rail-widget.ts` | Conditional `WorkRailContribution.svelte` backed only by an eligible canonical contribution |
| `apps/pi-extension/src/mission-canvas-session-inventory.ts` | Exact-identity session/attachment Work Surface contribution rendered in the Mission Canvas tab |
| `apps/pi-extension/src/scoped-surface-refresh.ts` | Desktop event client, invalidation controller, projection refresh, replay, and stale-response rejection |
| Pi overlay commands and key interactions | Semantic Desktop commands and canonical operation bindings; compatibility commands may present the exact Desktop Workstream |
| Terminal overlay fallback | Separate Agent TUI tab or truthful bounded compatibility fallback; never the production Mission Canvas GUI |

## Completion invariant

Every task in the completion DAG, Desktop callgraph, executable callgraph, and transition graph references `CARDINAL-135-SVELTE-001`. The broad completion DAG is requirement/dependency provenance, not direct edit authority. Models execute only the 133-task executable graph by loading the task’s bounded file under `docs/contracts/spec135-svelte-task-packets/`, obeying its stop conditions, ordered steps, exact targets, validation commands, evidence artifact, and reopening criteria. A task can close only when its cited Spec 135 behavior is available through the Svelte Mission Canvas destination, or when it explicitly implements a shared Core/generated-runtime dependency used by that destination. Screenshots, fixtures, terminal output, or scaffolds alone cannot establish closure.
