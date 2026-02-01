# Focusa — Finalized Documentation Set

**55 canonical docs** extracted from 395-message ChatGPT conversation (1.4MB, 56K lines).
Only the **latest finalized version** of each doc is included. Superseded drafts eliminated.

**Source conversation:** `/data/wirebot/focusa/focusa-chatgpt-conversation.md`

---

## What Is Focusa?

Focusa is a **local-first cognitive governance framework** for AI agents. It preserves focus, intent, and meaning across long-running AI sessions by separating cognition from conversation. It is harness-agnostic (works with Letta, Claude Code, Codex CLI, etc.), deterministic, and human-aligned.

---

## Top-Level Docs

| File | Description |
|------|-------------|
| [README.md](README.md) | Project overview |
| [PRD.md](PRD.md) | Product Requirements Document (final) |
| [AGENTS.md](AGENTS.md) | Agent protocol (Beads-centered) |
| [00-glossary.md](00-glossary.md) | **LOCKED** canonical glossary — all terms authoritative |

---

## Core Architecture (Gen2 — Canonical Terminology)

| # | File | Subsystem |
|---|------|-----------|
| 01 | [01-architecture-overview.md](01-architecture-overview.md) | MVP architecture overview |
| 02 | [02-runtime-daemon.md](02-runtime-daemon.md) | Runtime daemon, state, persistence |
| 03 | [03-focus-stack.md](03-focus-stack.md) | Focus Stack & Focus Frames |
| 04 | [04-focus-gate.md](04-focus-gate.md) | Focus Gate (RAS-inspired salience filter) |
| 05 | [05-intuition-engine.md](05-intuition-engine.md) | Intuition Engine (subconscious pattern detection) |
| 06 | [06-focus-state.md](06-focus-state.md) | Focus State (current state of mind) |
| 07 | [07-reference-store.md](07-reference-store.md) | Reference Store (externalized artifact memory) |
| 08 | [08-expression-engine.md](08-expression-engine.md) | Expression Engine (prompt assembly) |
| 09 | [09-proxy-adapter.md](09-proxy-adapter.md) | Proxy & harness adapters |
| 10 | [10-monorepo-layout.md](10-monorepo-layout.md) | Monorepo layout |
| 11 | [11-menubar-ui-spec.md](11-menubar-ui-spec.md) | Menubar UI (Tauri + SvelteKit) |

---

## Autonomy & Governance

| # | File | Subsystem |
|---|------|-----------|
| 12 | [12-autonomy-scoring.md](12-autonomy-scoring.md) | Autonomy scoring & earned capability |
| 13 | [13-autonomy-ui.md](13-autonomy-ui.md) | Autonomy visualization (CLI + menubar) |
| 14 | [14-uxp-ufi-schema.md](14-uxp-ufi-schema.md) | User Experience Calibration (UXP/UFI) |
| 15 | [15-agent-schema.md](15-agent-schema.md) | Agent definition (UPDATED, AUTHORITATIVE) |
| 16 | [16-agent-constitution.md](16-agent-constitution.md) | Agent constitution |
| 16b | [16-constitution-synthesizer.md](16-constitution-synthesizer.md) | Constitution synthesizer |

---

## Provenance & Caching

| # | File | Subsystem |
|---|------|-----------|
| 17 | [17-context-lineage-tree.md](17-context-lineage-tree.md) | Context Lineage Tree (CLT) — full provenance |
| 18 | [18-cache-permission-matrix.md](18-cache-permission-matrix.md) | Cache permission matrix |
| 19 | [19-intentional-cache-busting.md](19-intentional-cache-busting.md) | Intentional cache busting triggers |

---

## Data & Training

| # | File | Subsystem |
|---|------|-----------|
| 20 | [20-training-dataset-schema.md](20-training-dataset-schema.md) | Training dataset schema |
| 21 | [21-data-export-cli.md](21-data-export-cli.md) | Data export CLI (`focusa export`) |
| 22 | [22-data-contribution.md](22-data-contribution.md) | Opt-in background data contribution |

---

## Capabilities API

| # | File | Subsystem |
|---|------|-----------|
| 23 | [23-capabilities-api.md](23-capabilities-api.md) | Capabilities API |
| 24 | [24-capabilities-cli.md](24-capabilities-cli.md) | Capabilities CLI |
| 25 | [25-capability-permissions.md](25-capability-permissions.md) | Capability permissions model |
| 26 | [26-agent-capability-scope.md](26-agent-capability-scope.md) | Agent capability scope model |

---

## TUI & Telemetry

| # | File | Subsystem |
|---|------|-----------|
| 27 | [27-tui-spec.md](27-tui-spec.md) | TUI specification (ratatui) |
| 28 | [28-ratatui-component-tree.md](28-ratatui-component-tree.md) | TUI component tree |
| 29 | [29-telemetry-spec.md](29-telemetry-spec.md) | Cognitive Telemetry Layer (CTL) + update |
| 30 | [30-telemetry-schema.md](30-telemetry-schema.md) | Telemetry event schema |
| 31 | [31-telemetry-api.md](31-telemetry-api.md) | Telemetry capabilities API |
| 32 | [32-telemetry-tui.md](32-telemetry-tui.md) | Telemetry TUI integration |

---

## Advanced Systems

| # | File | Subsystem |
|---|------|-----------|
| 33 | [33-acp-proxy-spec.md](33-acp-proxy-spec.md) | ACP proxy & observation integration |
| 34 | [34-agent-skills-spec.md](34-agent-skills-spec.md) | Agent skill bundles |
| 35 | [35-skill-to-capabilities-mapping.md](35-skill-to-capabilities-mapping.md) | Skills → Capabilities mapping |
| 36 | [36-reliability-focus-mode.md](36-reliability-focus-mode.md) | Reliability Focus Mode + AIS update |
| 37 | [37-autonomy-calibration-spec.md](37-autonomy-calibration-spec.md) | Autonomy calibration (AUTHORITATIVE) |

---

## Threads & Concurrency

| # | File | Subsystem |
|---|------|-----------|
| 38 | [38-thread-thesis-spec.md](38-thread-thesis-spec.md) | Thread thesis (cognitive workspaces) |
| 39 | [39-thread-lifecycle-spec.md](39-thread-lifecycle-spec.md) | Thread lifecycle |
| 40 | [40-instance-session-attachment-spec.md](40-instance-session-attachment-spec.md) | Instance/Session/Attachment concurrency |
| 41 | [41-proposal-resolution-engine.md](41-proposal-resolution-engine.md) | Proposal Resolution Engine (PRE) |

---

## Gen1 Docs (Not Superseded — Unique Topics)

These docs from the initial spec cover topics that were NOT rewritten in the Gen2 terminology refresh. They remain authoritative for their topics, with UPDATE patches merged in.

| File | Topic | Notes |
|------|-------|-------|
| [G1-07-ascc.md](G1-07-ascc.md) | Anchored Structured Context Checkpointing (ASCC) | + Pinning & Degradation update |
| [G1-09-memory.md](G1-09-memory.md) | Semantic + Procedural Memory | + Trust Model update |
| [G1-10-workers.md](G1-10-workers.md) | Background Workers & Async Cognition | |
| [G1-12-api.md](G1-12-api.md) | Local HTTP API Specification | |
| [G1-13-cli.md](G1-13-cli.md) | CLI Contract | |
| [G1-16-testing.md](G1-16-testing.md) | Testing & Acceptance | + New Acceptance Criteria update |

---

## Implementation Docs

| File | Description |
|------|-------------|
| [bootstrap-prompt.md](bootstrap-prompt.md) | Engineer agent bootstrap prompt (MVP) |
| [bootstrap-prompt-rust.md](bootstrap-prompt-rust.md) | Rust-first engineer agent bootstrap prompt |
| [core-reducer.md](core-reducer.md) | Focusa-Core Reducer — canonical pseudocode (AUTHORITATIVE) |

---

## Gen1 Implementation Detail Supplements

These Gen1 docs contain **data models, algorithms, schemas, acceptance tests, and implementation specifics** that Gen2 docs intentionally kept high-level. Read Gen2 for concepts, read these for implementation.

| File | Topic | Key Content Gen2 Lacks |
|------|-------|----------------------|
| [G1-detail-00-doc-suite-readme.md](G1-detail-00-doc-suite-readme.md) | Original doc suite README | Doc numbering rationale, reading order |
| [G1-detail-03-runtime-daemon.md](G1-detail-03-runtime-daemon.md) | Runtime Daemon detail | `AppState` struct, process model, event log, persistence rules, startup/recovery, shutdown, reducer |
| [G1-detail-04-proxy-adapter.md](G1-detail-04-proxy-adapter.md) | Proxy Adapter detail | Integration modes (wrap CLI vs HTTP proxy), turn data shapes, daemon endpoints, validation checklist |
| [G1-detail-05-focus-stack-hec.md](G1-detail-05-focus-stack-hec.md) | Focus Stack (HEC) detail | Frame/FrameId/FrameRecord/FrameStats data model, PushFrame/PopFrame operations, persistence, acceptance tests |
| [G1-detail-06-focus-gate.md](G1-detail-06-focus-gate.md) | Focus Gate detail | 5-step algorithm, Candidate/Signal/CandidateState data model, pressure mechanics, pinning, temporal signals |
| [G1-detail-08-ecs.md](G1-detail-08-ecs.md) | Externalized Context Store (ECS) | Handle data model (HandleId/HandleKind/HandleRef), StoreArtifact/ResolveHandle ops, session scoping, human pinning |
| [G1-detail-11-prompt-assembly.md](G1-detail-11-prompt-assembly.md) | Prompt Assembly detail | 7-slot structure, budget contract, delta injection, handle rehydration, explicit degradation strategy |
| [G1-detail-15-events-observability.md](G1-detail-15-events-observability.md) | Events & Observability detail | Complete event type taxonomy (Stack, Gate, ASCC, ECS, Memory, Prompt, Worker, Adapter events), replay invariant |

---

## PRD Supplements

| File | Description |
|------|-------------|
| [G1-detail-PRD-gen2-intermediate.md](G1-detail-PRD-gen2-intermediate.md) | Gen2 intermediate PRD (between Gen1 full PRD and final delta) |
| [PRD-delta-threads.md](PRD-delta-threads.md) | Thread concept section for PRD |
| [PRD-delta-thread-workspaces.md](PRD-delta-thread-workspaces.md) | Thread as cognitive workspace section |

---

## Reading Order

1. **Start here:** [00-glossary.md](00-glossary.md) — defines all canonical terms
2. **Architecture:** [01-architecture-overview.md](01-architecture-overview.md) → [02-runtime-daemon.md](02-runtime-daemon.md)
3. **Core subsystems:** 03 → 04 → 05 → 06 → 07 → 08 (Focus Stack → Gate → Intuition → State → Reference → Expression)
4. **For implementation depth:** Read the `G1-detail-*` counterpart of each Gen2 doc
5. **Memory:** [G1-07-ascc.md](G1-07-ascc.md) + [G1-09-memory.md](G1-09-memory.md) (Gen1 only — not yet rewritten)
6. **Autonomy:** 12 → 13 → 37
7. **Agent model:** 15 → 16 → 16-constitution-synthesizer
8. **Advanced:** 17 (CLT) → 36 (Reliability) → 38-41 (Threads/Concurrency/Proposals)
