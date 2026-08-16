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
| current | [current/TAURI_MENUBAR_UP_TO_SPEED_SPEC.md](current/TAURI_MENUBAR_UP_TO_SPEED_SPEC.md) | Current implementation spec for a fully up-to-speed runtime app |
| current | [current/TAURI_MENUBAR_IMPLEMENTATION_GAPS.md](current/TAURI_MENUBAR_IMPLEMENTATION_GAPS.md) | Remaining implementation gaps and first implementation slice for the menubar runtime view |
| current | [current/FOCUSA_BRAIN_BODY_ANALOGY_GAP_MAP.md](current/FOCUSA_BRAIN_BODY_ANALOGY_GAP_MAP.md) | Whole-organism brain/body analogy, maturity gaps, and exhaustive docs cross-reference |
| current | [current/FOCUSA_FEATURE_MATURITY_AUDIT_2026-05-26.md](current/FOCUSA_FEATURE_MATURITY_AUDIT_2026-05-26.md) | Code-based 1–10 feature maturity ratings and underdeveloped workflow gaps |
| current | [current/DATASET_PREDICTION_SUBSTRATE.md](current/DATASET_PREDICTION_SUBSTRATE.md) | Dataset-agnostic prediction substrate, with stocks as the first domain adapter |
| current | [current/PREDICTION_METACOG_SIGNAL_SUBSTRATE.md](current/PREDICTION_METACOG_SIGNAL_SUBSTRATE.md) | Focusa-native prediction/metacognition/ontology flywheel substrate |
| current | [current/PREDICTIVE_METACOG_MATURITY_EVAL_2026-05-26.md](current/PREDICTIVE_METACOG_MATURITY_EVAL_2026-05-26.md) | Current maturity verdict for predictive and metacognitive feature sets |
| current | [current/END_OF_TASK_LEARNING_LOOP.md](current/END_OF_TASK_LEARNING_LOOP.md) | Required prediction/metacog closure loop for compaction cards, trajectory reviews, and final work reports |
| current | [current/PROJECT_INTELLIGENCE_FLYWHEEL.md](current/PROJECT_INTELLIGENCE_FLYWHEEL.md) | Ontology-grounded project-card flywheel for trajectory bootstrap/re-bootstrap, prediction, and metacog compounding |
| current | [current/PREDICTION_ALGORITHMS_IMPLEMENTED.md](current/PREDICTION_ALGORITHMS_IMPLEMENTED.md) | Implemented lightweight prediction formulas behind project-card algorithmic intelligence |
| 137 | [137-focusa-temporal-authority-deadlines-urgency-grounded-forecasting-spec.md](137-focusa-temporal-authority-deadlines-urgency-grounded-forecasting-spec.md) | Temporal authority, deadlines, urgency, uncertainty-aware timing, and grounded time forecasts |
| 138 | [138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md](138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md) | Maximal prediction, outcome, calibration, metacognitive learning, transfer, and epistemic governance substrate |
| 139 | [139-distributed-presence-environment-awareness-execution-placement-and-multi-daemon-coordination-spec.md](139-distributed-presence-environment-awareness-execution-placement-and-multi-daemon-coordination-spec.md) | Distributed presence primacy, environment identity, execution placement, expensive-operation deduplication, and multi-daemon coordination |
| 140 | [140-project-agent-runtime-constitution-instruction-authority-system-prompt-and-cross-harness-compiler-spec.md](140-project-agent-runtime-constitution-instruction-authority-system-prompt-and-cross-harness-compiler-spec.md) | C.R.I.S.T.-derived Project Agent Runtime Constitution, instruction authority graph, Pi system-prompt compiler, AGENTS/rules/skills, enforcement, and cross-harness delivery |
| 156 | [156-focusa-project-scoped-credential-authority-secret-broker-delegated-autonomy-mfa-totp-and-cross-surface-injection-spec.md](156-focusa-project-scoped-credential-authority-secret-broker-delegated-autonomy-mfa-totp-and-cross-surface-injection-spec.md) | Provider-neutral, project-scoped Credential Authority, secret custody/use separation, delegated autonomy, MFA/TOTP, and cross-surface injection |
| contract | [contracts/spec139-complete-feature-ledger.v1.yaml](contracts/spec139-complete-feature-ledger.v1.yaml) | Initial machine-readable Spec 139 implementation and closure ledger |
| contract | [contracts/spec140-complete-feature-ledger.v1.yaml](contracts/spec140-complete-feature-ledger.v1.yaml) | Initial machine-readable Spec 140 implementation and closure ledger |
| evidence | [evidence/spec138-prediction-metacognition-maximal-primitives-audit-2026-07-21.md](evidence/spec138-prediction-metacognition-maximal-primitives-audit-2026-07-21.md) | Code-reality audit and maximal primitive derivation supporting Spec 138 |
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

---

## Spec 135 — Professional Workspaces and C.R.I.S.T. Delivery Contract

Start with the [current authoritative Delivery Contract](135-series-current-manifest.md). It resolves framework, ownership, sequencing, browser-proof, generated-UI, compatibility, and decomposition conflicts across the series.

| Spec | Required subject |
|---|---|
| [135](135-focusa-professional-workspaces-and-crist-project-genesis-master-spec.md) | Master product and closure contract |
| [135A](135a-workspace-projection-pi-sidebar-work-rail-and-vertical-ux-spec.md) | Workspace projection, Mission Canvas, Work Rail, themes, vertical UX |
| [135B](135b-crist-project-genesis-context-role-interview-spec-tasks.md) | Context, Role, Interview, Spec, Tasks |
| [135C](135c-uiai-rich-artifact-live-refresh-and-research-bridge-spec.md) | UIAI artifacts, browser identity, FPV, live refresh |
| [135D](135d-complete-implementation-order-framework-reuse-performance-and-no-deferral-spec.md) | Full implementation graph and no-deferral law |
| [135E](135e-cross-spec-amendments-migration-and-closure-matrix.md) | Migration, compatibility, precedence, closure |
| [135F](135f-domain-general-ontology-core-semantic-graph-domain-packs-and-reactive-context-spec.md) | Ontology core, semantic graph, domain packs |
| [135G](135g-multiplexed-mission-canvas-work-surfaces-session-attachments-and-browser-context-isolation-spec.md) | Multiplexed Work Surfaces and browser isolation |
| [135H](135h-cross-functional-alpha-grill-interview-and-implementation-acceleration-spec.md) | Grill Interview, Cross-Functional Alpha, speed law |
| [135I](135i-real-time-generated-crist-ui-nontechnical-onboarding-and-core-api-integration-spec.md) | Real-time generated nontechnical UI and typed actions |
| [135J](135j-core-api-operation-registry-durable-ui-stream-and-runtime-reuse-hardening-spec.md) | Operation Registry, durable stream, runtime reuse |
| [135K](135k-uxp-ufi-adaptive-generated-ui-friction-learning-and-nontechnical-usability-spec.md) | UXP/UFI adaptation and nontechnical usability proof |

Required implementation inputs:

- [current code-reality and speed audit](current/SPEC135_REALTIME_GENERATED_UI_SPEED_AND_CORE_INTEGRATION_AUDIT_2026-07-18.md)
- [implementation acceleration directive](agent/spec135-implementation-acceleration-directive.md)
- [real-time generated UI directive](agent/spec135-real-time-generated-ui-directive.md)
- [UXP/UFI generated UI directive](agent/spec135-uxp-ufi-generated-ui-directive.md)

Key locked decisions:

```text
UIAI Engine Eval owns browser proof; Playwright is forbidden in Focusa.
A2UI web_core + permanent Lit renderer + Focusa Svelte Custom Elements.
Native durable Focusa stream first; AG-UI is compatibility, not authority.
JSON Schema 2020-12 + OpenAPI 3.0.3 + generated TypeScript clients and portable OpenAPI/JSON Schema contracts.
Pi RPC/Spec 133 owns model execution; Vercel AI SDK runtime is not adopted.
Every feature submits reusable behavior to greater Focusa primitives.
Every requirement remains in the machine-readable closure graph.
```

## 2026-08-15 session additions

- docs/162-remote-workspace-binding-design.md — RemoteWorkspaceBinding (#89)
- docs/163-safe-self-adaptive-compaction-policy-controller-design.md (#112)
- docs/164-workstream-rooted-canonical-runtime-design.md (#125)
- docs/current/CONSOLIDATION_AUDIT_2026-08-15.md (#52)
- docs/current/LICENSING_DIVERGENCE_AUDIT_2026-08-15.md (#119)
- docs/current/CONVERGENCE_STATE_2026-08-15.md (#101)
- docs/current/PROJECT_MARKER_PATHS.md (#243)
- docs/current/BACKGROUND_EXECUTION_AND_COMPLETION_NOTIFICATION.md (#311)
