# Focusa Agent Workforce — Browser Extension Concept (#174)

**Status:** concept (brainstorm crystallized 2026-08-23) · **Related:** #173, #175

## Vision

A separate, complete UI living in the browser: mission control for a person's
agent workforce. Anyone using Focusa spins up agents with roles, capabilities,
and objectives; manages them directly from the extension; watches them work;
and audits everything. **The extension is a window, not the runtime** — agents
live in the daemon(s), keep working when the browser closes, and the extension
reconnects live over the daemon SSE stream.

## Object model

```
Owner ── pairs ──► N Daemons (local laptop, VPS, …)
Owner ── owns ──► Projects ── contain ──► Workstreams ── contain ──► Task Graphs (DAG)
Task Graph nodes ── executed by ──► Agents
Agents ── instantiated from ──► Roles (= Objective + Capabilities + Secret Scopes
                                        + Model Tier + Budget + Lifespan)
Managers (stateful) ── delegate subsets of their allowance to ──► Crew (stateless)
Speed dial: fanout width 2x ▸ 4x ▸ 6x (FanoutPlan lanes, wait-for-all join)
```

- Stateful Managers: persistent memory via daemon awareness substrate; delegate,
  review, replan; survive browser close.
- Stateless Crew: spawned per node via focusa bg/workloop machinery; expire on
  completion; cost-tracked per unit of work.
- Quality gates: Manager reviews crew output against node acceptance criteria
  before done — production-consistency discipline encoded in the graph.
- Human checkpoint nodes: graph pauses for approval; extension badges the owner.

## Extension surfaces

1. Roster view (managers above, crew swarms below; spend meters)
2. Task-graph canvas (status colors; speed dial; mid-flight inject/reprioritize)
3. Live agent view — embed UIAI FPV stream: watch crew drive a real browser
4. Approvals inbox (broker step-up consents, checkpoints, risky actions)
5. Audit timeline (shared ledger schema; filter by agent/project/secret/daemon;
   exportable client-facing report)
6. Evidence cards on nodes — proof artifacts previewed inline with
   verified/unverified status; closure requires evidence (no false closures)
7. Direction bar — natural-language steering routed to the owning Manager

## Multi-daemon topology (decided 2026-08-23)

- Owner Identity pairs with each daemon (menubar bridge precedent: pairing
  token + device id + protocol-versioned nonce handshake).
- Daemons render as named endpoints under ONE owner lens; projects/workforces/
  audits tagged with owner identity — no cross-daemon ownership confusion.
- Cross-daemon rollups (unified roster/audit): follow best software practice —
  query-at-render federation first (each daemon remains source of truth for its
  own ledger), replicate only if performance demands later.

## Concurrent direction (decided 2026-08-23)

CRDT-inspired convergence: direction edits are intents merged at the daemon.
Scalar fields (priority, status) = last-writer-wins with logical clocks;
ordered collections (queue order) = list-CRDT style commutative ops. Managers
act as semantic arbiters where intent conflicts are ambiguous. Extension,
Cockpit, CLI are equal-intent clients.

## Shared lexicon (all surfaces, defined once)

Owner · Daemon · Project · Workstream · Task Graph · Node · Manager · Crew ·
Role · Capability · Tool Bundle · Secret Scope · Objective · Acceptance
Criteria · Evidence · Approval · Audit Entry

Design rule: every agent action is inspectable; every human touchpoint is a
consent or a direction. Agent-first, human-friendly.

## Future (deliberately out of scope here)

Human-side Focusa Browser Extension: capture-save into vaults, share-session-
with-agent consent flows. Planned separately; will join this mesh later.

## Live interaction layer (added 2026-08-23)

1. **Real-time speech-to-text** (GPT realtime transcription): always-available
   voice input in the extension feeding the Direction Bar — speak to steer,
   spawn, approve, or redirect managers mid-flight. Voice is just another
   intent client into the same CRDT-merged direction pipeline; transcribed
   intents appear in the audit timeline like typed ones.
2. **Focusa Radar embedded** (per its own spec 164,
   `spec/164-focusa-radar`: workstream-scoped ambient intelligence, agent
   inbox, autonomous multi-agent execution): the extension surfaces Radar as a
   live pane — ambient awareness of workstream state plus the agent inbox — so
   monitoring, direction, and Radar-driven autonomous execution share one
   surface and one lexicon.

Together with the task-graph canvas these make the extension genuinely *live*:
speak to direct, watch graphs move, receive Radar signals, approve from the
inbox — without leaving the browser.

## Elon-rule trim pass (2026-08-23)

Deleted from launch scope (re-enter only when reality demands): Radar pane
(waits on Spec 164 maturity), Agency/Client Contexts (design leaves room;
zero code until a customer demands separation), local Whisper STT (one-line
seam kept in place), Tailscale transport tier. Launch surfaces: pairing,
roster, task-graph canvas, audit timeline. Voice via OpenAI realtime only.
