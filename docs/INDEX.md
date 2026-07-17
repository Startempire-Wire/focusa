# Focusa — Current Documentation Snapshot

**Current public docs map with core architecture, runtime surfaces, ontology/Pi alignment, and implementation evidence.**
Focusa remains under active development; docs describe logical snapshots and may include older design-direction material.

**Current docs/runtime snapshot:** `v0.9.13-dev`

Older architecture/spec documents may retain historical `v0.9.0-dev` snapshot language when describing design-era context.

---

## What Is Focusa?

Focusa is a **local-first cognitive governance framework** for AI agents. It preserves focus, intent, and meaning across long-running AI sessions by separating cognition from conversation. It is harness-agnostic (works with Letta, Claude Code, Codex CLI, etc.), deterministic, and human-aligned.

---

## Top-Level Docs

| File                             | Description                                             |
| -------------------------------- | ------------------------------------------------------- |
| [README.md](README.md)           | Project overview                                        |
| [PRD.md](PRD.md)                 | Product Requirements Document (final)                   |
| [AGENTS.md](AGENTS.md)           | Agent protocol (Beads-centered)                         |
| [agent/01-focusa-agent-docs-index.md](agent/01-focusa-agent-docs-index.md) | Public-safe agent architecture, commands, API, Workpoints, Trajectory, and boundary guide |
| [00-glossary.md](00-glossary.md) | **LOCKED** canonical glossary — all terms authoritative |

---

## Core Architecture (Gen2 — Canonical Terminology)

| #   | File                                                       | Subsystem                                         |
| --- | ---------------------------------------------------------- | ------------------------------------------------- |
| 01  | [01-architecture-overview.md](01-architecture-overview.md) | Architecture overview and design direction        |
| 02  | [02-runtime-daemon.md](02-runtime-daemon.md)               | Runtime daemon, state, persistence                |
| 03  | [03-focus-stack.md](03-focus-stack.md)                     | Focus Stack & Focus Frames                        |
| 04  | [04-focus-gate.md](04-focus-gate.md)                       | Focus Gate (RAS-inspired salience filter)         |
| 05  | [05-intuition-engine.md](05-intuition-engine.md)           | Intuition Engine (subconscious pattern detection) |
| 06  | [06-focus-state.md](06-focus-state.md)                     | Focus State (current state of mind)               |
| 07  | [07-reference-store.md](07-reference-store.md)             | Reference Store (externalized artifact memory)    |
| 08  | [08-expression-engine.md](08-expression-engine.md)         | Expression Engine (prompt assembly)               |
| 09  | [09-proxy-adapter.md](09-proxy-adapter.md)                 | Proxy & harness adapters                          |
| 10  | [10-monorepo-layout.md](10-monorepo-layout.md)             | Monorepo layout                                   |
| 11  | [11-menubar-ui-spec.md](11-menubar-ui-spec.md)             | Older menubar UI design direction; reconcile with current audit/spec before implementation |
| current | [current/TAURI_MENUBAR_FUNCTIONALITY_AUDIT.md](current/TAURI_MENUBAR_FUNCTIONALITY_AUDIT.md) | Current Tauri menubar lag-behind audit |
| current | [current/TAURI_MENUBAR_UP_TO_SPEED_SPEC.md](current/TAURI_MENUBAR_UP_TO_SPEED_SPEC.md) | Current implementation spec for a fully up-to-speed runtime cockpit app |
| current | [current/TAURI_MENUBAR_IMPLEMENTATION_GAPS.md](current/TAURI_MENUBAR_IMPLEMENTATION_GAPS.md) | Remaining implementation gaps and first implementation slice for the menubar cockpit |
| current | [current/FOCUSA_BRAIN_BODY_ANALOGY_GAP_MAP.md](current/FOCUSA_BRAIN_BODY_ANALOGY_GAP_MAP.md) | Whole-organism brain/body analogy, maturity gaps, and exhaustive docs cross-reference |
| current | [current/FOCUSA_FEATURE_MATURITY_AUDIT_2026-05-26.md](current/FOCUSA_FEATURE_MATURITY_AUDIT_2026-05-26.md) | Code-based 1–10 feature maturity ratings and underdeveloped workflow gaps |
| current | [current/DATASET_PREDICTION_SUBSTRATE.md](current/DATASET_PREDICTION_SUBSTRATE.md) | Dataset-agnostic prediction substrate, with stocks as the first domain adapter |
| current | [current/PREDICTION_METACOG_SIGNAL_SUBSTRATE.md](current/PREDICTION_METACOG_SIGNAL_SUBSTRATE.md) | Focusa-native prediction/metacognition/ontology flywheel substrate |
| current | [current/PREDICTIVE_METACOG_MATURITY_EVAL_2026-05-26.md](current/PREDICTIVE_METACOG_MATURITY_EVAL_2026-05-26.md) | Current maturity verdict for predictive and metacognitive feature sets |
| current | [current/END_OF_TASK_LEARNING_LOOP.md](current/END_OF_TASK_LEARNING_LOOP.md) | Required prediction/metacog closure loop for compaction cards, trajectory reviews, and final work reports |
| current | [current/PROJECT_INTELLIGENCE_FLYWHEEL.md](current/PROJECT_INTELLIGENCE_FLYWHEEL.md) | Ontology-grounded project-card flywheel for trajectory bootstrap/re-bootstrap, prediction, and metacog compounding |
| current | [current/PREDICTION_ALGORITHMS_IMPLEMENTED.md](current/PREDICTION_ALGORITHMS_IMPLEMENTED.md) | Implemented lightweight prediction formulas behind project-card algorithmic intelligence |
| current | [current/AUTONOMIC_CODING_WORKFLOW_GOVERNOR.md](current/AUTONOMIC_CODING_WORKFLOW_GOVERNOR.md) | Proposed project-vitals/stuck-detector/governor layer for continuous coding agents |
| current | [current/FOCUSA_SECURITY_REVIEW_2026-05-26.md](current/FOCUSA_SECURITY_REVIEW_2026-05-26.md) | Five-part whole-project security review and remediation backlog |
| current | [current/FOCUSA_SECURITY_STANDARD_MATRIX_REVIEW_2026-05-26.md](current/FOCUSA_SECURITY_STANDARD_MATRIX_REVIEW_2026-05-26.md) | Focusa mapped against OWASP ASVS, OWASP API Top 10, CWE Top 25, STRIDE, and CIS Controls v8 |
| current | [current/API_ROUTE_PERMISSION_MATRIX.md](current/API_ROUTE_PERMISSION_MATRIX.md) | Intended API route scopes and route-family authorization baseline |
| current | [current/API_RESOURCE_LIMITS.md](current/API_RESOURCE_LIMITS.md) | API request body limit and resource-exhaustion posture |
| current | [current/PATH_TRAVERSAL_SECURITY_TESTS.md](current/PATH_TRAVERSAL_SECURITY_TESTS.md) | CWE-22 path traversal coverage and path-sensitive route inventory |
| current | [current/TAMPER_EVIDENT_EVENT_CHAIN.md](current/TAMPER_EVIDENT_EVENT_CHAIN.md) | SQLite event hash-chain checkpoints for repudiation detection |
| current | [current/DATA_RETENTION_BACKUP_DELETION_POLICY.md](current/DATA_RETENTION_BACKUP_DELETION_POLICY.md) | Local-first persisted-state retention, backup, restore, and deletion policy |
| current | [current/RUSTSEC_INFORMATIONAL_EXCEPTIONS.md](current/RUSTSEC_INFORMATIONAL_EXCEPTIONS.md) | Accepted informational RustSec exceptions and review triggers |
| current | [current/DYNAMIC_API_SECURITY_SMOKE.md](current/DYNAMIC_API_SECURITY_SMOKE.md) | Dynamic local API malformed JSON and oversized-body security smoke |
| current | [current/PERSISTED_STATE_PRIVACY_CLASSES.md](current/PERSISTED_STATE_PRIVACY_CLASSES.md) | Privacy classes and handling rules for Focusa persisted state |
| current | [current/SECURITY_COMMAND_BOUNDARY.md](current/SECURITY_COMMAND_BOUNDARY.md) | Reviewed shell/external command boundary and runtime unwrap static policy |

---

## Autonomy & Governance

| #   | File                                                             | Subsystem                                 |
| --- | ---------------------------------------------------------------- | ----------------------------------------- |
| 12  | [12-autonomy-scoring.md](12-autonomy-scoring.md)                 | Autonomy scoring & earned capability      |
| 13  | [13-autonomy-ui.md](13-autonomy-ui.md)                           | Autonomy visualization (CLI + menubar)    |
| 14  | [14-uxp-ufi-schema.md](14-uxp-ufi-schema.md)                     | User Experience Calibration (UXP/UFI)     |
| 15  | [15-agent-schema.md](15-agent-schema.md)                         | Agent definition (UPDATED, AUTHORITATIVE) |
| 16  | [16-agent-constitution.md](16-agent-constitution.md)             | Agent constitution                        |
| 16b | [16-constitution-synthesizer.md](16-constitution-synthesizer.md) | Constitution synthesizer                  |

---

## Provenance & Caching

| #   | File                                                               | Subsystem                                    |
| --- | ------------------------------------------------------------------ | -------------------------------------------- |
| 17  | [17-context-lineage-tree.md](17-context-lineage-tree.md)           | Context Lineage Tree (CLT) — full provenance |
| 18  | [18-cache-permission-matrix.md](18-cache-permission-matrix.md)     | Cache permission matrix                      |
| 19  | [19-intentional-cache-busting.md](19-intentional-cache-busting.md) | Intentional cache busting triggers           |

---

## Data & Training

| #   | File                                                           | Subsystem                           |
| --- | -------------------------------------------------------------- | ----------------------------------- |
| 20  | [20-training-dataset-schema.md](20-training-dataset-schema.md) | Training dataset schema             |
| 21  | [21-data-export-cli.md](21-data-export-cli.md)                 | Data export CLI (`focusa export`)   |
| 22  | [22-data-contribution.md](22-data-contribution.md)             | Opt-in background data contribution |

---

## Capabilities API

| #   | File                                                         | Subsystem                    |
| --- | ------------------------------------------------------------ | ---------------------------- |
| 23  | [23-capabilities-api.md](23-capabilities-api.md)             | Capabilities API             |
| 24  | [24-capabilities-cli.md](24-capabilities-cli.md)             | Capabilities CLI             |
| 25  | [25-capability-permissions.md](25-capability-permissions.md) | Capability permissions model |
| 26  | [26-agent-capability-scope.md](26-agent-capability-scope.md) | Agent capability scope model |

---

## TUI & Telemetry

| #   | File                                                         | Subsystem                                |
| --- | ------------------------------------------------------------ | ---------------------------------------- |
| 27  | [27-tui-spec.md](27-tui-spec.md)                             | TUI specification (ratatui)              |
| 28  | [28-ratatui-component-tree.md](28-ratatui-component-tree.md) | TUI component tree                       |
| 29  | [29-telemetry-spec.md](29-telemetry-spec.md)                 | Cognitive Telemetry Layer (CTL) + update |
| 30  | [30-telemetry-schema.md](30-telemetry-schema.md)             | Telemetry event schema                   |
| 31  | [31-telemetry-api.md](31-telemetry-api.md)                   | Telemetry capabilities API               |
| 32  | [32-telemetry-tui.md](32-telemetry-tui.md)                   | Telemetry TUI integration                |

---

## Ontology / Pi Alignment Addenda

| File                                                                                                       | Topic                                          |
| ---------------------------------------------------------------------------------------------------------- | ---------------------------------------------- |
| [45-ontology-overview.md](45-ontology-overview.md)                                                         | Ontology overview                              |
| [46-ontology-core-primitives.md](46-ontology-core-primitives.md)                                           | Core ontology primitives                       |
| [47-ontology-software-world.md](47-ontology-software-world.md)                                             | Software world ontology                        |
| [48-ontology-links-actions.md](48-ontology-links-actions.md)                                               | Links and actions                              |
| [49-working-sets-and-slices.md](49-working-sets-and-slices.md)                                             | Working sets and slices                        |
| [50-ontology-classification-and-reducer.md](50-ontology-classification-and-reducer.md)                     | Classification and reducer                     |
| [51-ontology-expression-and-proxy.md](51-ontology-expression-and-proxy.md)                                 | Expression and proxy integration               |
| [52-pi-extension-contract.md](52-pi-extension-contract.md)                                                 | Pi extension contract                          |
| [53-pi-behavioral-alignment-contract.md](53-pi-behavioral-alignment-contract.md)                           | Behavioral alignment contract                  |
| [54-pi-visible-output-boundary.md](54-pi-visible-output-boundary.md)                                       | Visible output boundary                        |
| [54a-operator-priority-and-subject-preservation.md](54a-operator-priority-and-subject-preservation.md)     | Operator priority and subject preservation     |
| [54b-context-injection-and-attention-routing.md](54b-context-injection-and-attention-routing.md)           | Context injection and attention routing        |
| [55-tool-action-contracts.md](55-tool-action-contracts.md)                                                 | Tool action contracts                          |
| [55-tool-action-contracts-impl.md](55-tool-action-contracts-impl.md)                                       | Tool action implementation notes               |
| [56-trace-checkpoints-recovery.md](56-trace-checkpoints-recovery.md)                                       | Trace, checkpoints, recovery                   |
| [57-golden-tasks-and-evals.md](57-golden-tasks-and-evals.md)                                               | Golden tasks and evals                         |
| [58-visual-ui-ontology-core.md](58-visual-ui-ontology-core.md)                                             | Visual/UI ontology core                        |
| [59-visual-ui-reverse-engineering.md](59-visual-ui-reverse-engineering.md)                                 | Visual/UI reverse engineering                  |
| [60-visual-ui-verification-and-critique.md](60-visual-ui-verification-and-critique.md)                     | Visual/UI verification and critique            |
| [61-domain-general-cognition-core.md](61-domain-general-cognition-core.md)                                 | Domain-general cognition core                  |
| [62-visual-ui-evidence-and-workflow.md](62-visual-ui-evidence-and-workflow.md)                             | Visual/UI evidence and workflow                |
| [63-visual-ui-invention-and-variation.md](63-visual-ui-invention-and-variation.md)                         | Visual/UI invention and variation              |
| [64-visual-ui-to-implementation.md](64-visual-ui-to-implementation.md)                                     | Visual/UI to implementation                    |
| [65-visual-ui-focusa-integration.md](65-visual-ui-focusa-integration.md)                                   | Visual/UI Focusa integration                   |
| [66-affordance-and-execution-environment-ontology.md](66-affordance-and-execution-environment-ontology.md) | Affordance and execution environment ontology  |
| [67-query-scope-and-relevance-control.md](67-query-scope-and-relevance-control.md)                         | Query scope and relevance control              |
| [68-current-ask-and-scope-integration.md](68-current-ask-and-scope-integration.md)                         | Current ask and scope integration              |
| [69-scope-failure-and-relevance-tracing.md](69-scope-failure-and-relevance-tracing.md)                     | Scope failure and relevance tracing            |
| [70-shared-interfaces-statuses-and-lifecycle.md](70-shared-interfaces-statuses-and-lifecycle.md)           | Shared interfaces, statuses, and lifecycle     |
| [71-governing-priors-and-scalar-weights.md](71-governing-priors-and-scalar-weights.md)                     | Governing priors and scalar weights            |
| [72-agent-identity-role-and-self-model-ontology.md](72-agent-identity-role-and-self-model-ontology.md)     | Agent identity, role, and self-model ontology  |
| [73-intention-commitment-and-self-regulation.md](73-intention-commitment-and-self-regulation.md)           | Intention, commitment, and self-regulation     |
| [74-identity-and-reference-resolution.md](74-identity-and-reference-resolution.md)                         | Identity and reference resolution              |
| [75-projection-and-view-semantics.md](75-projection-and-view-semantics.md)                                 | Projection and view semantics                  |
| [76-retention-forgetting-and-decay-policy.md](76-retention-forgetting-and-decay-policy.md)                 | Retention, forgetting, and decay policy        |
| [77-ontology-governance-versioning-and-migration.md](77-ontology-governance-versioning-and-migration.md)   | Ontology governance, versioning, and migration |

## Recent Implementation / Hardening Specs

| # | File | Subsystem |
| --- | --- | --- |
| 89 | [89-focusa-tool-suite-improvement-hardening-spec.md](89-focusa-tool-suite-improvement-hardening-spec.md) | Tool-suite hardening |
| 90 | [90-ontology-backed-tool-contracts-parity-spec.md](90-ontology-backed-tool-contracts-parity-spec.md) | Ontology-backed tool contracts |
| 91 | [91-live-tool-contract-proof-harness-spec.md](91-live-tool-contract-proof-harness-spec.md) | Live tool proof harness |
| 92 | [92-agent-first-polish-hooks-efficiency-spec.md](92-agent-first-polish-hooks-efficiency-spec.md) | Agent-first polish, recovery, prediction |
| 93 | [93-non-pi-agent-focusa-awareness-spec.md](93-non-pi-agent-focusa-awareness-spec.md) | Non-Pi agent awareness |
| 94 | [94-focusa-intent-preserving-memory-rpc-optimization-sow.md](94-focusa-intent-preserving-memory-rpc-optimization-sow.md) | Intent-preserving memory/RPC optimization |
| 95 | [95-focusa-ontology-low-latency-intelligence-enhancer-sow.md](95-focusa-ontology-low-latency-intelligence-enhancer-sow.md) | Ontology low-latency intelligence |
| 96 | [96-trajectory-projection-and-daemon-stability-spec.md](96-trajectory-projection-and-daemon-stability-spec.md) | Trajectory projection and daemon stability |
| 97 | [97-focusa-reflex-primitives-spec.md](97-focusa-reflex-primitives-spec.md) | Universal reflex primitives |
| 98 | [98-project-root-crdt-reconciliation-foundation-spec.md](98-project-root-crdt-reconciliation-foundation-spec.md) | Project-root CRDT reconciliation foundation |
| 99 | [99-original-intent-vs-implementation-audit.md](99-original-intent-vs-implementation-audit.md) | Original intent vs implementation audit |

## Spec 135 — Professional Workspaces and C.R.I.S.T. Project Genesis Series

Spec 135 and companions form one required implementation and closure set.

| # | File | Subsystem |
| --- | --- | --- |
| 135 | [135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md) | Master professional workspace and Project Genesis contract |
| 135A | [135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md](135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md) | Workspace projection, Pi sidebar, Work Rail, themes, vertical UX |
| 135B | [135b-crist-project-genesis-context-role-interview-spec-tasks.md](135b-crist-project-genesis-context-role-interview-spec-tasks.md) | C.R.I.S.T. Context, Role, Interview, Spec, and Tasks |
| 135C | [135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md](135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md) | UIAI rich artifacts, browser research, FPV, and live refresh |
| 135D | [135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md](135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md) | Complete build graph, reuse, performance, and no-deferral constitution |
| 135E | [135e-cross-spec-amendments-migration-and-closure-matrix.md](135e-cross-spec-amendments-migration-and-closure-matrix.md) | Cross-spec amendments, migration, compatibility, and closure matrix |

---

## Advanced Systems

| #   | File                                                                       | Subsystem                            |
| --- | -------------------------------------------------------------------------- | ------------------------------------ |
| 33  | [33-acp-proxy-spec.md](33-acp-proxy-spec.md)                               | ACP proxy & observation integration  |
| 34  | [34-agent-skills-spec.md](34-agent-skills-spec.md)                         | Agent skill bundles                  |
| 35  | [35-skill-to-capabilities-mapping.md](35-skill-to-capabilities-mapping.md) | Skills → Capabilities mapping        |
| 36  | [36-reliability-focus-mode.md](36-reliability-focus-mode.md)               | Reliability Focus Mode + AIS update  |
| 37  | [37-autonomy-calibration-spec.md](37-autonomy-calibration-spec.md)         | Autonomy calibration (AUTHORITATIVE) |

---

## Threads & Concurrency

| #   | File                                                                             | Subsystem                                                           |
| --- | -------------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| 38  | [38-thread-thesis-spec.md](38-thread-thesis-spec.md)                             | Thread thesis (cognitive workspaces)                                |
| 39  | [39-thread-lifecycle-spec.md](39-thread-lifecycle-spec.md)                       | Thread lifecycle                                                    |
| 40  | [40-instance-session-attachment-spec.md](40-instance-session-attachment-spec.md) | Instance/Session/Attachment concurrency                             |
| 41  | [41-proposal-resolution-engine.md](41-proposal-resolution-engine.md)             | Proposal Resolution Engine (PRE)                                    |
| 43  | [43-multi-device-sync.md](43-multi-device-sync.md)                               | Multi-device local-first sync (observations + per-thread ownership) |
| 98  | [98-project-root-crdt-reconciliation-foundation-spec.md](98-project-root-crdt-reconciliation-foundation-spec.md) | Project-root source of truth and CRDT reconciliation foundation |

---

## Gen1 Docs (Not Superseded — Unique Topics)

These docs from the initial spec cover topics that were NOT rewritten in the Gen2 terminology refresh. They remain authoritative for their topics, with UPDATE patches merged in.

| File                                                 | Topic                                            | Notes                                        |
| ---------------------------------------------------- | ------------------------------------------------ | -------------------------------------------- |
| [G1-07-ascc.md](G1-07-ascc.md)                       | Anchored Structured Context Checkpointing (ASCC) | + Pinning & Degradation update               |
| [G1-09-memory.md](G1-09-memory.md)                   | Semantic + Procedural Memory                     | + Trust Model update                         |
| [G1-10-workers.md](G1-10-workers.md)                 | Background Workers & Async Cognition             |                                              |
| [G1-12-api.md](G1-12-api.md)                         | Local HTTP API Specification                     |                                              |
| [G1-13-cli.md](G1-13-cli.md)                         | CLI Contract                                     | Includes export + multi-device sync commands |
| [G1-14-reflection-loop.md](G1-14-reflection-loop.md) | Reflection Loop Overlay                          | Policy-safe meta-cognition loop contract     |
| [G1-16-testing.md](G1-16-testing.md)                 | Testing & Acceptance                             | + New Acceptance Criteria update             |

---

## Implementation Docs

| File                                                 | Description                                                |
| ---------------------------------------------------- | ---------------------------------------------------------- |
| [bootstrap-prompt.md](bootstrap-prompt.md)           | Engineer agent bootstrap prompt                      |
| [bootstrap-prompt-rust.md](bootstrap-prompt-rust.md) | Rust-first engineer agent bootstrap prompt                 |
| [core-reducer.md](core-reducer.md)                   | Focusa-Core Reducer — canonical pseudocode (AUTHORITATIVE) |

---

## Gen1 Implementation Detail Supplements

These Gen1 docs contain **data models, algorithms, schemas, acceptance tests, and implementation specifics** that Gen2 docs intentionally kept high-level. Read Gen2 for concepts, read these for implementation.

| File                                                                         | Topic                            | Key Content Gen2 Lacks                                                                                             |
| ---------------------------------------------------------------------------- | -------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| [G1-detail-00-doc-suite-readme.md](G1-detail-00-doc-suite-readme.md)         | Original doc suite README        | Doc numbering rationale, reading order                                                                             |
| [G1-detail-03-runtime-daemon.md](G1-detail-03-runtime-daemon.md)             | Runtime Daemon detail            | `AppState` struct, process model, event log, persistence rules, startup/recovery, shutdown, reducer                |
| [G1-detail-04-proxy-adapter.md](G1-detail-04-proxy-adapter.md)               | Proxy Adapter detail             | Integration modes (wrap CLI vs HTTP proxy), turn data shapes, daemon endpoints, validation checklist               |
| [G1-detail-05-focus-stack-hec.md](G1-detail-05-focus-stack-hec.md)           | Focus Stack (HEC) detail         | Frame/FrameId/FrameRecord/FrameStats data model, PushFrame/PopFrame operations, persistence, acceptance tests      |
| [G1-detail-06-focus-gate.md](G1-detail-06-focus-gate.md)                     | Focus Gate detail                | 5-step algorithm, Candidate/Signal/CandidateState data model, pressure mechanics, pinning, temporal signals        |
| [G1-detail-08-ecs.md](G1-detail-08-ecs.md)                                   | Externalized Context Store (ECS) | Handle data model (HandleId/HandleKind/HandleRef), StoreArtifact/ResolveHandle ops, session scoping, human pinning |
| [G1-detail-11-prompt-assembly.md](G1-detail-11-prompt-assembly.md)           | Prompt Assembly detail           | 7-slot structure, budget contract, delta injection, handle rehydration, explicit degradation strategy              |
| [G1-detail-15-events-observability.md](G1-detail-15-events-observability.md) | Events & Observability detail    | Complete event type taxonomy (Stack, Gate, ASCC, ECS, Memory, Prompt, Worker, Adapter events), replay invariant    |

---

## PRD Supplements

| File                                                                     | Description                                                   |
| ------------------------------------------------------------------------ | ------------------------------------------------------------- |
| [G1-detail-PRD-gen2-intermediate.md](G1-detail-PRD-gen2-intermediate.md) | Gen2 intermediate PRD snapshot |
| [PRD-delta-threads.md](PRD-delta-threads.md)                             | Thread concept section for PRD                                |
| [PRD-delta-thread-workspaces.md](PRD-delta-thread-workspaces.md)         | Thread as cognitive workspace section                         |

---

## Reading Order

1. **Start here:** [00-glossary.md](00-glossary.md) — defines all canonical terms
2. **Architecture:** [01-architecture-overview.md](01-architecture-overview.md) → [02-runtime-daemon.md](02-runtime-daemon.md)
3. **Core subsystems:** 03 → 04 → 05 → 06 → 07 → 08 (Focus Stack → Gate → Intuition → State → Reference → Expression)
4. **For implementation depth:** Read the `G1-detail-*` counterpart of each Gen2 doc
5. **Memory:** [G1-07-ascc.md](G1-07-ascc.md) + [G1-09-memory.md](G1-09-memory.md) (older detailed specs; compare with current README/evidence for runtime state)
6. **Autonomy:** 12 → 13 → 37
7. **Agent model:** 15 → 16 → 16-constitution-synthesizer
8. **Advanced:** 17 (CLT) → 36 (Reliability) → 38-41 (Threads/Concurrency/Proposals)
9. **Professional workspaces and Project Genesis:** 135 → 135A → 135B → 135C → 135D → 135E
