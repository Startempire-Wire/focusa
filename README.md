# Focusa

[![CI](https://github.com/Startempire-Wire/focusa/actions/workflows/ci.yml/badge.svg)](https://github.com/Startempire-Wire/focusa/actions/workflows/ci.yml)
[![Release](https://github.com/Startempire-Wire/focusa/actions/workflows/release.yml/badge.svg)](https://github.com/Startempire-Wire/focusa/actions/workflows/release.yml)
[![Dev Release Tag](https://github.com/Startempire-Wire/focusa/actions/workflows/dev-release-tag.yml/badge.svg)](https://github.com/Startempire-Wire/focusa/actions/workflows/dev-release-tag.yml)
![Version](https://img.shields.io/badge/version-0.9.25--dev-blue)
![License](https://img.shields.io/badge/license-BSL--1.1-orange)
![Source Available](https://img.shields.io/badge/source-available-orange)
![Rust](https://img.shields.io/badge/rust-1.91%2B-dea584?logo=rust)
![Rust Edition](https://img.shields.io/badge/edition-2024-dea584?logo=rust)
![Cargo Workspace](https://img.shields.io/badge/cargo-workspace-dea584?logo=rust)
![Node](https://img.shields.io/badge/node-%3E%3D20-339933?logo=node.js)
![TypeScript](https://img.shields.io/badge/TypeScript-5.x-3178c6?logo=typescript)
![Tauri](https://img.shields.io/badge/Tauri-2.x-24c8db?logo=tauri)
![Svelte](https://img.shields.io/badge/Svelte-5-ff3e00?logo=svelte)
![Local First](https://img.shields.io/badge/local--first-agent%20infrastructure-7c3aed)
![Operator Preview](https://img.shields.io/badge/status-operator%20preview-22c55e)
![Mission Cohesion](https://img.shields.io/badge/category-mission%20cohesion%20layer-8b5cf6)
![Context Authority](https://img.shields.io/badge/context-authority%20gates-0ea5e9)
![Evidence Backed](https://img.shields.io/badge/evidence-backed%20workflows-14b8a6)
![Workpoints](https://img.shields.io/badge/primitive-Workpoints-6366f1)
![Trajectory](https://img.shields.io/badge/primitive-Trajectory-a855f7)
![CLI](https://img.shields.io/badge/interface-CLI-111827)
![HTTP API](https://img.shields.io/badge/interface-HTTP%20API-0ea5e9)
![TUI](https://img.shields.io/badge/interface-TUI-6366f1)
![Pi Extension](https://img.shields.io/badge/integration-Pi%20Extension-f97316)
![macOS Menubar](https://img.shields.io/badge/app-macOS%20menubar-black?logo=apple)
![Focusa.dev](https://img.shields.io/badge/site-focusa.dev-22d3ee)

## Kill the chat. Keep the mission.

**Focusa is a local-first mission cohesion layer for AI coding agents.**

Claude, Codex, OpenCode, OpenClaw, Pi, and other coding agents can move fast — until the session gets long, context compacts, the mission drifts, proof gets buried, or the next agent has to start over.

Focusa is a local-first cognitive runtime for systematic AI execution. It gives long-running AI work a durable operating language for ProjectIdentity, Continuity ID, HLT, MLG, STG, Waypoints, Workpoints, Evidence Refs, Context Cognition, Context Authority, and proof-backed continuation outside the fragile chat window.

It does not replace your agent.

It helps your agent keep the mission.

> **One-line pitch:** Focusa turns long AI chat into long-running AI project work.

---

## Current Snapshot

**Version:** `v0.9.25-dev`
**Release track:** Focusa Operator Preview + Context Authority hardening
**Runtime state:** Rust daemon, HTTP API, CLI, TUI, Pi extension, and menubar web/macOS package proof are implemented and live-tested. Context-authority CLI gates now protect risky mutations.
**Development state:** Focusa is actively evolving. This README describes the current released snapshot, not a finished product.

**Context Authority update:** Focusa now includes mutation-time context gates for the Phone Bridge incident class. See [`docs/current/AUTHORITY_MODEL.md`](docs/current/AUTHORITY_MODEL.md) for the canonical authority table and [`docs/current/CONTEXT_AUTHORITY_CURRENT.md`](docs/current/CONTEXT_AUTHORITY_CURRENT.md) for `focusa action preflight`, `focusa action classify-intent`, `focusa env contract`, `focusa runtime inventory`, `focusa binary preflight-install`, Phone Bridge preflight JSON, and HLT/TL degraded-placeholder behavior.

## Why Focusa is amazing for developers

If you ship code with AI agents today, you already know these pain points. Focusa is the first toolchain built to make them go away.

- **Compaction is not a wipeout.** A typed `Workpoint` survives any model context reset. Resume from the exact next step instead of re-discovering what the agent was doing.
- **Evidence is structural, not a screenshot.** Every claim the agent makes can be linked to a file, a test, a route, or a screenshot. Auditors (and your future self) can replay the proof.
- **Trajectory is a ladder, not a vibe.** HLT / MLG / STG + waypoints give the agent a typed north star. When it drifts, the ladder catches it.
- **Predictions are trackable, not mystical.** The agent records what it expects, you evaluate the outcome, calibration improves across sessions.
- **Metacognition compounds.** Lessons learned today are retrievable tomorrow. The agent doesn't rediscover the same mistakes.
- **Multi-agent is first-class.** Project roots, continuity IDs, and writer arbitration let multiple agents work in the same repo without stepping on each other.
- **Local-first, audit-ready.** Everything runs on your machine or your VPS. The daemon is a typed HTTP API. Nothing leaks to a third-party model.
- **Real observability.** Hot-path latency, cold-path cost, resource pressure, and degraded modes are surfaced in tools, not buried in logs.
- **Real GUI.** The Tauri menubar app shows live focus, workpoint, and trajectory state on macOS — not a CLI you have to remember.
- **Public surface ready.** Tools emit typed envelopes, project cards are shareable, and every public surface is opt-in and redacted by default.

**A developer, not a demo.** Focusa is built by developers who run long agent sessions daily. The roadmap is "what we wished existed" and ships in days, not quarters.

---

## What you can do today

- **Run a long Pi session and keep it.** Start a real AI coding session, create a Workpoint, attach evidence, recover after compaction/session drift, and continue without losing the thread.
- **Inspect any past work.** `focusa_traverse` walks the project graph, `focusa_hlt_history` shows the exact HLT ladder at any point in time, `focusa_metacog_recent_reflections` shows the agent's recent lessons.
- **Stop off-context mutation.** `focusa action preflight` blocks task substitution such as installing a release asset during Phone Bridge pairing on a live build host; `focusa env contract`, `focusa runtime inventory`, and `focusa binary preflight-install` expose the facts behind that verdict.
- **Test the Tauri menubar cockpit.** `apps/menubar` is in active testing: Svelte/web checks and pairing flows are proven locally, while native macOS `.app`, Keychain, restart, and OS lifecycle proof remain tracked testing work.
- **Wire it into your CI.** GitHub Actions runs Rust tests, strict spec gates, static audits, and packaging/release checks; native menubar runtime evidence is reported separately from CI/static proof.
- **Stream public cards.** With `FOCUSA_PUBLIC_STREAM=1`, tool calls become typed public cards — perfect for showcasing live agent work.

---

## Recent additions in this snapshot

- **The generated Focusa tool surface is current.** See [docs/current/generated/tool-surface-summary.md](docs/current/generated/tool-surface-summary.md) for live counts across Pi tools, API parity, CLI parity, docs coverage, and families.
- **Spec 103 — Call Stack Architecture Blueprint** *(⚠ deferred: design-forward, not production-hardened)*: `focusa_call_stack_design` writes a typed, append-only call stack design (entry → handlers → services → adapters → storage → output) for a feature before implementation; `focusa_call_stack_verify` checks the saved design for implementation drift. The design is linkable as `focusa_evidence` to an active Workpoint and is the first-class artifact an agent consumes before writing code. See `docs/103-call-stack-architecture-blueprint-spec.md`, `docs/focusa-tools/tools/focusa_call_stack_design.md`, and `docs/focusa-tools/tools/focusa_call_stack_verify.md`.
- **Spec 105 — Agent DX/UX**: `focusa_dxux_report`, `focusa_dxux_requirement`, `focusa_dxux_explain`, `focusa_dxux_digest`, `focusa preflight`, and `focusa explain <failure>` expose real reliability/authority, doability, recovery, evidence, drift, and compact-resume UX surfaces. See `docs/105-agent-dx-ux-merged-scope-spec.md`, `docs/focusa-tools/tools/focusa_dxux_report.md`, `docs/focusa-tools/tools/focusa_dxux_requirement.md`, `docs/focusa-tools/tools/focusa_dxux_explain.md`, and `docs/focusa-tools/tools/focusa_dxux_digest.md`.
- **Spec 106 — Vision Tightening**: Focusa's vocabulary and product identity are now tighter without flattening the cognitive model: ProjectIdentity, Continuity ID, HLT/MLG/STG, Workpoints, Evidence, Context Cognition, Context Authority, public stream redaction, UIAI diagnostics, Golden Workflow, and glossary surfaces are canonical. See `docs/106-focusa-vision-tightening-spec.md`, `docs/current/AUTHORITY_MODEL.md`, `docs/current/GOLDEN_WORKFLOW.md`, and `docs/current/FOCUSA_GLOSSARY_LINKED_DOCS_UI.md`.
- **Utility card / bootstrap card**: `focusa_utility_card` and `focusa utility card` expose compact startup, post-compaction, recovery, and tool-brevity guidance from current core surfaces. See `docs/focusa-tools/tools/focusa_utility_card.md`.
- **Spec 107 — Spec-first feature lifecycle and claim discipline**: new Focusa features must follow Idea → New Spec → bd/task decomposition → Implementation → tests/proofs → bd/task closure, and partial/surrogate evidence must not be claimed as completion. See `docs/107-spec-first-feature-lifecycle-and-claim-discipline-spec.md`.
- **Spec 101 — Bloatgaurd budgets**: `focusa_bloatgaurd_report`, `focusa_bloatgaurd_domain`, `focusa_bloatgaurd_tokenbloat_report`, `focusa_bloatgaurd_tokenbloat_domain`, `focusa_bloatgaurd_gate_modes`, `focusa_bloatgaurd_gate_mode`, `focusa_bloatgaurd_profiles`, `focusa_bloatgaurd_profile`, `focusa_bloatgaurd_routines`, `focusa_bloatgaurd_routine`, and `focusa_bloatgaurd_rollout` expose read-only budget domains 5.1–5.10, gate modes A/B/C, profile presets, routines, and rollout proof across Rust/API/CLI/Pi/menubar surfaces. See `docs/101-focusa-bloatgaurd-spec.md`, `docs/focusa-tools/tools/focusa_bloatgaurd_report.md`, `docs/focusa-tools/tools/focusa_bloatgaurd_domain.md`, `docs/focusa-tools/tools/focusa_bloatgaurd_tokenbloat_report.md`, `docs/focusa-tools/tools/focusa_bloatgaurd_tokenbloat_domain.md`, `docs/focusa-tools/tools/focusa_bloatgaurd_gate_modes.md`, and `docs/focusa-tools/tools/focusa_bloatgaurd_gate_mode.md`, `docs/focusa-tools/tools/focusa_bloatgaurd_profiles.md`, `docs/focusa-tools/tools/focusa_bloatgaurd_profile.md`, `docs/focusa-tools/tools/focusa_bloatgaurd_routines.md`, `docs/focusa-tools/tools/focusa_bloatgaurd_routine.md`, and `docs/focusa-tools/tools/focusa_bloatgaurd_rollout.md`.
- **Spec 100 — Context Cognition (Phases 1–3)**: `focusa_context_cognition` builds the bounded, advisory `ContextCognitionPacket` (scope, authority, freshness, selected context, ontology frame, evidence frame, reasoning frame, optimization frame, route frame). `focusa_context_cognition_render` returns a compact text render; `focusa_context_cognition_proof` returns bounded proof commands; `focusa_context_cognition_curate` (Phase 3 — Context Curator) does token-budgeted context selection with labeled exclusions (`low_score` / `over_budget`). Never mutates state. The Cognition Optimizer (Phase 5) is the next slice. See `docs/100-context-cognition-spec.md`.
- **Spec 100 — Context Cognition (Phases 4–5 — feedback loop, CQRS)** *(⚠ deferred: design-forward, not production-hardened)*: `focusa_context_cognition_curate_eval` (Phase 4) runs a curator eval case, computes precision/recall/F1, appends to `data/curator-eval-ledger/{hash}/eval-runs.jsonl`. `focusa_context_cognition_curate_optimize` (Phase 5) submits a candidate artifact and gets the `promote | rollback` decision per §15 promotion rule, appends to `data/cognition-optimizer-artifacts/{hash}/artifacts.jsonl`. `focusa_context_cognition_optimizer_artifacts` lists the versioned artifact ledger. CQRS read/write split (GET = read, POST = write); see §15.1.
- **Human-readable templates**: every metacog and predict tool now returns `tool | summary; ids: capture_id=… rehydrate_id=…; fields: lesson=… why=… confidence=…; next: focusa_… → focusa_…` so the operator can pick up the existing hash IDs without digging into details.
- **Project intelligence flywheel**: `focusa_project_card` fuses ProjectIdentity, ontology, trajectory, Workpoint/evidence, prediction stats, outcomes, elapsed/token efficiency, and metacog prompts; `focusa_project_card_outcome` feeds verified outcomes back into learned weights.
- **Session transfer**: `focusa_session_transfer` provides save/continue semantics for long Pi/Focusa work without forking continuity.
- **UIAI browser diagnostics integration**: scoped UIAI browser sessions and reliability reports emit Focusa-ready `focusa_evidence` handles; `focusa_browser_diagnostics_intake` turns diagnostics into evidence, active-object hints, predictions, and optional metacog signals.
- **Doctor browser awareness**: `focusa_tool_doctor` surfaces UIAI browser health/pressure so browser failures are visible during Focusa troubleshooting.
- **Spec 108 — Awareness substrate**: `focusa_awareness_packet` renders a surface-aware AwarenessPacket with DVS-scored visible lines, suppressed lines, next_tools, and recovery_tools. Surfaces: reload, post_compaction, warning, tool_guidance, uiai_bridge. See `docs/108-focusa-awareness-substrate-spec.md` and `docs/focusa-tools/tools/focusa_awareness_packet.md`.
- **Menubar cockpit testing**: the Svelte/Tauri menubar app has passing web build/check proof and pairing API/web proof; native macOS `.app`, Keychain, restart persistence, screenshots/logs, and OS lifecycle validation remain testing work, tracked by `focusa-ui0y.15`, `focusa-qasy.25`, and native artifact beads.
- **Mac menubar OAuth-like device pairing with QR + PWA (focusa-ui0y)**: pair a Mac to a Focusa VPS via three handoff modes — CLI (SSH), QR + phone (Telegram/Discord-style), or QR + VPS browser. Set `FOCUSA_PAIRING_URL` on the daemon to your public VPS hostname; the Mac menubar renders a QR encoding the PWA helper page at `/pair/{device_id}`. Architecture: [`docs/53-focusa-device-pairing-spec.md`](docs/53-focusa-device-pairing-spec.md). Each Focusa install is its own trust root — multi-tenant safe, no shared registry.
- **Strict CI proof**: GitHub CI passes Rust tests/clippy and strict spec/static gates on every push to `main`; native menubar runtime proof is not treated as complete until actual macOS evidence exists.

---

## Why Focusa exists

Long agent sessions fail in predictable ways:

- **Conversation is mistaken for memory.** When the model context is compacted or overflows, decisions, constraints, evidence, and next steps become lossy prose.
- **The active task drifts.** Agents keep working, but not always on the same object, scope, or operator intent.
- **Proof gets buried in logs.** A test result, API response, or file path may be visible once and then disappear into transcript noise.
- **Learning is ungrounded.** Agents can record lessons, but without evidence, quality gates, or evaluation loops, those lessons become another pile of notes.
- **Autonomy is hard to trust.** The operator needs visible state, checkpoints, rollback points, and writer ownership instead of hidden memory writes.

Focusa was created to move durable meaning out of raw conversation and into typed, inspectable, local state.

---

## What Focusa is

Focusa is a local cognitive runtime that runs beside an agent harness such as Pi. It does not replace the agent or the model. It gives the agent structured memory, continuity, evidence handling, and governance surfaces.

In plain terms, Focusa gives an agent:

- a **current state of mind** (`Focus State`),
- a **continuation contract** after compaction (`Workpoint`),
- a way to **save proof without bloating prompts** (`Evidence` + handles),
- a **lineage/snapshot system** for recovery,
- a **metacognition loop** for reusable learning, persisted evaluations, and bounded promotion back into retrieval memory,
- a **work-loop control surface** with writer ownership and hot-path dispatch-readiness diagnostics,
- and a common result envelope so tools return predictable status, retry, evidence, and next-tool guidance.

Focusa is local-first. State lives on the machine running the daemon, under the project/data directory, and can be inspected through the CLI/API.

### What Focusa is not

- Not a model.
- Not a chatbot.
- Not a replacement for Pi, Claude Code, Codex, or other harnesses.
- Not a generic RAG system.
- Not a cloud memory service.
- Not finished or frozen; it is under active development.

---

## What a user can expect from Focusa-enhanced agents

When Focusa is working well, an agent should:

1. **Resume cleanly after compaction.** It should call `focusa_workpoint_resume` and continue from a typed packet instead of guessing from the transcript tail.
2. **Keep the current mission visible.** Intent, focus, constraints, failures, recent results, and next steps are stored in bounded fields.
3. **Preserve decisions and why they were made.** Decisions are concise architectural records, not buried paragraphs.
4. **Treat evidence as first-class.** Test output, API proof, release checks, and file references can be linked to the active Workpoint.
5. **Notice drift.** Workpoint drift checks can tell whether the agent is still working on the expected action/object.
6. **Avoid prompt bloat.** Large outputs become handles or evidence refs instead of raw transcript paste.
7. **Recover from uncertainty.** Tool results include status, retry posture, canonical/degraded state, and next-tool hints.
8. **Learn with discipline.** Metacognition tools include quality gates, evidence refs, persisted evaluation records, and bounded promotion back into retrieval memory instead of unconstrained note-taking.
9. **Respect ownership.** Work-loop mutation tools expose writer conflicts and preflight state instead of silently taking over.
10. **Remain inspectable.** The CLI/API expose state, lineage, snapshots, events, memory, ontology, Workpoints, and tool health.
11. **Follow the trajectory ladder.** HLT (High-Level Trajectory) → MLG (Mid-Level Goal) → STG (Short-Term Goal) → Waypoints steer toward the project north star; the Workpoint remains the immediate continuation contract.
12. **Plan proactively from the HLT.** Once the HLT is known, the agent should derive MLGs/STGs/Waypoints and keep moving toward them instead of passively waiting or reacting turn-by-turn, unless a risk/approval gate blocks action.
13. **Defer while offering routes.** The operator has authority; the agent should still actively offer HLT-aligned Waypoints, STGs, and MLGs as optional route guidance along the way.

---

## Operator Preview maturity table

| Surface | Status | Release posture |
|---|---|---|
| Workpoint checkpoint/resume | Implemented | Primary supported workflow |
| Evidence link | Implemented | Primary supported workflow |
| `focusa onboard` | Implemented | First-run Operator Preview flow |
| `focusa doctor` | Implemented | Supported diagnostics/repair surface |
| Manual awareness card | Implemented | Supported non-Pi fallback path |
| `focusa status --operator` | Implemented | One-screen session card for buyers/operators |
| `focusa workpoint resume --copy-prompt` | Implemented | Paste-ready manual continuation packet for non-Pi agents |
| Golden demo script | Implemented | `scripts/demo-workpoint-happy-path.sh` proves the happy path |
| Trajectory ladder | Implemented/advisory | HLT → MLG → STG → Waypoints steer toward the project north star |
| Trajectory view | Implemented/advisory | Supported orientation layer; not task authority |
| Pi extension | Implemented | Best-supported deep harness path |
| CLI/API | Implemented | Supported operator surface |
| Work-loop | Implemented but advanced | Preview/advanced |
| Metacognition | Implemented, bounded | Preview/advanced |
| Ontology governance | Partial/design-forward | Experimental unless marked current in `docs/current/` | Spec103, Spec100 Ph4-5 deferred: design-forward, not production-hardened |
| GUI/menubar | Testing preview cockpit | Web build/checks and local pairing flows pass; native macOS `.app`, Keychain, restart, screenshots/logs, and OS lifecycle proof remain open |
| Mac device pairing (focusa-ui0y) | Implemented surfaces; E2E testing open | `focusa device pair-qr` + menubar QR render + PWA helper page at `/pair/{device_id}` exist; actual Mac E2E proof remains tracked by `focusa-ui0y.15` |
| Team/multi-user/cloud sync | Future | Not in preview |

---

## Current testing boundaries

- **Menubar is in testing**, not release-final native proof. Web/Svelte checks, local pairing API/web flows, QR/PWA paths, and static audits are good evidence; actual macOS `.app` launch, Keychain persistence, restart persistence, screenshots/logs, and native Tauri window/menu/invoke lifecycle remain open.
- **Spec106 product QA hardening remains active.** `docs/evidence/REAL_BROWSER_PRODUCT_QA_2026-06-14.md` records real browser/product findings after the static/architecture pass; open beads track native blockers and deeper product QA.
- **Completion claims are now spec-disciplined.** Spec107 requires Idea → New Spec → bd/task decomposition → Implementation → tests/proofs → bd/task closure and forbids partial/surrogate evidence as completion proof.

---

## Current architecture snapshot

```text
Agent harness / Pi session
        │
        │ focusa_* tools, commands, lifecycle hooks
        ▼
Pi Focusa extension ── thin adapter, no parallel memory
        │
        │ HTTP JSON calls
        ▼
Focusa daemon / API (Rust)
        │
        ├─ Focus State: bounded current cognitive state
        ├─ Workpoint: typed continuation and evidence spine
        ├─ Core reducer: deterministic state transitions
        ├─ Ontology: objects, links, working sets, active refs
        ├─ Lineage / CLT: branch-aware context history
        ├─ Tree snapshots: recoverable state checkpoints
        ├─ Metacognition: capture/retrieve/reflect/adjust/evaluate
        ├─ Work-loop: continuous execution state and writer control
        ├─ ECS / references: externalized handles for large content
        └─ CLI/API parity surfaces
```

### Core crate: `focusa-core`

The core crate owns data types and reducer logic. Important current state includes:

- `FocusaState` — session, focus stack, Focus State, gate, memory, telemetry, ontology, Workpoint, and continuous work state.
- `FocusState` — bounded slots for intent, current focus, decisions, constraints, failures, next steps, open questions, recent results, notes, and artifacts.
- `WorkpointState` — active Workpoint ID, records, resume events, drift events, degraded fallbacks.
- `OntologyState` — proposals, objects/links/status changes, working-set refreshes, verification records, and delta log.
- `FocusaEvent` — reducer-owned event taxonomy for Focus State, ontology, Workpoint, continuous work, telemetry, memory, and related state transitions.

The reducer is the authority for state mutation. API routes and Pi tools should submit typed events or commands; they should not become alternate memory systems.

### API crate: `focusa-api`

The daemon exposes local HTTP endpoints under `/v1`. Current important namespaces include:

- `/v1/health`, `/v1/status`
- `/v1/focus/*`
- `/v1/workpoint/*`
- `/v1/work-loop/*`
- `/v1/lineage/*`
- `/v1/ontology/*`
- `/v1/metacognition/*`
- `/v1/threads/*`, `/v1/instances/*`
- `/v1/capabilities/*`
- telemetry, memory, ECS/reference, gate, proposals, autonomy, cache, and token surfaces

The Workpoint release path now waits for reducer-visible state before reporting success:

- `POST /v1/workpoint/checkpoint` returns `accepted` only after the new active Workpoint is visible to `/current` and `/resume`.
- `POST /v1/workpoint/evidence/link` returns `accepted` only after linked evidence is visible in Workpoint state.
- If reducer state has not materialized yet, the route returns `pending` with retry guidance instead of pretending the operation is complete.

### CLI crate: `focusa-cli`

The CLI is the operator/debug surface for the daemon. Current command domains include:

```text
start, stop, status, doctor, cleanup, continue, focus, stack, gate, memory,
ecs, env, events, turns, state, clt, lineage, autonomy, constitution,
telemetry, rfm, release, proposals, predict, reflect, metacognition,
ontology, skills, thread, export, contribute, cache, workpoint, tokens, wrap
```

Most commands support human-readable output, and the top-level CLI supports `--json` for machine-readable workflows.

### Pi extension

The Pi extension is the main agent-facing integration. It registers 63 current `focusa_*` tools grouped into these families:

- **Focus State:** scratch, decide, constraint, failure, intent, current focus, next step, open question, recent result, note.
- **Project Identity and project intelligence:** resolve/verify the project folder before trusting carryover; generate project cards, attach project-card outcomes, and save/continue long sessions with `focusa_session_transfer`.
- **Trajectory:** view, define, assess, checkpoint/resume, and propose advisory Workpoint candidates.
- **Workpoint:** checkpoint, resume, link evidence, active object resolve, evidence capture, browser diagnostics intake, and scope-safe evidence handles.
- **Traversal/reflexes:** bounded `focusa_traverse` slices across lineage, ontology, evidence, telemetry, Workpoints, registries, plus read-only Spec97 Reflex Primitive summaries via `focusa_reflex_primitives`.
- **Work-loop:** writer status, status, control, context, checkpoint, select next.
- **Tree/lineage:** head, path, snapshot, diff, restore, recent snapshots, compare latest, lineage tree, LI extraction.
- **Metacognition:** capture, retrieve, reflect, plan adjustment, evaluate outcome, recent reflections, recent adjustments, loop run, doctor.
- **Prediction loop:** record, recent, evaluate, stats, and project-card outcome feedback for bounded inspectable predictions and lightweight algorithmic guidance.
- **State hygiene:** doctor, plan, approval-safe apply.
- **Tool doctor/resource mode:** diagnostic entrypoint, UIAI browser health/pressure visibility, plus LowMem/emergency posture control.
- **SilentSessions:** tmux-backed background Pi session list/start/reopen/tail/send/kill with explicit approval gates.
- **Workpoint project-folder guard:** project/session-bound resume packets reject cross-project continuation.
- **Compaction fallback guard:** Pi replacement compaction hydrates sparse fields from related canonical sources instead of emitting bare `none`.

Every `focusa_*` tool is expected to expose a common `tool_result_v1` result envelope with status, canonical/degraded flags, retry guidance, side effects, evidence refs, next-tool hints, and bounded `reflex_suggestions` when a recurring recovery primitive applies.

---

## Workpoint continuity

`project_root` means the project folder/container that holds related files. Trajectory is the functional route: current state → desired outcome → waypoint goals. A Workpoint is a typed continuation record. It preserves:

- mission / current ask,
- active object refs,
- action intent,
- verification records,
- blockers and drift boundaries,
- next slice / exact next action,
- canonical vs degraded state.

Use Workpoints whenever raw conversation becomes unreliable:

```text
Before compaction or risky handoff:
  focusa_workpoint_checkpoint

After compaction, model switch, fork, or uncertainty:
  focusa_workpoint_resume

After tests, release proof, API evidence, or file proof:
  focusa_workpoint_link_evidence
```

A non-canonical Workpoint is a fallback hint, not truth. The agent should say it is degraded and recover through a canonical Focusa read when possible.

---

## Metacognition and learning

Focusa metacognition is for reusable learning. It is not a dumping ground for every thought.

The loop is:

1. `focusa_metacog_capture` — store a reusable signal with rationale/evidence.
2. `focusa_metacog_retrieve` — search prior learning before planning.
3. `focusa_metacog_reflect` — generate hypotheses and strategy updates.
4. `focusa_metacog_plan_adjust` — turn reflection into a tracked adjustment.
5. `focusa_metacog_evaluate_outcome` — decide whether the adjustment improved results.

Spec89 added quality-gate details and suggested metrics so weak, vague, or low-evidence learning can be improved before it influences future behavior.

---

## State hygiene

Focusa state should be useful, not hoarded. The current hygiene tools are intentionally proposal-first:

- `focusa_state_hygiene_doctor` diagnoses stale or duplicate signals without mutation.
- `focusa_state_hygiene_plan` creates a proposed cleanup path.
- `focusa_state_hygiene_apply` is approval-gated and non-destructive in the current snapshot.

No Focusa tool should be silently deleted or demoted as a shortcut. Weak tools should be clarified, hardened, merged upward, or redesigned.

---

## Quick start

### Build and run locally

```bash
git clone <repo-url> focusa
cd focusa
cargo build --release -p focusa-api -p focusa-cli

# Start daemon in foreground
cargo run --bin focusa-daemon

# In another shell
cargo run --bin focusa -- status
cargo run --bin focusa -- status --operator
cargo run --bin focusa -- onboard --agent manual
cargo run --bin focusa -- project identity --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --json
cargo run --bin focusa -- workpoint current --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --continuity-id cont-1

# Manual continuation packet for non-Pi agents
cargo run --bin focusa -- workpoint resume --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --continuity-id cont-1 --copy-prompt

# Optional golden proof loop
bash scripts/demo-workpoint-happy-path.sh
```

Default API URL:

```text
http://127.0.0.1:8787
```

### Installed service pattern

A deployed local service typically runs the built daemon from the checkout or from an install directory on `PATH`:

```bash
./target/release/focusa-daemon
# or, after installing/copying binaries:
focusa-daemon
```

Health check:

```bash
curl -sS http://127.0.0.1:8787/v1/health | jq .
```

Production deployment checklist: `docs/production-deployment-guide.md`

Automated tagged release → live deploy path: `docs/live-release-automation.md`

### CLI examples

Project-scoped commands should carry `--project-root` plus a stable `--continuity-id`. Use the repo folder (or `FOCUSA_PROJECT_ROOT`) for `--project-root`; use a repeatable workstream id such as `cont-1`, a ticket id, or your agent continuity id for `--continuity-id`.

```bash
# Daemon status
focusa status

# Discover the scoped project identity first
focusa project identity --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --json

# Current Workpoint for a stable logical workstream
focusa workpoint current --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --continuity-id cont-1

# Resume packet for that workstream
focusa workpoint resume --project-root "${FOCUSA_PROJECT_ROOT:-$PWD}" --continuity-id cont-1

# Drift check
focusa workpoint drift-check \
  --latest-action 'release verify Spec89FocusaToolSuite live_api cli pi_tool' \
  --expected-action-type release_verify

# Ontology surfaces
focusa ontology primitives
focusa ontology world
focusa ontology slices
```

### API examples

```bash
# Health
curl -sS http://127.0.0.1:8787/v1/health | jq .

# Current Workpoint
curl -sS http://127.0.0.1:8787/v1/workpoint/current | jq .

# Resume Workpoint
curl -sS -X POST http://127.0.0.1:8787/v1/workpoint/resume \
  -H 'content-type: application/json' \
  -d '{"mode":"operator"}' | jq .
```

### Pi skill and tools

The current Focusa Pi skill lives in:

- project: `.pi/skills/focusa/SKILL.md`
- extension package: `apps/pi-extension/skills/focusa/SKILL.md`
- installed global copy: `~/.pi/skills/focusa/SKILL.md`

If Pi reports `description is required`, the skill is missing YAML frontmatter. A valid Focusa skill starts with:

```yaml
---
name: focusa
description: Use when preserving Focusa cognitive state, resuming after compaction/model switch/context overflow, linking evidence to Workpoints, using Focus State, work-loop, lineage/tree, metacognition, state-hygiene, or diagnosing Focusa tool readiness.
---
```

Validate skill loading with Pi's actual loader, resolving the global npm install instead of assuming a host-specific Node path:

```bash
PI_AGENT_DIR="${PI_AGENT_DIR:-$HOME/.pi/agent}"
PI_PKG_ROOT="${PI_PKG_ROOT:-$(npm root -g)/@mariozechner/pi-coding-agent}"
PI_PKG_ROOT="$PI_PKG_ROOT" PI_AGENT_DIR="$PI_AGENT_DIR" node --input-type=module - <<'NODE'
const mod = await import(`${process.env.PI_PKG_ROOT}/dist/core/skills.js`);
const r = mod.loadSkills({ cwd: process.cwd(), agentDir: process.env.PI_AGENT_DIR, skillPaths: [], includeDefaults: true });
console.log(r.skills.map(s => [s.name, s.description.length, s.filePath]));
console.log(r.diagnostics);
NODE
```

---

## Current repository layout

```text
focusa/
├── README.md                         # GitHub-facing overview
├── Cargo.toml                        # Rust workspace
├── crates/
│   ├── focusa-core/                  # Reducer, state, event types, memory, workers
│   ├── focusa-api/                   # Local daemon / HTTP API binary focusa-daemon
│   ├── focusa-cli/                   # CLI binary focusa
│   └── focusa-tui/                   # TUI crate
├── apps/
│   ├── pi-extension/                 # Pi integration and Focusa tools
│   └── menubar/                      # Svelte/Tauri ambient runtime cockpit
├── docs/                             # Specs, evidence, audits, operator guides
├── tests/                            # Contract and live-stress scripts
└── .pi/skills/focusa/                # Project-local Focusa skill
```

Some older docs describe planned GUI/proxy/autonomy surfaces in more detail than the current runtime exposes. Treat those as design direction unless the README/current evidence says they are released in this snapshot.

---

## Current live proof

Current release proof is documented in:

```text
docs/evidence/PUBLIC_DOCS_RELEASE_SYNC_2026-05-26.md
docs/evidence/SPEC89_REAL_RELEASE_LIVE_PROOF_2026-04-28.md
```

Current proof verifies pushed GitHub CI plus rebuilt local daemon/CLI checks. Historical Spec89 proof verified direct live API/CLI/Pi tool probes, including:

- daemon health,
- Workpoint checkpoint/current/resume,
- Workpoint evidence link visible in resume,
- Focus State update,
- metacognition capture,
- work-loop status,
- CLI Workpoint current and drift-check,
- Pi `focusa_workpoint_resume`.

Current hardening gates also cover bounded CLI smoke, tool stress, extended soak, parallel hot-route load, context-pressure warning copy, dynamic choreography weighting, project-card/session-transfer surfaces, UIAI browser diagnostics evidence flow, menubar web/macOS package proof, and safe audit/profiling checks. These prove Focusa preserves scoped project/trajectory/Workpoint/evidence anchors under pressure instead of treating context pressure as lost continuity.

Focusa-native dogfood is documented in `docs/current/FOCUSA_DOGFOOD.md` and runnable with `bash tests/focusa_dogfood_test.sh`. It validates agent-facing UX loops across daemon health, tool contracts, trajectory, Workpoint continuity, evidence, metacognition, prediction, and resource pressure.

Proof marker:

```text
DIRECT_REAL_RELEASE_PROOF=PASS
```

---

## Design principles

1. **Meaning lives in typed state, not transcript luck.**
2. **Focusa is the cognitive authority; adapters stay thin.**
3. **Every important result should say whether it is canonical, degraded, retryable, or blocked.**
4. **Evidence should be linked, not pasted forever.**
5. **Agents should recover through explicit state reads, not memory vibes.**
6. **Operator steering wins over automation.**
7. **Local-first and inspectable beats hidden cloud memory.**
8. **Focusa remains evolvable; docs describe snapshots, not permanent completion.**

---

## Documentation map

Start here:

- `docs/README.md` — documentation index for the current snapshot.
- `docs/focusa-tools/README.md` — focused docs for every current `focusa_*` tool family, with tool descriptions and examples.
- `docs/focusa-tools/workpoint.md` — Workpoint checkpoint/resume/evidence/object-resolution tools.
- `docs/focusa-tools/focus-state.md` — Focus State and scratchpad tools.
- `docs/focusa-tools/work-loop.md` — continuous work-loop writer/status/control tools.
- `docs/focusa-tools/metacognition.md` — metacog capture/retrieve/reflect/adjust/evaluate tools.
- `docs/focusa-tools/tree-lineage.md` — lineage, tree, snapshot, diff, restore, and LI extraction tools.
- `docs/focusa-tools/diagnostics-hygiene.md` — tool doctor and state hygiene tools.
- `docs/SPEC89_HARDENED_FOCUSA_TOOL_OPERATOR_GUIDE_2026-04-28.md` — when to use each hardened Focusa tool.
- `docs/88-ontology-backed-workpoint-continuity.md` — Workpoint continuity design.
- `docs/89-focusa-tool-suite-improvement-hardening-spec.md` — current tool-suite hardening snapshot.
- `docs/evidence/SPEC89_REAL_RELEASE_LIVE_PROOF_2026-04-28.md` — released runtime proof.

### Current-build references

These docs describe only the current present build/snapshot surfaces:

- [`CHANGELOG.md`](CHANGELOG.md) — current snapshot change history.
- [`docs/current/CURRENT_RUNTIME_STATUS.md`](docs/current/CURRENT_RUNTIME_STATUS.md) — implemented runtime status and current limits.
- [`docs/current/API_REFERENCE_CURRENT.md`](docs/current/API_REFERENCE_CURRENT.md) — current API route inventory generated from route registrations.
- [`docs/current/CLI_REFERENCE_CURRENT.md`](docs/current/CLI_REFERENCE_CURRENT.md) — current CLI command inventory from `focusa --help`.
- [`docs/current/PI_EXTENSION_AND_SKILLS_GUIDE.md`](docs/current/PI_EXTENSION_AND_SKILLS_GUIDE.md) — Pi extension and skill locations/validation.
- [`docs/current/WORKPOINT_LIFECYCLE_GUIDE.md`](docs/current/WORKPOINT_LIFECYCLE_GUIDE.md) — current Workpoint usage and recovery flow.
- [`docs/current/TOOL_RESULT_ENVELOPE_V1.md`](docs/current/TOOL_RESULT_ENVELOPE_V1.md) — current structured tool result contract.
- [`docs/current/TROUBLESHOOTING_CURRENT.md`](docs/current/TROUBLESHOOTING_CURRENT.md) — current troubleshooting runbook.
- [`docs/current/FOCUSA_OPERATOR_PREVIEW_PROOF.md`](docs/current/FOCUSA_OPERATOR_PREVIEW_PROOF.md) — buyer-readable Operator Preview proof pack and release gates.
- [`docs/current/PORTABILITY_AUDIT.md`](docs/current/PORTABILITY_AUDIT.md) — external tester portability matrix, fixed gaps, and remaining caveats.
- [`docs/current/VALIDATION_AND_RELEASE_PROOF.md`](docs/current/VALIDATION_AND_RELEASE_PROOF.md) — current validation and real runtime proof expectations.
- [`docs/current/PRODUCTION_RELEASE_COMMANDS.md`](docs/current/PRODUCTION_RELEASE_COMMANDS.md) — copy/paste commands for release, restart, GitHub proof, and cleanup.
- [`docs/production-deployment-guide.md`](docs/production-deployment-guide.md) — production deployment reference for reverse proxy, TLS, pairing URL, rate limits, and logs.
- [`templates/workpoint-session.md`](templates/workpoint-session.md), [`templates/evidence-checklist.md`](templates/evidence-checklist.md), [`templates/agentops-sop.md`](templates/agentops-sop.md) — buyer-ready Operator Preview session templates.
- [`docs/92-agent-first-polish-hooks-efficiency-spec.md`](docs/92-agent-first-polish-hooks-efficiency-spec.md) — next polish spec for hooks, token/cache UX, agent command center, and predictive power.
- [`docs/current/HOOK_COVERAGE.md`](docs/current/HOOK_COVERAGE.md) — current Pi hook coverage and Spec92 hook telemetry commands.
- [`docs/current/EFFICIENCY_GUIDE.md`](docs/current/EFFICIENCY_GUIDE.md) — current token-budget telemetry and planned cache metadata commands.
- [`docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md`](docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md) — current doctor/continue command-center usage and envelopes.
- [`docs/current/DAEMON_RESILIENCE.md`](docs/current/DAEMON_RESILIENCE.md) — live daemon restart hardening and Pi in-session holdover/kickstart behavior.
- [`docs/current/ERROR_EMPTY_STATES.md`](docs/current/ERROR_EMPTY_STATES.md) — recovery-first CLI/API failure and empty-state envelopes.
- [`docs/current/MAC_APP_MISSION_CONTROL.md`](docs/current/MAC_APP_MISSION_CONTROL.md) — Mac mission-control cards for daemon/workpoint/work-loop/token/cache/release state.
- [`docs/current/PREDICTIVE_POWER_GUIDE.md`](docs/current/PREDICTIVE_POWER_GUIDE.md) — prediction record/evaluation/stats API and CLI guide.
- [`docs/current/PROJECT_INTELLIGENCE_FLYWHEEL.md`](docs/current/PROJECT_INTELLIGENCE_FLYWHEEL.md) — ontology-grounded project-card flywheel for trajectory bootstrap/re-bootstrap, prediction, and metacog compounding.
- [`docs/current/PREDICTION_ALGORITHMS_IMPLEMENTED.md`](docs/current/PREDICTION_ALGORITHMS_IMPLEMENTED.md) — implemented lightweight prediction formulas behind project-card algorithmic intelligence.
- [`docs/current/UIAI_BROWSER_DIAGNOSTICS_FOCUSA_INTEGRATION_SPEC.md`](docs/current/UIAI_BROWSER_DIAGNOSTICS_FOCUSA_INTEGRATION_SPEC.md) — local UIAI browser diagnostics evidence ingestion, scoped `focusa_evidence` artifacts, and Workpoint/prediction flow.
- [`docs/current/AGENT_COMMAND_COOKBOOK.md`](docs/current/AGENT_COMMAND_COOKBOOK.md) — copy/paste agent workflows for start/risky edit/compaction/daemon/release/Mac/prediction/cleanup.
- [`docs/evidence/PUBLIC_DOCS_RELEASE_SYNC_2026-05-26.md`](docs/evidence/PUBLIC_DOCS_RELEASE_SYNC_2026-05-26.md) — current public docs sync, Guardian/public-tree secret audit, GitHub CI proof, UIAI evidence proof, and menubar proof.
- [`docs/90-ontology-backed-tool-contracts-parity-spec.md`](docs/90-ontology-backed-tool-contracts-parity-spec.md) — Spec90 current tool contract/parity hardening plan.
- [`docs/current/FOCUSA_TOOL_CONTRACT_REGISTRY.md`](docs/current/FOCUSA_TOOL_CONTRACT_REGISTRY.md) — current machine-readable tool contract registry table.
- [`docs/91-live-tool-contract-proof-harness-spec.md`](docs/91-live-tool-contract-proof-harness-spec.md) — Spec91 live runtime proof harness for tool contracts.
- [`docs/current/LIVE_TOOL_CONTRACT_PROOF.md`](docs/current/LIVE_TOOL_CONTRACT_PROOF.md) — current live proof command and expected result.

### Individual Focusa tool docs

Each current `focusa_*` Pi tool has its own doc with purpose, usage guidance, example usage, expected result, recovery notes, and related tools. Current contract count: **72**.

| Tool | Family | Doc |
| --- | --- | --- |
| `focusa_project_identity` | Project Identity | [`docs/focusa-tools/tools/focusa_project_identity.md`](docs/focusa-tools/tools/focusa_project_identity.md) |
| `focusa_project_card` | Project Identity | [`docs/focusa-tools/tools/focusa_project_card.md`](docs/focusa-tools/tools/focusa_project_card.md) |
| `focusa_project_card_outcome` | Project Identity | [`docs/focusa-tools/tools/focusa_project_card_outcome.md`](docs/focusa-tools/tools/focusa_project_card_outcome.md) |
| `focusa_session_transfer` | Workpoint | [`docs/focusa-tools/tools/focusa_session_transfer.md`](docs/focusa-tools/tools/focusa_session_transfer.md) |
| `focusa_project_verify` | Project Identity | [`docs/focusa-tools/tools/focusa_project_verify.md`](docs/focusa-tools/tools/focusa_project_verify.md) |
| `focusa_trajectory_view` | Trajectory | [`docs/focusa-tools/tools/focusa_trajectory_view.md`](docs/focusa-tools/tools/focusa_trajectory_view.md) |
| `focusa_context_cognition` | Trajectory | [`docs/focusa-tools/tools/focusa_context_cognition.md`](docs/focusa-tools/tools/focusa_context_cognition.md) |
| `focusa_context_cognition_render` | Trajectory | [`docs/focusa-tools/tools/focusa_context_cognition_render.md`](docs/focusa-tools/tools/focusa_context_cognition_render.md) |
| `focusa_context_cognition_proof` | Trajectory | [`docs/focusa-tools/tools/focusa_context_cognition_proof.md`](docs/focusa-tools/tools/focusa_context_cognition_proof.md) |
| `focusa_context_cognition_curate` | Trajectory | [`docs/focusa-tools/tools/focusa_context_cognition_curate.md`](docs/focusa-tools/tools/focusa_context_cognition_curate.md) |
| `focusa_context_cognition_curate_eval` | Trajectory | [`docs/focusa-tools/tools/focusa_context_cognition_curate_eval.md`](docs/focusa-tools/tools/focusa_context_cognition_curate_eval.md) |
| `focusa_context_cognition_curate_optimize` | Trajectory | [`docs/focusa-tools/tools/focusa_context_cognition_curate_optimize.md`](docs/focusa-tools/tools/focusa_context_cognition_curate_optimize.md) |
| `focusa_context_cognition_optimizer_artifacts` | Trajectory | [`docs/focusa-tools/tools/focusa_context_cognition_optimizer_artifacts.md`](docs/focusa-tools/tools/focusa_context_cognition_optimizer_artifacts.md) |
| `focusa_trajectory_define_goal` | Trajectory | [`docs/focusa-tools/tools/focusa_trajectory_define_goal.md`](docs/focusa-tools/tools/focusa_trajectory_define_goal.md) |
| `focusa_trajectory_assess` | Trajectory | [`docs/focusa-tools/tools/focusa_trajectory_assess.md`](docs/focusa-tools/tools/focusa_trajectory_assess.md) |
| `focusa_trajectory_propose_workpoint` | Trajectory | [`docs/focusa-tools/tools/focusa_trajectory_propose_workpoint.md`](docs/focusa-tools/tools/focusa_trajectory_propose_workpoint.md) |
| `focusa_trajectory_checkpoint` | Trajectory | [`docs/focusa-tools/tools/focusa_trajectory_checkpoint.md`](docs/focusa-tools/tools/focusa_trajectory_checkpoint.md) |
| `focusa_trajectory_resume` | Trajectory | [`docs/focusa-tools/tools/focusa_trajectory_resume.md`](docs/focusa-tools/tools/focusa_trajectory_resume.md) |
| `focusa_hlt_history` | Trajectory | [`docs/focusa-tools/tools/focusa_hlt_history.md`](docs/focusa-tools/tools/focusa_hlt_history.md) |
| `focusa_traverse` | Traversal | [`docs/focusa-tools/tools/focusa_traverse.md`](docs/focusa-tools/tools/focusa_traverse.md) |
| `focusa_reflex_primitives` | Traversal | [`docs/focusa-tools/tools/focusa_reflex_primitives.md`](docs/focusa-tools/tools/focusa_reflex_primitives.md) |
| `focusa_predict_record` | Metacognition | [`docs/focusa-tools/tools/focusa_predict_record.md`](docs/focusa-tools/tools/focusa_predict_record.md) |
| `focusa_predict_recent` | Metacognition | [`docs/focusa-tools/tools/focusa_predict_recent.md`](docs/focusa-tools/tools/focusa_predict_recent.md) |
| `focusa_predict_evaluate` | Metacognition | [`docs/focusa-tools/tools/focusa_predict_evaluate.md`](docs/focusa-tools/tools/focusa_predict_evaluate.md) |
| `focusa_predict_stats` | Metacognition | [`docs/focusa-tools/tools/focusa_predict_stats.md`](docs/focusa-tools/tools/focusa_predict_stats.md) |
| `focusa_scratch` | Focus State | [`docs/focusa-tools/tools/focusa_scratch.md`](docs/focusa-tools/tools/focusa_scratch.md) |
| `focusa_decide` | Focus State | [`docs/focusa-tools/tools/focusa_decide.md`](docs/focusa-tools/tools/focusa_decide.md) |
| `focusa_constraint` | Focus State | [`docs/focusa-tools/tools/focusa_constraint.md`](docs/focusa-tools/tools/focusa_constraint.md) |
| `focusa_failure` | Focus State | [`docs/focusa-tools/tools/focusa_failure.md`](docs/focusa-tools/tools/focusa_failure.md) |
| `focusa_intent` | Focus State | [`docs/focusa-tools/tools/focusa_intent.md`](docs/focusa-tools/tools/focusa_intent.md) |
| `focusa_current_focus` | Focus State | [`docs/focusa-tools/tools/focusa_current_focus.md`](docs/focusa-tools/tools/focusa_current_focus.md) |
| `focusa_next_step` | Focus State | [`docs/focusa-tools/tools/focusa_next_step.md`](docs/focusa-tools/tools/focusa_next_step.md) |
| `focusa_open_question` | Focus State | [`docs/focusa-tools/tools/focusa_open_question.md`](docs/focusa-tools/tools/focusa_open_question.md) |
| `focusa_recent_result` | Focus State | [`docs/focusa-tools/tools/focusa_recent_result.md`](docs/focusa-tools/tools/focusa_recent_result.md) |
| `focusa_note` | Focus State | [`docs/focusa-tools/tools/focusa_note.md`](docs/focusa-tools/tools/focusa_note.md) |
| `focusa_work_loop_writer_status` | Work-loop | [`docs/focusa-tools/tools/focusa_work_loop_writer_status.md`](docs/focusa-tools/tools/focusa_work_loop_writer_status.md) |
| `focusa_work_loop_status` | Work-loop | [`docs/focusa-tools/tools/focusa_work_loop_status.md`](docs/focusa-tools/tools/focusa_work_loop_status.md) |
| `focusa_work_loop_control` | Work-loop | [`docs/focusa-tools/tools/focusa_work_loop_control.md`](docs/focusa-tools/tools/focusa_work_loop_control.md) |
| `focusa_work_loop_context` | Work-loop | [`docs/focusa-tools/tools/focusa_work_loop_context.md`](docs/focusa-tools/tools/focusa_work_loop_context.md) |
| `focusa_work_loop_checkpoint` | Work-loop | [`docs/focusa-tools/tools/focusa_work_loop_checkpoint.md`](docs/focusa-tools/tools/focusa_work_loop_checkpoint.md) |
| `focusa_work_loop_select_next` | Work-loop | [`docs/focusa-tools/tools/focusa_work_loop_select_next.md`](docs/focusa-tools/tools/focusa_work_loop_select_next.md) |
| `focusa_state_hygiene_doctor` | Diagnostics / Hygiene | [`docs/focusa-tools/tools/focusa_state_hygiene_doctor.md`](docs/focusa-tools/tools/focusa_state_hygiene_doctor.md) |
| `focusa_state_hygiene_plan` | Diagnostics / Hygiene | [`docs/focusa-tools/tools/focusa_state_hygiene_plan.md`](docs/focusa-tools/tools/focusa_state_hygiene_plan.md) |
| `focusa_state_hygiene_apply` | Diagnostics / Hygiene | [`docs/focusa-tools/tools/focusa_state_hygiene_apply.md`](docs/focusa-tools/tools/focusa_state_hygiene_apply.md) |
| `focusa_silent_sessions` | Work-loop | [`docs/focusa-tools/tools/focusa_silent_sessions.md`](docs/focusa-tools/tools/focusa_silent_sessions.md) |
| `focusa_tool_doctor` | Diagnostics / Hygiene | [`docs/focusa-tools/tools/focusa_tool_doctor.md`](docs/focusa-tools/tools/focusa_tool_doctor.md) |
| `focusa_agent_prompt` | Focus State | [`docs/focusa-tools/tools/focusa_agent_prompt.md`](docs/focusa-tools/tools/focusa_agent_prompt.md) |
| `focusa_resource_mode` | Diagnostics / Hygiene | [`docs/focusa-tools/tools/focusa_resource_mode.md`](docs/focusa-tools/tools/focusa_resource_mode.md) |
| `focusa_active_object_resolve` | Workpoint | [`docs/focusa-tools/tools/focusa_active_object_resolve.md`](docs/focusa-tools/tools/focusa_active_object_resolve.md) |
| `focusa_evidence_capture` | Workpoint | [`docs/focusa-tools/tools/focusa_evidence_capture.md`](docs/focusa-tools/tools/focusa_evidence_capture.md) |
| `focusa_browser_diagnostics_intake` | Workpoint | [`docs/focusa-tools/tools/focusa_browser_diagnostics_intake.md`](docs/focusa-tools/tools/focusa_browser_diagnostics_intake.md) |
| `focusa_workpoint_checkpoint` | Workpoint | [`docs/focusa-tools/tools/focusa_workpoint_checkpoint.md`](docs/focusa-tools/tools/focusa_workpoint_checkpoint.md) |
| `focusa_call_stack_design` | Workpoint | [`docs/focusa-tools/tools/focusa_call_stack_design.md`](docs/focusa-tools/tools/focusa_call_stack_design.md) |
| `focusa_device_pair_start` | Session Transfer | [`docs/focusa-tools/tools/focusa_device_pair_start.md`](docs/focusa-tools/tools/focusa_device_pair_start.md) |
| `focusa_device_pair_qr` | Session Transfer | [`docs/focusa-tools/tools/focusa_device_pair_qr.md`](docs/focusa-tools/tools/focusa_device_pair_qr.md) |
| `focusa_device_pair_complete` | Session Transfer | [`docs/focusa-tools/tools/focusa_device_pair_complete.md`](docs/focusa-tools/tools/focusa_device_pair_complete.md) |
| `focusa_device_pair_status` | Session Transfer | [`docs/focusa-tools/tools/focusa_device_pair_status.md`](docs/focusa-tools/tools/focusa_device_pair_status.md) |
| `focusa_device_pair_list` | Session Transfer | [`docs/focusa-tools/tools/focusa_device_pair_list.md`](docs/focusa-tools/tools/focusa_device_pair_list.md) |
| `focusa_device_pair_revoke` | Session Transfer | [`docs/focusa-tools/tools/focusa_device_pair_revoke.md`](docs/focusa-tools/tools/focusa_device_pair_revoke.md) |
| `focusa_workpoint_link_evidence` | Workpoint | [`docs/focusa-tools/tools/focusa_workpoint_link_evidence.md`](docs/focusa-tools/tools/focusa_workpoint_link_evidence.md) |
| `focusa_workpoint_resume` | Workpoint | [`docs/focusa-tools/tools/focusa_workpoint_resume.md`](docs/focusa-tools/tools/focusa_workpoint_resume.md) |
| `focusa_tree_head` | Tree / Lineage | [`docs/focusa-tools/tools/focusa_tree_head.md`](docs/focusa-tools/tools/focusa_tree_head.md) |
| `focusa_tree_path` | Tree / Lineage | [`docs/focusa-tools/tools/focusa_tree_path.md`](docs/focusa-tools/tools/focusa_tree_path.md) |
| `focusa_tree_snapshot_state` | Tree / Lineage | [`docs/focusa-tools/tools/focusa_tree_snapshot_state.md`](docs/focusa-tools/tools/focusa_tree_snapshot_state.md) |
| `focusa_tree_restore_state` | Tree / Lineage | [`docs/focusa-tools/tools/focusa_tree_restore_state.md`](docs/focusa-tools/tools/focusa_tree_restore_state.md) |
| `focusa_tree_diff_context` | Tree / Lineage | [`docs/focusa-tools/tools/focusa_tree_diff_context.md`](docs/focusa-tools/tools/focusa_tree_diff_context.md) |
| `focusa_metacog_capture` | Metacognition | [`docs/focusa-tools/tools/focusa_metacog_capture.md`](docs/focusa-tools/tools/focusa_metacog_capture.md) |
| `focusa_metacog_retrieve` | Metacognition | [`docs/focusa-tools/tools/focusa_metacog_retrieve.md`](docs/focusa-tools/tools/focusa_metacog_retrieve.md) |
| `focusa_metacog_reflect` | Metacognition | [`docs/focusa-tools/tools/focusa_metacog_reflect.md`](docs/focusa-tools/tools/focusa_metacog_reflect.md) |
| `focusa_metacog_plan_adjust` | Metacognition | [`docs/focusa-tools/tools/focusa_metacog_plan_adjust.md`](docs/focusa-tools/tools/focusa_metacog_plan_adjust.md) |
| `focusa_metacog_evaluate_outcome` | Metacognition | [`docs/focusa-tools/tools/focusa_metacog_evaluate_outcome.md`](docs/focusa-tools/tools/focusa_metacog_evaluate_outcome.md) |
| `focusa_tree_recent_snapshots` | Tree / Lineage | [`docs/focusa-tools/tools/focusa_tree_recent_snapshots.md`](docs/focusa-tools/tools/focusa_tree_recent_snapshots.md) |
| `focusa_tree_snapshot_compare_latest` | Tree / Lineage | [`docs/focusa-tools/tools/focusa_tree_snapshot_compare_latest.md`](docs/focusa-tools/tools/focusa_tree_snapshot_compare_latest.md) |
| `focusa_metacog_recent_reflections` | Metacognition | [`docs/focusa-tools/tools/focusa_metacog_recent_reflections.md`](docs/focusa-tools/tools/focusa_metacog_recent_reflections.md) |
| `focusa_metacog_recent_adjustments` | Metacognition | [`docs/focusa-tools/tools/focusa_metacog_recent_adjustments.md`](docs/focusa-tools/tools/focusa_metacog_recent_adjustments.md) |
| `focusa_metacog_loop_run` | Metacognition | [`docs/focusa-tools/tools/focusa_metacog_loop_run.md`](docs/focusa-tools/tools/focusa_metacog_loop_run.md) |
| `focusa_metacog_doctor` | Metacognition | [`docs/focusa-tools/tools/focusa_metacog_doctor.md`](docs/focusa-tools/tools/focusa_metacog_doctor.md) |
| `focusa_lineage_tree` | Tree / Lineage | [`docs/focusa-tools/tools/focusa_lineage_tree.md`](docs/focusa-tools/tools/focusa_lineage_tree.md) |
| `focusa_li_tree_extract` | Tree / Lineage | [`docs/focusa-tools/tools/focusa_li_tree_extract.md`](docs/focusa-tools/tools/focusa_li_tree_extract.md) |

### Focusa skills

The main `focusa` skill is the router and mental model. Focused companion skills provide progressive-disclosure playbooks for high-value workflows:

- `.pi/skills/focusa/SKILL.md` / `apps/pi-extension/skills/focusa/SKILL.md` — main Focusa router skill.
- `.pi/skills/focusa-workpoint/SKILL.md` — Workpoint recovery, evidence linking, drift-safe handoff.
- `.pi/skills/focusa-metacognition/SKILL.md` — reusable learning and quality-gated reflection.
- `.pi/skills/focusa-work-loop/SKILL.md` — continuous work-loop ownership/control.
- `.pi/skills/focusa-cli-api/SKILL.md` — daemon, CLI, API, release-proof operations.
- `.pi/skills/focusa-troubleshooting/SKILL.md` — degraded/offline/pending/blocked recovery.
- `.pi/skills/focusa-docs-maintenance/SKILL.md` — public docs, skills, evidence, and snapshot wording.
- `.pi/skills/predictive-power/SKILL.md` — prediction record/evaluation/stats workflows.

## Current polish, security, and release docs

- [Agent Awareness Quickstart](docs/current/AGENT_AWARENESS_QUICKSTART.md)
- [Focusa Agent Utility Card](docs/current/FOCUSA_AGENT_UTILITY_CARD.md)
- [Friendly Focusa Onboarding Q](docs/current/FOCUSA_FRIENDLY_ONBOARDING.md)
- [Focusa Tool Choreography Map](docs/current/FOCUSA_TOOL_CHOREOGRAPHY_MAP.md)
- [Model-Visible Awareness Surfaces](docs/current/FOCUSA_MODEL_VISIBLE_AWARENESS.md)
- [Model Forgetting / Scope Override Incident and Attention Guard Spec](docs/current/PROJECT_SCOPE_OVERRIDE_INCIDENT_AND_GUARD_SPEC_2026-05-31.md)
- [Tool Implementation-to-Spec Audit](docs/current/FOCUSA_TOOL_IMPLEMENTATION_SPEC_AUDIT.md)
- [Non-Pi Agent Focusa Usage](docs/current/NON_PI_AGENT_FOCUSA_USAGE.md)
- [Predictive Power Guide](docs/current/PREDICTIVE_POWER_GUIDE.md)
- [Project Intelligence Flywheel](docs/current/PROJECT_INTELLIGENCE_FLYWHEEL.md)
- [Prediction Algorithms Implemented](docs/current/PREDICTION_ALGORITHMS_IMPLEMENTED.md)
- [UIAI Browser Diagnostics → Focusa Integration](docs/current/UIAI_BROWSER_DIAGNOSTICS_FOCUSA_INTEGRATION_SPEC.md)
- [Agent Command Cookbook](docs/current/AGENT_COMMAND_COOKBOOK.md)
- [Doctor / Continue / Release Prove](docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md)
- [Workpoint Project Folder + Continuity Guard](docs/current/WORKPOINT_SESSION_SCOPE_GUARD.md)
- [Compaction Fallbacks](docs/current/COMPACTION_FALLBACKS.md)
- [Daemon Resilience](docs/current/DAEMON_RESILIENCE.md)
- [Efficiency Guide](docs/current/EFFICIENCY_GUIDE.md)
- [Hook Coverage](docs/current/HOOK_COVERAGE.md)
- [Trajectory Tool Index](docs/focusa-tools/trajectory.md)
- [Trajectory GTM and Companion Gap Assessment](docs/current/TRAJECTORY_GTM_AND_GAPS.md)
- [Tauri Menubar Functionality Audit](docs/current/TAURI_MENUBAR_FUNCTIONALITY_AUDIT.md)
- [Tauri Menubar Up-to-Speed Spec](docs/current/TAURI_MENUBAR_UP_TO_SPEED_SPEC.md)
- [Focusa Security Review](docs/current/FOCUSA_SECURITY_REVIEW_2026-05-26.md)
- [Focusa Security Standard Matrix Review](docs/current/FOCUSA_SECURITY_STANDARD_MATRIX_REVIEW_2026-05-26.md)
- [Validation and Release Proof](docs/current/VALIDATION_AND_RELEASE_PROOF.md)
- [Spec92 Full Rollout Proof](docs/evidence/SPEC92_FULL_ROLLOUT_PROOF_2026-04-28.md)
- [Public Docs Release Sync 2026-05-26](docs/evidence/PUBLIC_DOCS_RELEASE_SYNC_2026-05-26.md)

## License

Focusa is source-available under the Focusa Business Source License 1.1 in `LICENSE.md`.

Free use is limited to personal, educational, evaluation, and non-commercial local use. Commercial, team/company, hosted-service, client-delivery, redistribution, or product-embedding use requires a paid commercial license from Startempire Wire; see `COMMERCIAL.md`.

Trademark use is governed separately by `TRADEMARKS.md`. External contributions require prior approval; see `CONTRIBUTING.md`. Commercial support expectations are in `SUPPORT_TERMS.md`. Common licensing questions are answered in `LICENSE-FAQ.md`.
