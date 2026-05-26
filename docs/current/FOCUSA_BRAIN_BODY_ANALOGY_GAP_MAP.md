# Focusa Brain/Body Analogy Gap Map and Documentation Cross-Reference

Generated/updated: 2026-05-26

Purpose: map Focusa’s software functions to a brain/body/organism analogy, identify completeness gaps, and provide a single cross-reference over the documentation corpus.

> Status note: Focusa is an active development snapshot. Treat `docs/current/*`, live proof docs, and tool docs as current runtime references; older numbered specs preserve design intent unless reconciled by current docs or evidence.

## Short answer: what completes the analogy?

Focusa already has major cognitive organs: conscious state, subconscious/background cognition, decision matrix, reflexes, memory, lineage, and metacognition. The analogy becomes more complete when Focusa is treated as a **whole adaptive organism**, not only a mind.

The missing completeness layer is the integrated loop: **senses → attention → state → goals → action → proof → learning → homeostasis → social coordination → development**.

## Brain/body system map

| Organism function | Focusa analogue now | Primary docs | Gap / next maturity frontier |
| --- | --- | --- | --- |
| Conscious attention / working mind | Focus State, Focus Stack, current ask/scope, model-visible awareness | [06-focus-state](../06-focus-state.md), [03-focus-stack](../03-focus-stack.md), [68-current-ask-and-scope-integration](../68-current-ask-and-scope-integration.md), [FOCUSA_MODEL_VISIBLE_AWARENESS](./FOCUSA_MODEL_VISIBLE_AWARENESS.md) | Sharper precedence rules when multiple current-state sources conflict. |
| Pre-conscious salience / RAS | Focus Gate, scope/relevance control, active object resolution | [04-focus-gate](../04-focus-gate.md), [G1-detail-06-focus-gate](../G1-detail-06-focus-gate.md), [67-query-scope-and-relevance-control](../67-query-scope-and-relevance-control.md), [74-identity-and-reference-resolution](../74-identity-and-reference-resolution.md) | More measured salience scoring from real tool outcomes. |
| Subconscious / background cognition | Intuition Engine, bounded secondary cognition, work-loop, silent sessions | [05-intuition-engine](../05-intuition-engine.md), [78-bounded-secondary-cognition-and-persistent-autonomy](../78-bounded-secondary-cognition-and-persistent-autonomy.md), [79-focusa-governed-continuous-work-loop](../79-focusa-governed-continuous-work-loop.md), [DOC78_SECONDARY_COGNITION_CALLSITE_AUDIT](../DOC78_SECONDARY_COGNITION_CALLSITE_AUDIT_2026-04-13.md) | Broader structured subconscious prompt programs and bounded autonomy proofs. |
| Reflex arc | Spec97 reflex primitives, tool-result recovery hints, no-deadend envelopes | [97-focusa-reflex-primitives-spec](../97-focusa-reflex-primitives-spec.md), [TOOL_RESULT_ENVELOPE_V1](./TOOL_RESULT_ENVELOPE_V1.md), [TROUBLESHOOTING_CURRENT](./TROUBLESHOOTING_CURRENT.md) | Expand from recovery reflexes into positive execution reflexes while staying advisory. |
| Executive function / prefrontal cortex | Trajectory, Workpoint, decisions, operator priority, tool choreography | [TRAJECTORY_GTM_AND_GAPS](./TRAJECTORY_GTM_AND_GAPS.md), [WORKPOINT_LIFECYCLE_GUIDE](./WORKPOINT_LIFECYCLE_GUIDE.md), [54a-operator-priority-and-subject-preservation](../54a-operator-priority-and-subject-preservation.md), [FOCUSA_TOOL_CHOREOGRAPHY_MAP](./FOCUSA_TOOL_CHOREOGRAPHY_MAP.md) | Explicit arbitration among trajectory, operator ask, Workpoint, and work-loop. |
| Episodic memory / hippocampus | Context Lineage Tree, snapshots, Workpoint checkpoints, evidence refs | [17-context-lineage-tree](../17-context-lineage-tree.md), [56-trace-checkpoints-recovery](../56-trace-checkpoints-recovery.md), [WORKPOINT_SESSION_SCOPE_GUARD](./WORKPOINT_SESSION_SCOPE_GUARD.md), [focusa-tools/tree-lineage](../focusa-tools/tree-lineage.md) | More automatic event segmentation and episode summarization. |
| Semantic/procedural memory | Reference Store, ECS, ontology, skills, tool contracts | [07-reference-store](../07-reference-store.md), [G1-detail-08-ecs](../G1-detail-08-ecs.md), [45-ontology-overview](../45-ontology-overview.md), [34-agent-skills-spec](../34-agent-skills-spec.md), [FOCUSA_TOOL_CONTRACT_REGISTRY](./FOCUSA_TOOL_CONTRACT_REGISTRY.md) | Tighter bridge between docs/specs and runtime ontology retrieval. |
| Motor system / action body | CLI/API/Pi extension/tool surface/action contracts | [API_REFERENCE_CURRENT](./API_REFERENCE_CURRENT.md), [CLI_REFERENCE_CURRENT](./CLI_REFERENCE_CURRENT.md), [44-pi-focusa-integration-spec](../44-pi-focusa-integration-spec.md), [55-tool-action-contracts](../55-tool-action-contracts.md) | More end-to-end actuator proof across non-Pi agents and UI. |
| Sensory system | Telemetry, observability, active object traversal, non-Pi awareness | [29-telemetry-spec](../29-telemetry-spec.md), [30-telemetry-schema](../30-telemetry-schema.md), [31-telemetry-api](../31-telemetry-api.md), [NON_PI_AGENT_FOCUSA_USAGE](./NON_PI_AGENT_FOCUSA_USAGE.md) | First-class sensor taxonomy: filesystem, process, git, daemon, UI, user, CI, web. |
| Homeostasis / autonomic regulation | ResourceMode, LowMem, daemon resilience, efficiency telemetry | [EFFICIENCY_GUIDE](./EFFICIENCY_GUIDE.md), [DAEMON_RESILIENCE](./DAEMON_RESILIENCE.md), [RUNTIME_CONFIG_KEYS](./RUNTIME_CONFIG_KEYS.md), [82-focusa-memory-optimization-spec](../82-focusa-memory-optimization-spec.md) | Global “physiology” model for token, memory, latency, daemon, and attention pressure. |
| Immune system / error containment | Doctor, hygiene, troubleshooting, safety envelopes, validation proof | [TOOL_RELIABILITY_AUDIT](./TOOL_RELIABILITY_AUDIT.md), [TROUBLESHOOTING_CURRENT](./TROUBLESHOOTING_CURRENT.md), [ERROR_EMPTY_STATES](./ERROR_EMPTY_STATES.md), [VALIDATION_AND_RELEASE_PROOF](./VALIDATION_AND_RELEASE_PROOF.md) | Threat/contamination model for stale state, bad evidence, and tool misuse. |
| Learning / plasticity | Metacognition, predictive power, evals, calibration | [focusa-tools/metacognition](../focusa-tools/metacognition.md), [PREDICTIVE_POWER_GUIDE](./PREDICTIVE_POWER_GUIDE.md), [80-pi-tree-li-metacognition-tooling-spec](../80-pi-tree-li-metacognition-tooling-spec.md), [57-golden-tasks-and-evals](../57-golden-tasks-and-evals.md) | Learning promotion policy tied to measured outcome deltas, not narrative confidence. |
| Self-model / identity | Agent schema, constitution, project identity, session scope guard | [15-agent-schema](../15-agent-schema.md), [16-agent-constitution](../16-agent-constitution.md), [72-agent-identity-role-and-self-model-ontology](../72-agent-identity-role-and-self-model-ontology.md), [focusa-tools/project-identity](../focusa-tools/project-identity.md) | Clear separation between model persona, operator identity, project identity, and Workpoint identity. |
| Social cognition / communication | Operator preview, agent awareness quickstart, non-Pi usage, command cookbook | [FOCUSA_OPERATOR_PREVIEW_PROOF](./FOCUSA_OPERATOR_PREVIEW_PROOF.md), [AGENT_AWARENESS_QUICKSTART](./AGENT_AWARENESS_QUICKSTART.md), [AGENT_COMMAND_COOKBOOK](./AGENT_COMMAND_COOKBOOK.md), [93-non-pi-agent-focusa-awareness-spec](../93-non-pi-agent-focusa-awareness-spec.md) | Richer multi-agent handoff and shared-world coordination protocols. |
| Development / growth | Spec ladder, decomposition, release proof, dogfood loop | [FOCUSA_DOGFOOD](./FOCUSA_DOGFOOD.md), [89-focusa-tool-suite-improvement-hardening-spec](../89-focusa-tool-suite-improvement-hardening-spec.md), [91-live-tool-contract-proof-harness-spec](../91-live-tool-contract-proof-harness-spec.md), [LIVE_TOOL_CONTRACT_PROOF](./LIVE_TOOL_CONTRACT_PROOF.md) | A living “organism maturity score” driven by proof, gaps, and user outcomes. |
| Sleep / consolidation | Compaction fallback, checkpointing, evidence capture, summaries | [COMPACTION_FALLBACKS](./COMPACTION_FALLBACKS.md), [G1-07-ascc](../G1-07-ascc.md), [56-trace-checkpoints-recovery](../56-trace-checkpoints-recovery.md), [WORKPOINT_LIFECYCLE_GUIDE](./WORKPOINT_LIFECYCLE_GUIDE.md) | Scheduled consolidation cycles that prune, summarize, and promote durable knowledge. |
| Mood/endocrine/global modulation | Autonomy scoring, governing priors, scalar weights, reliability focus mode | [12-autonomy-scoring](../12-autonomy-scoring.md), [36-reliability-focus-mode](../36-reliability-focus-mode.md), [37-autonomy-calibration-spec](../37-autonomy-calibration-spec.md), [71-governing-priors-and-scalar-weights](../71-governing-priors-and-scalar-weights.md) | Global operating posture: cautious, exploratory, release, recovery, low-resource, high-proof. |

## Cross-reference reading routes

- **Current runtime truth first:** [CURRENT_RUNTIME_STATUS](./CURRENT_RUNTIME_STATUS.md) → [API_REFERENCE_CURRENT](./API_REFERENCE_CURRENT.md) → [CLI_REFERENCE_CURRENT](./CLI_REFERENCE_CURRENT.md) → [VALIDATION_AND_RELEASE_PROOF](./VALIDATION_AND_RELEASE_PROOF.md).
- **Agent work loop:** [AGENT_AWARENESS_QUICKSTART](./AGENT_AWARENESS_QUICKSTART.md) → [FOCUSA_AGENT_UTILITY_CARD](./FOCUSA_AGENT_UTILITY_CARD.md) → [FOCUSA_TOOL_CHOREOGRAPHY_MAP](./FOCUSA_TOOL_CHOREOGRAPHY_MAP.md) → [WORKPOINT_LIFECYCLE_GUIDE](./WORKPOINT_LIFECYCLE_GUIDE.md).
- **Recovery route:** [TOOL_RESULT_ENVELOPE_V1](./TOOL_RESULT_ENVELOPE_V1.md) → [TROUBLESHOOTING_CURRENT](./TROUBLESHOOTING_CURRENT.md) → [ERROR_EMPTY_STATES](./ERROR_EMPTY_STATES.md) → [DAEMON_RESILIENCE](./DAEMON_RESILIENCE.md).
- **Learning route:** [PREDICTIVE_POWER_GUIDE](./PREDICTIVE_POWER_GUIDE.md) → [focusa-tools/metacognition](../focusa-tools/metacognition.md) → [80-pi-tree-li-metacognition-tooling-spec](../80-pi-tree-li-metacognition-tooling-spec.md) → [57-golden-tasks-and-evals](../57-golden-tasks-and-evals.md).
- **Organism/spec route:** [01-architecture-overview](../01-architecture-overview.md) → [UNIFIED_ORGANISM_SPEC](../UNIFIED_ORGANISM_SPEC.md) → [INTEGRATION_SPEC](../INTEGRATION_SPEC.md) → [61-domain-general-cognition-core](../61-domain-general-cognition-core.md) → [78-bounded-secondary-cognition-and-persistent-autonomy](../78-bounded-secondary-cognition-and-persistent-autonomy.md).

## Gap backlog from the organism analogy

| Gap | Why it matters | Candidate doc/spec home |
| --- | --- | --- |
| Sensor taxonomy | Completes the input side of the organism: user, files, git, CI, daemon, UI, web, MCP, resource signals. | Extend telemetry/ontology docs: [29](../29-telemetry-spec.md), [45](../45-ontology-overview.md), [66](../66-affordance-and-execution-environment-ontology.md). |
| Actuator taxonomy | Separates observation from action and clarifies safe motor permissions. | Extend action/capability docs: [23](../23-capabilities-api.md), [25](../25-capability-permissions.md), [55](../55-tool-action-contracts.md). |
| Homeostasis model | Unifies token, latency, daemon health, memory, and attention pressure. | Extend [EFFICIENCY_GUIDE](./EFFICIENCY_GUIDE.md), [DAEMON_RESILIENCE](./DAEMON_RESILIENCE.md), [82](../82-focusa-memory-optimization-spec.md). |
| Sleep/consolidation cycle | Turns compaction from emergency recovery into planned memory maintenance. | Extend [COMPACTION_FALLBACKS](./COMPACTION_FALLBACKS.md), [G1-07-ascc](../G1-07-ascc.md), [76](../76-retention-forgetting-and-decay-policy.md). |
| Immune/stale-state model | Makes bad state, stale evidence, wrong project scope, and unsafe mutation first-class contaminants. | Extend [TROUBLESHOOTING_CURRENT](./TROUBLESHOOTING_CURRENT.md), [WORKPOINT_SESSION_SCOPE_GUARD](./WORKPOINT_SESSION_SCOPE_GUARD.md), [TOOL_RELIABILITY_AUDIT](./TOOL_RELIABILITY_AUDIT.md). |
| Social/multi-agent coordination | Completes organism-in-environment: agents, operator, Discord/Mac/Pi/non-Pi handoffs. | Extend [NON_PI_AGENT_FOCUSA_USAGE](./NON_PI_AGENT_FOCUSA_USAGE.md), [93](../93-non-pi-agent-focusa-awareness-spec.md), [43](../43-multi-device-sync.md). |
| Developmental maturity score | Connects dogfood, proof, prediction accuracy, and gap closure into one maturity model. | Extend [FOCUSA_DOGFOOD](./FOCUSA_DOGFOOD.md), [LIVE_TOOL_CONTRACT_PROOF](./LIVE_TOOL_CONTRACT_PROOF.md), [PREDICTIVE_POWER_GUIDE](./PREDICTIVE_POWER_GUIDE.md). |

## Documentation corpus cross-reference

This section references every Markdown file under `docs/` found at generation time: **438 docs including this file**.

### This map

| Doc | Role |
| --- | --- |
| [docs/current/FOCUSA_BRAIN_BODY_ANALOGY_GAP_MAP.md](./FOCUSA_BRAIN_BODY_ANALOGY_GAP_MAP.md) | Whole-organism analogy, gap map, and docs cross-reference. |

### Top-level product and navigation docs

| Doc | Title / role |
| --- | --- |
| [docs/AGENTS.md](../AGENTS.md) | AGENTS.md — Focusa Local Agent Protocol (Beads-Centered) |
| [docs/INDEX.md](../INDEX.md) | Focusa — Current Documentation Snapshot |
| [docs/PRD-delta-thread-workspaces.md](../PRD-delta-thread-workspaces.md) | Threads (Cognitive Workspaces) |
| [docs/PRD-delta-threads.md](../PRD-delta-threads.md) | Core Concept: Threads |
| [docs/PRD.md](../PRD.md) | Focusa — Product Requirements Document (PRD) |
| [docs/README.md](../README.md) | Focusa Docs |
| [docs/fixtures/api/README.md](../fixtures/api/README.md) | Focusa API fixture payloads |

### Current runtime docs

| Doc | Title / role |
| --- | --- |
| [docs/current/AGENT_AWARENESS_QUICKSTART.md](./AGENT_AWARENESS_QUICKSTART.md) | Agent Awareness Quickstart |
| [docs/current/AGENT_COMMAND_COOKBOOK.md](./AGENT_COMMAND_COOKBOOK.md) | Agent Command Cookbook |
| [docs/current/API_REFERENCE_CURRENT.md](./API_REFERENCE_CURRENT.md) | Current API Route Inventory |
| [docs/current/CLI_REFERENCE_CURRENT.md](./CLI_REFERENCE_CURRENT.md) | Current CLI Reference |
| [docs/current/COMPACTION_FALLBACKS.md](./COMPACTION_FALLBACKS.md) | Compaction Fallbacks |
| [docs/current/CURRENT_RUNTIME_STATUS.md](./CURRENT_RUNTIME_STATUS.md) | Current Runtime Status |
| [docs/current/DAEMON_RESILIENCE.md](./DAEMON_RESILIENCE.md) | Daemon Resilience and In-Session Kickstart |
| [docs/current/DOCTOR_CONTINUE_RELEASE_PROVE.md](./DOCTOR_CONTINUE_RELEASE_PROVE.md) | Doctor, Continue, and Release Proof Commands |
| [docs/current/EFFICIENCY_GUIDE.md](./EFFICIENCY_GUIDE.md) | Efficiency Guide |
| [docs/current/ERROR_EMPTY_STATES.md](./ERROR_EMPTY_STATES.md) | Error and Empty-State Envelopes |
| [docs/current/FOCUSA_AGENT_UTILITY_CARD.md](./FOCUSA_AGENT_UTILITY_CARD.md) | Focusa Agent Utility Card |
| [docs/current/FOCUSA_DOGFOOD.md](./FOCUSA_DOGFOOD.md) | Focusa Native Dogfood |
| [docs/current/FOCUSA_FRIENDLY_ONBOARDING.md](./FOCUSA_FRIENDLY_ONBOARDING.md) | Friendly Focusa Onboarding Q |
| [docs/current/FOCUSA_MODEL_VISIBLE_AWARENESS.md](./FOCUSA_MODEL_VISIBLE_AWARENESS.md) | Focusa Model-Visible Awareness Surfaces |
| [docs/current/FOCUSA_OPERATOR_PREVIEW_PROOF.md](./FOCUSA_OPERATOR_PREVIEW_PROOF.md) | Focusa Operator Preview Proof |
| [docs/current/FOCUSA_TOOL_CHOREOGRAPHY_MAP.md](./FOCUSA_TOOL_CHOREOGRAPHY_MAP.md) | Focusa Tool Choreography Map |
| [docs/current/FOCUSA_TOOL_CONTRACT_REGISTRY.md](./FOCUSA_TOOL_CONTRACT_REGISTRY.md) | Focusa Tool Contract Registry |
| [docs/current/FOCUSA_TOOL_IMPLEMENTATION_SPEC_AUDIT.md](./FOCUSA_TOOL_IMPLEMENTATION_SPEC_AUDIT.md) | Focusa Tool Implementation-to-Spec Audit |
| [docs/current/HOOK_COVERAGE.md](./HOOK_COVERAGE.md) | Focusa Hook Coverage |
| [docs/current/LIVE_TOOL_CONTRACT_PROOF.md](./LIVE_TOOL_CONTRACT_PROOF.md) | Live Tool Contract Proof |
| [docs/current/MAC_APP_MISSION_CONTROL.md](./MAC_APP_MISSION_CONTROL.md) | Mac App Mission Control |
| [docs/current/NON_PI_AGENT_FOCUSA_USAGE.md](./NON_PI_AGENT_FOCUSA_USAGE.md) | Non-Pi Agent Focusa Usage |
| [docs/current/PI_EXTENSION_AND_SKILLS_GUIDE.md](./PI_EXTENSION_AND_SKILLS_GUIDE.md) | Pi Extension and Skills Guide |
| [docs/current/PORTABILITY_AUDIT.md](./PORTABILITY_AUDIT.md) | Focusa Portability Audit — External Tester Readiness |
| [docs/current/PREDICTIVE_POWER_GUIDE.md](./PREDICTIVE_POWER_GUIDE.md) | Predictive Power Guide |
| [docs/current/PRODUCTION_RELEASE_COMMANDS.md](./PRODUCTION_RELEASE_COMMANDS.md) | Production Release Commands |
| [docs/current/RUNTIME_CONFIG_KEYS.md](./RUNTIME_CONFIG_KEYS.md) | Runtime Configuration Keys |
| [docs/current/TAURI_MENUBAR_FUNCTIONALITY_AUDIT.md](./TAURI_MENUBAR_FUNCTIONALITY_AUDIT.md) | Tauri Menubar Functionality Audit |
| [docs/current/TAURI_MENUBAR_IMPLEMENTATION_GAPS.md](./TAURI_MENUBAR_IMPLEMENTATION_GAPS.md) | Tauri Menubar Implementation Gaps |
| [docs/current/TAURI_MENUBAR_UP_TO_SPEED_SPEC.md](./TAURI_MENUBAR_UP_TO_SPEED_SPEC.md) | Tauri Menubar Up-to-Speed Spec |
| [docs/current/TOOL_RELIABILITY_AUDIT.md](./TOOL_RELIABILITY_AUDIT.md) | Focusa Tool Reliability Audit |
| [docs/current/TOOL_RESULT_ENVELOPE_V1.md](./TOOL_RESULT_ENVELOPE_V1.md) | Tool Result Envelope v1 |
| [docs/current/TRAJECTORY_GTM_AND_GAPS.md](./TRAJECTORY_GTM_AND_GAPS.md) | Focusa Trajectory GTM and Companion Gap Assessment |
| [docs/current/TROUBLESHOOTING_CURRENT.md](./TROUBLESHOOTING_CURRENT.md) | Current Troubleshooting Guide |
| [docs/current/VALIDATION_AND_RELEASE_PROOF.md](./VALIDATION_AND_RELEASE_PROOF.md) | Validation and Release Proof |
| [docs/current/WORKPOINT_LIFECYCLE_GUIDE.md](./WORKPOINT_LIFECYCLE_GUIDE.md) | Workpoint Lifecycle Guide |
| [docs/current/WORKPOINT_SESSION_SCOPE_GUARD.md](./WORKPOINT_SESSION_SCOPE_GUARD.md) | Workpoint Project Folder + Continuity Guard |

### Core architecture specs

| Doc | Title / role |
| --- | --- |
| [docs/00-glossary.md](../00-glossary.md) | docs/00-glossary.md — Focusa Canonical Glossary (LOCKED) |
| [docs/01-architecture-overview.md](../01-architecture-overview.md) | docs/01-architecture-overview.md — MVP Architecture Overview |
| [docs/02-runtime-daemon.md](../02-runtime-daemon.md) | docs/02-runtime-daemon.md — Focusa Runtime & Daemon (MVP) |
| [docs/03-focus-stack.md](../03-focus-stack.md) | docs/03-focus-stack.md — Focus Stack & Focus Frames (MVP) |
| [docs/04-focus-gate.md](../04-focus-gate.md) | docs/04-focus-gate.md — Focus Gate (MVP) |
| [docs/05-intuition-engine.md](../05-intuition-engine.md) | docs/05-intuition-engine.md — Intuition Engine (MVP) |
| [docs/06-focus-state.md](../06-focus-state.md) | docs/06-focus-state.md — Focus State (MVP) |
| [docs/07-reference-store.md](../07-reference-store.md) | docs/07-reference-store.md — Reference Store (MVP) |
| [docs/08-expression-engine.md](../08-expression-engine.md) | docs/08-expression-engine.md — Expression Engine (MVP) |
| [docs/09-proxy-adapter.md](../09-proxy-adapter.md) | docs/09-proxy-adapter.md — Proxy & Harness Adapters (MVP) |
| [docs/10-monorepo-layout.md](../10-monorepo-layout.md) | docs/10-monorepo-layout.md — Focusa Monorepo Layout (MVP) |
| [docs/11-menubar-ui-spec.md](../11-menubar-ui-spec.md) | docs/11-menubar-ui-spec.md — Focusa Menubar UI (MVP) |

### Autonomy and governance specs

| Doc | Title / role |
| --- | --- |
| [docs/12-autonomy-scoring.md](../12-autonomy-scoring.md) | docs/12-autonomy-scoring.md — Autonomy Scoring & Earned Capability (MVP+) |
| [docs/13-autonomy-ui.md](../13-autonomy-ui.md) | docs/13-autonomy-ui.md — Autonomy Visualization (CLI + Menubar) |
| [docs/14-uxp-ufi-schema.md](../14-uxp-ufi-schema.md) | docs/14-uxp-ufi-schema.md — User Experience Calibration (AUTHORITATIVE) |
| [docs/15-agent-schema.md](../15-agent-schema.md) | docs/15-agent-schema.md — Agent Definition (UPDATED, AUTHORITATIVE) |
| [docs/16-agent-constitution.md](../16-agent-constitution.md) | docs/16-agent-constitution.md — Agent Constitution (AUTHORITATIVE) |
| [docs/16-constitution-synthesizer.md](../16-constitution-synthesizer.md) | docs/16-constitution-synthesizer.md — Constitution Synthesizer (AUTHORITATIVE) |

### Provenance, cache, data specs

| Doc | Title / role |
| --- | --- |
| [docs/17-context-lineage-tree.md](../17-context-lineage-tree.md) | docs/17-context-lineage-tree.md — Context Lineage Tree (CLT) Specification (AUTHORITATIVE) |
| [docs/18-cache-permission-matrix.md](../18-cache-permission-matrix.md) | docs/18-cache-permission-matrix.md — Cache Permission Matrix (AUTHORITATIVE) |
| [docs/19-intentional-cache-busting.md](../19-intentional-cache-busting.md) | docs/19-intentional-cache-busting.md — Intentional Cache Busting Triggers (AUTHORITATIVE) |
| [docs/20-training-dataset-schema.md](../20-training-dataset-schema.md) | docs/20-training-dataset-schema.md — Focusa Training Dataset Schema (AUTHORITATIVE) |
| [docs/21-data-export-cli.md](../21-data-export-cli.md) | docs/21-data-export-cli.md — Focusa Data Export CLI Specification (AUTHORITATIVE) |
| [docs/22-data-contribution.md](../22-data-contribution.md) | docs/22-data-contribution.md — Opt-In Background Data Contribution (AUTHORITATIVE) |

### Capabilities, telemetry, skills specs

| Doc | Title / role |
| --- | --- |
| [docs/23-capabilities-api.md](../23-capabilities-api.md) | docs/23-capabilities-api.md — Focusa Capabilities API (AUTHORITATIVE) |
| [docs/24-capabilities-cli.md](../24-capabilities-cli.md) | docs/24-capabilities-cli.md — Focusa Capabilities CLI Specification (AUTHORITATIVE) |
| [docs/25-capability-permissions.md](../25-capability-permissions.md) | docs/25-capability-permissions.md — Capability Permissions Model (AUTHORITATIVE) |
| [docs/26-agent-capability-scope.md](../26-agent-capability-scope.md) | docs/26-agent-capability-scope.md — Agent Capability Scope Model (AUTHORITATIVE) |
| [docs/27-tui-spec.md](../27-tui-spec.md) | docs/27-tui-spec.md — Focusa TUI Specification (ratatui) (AUTHORITATIVE) |
| [docs/28-ratatui-component-tree.md](../28-ratatui-component-tree.md) | docs/28-ratatui-component-tree.md — Focusa TUI Component Tree (AUTHORITATIVE) |
| [docs/29-telemetry-spec.md](../29-telemetry-spec.md) | docs/29-telemetry-spec.md — Cognitive Telemetry Layer (CTL) Specification (AUTHORITATIVE) |
| [docs/30-telemetry-schema.md](../30-telemetry-schema.md) | docs/30-telemetry-schema.md — Telemetry Event Schema (AUTHORITATIVE) |
| [docs/31-telemetry-api.md](../31-telemetry-api.md) | docs/31-telemetry-api.md — Telemetry Capabilities API (AUTHORITATIVE) |
| [docs/32-telemetry-tui.md](../32-telemetry-tui.md) | docs/32-telemetry-tui.md — Telemetry TUI Integration (AUTHORITATIVE) |
| [docs/33-acp-proxy-spec.md](../33-acp-proxy-spec.md) | docs/33-acp-proxy-spec.md — ACP Proxy & Observation Integration (AUTHORITATIVE) |
| [docs/34-agent-skills-spec.md](../34-agent-skills-spec.md) | docs/34-agent-skills-spec.md — Focusa Agent Skill Bundle (AUTHORITATIVE) |
| [docs/35-skill-to-capabilities-mapping.md](../35-skill-to-capabilities-mapping.md) | docs/35-skill-to-capabilities-mapping.md — Agent Skills → Capabilities API (AUTHORITATIVE) |
| [docs/36-reliability-focus-mode.md](../36-reliability-focus-mode.md) | docs/36-reliability-focus-mode.md |
| [docs/37-autonomy-calibration-spec.md](../37-autonomy-calibration-spec.md) | docs/37-autonomy-calibration-spec.md |

### Threads, concurrency, Pi integration specs

| Doc | Title / role |
| --- | --- |
| [docs/38-thread-thesis-spec.md](../38-thread-thesis-spec.md) | docs/38-thread-thesis-spec.md |
| [docs/39-thread-lifecycle-spec.md](../39-thread-lifecycle-spec.md) | docs/39-thread-lifecycle-spec.md |
| [docs/40-instance-session-attachment-spec.md](../40-instance-session-attachment-spec.md) | docs/40-instance-session-attachment-spec.md |
| [docs/41-proposal-resolution-engine.md](../41-proposal-resolution-engine.md) | docs/41-proposal-resolution-engine.md |
| [docs/42-magic-harness-shims.md](../42-magic-harness-shims.md) | docs/42-magic-harness-shims.md — Magic Harness Shims (Desired UX) |
| [docs/42-menubar-ux-improvements.md](../42-menubar-ux-improvements.md) | docs/42-menubar-ux-improvements.md — Menubar UX Improvement Spec |
| [docs/43-multi-device-sync.md](../43-multi-device-sync.md) | docs/43-multi-device-sync.md — Multi-Device Local-First Sync (AUTHORITATIVE) |
| [docs/44-pi-focusa-integration-spec.md](../44-pi-focusa-integration-spec.md) | 44 — Pi × Focusa Integration Spec (Proxy-First, Extension-Thin) |

### Ontology and cognition specs

| Doc | Title / role |
| --- | --- |
| [docs/45-ontology-overview.md](../45-ontology-overview.md) | Focusa Ontology Overview |
| [docs/46-ontology-core-primitives.md](../46-ontology-core-primitives.md) | Ontology Core Primitives |
| [docs/47-ontology-software-world.md](../47-ontology-software-world.md) | Ontology Software World |
| [docs/48-ontology-links-actions.md](../48-ontology-links-actions.md) | Ontology Links and Actions |
| [docs/49-working-sets-and-slices.md](../49-working-sets-and-slices.md) | Working Sets and Slices |
| [docs/50-ontology-classification-and-reducer.md](../50-ontology-classification-and-reducer.md) | Ontology Classification and Reducer Integration |
| [docs/51-ontology-expression-and-proxy.md](../51-ontology-expression-and-proxy.md) | Ontology Expression and Proxy Integration |
| [docs/52-pi-extension-contract.md](../52-pi-extension-contract.md) | Pi Extension Contract |
| [docs/53-pi-behavioral-alignment-contract.md](../53-pi-behavioral-alignment-contract.md) | Pi Behavioral Alignment Contract |
| [docs/54-pi-visible-output-boundary.md](../54-pi-visible-output-boundary.md) | Pi Visible Output Boundary |
| [docs/54a-operator-priority-and-subject-preservation.md](../54a-operator-priority-and-subject-preservation.md) | Operator Priority and Subject Preservation |
| [docs/54b-context-injection-and-attention-routing.md](../54b-context-injection-and-attention-routing.md) | Context Injection and Attention Routing |
| [docs/55-tool-action-contracts-impl.md](../55-tool-action-contracts-impl.md) | Tool and Action Contracts — Implementation Notes |
| [docs/55-tool-action-contracts.md](../55-tool-action-contracts.md) | Tool and Action Contracts |
| [docs/56-trace-checkpoints-recovery.md](../56-trace-checkpoints-recovery.md) | Trace, Checkpoints, and Recovery |
| [docs/57-golden-tasks-and-evals.md](../57-golden-tasks-and-evals.md) | Golden Tasks and Evals |
| [docs/58-visual-ui-ontology-core.md](../58-visual-ui-ontology-core.md) | Visual/UI Ontology Core |
| [docs/59-visual-ui-reverse-engineering.md](../59-visual-ui-reverse-engineering.md) | Visual/UI Reverse Engineering |
| [docs/60-visual-ui-verification-and-critique.md](../60-visual-ui-verification-and-critique.md) | Visual/UI Verification and Critique |
| [docs/61-domain-general-cognition-core.md](../61-domain-general-cognition-core.md) | Domain-General Cognition Core |
| [docs/62-visual-ui-evidence-and-workflow.md](../62-visual-ui-evidence-and-workflow.md) | Visual/UI Evidence and Workflow |
| [docs/63-visual-ui-invention-and-variation.md](../63-visual-ui-invention-and-variation.md) | Visual/UI Invention and Variation |
| [docs/64-visual-ui-to-implementation.md](../64-visual-ui-to-implementation.md) | Visual/UI to Implementation |
| [docs/65-visual-ui-focusa-integration.md](../65-visual-ui-focusa-integration.md) | Visual/UI Integration with Focusa |
| [docs/66-affordance-and-execution-environment-ontology.md](../66-affordance-and-execution-environment-ontology.md) | Affordance and Execution-Environment Ontology |
| [docs/67-query-scope-and-relevance-control.md](../67-query-scope-and-relevance-control.md) | Query Scope and Relevance Control |
| [docs/68-current-ask-and-scope-integration.md](../68-current-ask-and-scope-integration.md) | Current Ask and Scope Integration |
| [docs/69-scope-failure-and-relevance-tracing.md](../69-scope-failure-and-relevance-tracing.md) | Scope Failure and Relevance Tracing |
| [docs/70-shared-interfaces-statuses-and-lifecycle.md](../70-shared-interfaces-statuses-and-lifecycle.md) | Shared Interfaces, Statuses, and Lifecycle |
| [docs/71-governing-priors-and-scalar-weights.md](../71-governing-priors-and-scalar-weights.md) | Governing Priors and Scalar Weights |
| [docs/72-agent-identity-role-and-self-model-ontology.md](../72-agent-identity-role-and-self-model-ontology.md) | Agent Identity, Role, and Self-Model Ontology |
| [docs/73-intention-commitment-and-self-regulation.md](../73-intention-commitment-and-self-regulation.md) | Intention, Commitment, and Self-Regulation |
| [docs/74-identity-and-reference-resolution.md](../74-identity-and-reference-resolution.md) | Identity and Reference Resolution |
| [docs/75-projection-and-view-semantics.md](../75-projection-and-view-semantics.md) | Projection and View Semantics |
| [docs/76-retention-forgetting-and-decay-policy.md](../76-retention-forgetting-and-decay-policy.md) | Retention, Forgetting, and Decay Policy |
| [docs/77-ontology-governance-versioning-and-migration.md](../77-ontology-governance-versioning-and-migration.md) | Ontology Governance, Versioning, and Migration |
| [docs/78-bounded-secondary-cognition-and-persistent-autonomy.md](../78-bounded-secondary-cognition-and-persistent-autonomy.md) | Bounded Secondary Cognition and Persistent Autonomy |

### Implementation and hardening specs

| Doc | Title / role |
| --- | --- |
| [docs/79-focusa-governed-continuous-work-loop.md](../79-focusa-governed-continuous-work-loop.md) | 79 — Focusa-Governed Continuous Work Loop (FGCWL) |
| [docs/79_CONTINUOUS_WORK_LOOP_BD_DECOMPOSITION_2026-04-13.md](../79_CONTINUOUS_WORK_LOOP_BD_DECOMPOSITION_2026-04-13.md) | 79 Continuous Work Loop — Full BD Decomposition |
| [docs/80-pi-tree-li-metacognition-tooling-spec.md](../80-pi-tree-li-metacognition-tooling-spec.md) | 80 — Pi `/tree` × Focusa LI Metacognition Tooling Spec (Sharpened) |
| [docs/81-focusa-llm-tool-suite-and-cli-development-reset-spec.md](../81-focusa-llm-tool-suite-and-cli-development-reset-spec.md) | 81 — Focusa LLM Tool Suite + CLI Development Reset Spec |
| [docs/82-focusa-memory-optimization-spec.md](../82-focusa-memory-optimization-spec.md) | 82 — Focusa Memory Optimization Spec |
| [docs/83-pi-focusa-rpc-efficiency-spec.md](../83-pi-focusa-rpc-efficiency-spec.md) | 83 — Pi × Focusa RPC Efficiency Spec (A/B/C) |
| [docs/84-action-type-parity-spec.md](../84-action-type-parity-spec.md) | 84 — Action-Type Parity Closure Spec |
| [docs/85-relation-type-parity-spec.md](../85-relation-type-parity-spec.md) | 85 — Relation-Type Parity Closure Spec |
| [docs/86-shared-status-lifecycle-parity-spec.md](../86-shared-status-lifecycle-parity-spec.md) | 86 — Shared-Status Lifecycle Parity Closure Spec |
| [docs/87-focusa-first-class-tool-desirability-and-pickup-spec.md](../87-focusa-first-class-tool-desirability-and-pickup-spec.md) | 87 — Focusa First-Class Tool Desirability and Pickup Spec |
| [docs/88-ontology-backed-workpoint-continuity.md](../88-ontology-backed-workpoint-continuity.md) | 88 — Ontology-backed Workpoint Continuity and Pi Compaction Integration |
| [docs/89-focusa-tool-suite-improvement-hardening-spec.md](../89-focusa-tool-suite-improvement-hardening-spec.md) | 89 — Focusa Tool Suite Improvement and Hardening Spec |
| [docs/90-ontology-backed-tool-contracts-parity-spec.md](../90-ontology-backed-tool-contracts-parity-spec.md) | Spec90 — Ontology-Backed Focusa Tool Contracts and Parity Hardening |
| [docs/91-live-tool-contract-proof-harness-spec.md](../91-live-tool-contract-proof-harness-spec.md) | Spec91 — Live Tool Contract Proof Harness |
| [docs/92-agent-first-polish-hooks-efficiency-spec.md](../92-agent-first-polish-hooks-efficiency-spec.md) | Spec92 — Agent-First Polish, Hook Coverage, Token Efficiency, Cache UX, and Predictive Power |
| [docs/93-non-pi-agent-focusa-awareness-spec.md](../93-non-pi-agent-focusa-awareness-spec.md) | Spec93 — Non-Pi Agent Focusa Awareness |
| [docs/94-focusa-intent-preserving-memory-rpc-optimization-sow.md](../94-focusa-intent-preserving-memory-rpc-optimization-sow.md) | 94 — Focusa Intent-Preserving Memory, Payload, and RPC Optimization SOW |
| [docs/95-focusa-ontology-low-latency-intelligence-enhancer-sow.md](../95-focusa-ontology-low-latency-intelligence-enhancer-sow.md) | 95 — Focusa Ontology Integration and Low-Latency Intelligence Enhancer SOW |
| [docs/96-trajectory-projection-and-daemon-stability-spec.md](../96-trajectory-projection-and-daemon-stability-spec.md) | 96 — Trajectory Projection and Daemon Stability Spec |
| [docs/97-focusa-reflex-primitives-spec.md](../97-focusa-reflex-primitives-spec.md) | 97 — Focusa Reflex Primitives Spec |

### Focused tool-family docs

| Doc | Title / role |
| --- | --- |
| [docs/focusa-tools/README.md](../focusa-tools/README.md) | Focusa Tool Docs |
| [docs/focusa-tools/diagnostics-hygiene.md](../focusa-tools/diagnostics-hygiene.md) | Diagnostics Hygiene Tool Index |
| [docs/focusa-tools/focus-state.md](../focusa-tools/focus-state.md) | Focus State Tool Index |
| [docs/focusa-tools/metacognition.md](../focusa-tools/metacognition.md) | Metacognition Tool Index |
| [docs/focusa-tools/predictive-power.md](../focusa-tools/predictive-power.md) | Predictive Power Tools |
| [docs/focusa-tools/project-identity.md](../focusa-tools/project-identity.md) | Project Identity tools |
| [docs/focusa-tools/stability-audit-2026-05-22.md](../focusa-tools/stability-audit-2026-05-22.md) | Focusa Pi Tool Stability Audit — 2026-05-22 |
| [docs/focusa-tools/trajectory.md](../focusa-tools/trajectory.md) | Trajectory Tool Index |
| [docs/focusa-tools/tree-lineage.md](../focusa-tools/tree-lineage.md) | Tree Lineage Tool Index |
| [docs/focusa-tools/work-loop.md](../focusa-tools/work-loop.md) | Work Loop Tool Index |
| [docs/focusa-tools/workpoint.md](../focusa-tools/workpoint.md) | Workpoint Tool Index |

### Per-tool docs

| Doc | Title / role |
| --- | --- |
| [docs/focusa-tools/tools/focusa_active_object_resolve.md](../focusa-tools/tools/focusa_active_object_resolve.md) | `focusa_active_object_resolve` |
| [docs/focusa-tools/tools/focusa_constraint.md](../focusa-tools/tools/focusa_constraint.md) | `focusa_constraint` |
| [docs/focusa-tools/tools/focusa_current_focus.md](../focusa-tools/tools/focusa_current_focus.md) | `focusa_current_focus` |
| [docs/focusa-tools/tools/focusa_decide.md](../focusa-tools/tools/focusa_decide.md) | `focusa_decide` |
| [docs/focusa-tools/tools/focusa_evidence_capture.md](../focusa-tools/tools/focusa_evidence_capture.md) | `focusa_evidence_capture` |
| [docs/focusa-tools/tools/focusa_failure.md](../focusa-tools/tools/focusa_failure.md) | `focusa_failure` |
| [docs/focusa-tools/tools/focusa_intent.md](../focusa-tools/tools/focusa_intent.md) | `focusa_intent` |
| [docs/focusa-tools/tools/focusa_li_tree_extract.md](../focusa-tools/tools/focusa_li_tree_extract.md) | `focusa_li_tree_extract` |
| [docs/focusa-tools/tools/focusa_lineage_tree.md](../focusa-tools/tools/focusa_lineage_tree.md) | `focusa_lineage_tree` |
| [docs/focusa-tools/tools/focusa_metacog_capture.md](../focusa-tools/tools/focusa_metacog_capture.md) | `focusa_metacog_capture` |
| [docs/focusa-tools/tools/focusa_metacog_doctor.md](../focusa-tools/tools/focusa_metacog_doctor.md) | `focusa_metacog_doctor` |
| [docs/focusa-tools/tools/focusa_metacog_evaluate_outcome.md](../focusa-tools/tools/focusa_metacog_evaluate_outcome.md) | `focusa_metacog_evaluate_outcome` |
| [docs/focusa-tools/tools/focusa_metacog_loop_run.md](../focusa-tools/tools/focusa_metacog_loop_run.md) | `focusa_metacog_loop_run` |
| [docs/focusa-tools/tools/focusa_metacog_plan_adjust.md](../focusa-tools/tools/focusa_metacog_plan_adjust.md) | `focusa_metacog_plan_adjust` |
| [docs/focusa-tools/tools/focusa_metacog_recent_adjustments.md](../focusa-tools/tools/focusa_metacog_recent_adjustments.md) | `focusa_metacog_recent_adjustments` |
| [docs/focusa-tools/tools/focusa_metacog_recent_reflections.md](../focusa-tools/tools/focusa_metacog_recent_reflections.md) | `focusa_metacog_recent_reflections` |
| [docs/focusa-tools/tools/focusa_metacog_reflect.md](../focusa-tools/tools/focusa_metacog_reflect.md) | `focusa_metacog_reflect` |
| [docs/focusa-tools/tools/focusa_metacog_retrieve.md](../focusa-tools/tools/focusa_metacog_retrieve.md) | `focusa_metacog_retrieve` |
| [docs/focusa-tools/tools/focusa_next_step.md](../focusa-tools/tools/focusa_next_step.md) | `focusa_next_step` |
| [docs/focusa-tools/tools/focusa_note.md](../focusa-tools/tools/focusa_note.md) | `focusa_note` |
| [docs/focusa-tools/tools/focusa_open_question.md](../focusa-tools/tools/focusa_open_question.md) | `focusa_open_question` |
| [docs/focusa-tools/tools/focusa_predict_evaluate.md](../focusa-tools/tools/focusa_predict_evaluate.md) | focusa_predict_evaluate |
| [docs/focusa-tools/tools/focusa_predict_recent.md](../focusa-tools/tools/focusa_predict_recent.md) | focusa_predict_recent |
| [docs/focusa-tools/tools/focusa_predict_record.md](../focusa-tools/tools/focusa_predict_record.md) | focusa_predict_record |
| [docs/focusa-tools/tools/focusa_predict_stats.md](../focusa-tools/tools/focusa_predict_stats.md) | focusa_predict_stats |
| [docs/focusa-tools/tools/focusa_project_identity.md](../focusa-tools/tools/focusa_project_identity.md) | `focusa_project_identity` |
| [docs/focusa-tools/tools/focusa_project_verify.md](../focusa-tools/tools/focusa_project_verify.md) | `focusa_project_verify` |
| [docs/focusa-tools/tools/focusa_recent_result.md](../focusa-tools/tools/focusa_recent_result.md) | `focusa_recent_result` |
| [docs/focusa-tools/tools/focusa_reflex_primitives.md](../focusa-tools/tools/focusa_reflex_primitives.md) | `focusa_reflex_primitives` |
| [docs/focusa-tools/tools/focusa_resource_mode.md](../focusa-tools/tools/focusa_resource_mode.md) | `focusa_resource_mode` |
| [docs/focusa-tools/tools/focusa_scratch.md](../focusa-tools/tools/focusa_scratch.md) | `focusa_scratch` |
| [docs/focusa-tools/tools/focusa_silent_sessions.md](../focusa-tools/tools/focusa_silent_sessions.md) | `focusa_silent_sessions` |
| [docs/focusa-tools/tools/focusa_state_hygiene_apply.md](../focusa-tools/tools/focusa_state_hygiene_apply.md) | `focusa_state_hygiene_apply` |
| [docs/focusa-tools/tools/focusa_state_hygiene_doctor.md](../focusa-tools/tools/focusa_state_hygiene_doctor.md) | `focusa_state_hygiene_doctor` |
| [docs/focusa-tools/tools/focusa_state_hygiene_plan.md](../focusa-tools/tools/focusa_state_hygiene_plan.md) | `focusa_state_hygiene_plan` |
| [docs/focusa-tools/tools/focusa_tool_doctor.md](../focusa-tools/tools/focusa_tool_doctor.md) | `focusa_tool_doctor` |
| [docs/focusa-tools/tools/focusa_trajectory_assess.md](../focusa-tools/tools/focusa_trajectory_assess.md) | focusa_trajectory_assess |
| [docs/focusa-tools/tools/focusa_trajectory_checkpoint.md](../focusa-tools/tools/focusa_trajectory_checkpoint.md) | focusa_trajectory_checkpoint |
| [docs/focusa-tools/tools/focusa_trajectory_define_goal.md](../focusa-tools/tools/focusa_trajectory_define_goal.md) | focusa_trajectory_define_goal |
| [docs/focusa-tools/tools/focusa_trajectory_propose_workpoint.md](../focusa-tools/tools/focusa_trajectory_propose_workpoint.md) | focusa_trajectory_propose_workpoint |
| [docs/focusa-tools/tools/focusa_trajectory_resume.md](../focusa-tools/tools/focusa_trajectory_resume.md) | focusa_trajectory_resume |
| [docs/focusa-tools/tools/focusa_trajectory_view.md](../focusa-tools/tools/focusa_trajectory_view.md) | focusa_trajectory_view |
| [docs/focusa-tools/tools/focusa_traverse.md](../focusa-tools/tools/focusa_traverse.md) | `focusa_traverse` |
| [docs/focusa-tools/tools/focusa_tree_diff_context.md](../focusa-tools/tools/focusa_tree_diff_context.md) | `focusa_tree_diff_context` |
| [docs/focusa-tools/tools/focusa_tree_head.md](../focusa-tools/tools/focusa_tree_head.md) | `focusa_tree_head` |
| [docs/focusa-tools/tools/focusa_tree_path.md](../focusa-tools/tools/focusa_tree_path.md) | `focusa_tree_path` |
| [docs/focusa-tools/tools/focusa_tree_recent_snapshots.md](../focusa-tools/tools/focusa_tree_recent_snapshots.md) | `focusa_tree_recent_snapshots` |
| [docs/focusa-tools/tools/focusa_tree_restore_state.md](../focusa-tools/tools/focusa_tree_restore_state.md) | `focusa_tree_restore_state` |
| [docs/focusa-tools/tools/focusa_tree_snapshot_compare_latest.md](../focusa-tools/tools/focusa_tree_snapshot_compare_latest.md) | `focusa_tree_snapshot_compare_latest` |
| [docs/focusa-tools/tools/focusa_tree_snapshot_state.md](../focusa-tools/tools/focusa_tree_snapshot_state.md) | `focusa_tree_snapshot_state` |
| [docs/focusa-tools/tools/focusa_work_loop_checkpoint.md](../focusa-tools/tools/focusa_work_loop_checkpoint.md) | `focusa_work_loop_checkpoint` |
| [docs/focusa-tools/tools/focusa_work_loop_context.md](../focusa-tools/tools/focusa_work_loop_context.md) | `focusa_work_loop_context` |
| [docs/focusa-tools/tools/focusa_work_loop_control.md](../focusa-tools/tools/focusa_work_loop_control.md) | `focusa_work_loop_control` |
| [docs/focusa-tools/tools/focusa_work_loop_select_next.md](../focusa-tools/tools/focusa_work_loop_select_next.md) | `focusa_work_loop_select_next` |
| [docs/focusa-tools/tools/focusa_work_loop_status.md](../focusa-tools/tools/focusa_work_loop_status.md) | `focusa_work_loop_status` |
| [docs/focusa-tools/tools/focusa_work_loop_writer_status.md](../focusa-tools/tools/focusa_work_loop_writer_status.md) | `focusa_work_loop_writer_status` |
| [docs/focusa-tools/tools/focusa_workpoint_checkpoint.md](../focusa-tools/tools/focusa_workpoint_checkpoint.md) | `focusa_workpoint_checkpoint` |
| [docs/focusa-tools/tools/focusa_workpoint_link_evidence.md](../focusa-tools/tools/focusa_workpoint_link_evidence.md) | `focusa_workpoint_link_evidence` |
| [docs/focusa-tools/tools/focusa_workpoint_resume.md](../focusa-tools/tools/focusa_workpoint_resume.md) | `focusa_workpoint_resume` |

### Evidence and proof docs

| Doc | Title / role |
| --- | --- |
| [docs/evidence/BEADS_DB_COMPACTION_2026-05-22.md](../evidence/BEADS_DB_COMPACTION_2026-05-22.md) | Beads DB event-payload compaction — 2026-05-22 |
| [docs/evidence/CURRENT_BUILD_DOC_GAPS_CLOSED_2026-04-28.md](../evidence/CURRENT_BUILD_DOC_GAPS_CLOSED_2026-04-28.md) | Current Build Documentation Gaps Closed — 2026-04-28 |
| [docs/evidence/DAEMON_RESILIENCE_LIVE_PROOF_2026-04-28.md](../evidence/DAEMON_RESILIENCE_LIVE_PROOF_2026-04-28.md) | Daemon Resilience Live Proof — 2026-04-28 |
| [docs/evidence/DOC50_ONTOLOGY_PARITY_CERTIFICATE_2026-04-21.md](../evidence/DOC50_ONTOLOGY_PARITY_CERTIFICATE_2026-04-21.md) | DOC50 Ontology Parity Certificate — 2026-04-21 |
| [docs/evidence/DOC78_COMPLETION_CERTIFICATE_2026-04-18.md](../evidence/DOC78_COMPLETION_CERTIFICATE_2026-04-18.md) | Doc78 Completion Certificate — 2026-04-18 |
| [docs/evidence/DOC78_PRODUCTION_RUNTIME_EVIDENCE_2026-04-17.md](../evidence/DOC78_PRODUCTION_RUNTIME_EVIDENCE_2026-04-17.md) | Doc78 Production Runtime Evidence — 2026-04-17 |
| [docs/evidence/DOC78_PRODUCTION_RUNTIME_SERIES_EVIDENCE_2026-04-18.md](../evidence/DOC78_PRODUCTION_RUNTIME_SERIES_EVIDENCE_2026-04-18.md) | Doc78 Production Runtime Series Evidence — 2026-04-18 |
| [docs/evidence/DOCS_SECRET_AUDIT_2026-04-28.md](../evidence/DOCS_SECRET_AUDIT_2026-04-28.md) | Docs Secret Audit — 2026-04-28 |
| [docs/evidence/FOCUSA_BATTERY_TEST_REPORT_2026-05-22.md](../evidence/FOCUSA_BATTERY_TEST_REPORT_2026-05-22.md) | Focusa Battery Test Report — 2026-05-22 |
| [docs/evidence/FOCUSA_FOCUSED_SKILLS_AND_TOOL_DOCS_RELEASE_2026-04-28.md](../evidence/FOCUSA_FOCUSED_SKILLS_AND_TOOL_DOCS_RELEASE_2026-04-28.md) | Focusa Focused Skills and Tool Docs Release — 2026-04-28 |
| [docs/evidence/FOCUSA_ONE_TOOL_PER_DOC_CORRECTION_2026-04-28.md](../evidence/FOCUSA_ONE_TOOL_PER_DOC_CORRECTION_2026-04-28.md) | Focusa One-Tool-Per-Doc Correction — 2026-04-28 |
| [docs/evidence/FOCUSA_SKILL_EVALUATION_AND_REPAIR_2026-04-28.md](../evidence/FOCUSA_SKILL_EVALUATION_AND_REPAIR_2026-04-28.md) | Focusa Skill Evaluation and Repair — 2026-04-28 |
| [docs/evidence/FOCUSA_TOOL_STRESS_EVIDENCE_2026-04-28.md](../evidence/FOCUSA_TOOL_STRESS_EVIDENCE_2026-04-28.md) | Focusa Tool Stress Evidence — 2026-04-28 |
| [docs/evidence/OPERATOR_PREVIEW_RELEASE_CHECKPOINT_2026-05-26.md](../evidence/OPERATOR_PREVIEW_RELEASE_CHECKPOINT_2026-05-26.md) | Operator Preview Release Checkpoint — 2026-05-26 |
| [docs/evidence/PRODUCTION_RELEASE_MAC_APP_GITHUB_FIX_2026-04-28.md](../evidence/PRODUCTION_RELEASE_MAC_APP_GITHUB_FIX_2026-04-28.md) | Production Release / Mac App / GitHub Fix — 2026-04-28 |
| [docs/evidence/PR_BUNDLE_SPEC83_86_2026-04-21.md](../evidence/PR_BUNDLE_SPEC83_86_2026-04-21.md) | PR Bundle — SPEC83 + SPEC84/85/86 |
| [docs/evidence/PUBLIC_DOCS_OPERATOR_PREVIEW_SYNC_2026-05-26.md](../evidence/PUBLIC_DOCS_OPERATOR_PREVIEW_SYNC_2026-05-26.md) | Public Docs Operator Preview Sync — 2026-05-26 |
| [docs/evidence/PUBLIC_DOCS_RUNTIME_ALIGNMENT_AUDIT_2026-04-28.md](../evidence/PUBLIC_DOCS_RUNTIME_ALIGNMENT_AUDIT_2026-04-28.md) | Public Docs Runtime Alignment Audit — 2026-04-28 |
| [docs/evidence/PUBLIC_DOCS_SPEC97_REFRESH_2026-05-25.md](../evidence/PUBLIC_DOCS_SPEC97_REFRESH_2026-05-25.md) | Public Docs Spec97 Refresh Evidence — 2026-05-25 |
| [docs/evidence/SPEC80_B1_1_HEAD_PATH_CONTRACT_FINALIZATION_2026-04-21.md](../evidence/SPEC80_B1_1_HEAD_PATH_CONTRACT_FINALIZATION_2026-04-21.md) | SPEC80 B1.1 — Head/Path Contract Finalization (Tree/Lineage) |
| [docs/evidence/SPEC80_B1_2_SNAPSHOT_RESTORE_DIFF_CONTRACT_FINALIZATION_2026-04-21.md](../evidence/SPEC80_B1_2_SNAPSHOT_RESTORE_DIFF_CONTRACT_FINALIZATION_2026-04-21.md) | SPEC80 B1.2 — Snapshot/Restore/Diff Contract Finalization |
| [docs/evidence/SPEC80_B2_1_CAPTURE_RETRIEVE_REFLECT_SCHEMAS_2026-04-21.md](../evidence/SPEC80_B2_1_CAPTURE_RETRIEVE_REFLECT_SCHEMAS_2026-04-21.md) | SPEC80 B2.1 — Capture/Retrieve/Reflect Schema Finalization |
| [docs/evidence/SPEC80_B2_2_PLAN_ADJUST_EVALUATE_SCHEMAS_2026-04-21.md](../evidence/SPEC80_B2_2_PLAN_ADJUST_EVALUATE_SCHEMAS_2026-04-21.md) | SPEC80 B2.2 — Plan-Adjust/Evaluate Schema Finalization |
| [docs/evidence/SPEC80_B3_1_IMPLEMENTED_ENDPOINT_BINDING_VALIDATION_2026-04-21.md](../evidence/SPEC80_B3_1_IMPLEMENTED_ENDPOINT_BINDING_VALIDATION_2026-04-21.md) | SPEC80 B3.1 — Implemented Endpoint Binding Validation |
| [docs/evidence/SPEC80_B3_2_PLANNED_ENDPOINT_BACKLOG_DERIVATION_2026-04-21.md](../evidence/SPEC80_B3_2_PLANNED_ENDPOINT_BACKLOG_DERIVATION_2026-04-21.md) | SPEC80 B3.2 — Planned Endpoint Backlog Derivation |
| [docs/evidence/SPEC80_B4_1_TYPED_ERROR_ENVELOPE_MAPPING_2026-04-21.md](../evidence/SPEC80_B4_1_TYPED_ERROR_ENVELOPE_MAPPING_2026-04-21.md) | SPEC80 B4.1 — Typed Error Envelope Mapping |
| [docs/evidence/SPEC80_B4_2_CAPABILITY_PERMISSION_MAPPING_2026-04-21.md](../evidence/SPEC80_B4_2_CAPABILITY_PERMISSION_MAPPING_2026-04-21.md) | SPEC80 B4.2 — Capability Permission Mapping |
| [docs/evidence/SPEC80_BASELINE_WINDOW_COMPUTATION_2026-04-21.md](../evidence/SPEC80_BASELINE_WINDOW_COMPUTATION_2026-04-21.md) | SPEC80 E2.1 — Baseline Window Computation Design |
| [docs/evidence/SPEC80_C1_1_EXPORT_EXECUTION_ENGINE_PLAN_2026-04-21.md](../evidence/SPEC80_C1_1_EXPORT_EXECUTION_ENGINE_PLAN_2026-04-21.md) | SPEC80 C1.1 — Export Execution Engine Plan |
| [docs/evidence/SPEC80_C1_2_EXPORT_VALIDATION_TEST_PLAN_2026-04-21.md](../evidence/SPEC80_C1_2_EXPORT_VALIDATION_TEST_PLAN_2026-04-21.md) | SPEC80 C1.2 — Export Validation + Tests Plan |
| [docs/evidence/SPEC80_C2_1_LINEAGE_COMMAND_SURFACE_DESIGN_2026-04-21.md](../evidence/SPEC80_C2_1_LINEAGE_COMMAND_SURFACE_DESIGN_2026-04-21.md) | SPEC80 C2.1 — Lineage Command Surface Design |
| [docs/evidence/SPEC80_C2_2_LINEAGE_CLI_CONTRACT_TESTS_PLAN_2026-04-21.md](../evidence/SPEC80_C2_2_LINEAGE_CLI_CONTRACT_TESTS_PLAN_2026-04-21.md) | SPEC80 C2.2 — Lineage CLI Contract Tests Plan |
| [docs/evidence/SPEC80_C3_1_METACOGNITION_COMMAND_SURFACE_DESIGN_2026-04-21.md](../evidence/SPEC80_C3_1_METACOGNITION_COMMAND_SURFACE_DESIGN_2026-04-21.md) | SPEC80 C3.1 — Metacognition Command Surface Design |
| [docs/evidence/SPEC80_C3_2_METACOGNITION_CLI_CONTRACT_TESTS_PLAN_2026-04-21.md](../evidence/SPEC80_C3_2_METACOGNITION_CLI_CONTRACT_TESTS_PLAN_2026-04-21.md) | SPEC80 C3.2 — Metacognition CLI Contract Tests Plan |
| [docs/evidence/SPEC80_C4_1_JSON_SCHEMA_REGISTRY_PLAN_2026-04-21.md](../evidence/SPEC80_C4_1_JSON_SCHEMA_REGISTRY_PLAN_2026-04-21.md) | SPEC80 C4.1 — JSON Schema Registry Plan |
| [docs/evidence/SPEC80_C4_2_COMPATIBILITY_POLICY_2026-04-21.md](../evidence/SPEC80_C4_2_COMPATIBILITY_POLICY_2026-04-21.md) | SPEC80 C4.2 — Compatibility Policy |
| [docs/evidence/SPEC80_CLAIM_VALIDATION_PROTOCOL_2026-04-21.md](../evidence/SPEC80_CLAIM_VALIDATION_PROTOCOL_2026-04-21.md) | SPEC80 Claim Validation Protocol — 2026-04-21 |
| [docs/evidence/SPEC80_CLI_JSON_COMPATIBILITY_POLICY_2026-04-21.md](../evidence/SPEC80_CLI_JSON_COMPATIBILITY_POLICY_2026-04-21.md) | SPEC80 C4.2 — CLI JSON Compatibility Policy |
| [docs/evidence/SPEC80_CLI_JSON_SCHEMA_REGISTRY_2026-04-21.md](../evidence/SPEC80_CLI_JSON_SCHEMA_REGISTRY_2026-04-21.md) | SPEC80 C4.1 — CLI JSON Schema Registry |
| [docs/evidence/SPEC80_CLI_METACOG_COMMAND_SURFACE_2026-04-21.md](../evidence/SPEC80_CLI_METACOG_COMMAND_SURFACE_2026-04-21.md) | SPEC80 CLI Metacognition Command Surface Design — 2026-04-21 |
| [docs/evidence/SPEC80_COMPACTION_SURVIVAL_SCENARIO_SPEC_2026-04-21.md](../evidence/SPEC80_COMPACTION_SURVIVAL_SCENARIO_SPEC_2026-04-21.md) | SPEC80 D4.1 — Compaction Survival Scenario Spec |
| [docs/evidence/SPEC80_COMPOUNDING_GATE_REPORT_GENERATOR_2026-04-21.md](../evidence/SPEC80_COMPOUNDING_GATE_REPORT_GENERATOR_2026-04-21.md) | SPEC80 E4.1 — Compounding Gate Report Generator Contract |
| [docs/evidence/SPEC80_D1_1_FORK_INTEGRITY_SCENARIO_SPEC_2026-04-21.md](../evidence/SPEC80_D1_1_FORK_INTEGRITY_SCENARIO_SPEC_2026-04-21.md) | SPEC80 D1.1 — Fork Integrity Scenario Spec |
| [docs/evidence/SPEC80_D1_2_TREE_NAVIGATION_RESTORE_SCENARIO_SPEC_2026-04-21.md](../evidence/SPEC80_D1_2_TREE_NAVIGATION_RESTORE_SCENARIO_SPEC_2026-04-21.md) | SPEC80 D1.2 — Tree Navigation Restore Scenario Spec |
| [docs/evidence/SPEC80_D2_1_MERGE_CONFLICT_VISIBILITY_TEST_SPEC_2026-04-21.md](../evidence/SPEC80_D2_1_MERGE_CONFLICT_VISIBILITY_TEST_SPEC_2026-04-21.md) | SPEC80 D2.1 — Merge Conflict Visibility Test Spec |
| [docs/evidence/SPEC80_D2_2_SNAPSHOT_API_IDEMPOTENCY_CONSISTENCY_SPEC_2026-04-21.md](../evidence/SPEC80_D2_2_SNAPSHOT_API_IDEMPOTENCY_CONSISTENCY_SPEC_2026-04-21.md) | SPEC80 D2.2 — Snapshot API Idempotency and Consistency Spec |
| [docs/evidence/SPEC80_D3_1_REFLECTION_METACOG_LATENCY_BUDGET_SPEC_2026-04-21.md](../evidence/SPEC80_D3_1_REFLECTION_METACOG_LATENCY_BUDGET_SPEC_2026-04-21.md) | SPEC80 D3.1 — Reflection/Metacog Latency Budget Spec |
| [docs/evidence/SPEC80_D3_2_RESTORE_COMPACTION_PERFORMANCE_BUDGET_SPEC_2026-04-21.md](../evidence/SPEC80_D3_2_RESTORE_COMPACTION_PERFORMANCE_BUDGET_SPEC_2026-04-21.md) | SPEC80 D3.2 — Restore/Compaction Performance Budget Spec |
| [docs/evidence/SPEC80_D4_1_COMPACTION_SURVIVAL_TEST_PACK_SPEC_2026-04-21.md](../evidence/SPEC80_D4_1_COMPACTION_SURVIVAL_TEST_PACK_SPEC_2026-04-21.md) | SPEC80 D4.1 — Compaction Survival Test Pack Spec |
| [docs/evidence/SPEC80_D4_2_SILENT_MUTATION_SENTINEL_CHECKS_SPEC_2026-04-21.md](../evidence/SPEC80_D4_2_SILENT_MUTATION_SENTINEL_CHECKS_SPEC_2026-04-21.md) | SPEC80 D4.2 — Silent Mutation Sentinel Checks Spec |
| [docs/evidence/SPEC80_E1_1_METRIC_EXTRACTION_PIPELINE_DESIGN_2026-04-21.md](../evidence/SPEC80_E1_1_METRIC_EXTRACTION_PIPELINE_DESIGN_2026-04-21.md) | SPEC80 E1.1 — Metric Extraction Pipeline Design |
| [docs/evidence/SPEC80_E1_2_THRESHOLD_EVALUATOR_DESIGN_2026-04-21.md](../evidence/SPEC80_E1_2_THRESHOLD_EVALUATOR_DESIGN_2026-04-21.md) | SPEC80 E1.2 — Threshold Evaluator Design |
| [docs/evidence/SPEC80_E2_1_BASELINE_WINDOW_COMPUTATION_SPEC_2026-04-21.md](../evidence/SPEC80_E2_1_BASELINE_WINDOW_COMPUTATION_SPEC_2026-04-21.md) | SPEC80 E2.1 — Baseline Window Computation Spec |
| [docs/evidence/SPEC80_E2_2_EVALUATION_CADENCE_AUTOMATION_SPEC_2026-04-21.md](../evidence/SPEC80_E2_2_EVALUATION_CADENCE_AUTOMATION_SPEC_2026-04-21.md) | SPEC80 E2.2 — Evaluation Cadence Automation Spec |
| [docs/evidence/SPEC80_E3_1_FORM_SCHEMA_ONTOLOGY_ALIGNMENT_SPEC_2026-04-21.md](../evidence/SPEC80_E3_1_FORM_SCHEMA_ONTOLOGY_ALIGNMENT_SPEC_2026-04-21.md) | SPEC80 E3.1 — Form Schema + Ontology Alignment Spec |
| [docs/evidence/SPEC80_E3_2_FORM_QUALITY_VALIDATOR_SPEC_2026-04-21.md](../evidence/SPEC80_E3_2_FORM_QUALITY_VALIDATOR_SPEC_2026-04-21.md) | SPEC80 E3.2 — Form Quality Validator Spec |
| [docs/evidence/SPEC80_E4_1_COMPOUNDING_GATE_REPORT_GENERATOR_SPEC_2026-04-21.md](../evidence/SPEC80_E4_1_COMPOUNDING_GATE_REPORT_GENERATOR_SPEC_2026-04-21.md) | SPEC80 E4.1 — Compounding Gate Report Generator Spec |
| [docs/evidence/SPEC80_E4_2_LEARNING_PROMOTION_DECISION_POLICY_SPEC_2026-04-21.md](../evidence/SPEC80_E4_2_LEARNING_PROMOTION_DECISION_POLICY_SPEC_2026-04-21.md) | SPEC80 E4.2 — Learning Promotion Decision Policy Spec |
| [docs/evidence/SPEC80_ENDPOINT_FALLBACK_BINDING_MATRIX_2026-04-21.md](../evidence/SPEC80_ENDPOINT_FALLBACK_BINDING_MATRIX_2026-04-21.md) | SPEC80 Endpoint + Fallback Binding Matrix (Operationalized) — 2026-04-21 |
| [docs/evidence/SPEC80_ERROR_PERMISSION_MODEL_2026-04-21.md](../evidence/SPEC80_ERROR_PERMISSION_MODEL_2026-04-21.md) | SPEC80 Error + Permission Model (Normalized) — 2026-04-21 |
| [docs/evidence/SPEC80_EVALUATION_CADENCE_AUTOMATION_2026-04-21.md](../evidence/SPEC80_EVALUATION_CADENCE_AUTOMATION_2026-04-21.md) | SPEC80 E2.2 — Evaluation Cadence Automation Design |
| [docs/evidence/SPEC80_EXPORT_EXECUTION_ENGINE_PLAN_2026-04-21.md](../evidence/SPEC80_EXPORT_EXECUTION_ENGINE_PLAN_2026-04-21.md) | SPEC80 C1.1 — Export Execution Engine Plan |
| [docs/evidence/SPEC80_EXPORT_VALIDATION_TEST_PLAN_2026-04-21.md](../evidence/SPEC80_EXPORT_VALIDATION_TEST_PLAN_2026-04-21.md) | SPEC80 C1.2 — Export Validation + Tests Plan |
| [docs/evidence/SPEC80_F1_1_PHASE_0_2_READINESS_CHECKS_SPEC_2026-04-21.md](../evidence/SPEC80_F1_1_PHASE_0_2_READINESS_CHECKS_SPEC_2026-04-21.md) | SPEC80 F1.1 — Phase 0-2 Readiness Checks Spec |
| [docs/evidence/SPEC80_F1_2_PHASE_3_4_EVIDENCE_CHECKS_SPEC_2026-04-21.md](../evidence/SPEC80_F1_2_PHASE_3_4_EVIDENCE_CHECKS_SPEC_2026-04-21.md) | SPEC80 F1.2 — Phase 3-4 Evidence Checks Spec |
| [docs/evidence/SPEC80_F2_1_CRITICAL_PATH_MAPPING_SPEC_2026-04-21.md](../evidence/SPEC80_F2_1_CRITICAL_PATH_MAPPING_SPEC_2026-04-21.md) | SPEC80 F2.1 — Critical Path Mapping Spec |
| [docs/evidence/SPEC80_F2_2_ROLLBACK_FALLBACK_PLAN_SPEC_2026-04-21.md](../evidence/SPEC80_F2_2_ROLLBACK_FALLBACK_PLAN_SPEC_2026-04-21.md) | SPEC80 F2.2 — Rollback and Fallback Plan Spec |
| [docs/evidence/SPEC80_F3_1_BEAD_METADATA_LINT_POLICY_SPEC_2026-04-21.md](../evidence/SPEC80_F3_1_BEAD_METADATA_LINT_POLICY_SPEC_2026-04-21.md) | SPEC80 F3.1 — Bead Metadata Lint Policy Spec |
| [docs/evidence/SPEC80_F3_2_CLOSURE_AUDIT_AUTOMATION_SPEC_2026-04-21.md](../evidence/SPEC80_F3_2_CLOSURE_AUDIT_AUTOMATION_SPEC_2026-04-21.md) | SPEC80 F3.2 — Closure Audit Automation Spec |
| [docs/evidence/SPEC80_F4_1_UTILIZATION_CRITERIA_VERIFIER_SPEC_2026-04-21.md](../evidence/SPEC80_F4_1_UTILIZATION_CRITERIA_VERIFIER_SPEC_2026-04-21.md) | SPEC80 F4.1 — Utilization Criteria Verifier Spec |
| [docs/evidence/SPEC80_F4_2_FINAL_OPERATOR_SIGNOFF_PACKET_SPEC_2026-04-21.md](../evidence/SPEC80_F4_2_FINAL_OPERATOR_SIGNOFF_PACKET_SPEC_2026-04-21.md) | SPEC80 F4.2 — Final Operator Sign-off Packet Spec |
| [docs/evidence/SPEC80_FOCUSA_ONTOLOGY_AUTHORITY_MAP_2026-04-21.md](../evidence/SPEC80_FOCUSA_ONTOLOGY_AUTHORITY_MAP_2026-04-21.md) | SPEC80 Focusa Ontology Authority Map — 2026-04-21 |
| [docs/evidence/SPEC80_FORK_INTEGRITY_SCENARIO_SPEC_2026-04-21.md](../evidence/SPEC80_FORK_INTEGRITY_SCENARIO_SPEC_2026-04-21.md) | SPEC80 D1.1 — Fork Integrity Scenario Spec |
| [docs/evidence/SPEC80_LABEL_TAXONOMY_ENFORCEMENT_2026-04-21.md](../evidence/SPEC80_LABEL_TAXONOMY_ENFORCEMENT_2026-04-21.md) | SPEC80 Label Taxonomy Enforcement — 2026-04-21 |
| [docs/evidence/SPEC80_LAYER_COMPLIANCE_REVIEW_CHECKLIST_2026-04-21.md](../evidence/SPEC80_LAYER_COMPLIANCE_REVIEW_CHECKLIST_2026-04-21.md) | SPEC80 Layer Compliance Review Checklist — 2026-04-21 |
| [docs/evidence/SPEC80_MERGE_CONFLICT_VISIBILITY_SCENARIO_SPEC_2026-04-21.md](../evidence/SPEC80_MERGE_CONFLICT_VISIBILITY_SCENARIO_SPEC_2026-04-21.md) | SPEC80 D2.1 — Merge Conflict Visibility Scenario Spec |
| [docs/evidence/SPEC80_METACOG_TOOL_CONTRACTS_2026-04-21.md](../evidence/SPEC80_METACOG_TOOL_CONTRACTS_2026-04-21.md) | SPEC80 Metacognitive Tool Contracts — 2026-04-21 |
| [docs/evidence/SPEC80_OUTCOME_METRIC_EXTRACTION_PIPELINE_2026-04-21.md](../evidence/SPEC80_OUTCOME_METRIC_EXTRACTION_PIPELINE_2026-04-21.md) | SPEC80 E1.1 — Outcome Metric Extraction Pipeline Design |
| [docs/evidence/SPEC80_OUTCOME_THRESHOLD_EVALUATOR_DESIGN_2026-04-21.md](../evidence/SPEC80_OUTCOME_THRESHOLD_EVALUATOR_DESIGN_2026-04-21.md) | SPEC80 E1.2 — Outcome Threshold Evaluator Design |
| [docs/evidence/SPEC80_PI_AUTHORITY_MAP_2026-04-21.md](../evidence/SPEC80_PI_AUTHORITY_MAP_2026-04-21.md) | SPEC80 Pi Authority Map — 2026-04-21 |
| [docs/evidence/SPEC80_REFLECTION_METACOG_LATENCY_BUDGET_2026-04-21.md](../evidence/SPEC80_REFLECTION_METACOG_LATENCY_BUDGET_2026-04-21.md) | SPEC80 D3.1 — Reflection/Metacognition Latency Budget |
| [docs/evidence/SPEC80_RESTORE_COMPACTION_PERFORMANCE_BUDGETS_2026-04-21.md](../evidence/SPEC80_RESTORE_COMPACTION_PERFORMANCE_BUDGETS_2026-04-21.md) | SPEC80 D3.2 — Restore/Compaction Performance Budgets |
| [docs/evidence/SPEC80_SECTION20_DECOMPOSITION_LANES_2026-04-21.md](../evidence/SPEC80_SECTION20_DECOMPOSITION_LANES_2026-04-21.md) | SPEC80 §20 Decomposition Lanes — 2026-04-21 |
| [docs/evidence/SPEC80_SILENT_MUTATION_SENTINEL_CHECKS_2026-04-21.md](../evidence/SPEC80_SILENT_MUTATION_SENTINEL_CHECKS_2026-04-21.md) | SPEC80 D4.2 — Silent Mutation Sentinel Checks |
| [docs/evidence/SPEC80_SNAPSHOT_IDEMPOTENCY_CONSISTENCY_PLAN_2026-04-21.md](../evidence/SPEC80_SNAPSHOT_IDEMPOTENCY_CONSISTENCY_PLAN_2026-04-21.md) | SPEC80 D2.2 — Snapshot API Idempotency + Consistency Plan |
| [docs/evidence/SPEC80_TOOL_LAYER_DECLARATION_CONTRACT_2026-04-21.md](../evidence/SPEC80_TOOL_LAYER_DECLARATION_CONTRACT_2026-04-21.md) | SPEC80 Tool-to-Layer Declaration Contract — 2026-04-21 |
| [docs/evidence/SPEC80_TOOL_SUITE_RUNTIME_EVIDENCE_2026-04-21.md](../evidence/SPEC80_TOOL_SUITE_RUNTIME_EVIDENCE_2026-04-21.md) | SPEC80 Tool Suite Runtime Evidence (2026-04-21) |
| [docs/evidence/SPEC80_TREE_LINEAGE_TOOL_CONTRACTS_2026-04-21.md](../evidence/SPEC80_TREE_LINEAGE_TOOL_CONTRACTS_2026-04-21.md) | SPEC80 Tree/Lineage Bridge Tool Contracts — 2026-04-21 |
| [docs/evidence/SPEC80_TREE_NAVIGATION_RESTORE_SCENARIO_SPEC_2026-04-21.md](../evidence/SPEC80_TREE_NAVIGATION_RESTORE_SCENARIO_SPEC_2026-04-21.md) | SPEC80 D1.2 — Tree Navigation Restore Scenario Spec |
| [docs/evidence/SPEC80_UTILIZATION_PROOF_PACK_PLAN_2026-04-21.md](../evidence/SPEC80_UTILIZATION_PROOF_PACK_PLAN_2026-04-21.md) | SPEC80 Utilization Proof Pack Plan — 2026-04-21 |
| [docs/evidence/SPEC81_CLI_HIGH_ORDER_WORKFLOWS_NOTE_2026-04-22.md](../evidence/SPEC81_CLI_HIGH_ORDER_WORKFLOWS_NOTE_2026-04-22.md) | SPEC81 CLI High-Order Workflows Note |
| [docs/evidence/SPEC81_CLI_RUNTIME_AND_ERROR_PATH_NOTE_2026-04-22.md](../evidence/SPEC81_CLI_RUNTIME_AND_ERROR_PATH_NOTE_2026-04-22.md) | SPEC81 CLI Runtime + Error Path Note |
| [docs/evidence/SPEC81_COMPLETION_MATRIX_2026-04-22.md](../evidence/SPEC81_COMPLETION_MATRIX_2026-04-22.md) | SPEC81 Completion Matrix |
| [docs/evidence/SPEC81_EXTENSION_RUNTIME_CONTRACT_NOTE_2026-04-22.md](../evidence/SPEC81_EXTENSION_RUNTIME_CONTRACT_NOTE_2026-04-22.md) | SPEC81 Extension Runtime Contract Note |
| [docs/evidence/SPEC81_HIGHEST_QUALITY_FOLLOWUP_NOTE_2026-04-22.md](../evidence/SPEC81_HIGHEST_QUALITY_FOLLOWUP_NOTE_2026-04-22.md) | SPEC81 Highest-Quality Follow-up Note |
| [docs/evidence/SPEC81_TOOL_HARDENING_IMPL_NOTE_2026-04-22.md](../evidence/SPEC81_TOOL_HARDENING_IMPL_NOTE_2026-04-22.md) | SPEC81 Tool Hardening Implementation Note |
| [docs/evidence/SPEC81_TOOL_SUITE_AUDIT_MATRIX_2026-04-22.md](../evidence/SPEC81_TOOL_SUITE_AUDIT_MATRIX_2026-04-22.md) | SPEC81 Tool Suite Audit Matrix |
| [docs/evidence/SPEC82_PI_EXTENSION_PERF_REEVALUATION_2026-04-22.md](../evidence/SPEC82_PI_EXTENSION_PERF_REEVALUATION_2026-04-22.md) | SPEC82 Pi Extension Perf Reevaluation |
| [docs/evidence/SPEC83_RPC_EFFICIENCY_RUNTIME_EVIDENCE_2026-04-21.md](../evidence/SPEC83_RPC_EFFICIENCY_RUNTIME_EVIDENCE_2026-04-21.md) | SPEC83 Runtime Evidence — Pi × Focusa RPC Efficiency |
| [docs/evidence/SPEC84_ACTION_TYPE_PARITY_AUDIT_MATRIX_2026-04-21.md](../evidence/SPEC84_ACTION_TYPE_PARITY_AUDIT_MATRIX_2026-04-21.md) | SPEC84 Action-Type Parity Audit Matrix |
| [docs/evidence/SPEC84_ACTION_TYPE_PARITY_RUNTIME_EVIDENCE_2026-04-21.md](../evidence/SPEC84_ACTION_TYPE_PARITY_RUNTIME_EVIDENCE_2026-04-21.md) | SPEC84 Runtime Evidence — Action-Type Parity |
| [docs/evidence/SPEC85_RELATION_TYPE_PARITY_AUDIT_MATRIX_2026-04-21.md](../evidence/SPEC85_RELATION_TYPE_PARITY_AUDIT_MATRIX_2026-04-21.md) | SPEC85 Relation-Type Parity Audit Matrix |
| [docs/evidence/SPEC85_RELATION_TYPE_PARITY_RUNTIME_EVIDENCE_2026-04-21.md](../evidence/SPEC85_RELATION_TYPE_PARITY_RUNTIME_EVIDENCE_2026-04-21.md) | SPEC85 Runtime Evidence — Relation-Type Parity |
| [docs/evidence/SPEC86_SHARED_STATUS_LIFECYCLE_AUDIT_MATRIX_2026-04-21.md](../evidence/SPEC86_SHARED_STATUS_LIFECYCLE_AUDIT_MATRIX_2026-04-21.md) | SPEC86 Shared-Status Lifecycle Parity Audit Matrix |
| [docs/evidence/SPEC86_SHARED_STATUS_LIFECYCLE_PARITY_RUNTIME_EVIDENCE_2026-04-21.md](../evidence/SPEC86_SHARED_STATUS_LIFECYCLE_PARITY_RUNTIME_EVIDENCE_2026-04-21.md) | SPEC86 Runtime Evidence — Shared Status Vocabulary Parity |
| [docs/evidence/SPEC87_COMPLETION_MATRIX_2026-04-22.md](../evidence/SPEC87_COMPLETION_MATRIX_2026-04-22.md) | SPEC87 Completion Matrix |
| [docs/evidence/SPEC87_EXISTING_TOOL_DESIRABILITY_UPGRADES_NOTE_2026-04-22.md](../evidence/SPEC87_EXISTING_TOOL_DESIRABILITY_UPGRADES_NOTE_2026-04-22.md) | SPEC87 Existing Tool Desirability Upgrades Note |
| [docs/evidence/SPEC87_HELPER_AND_COMPOSITE_TOOLS_NOTE_2026-04-22.md](../evidence/SPEC87_HELPER_AND_COMPOSITE_TOOLS_NOTE_2026-04-22.md) | SPEC87 Helper and Composite Tools Note |
| [docs/evidence/SPEC87_PICKUP_AND_EFFECTIVENESS_PROOF_NOTE_2026-04-22.md](../evidence/SPEC87_PICKUP_AND_EFFECTIVENESS_PROOF_NOTE_2026-04-22.md) | SPEC87 Pickup and Effectiveness Proof Note |
| [docs/evidence/SPEC87_TOOL_DESIRABILITY_AUDIT_MATRIX_2026-04-22.md](../evidence/SPEC87_TOOL_DESIRABILITY_AUDIT_MATRIX_2026-04-22.md) | SPEC87 Tool Desirability Audit Matrix |
| [docs/evidence/SPEC88_CRITICAL_IMPL_AUDIT_2026-04-28.md](../evidence/SPEC88_CRITICAL_IMPL_AUDIT_2026-04-28.md) | Spec88 Critical Implementation Audit — 2026-04-28 |
| [docs/evidence/SPEC88_CURRENT_WORKPOINT_2026-04-28.md](../evidence/SPEC88_CURRENT_WORKPOINT_2026-04-28.md) | Spec88 Current Workpoint — Compaction Handoff |
| [docs/evidence/SPEC88_GOLDEN_EVAL_EVIDENCE_2026-04-28.md](../evidence/SPEC88_GOLDEN_EVAL_EVIDENCE_2026-04-28.md) | Spec88 Golden Eval Evidence Packet — Workpoint Continuity |
| [docs/evidence/SPEC88_LIVE_OPERATIONAL_EVIDENCE_2026-04-28.md](../evidence/SPEC88_LIVE_OPERATIONAL_EVIDENCE_2026-04-28.md) | Spec88 Live Operational Evidence — 2026-04-28 |
| [docs/evidence/SPEC88_ROLLOUT_GATE_2026-04-28.md](../evidence/SPEC88_ROLLOUT_GATE_2026-04-28.md) | Spec88 Rollout Gate — Operator Docs and Skill Update |
| [docs/evidence/SPEC88_WORKPOINT_CONTRACT_MATRIX_2026-04-28.md](../evidence/SPEC88_WORKPOINT_CONTRACT_MATRIX_2026-04-28.md) | Spec88 Workpoint Contract Matrix |
| [docs/evidence/SPEC89_FOCUSA_TOOL_INVENTORY_2026-04-28.md](../evidence/SPEC89_FOCUSA_TOOL_INVENTORY_2026-04-28.md) | Spec89 Focusa Pi Tool Inventory — 2026-04-28 |
| [docs/evidence/SPEC89_LIVE_TOOL_BASELINE_2026-04-28.md](../evidence/SPEC89_LIVE_TOOL_BASELINE_2026-04-28.md) | Spec89 Live Focusa Tool Baseline — 2026-04-28 |
| [docs/evidence/SPEC89_PHASE0_COMPLETION_MATRIX_2026-04-28.md](../evidence/SPEC89_PHASE0_COMPLETION_MATRIX_2026-04-28.md) | Spec89 Phase 0 Completion Matrix — 2026-04-28 |
| [docs/evidence/SPEC89_PHASE1_ENVELOPE_EVIDENCE_2026-04-28.md](../evidence/SPEC89_PHASE1_ENVELOPE_EVIDENCE_2026-04-28.md) | Spec89 Phase 1 Unified Result Envelope Evidence — 2026-04-28 |
| [docs/evidence/SPEC89_PHASE2_WORKPOINT_SPINE_EVIDENCE_2026-04-28.md](../evidence/SPEC89_PHASE2_WORKPOINT_SPINE_EVIDENCE_2026-04-28.md) | Spec89 Phase 2 Workpoint Spine Evidence — 2026-04-28 |
| [docs/evidence/SPEC89_PHASE3_DOCTOR_RESOLVER_EVIDENCE_2026-04-28.md](../evidence/SPEC89_PHASE3_DOCTOR_RESOLVER_EVIDENCE_2026-04-28.md) | Spec89 Phase 3 Doctor/Resolver/Evidence Evidence — 2026-04-28 |
| [docs/evidence/SPEC89_PHASE4_WORK_LOOP_METACOG_EVIDENCE_2026-04-28.md](../evidence/SPEC89_PHASE4_WORK_LOOP_METACOG_EVIDENCE_2026-04-28.md) | Spec89 Phase 4 Work-loop UX and Metacog Quality Evidence — 2026-04-28 |
| [docs/evidence/SPEC89_PHASE5_DEDUPE_HYGIENE_EVIDENCE_2026-04-28.md](../evidence/SPEC89_PHASE5_DEDUPE_HYGIENE_EVIDENCE_2026-04-28.md) | Spec89 Phase 5 Dedupe and State Hygiene Evidence — 2026-04-28 |
| [docs/evidence/SPEC89_PHASE6_PICKUP_PARITY_STRESS_EVIDENCE_2026-04-28.md](../evidence/SPEC89_PHASE6_PICKUP_PARITY_STRESS_EVIDENCE_2026-04-28.md) | Spec89 Phase 6 Pickup, Parity, and Operational Stress Evidence — 2026-04-28 |
| [docs/evidence/SPEC89_PHASE7_CLOSURE_GUARDRAILS_EVIDENCE_2026-04-28.md](../evidence/SPEC89_PHASE7_CLOSURE_GUARDRAILS_EVIDENCE_2026-04-28.md) | Spec89 Phase 7 Closure and Maintenance Guardrails Evidence — 2026-04-28 |
| [docs/evidence/SPEC89_REAL_RELEASE_LIVE_PROOF_2026-04-28.md](../evidence/SPEC89_REAL_RELEASE_LIVE_PROOF_2026-04-28.md) | Spec89 Real Release Live Proof — 2026-04-28 |
| [docs/evidence/SPEC89_TOOL_CONTRACT_MATRIX_2026-04-28.md](../evidence/SPEC89_TOOL_CONTRACT_MATRIX_2026-04-28.md) | Spec89 / Spec55 Focusa Tool Contract Matrix — 2026-04-28 |
| [docs/evidence/SPEC89_TOOL_FAILURE_INVENTORY_2026-04-28.md](../evidence/SPEC89_TOOL_FAILURE_INVENTORY_2026-04-28.md) | Spec89 Tool Failure Inventory — 2026-04-28 |
| [docs/evidence/SPEC89_TOOL_RESULT_SCHEMA_MIGRATION_2026-04-28.md](../evidence/SPEC89_TOOL_RESULT_SCHEMA_MIGRATION_2026-04-28.md) | Spec89 Shared FocusaToolResult Schema and Migration Strategy — 2026-04-28 |
| [docs/evidence/SPEC89_TOOL_SPEC_MAPPING_2026-04-28.md](../evidence/SPEC89_TOOL_SPEC_MAPPING_2026-04-28.md) | Spec89 Tool-to-Spec Mapping — 2026-04-28 |
| [docs/evidence/SPEC89_URGENT_FIX_VALIDATION_2026-04-28.md](../evidence/SPEC89_URGENT_FIX_VALIDATION_2026-04-28.md) | Spec89 Urgent Focusa Tool Failure Fix Validation — 2026-04-28 |
| [docs/evidence/SPEC90_INITIAL_IMPLEMENTATION_2026-04-28.md](../evidence/SPEC90_INITIAL_IMPLEMENTATION_2026-04-28.md) | Spec90 Initial Implementation Evidence — 2026-04-28 |
| [docs/evidence/SPEC91_LIVE_TOOL_CONTRACT_PROOF_2026-04-28.md](../evidence/SPEC91_LIVE_TOOL_CONTRACT_PROOF_2026-04-28.md) | Spec91 Live Tool Contract Proof — 2026-04-28 |
| [docs/evidence/SPEC92_FULL_ROLLOUT_PROOF_2026-04-28.md](../evidence/SPEC92_FULL_ROLLOUT_PROOF_2026-04-28.md) | Spec92 Full Rollout Proof — 2026-04-28 |
| [docs/evidence/SPEC93_NON_PI_AWARENESS_ROLLOUT_PROOF_2026-04-29.md](../evidence/SPEC93_NON_PI_AWARENESS_ROLLOUT_PROOF_2026-04-29.md) | Spec93 Non-Pi Awareness Rollout Proof — 2026-04-29 |
| [docs/evidence/SPEC94_95_COMPLETION_PROOF_2026-05-03.md](../evidence/SPEC94_95_COMPLETION_PROOF_2026-05-03.md) | Spec94/95 Completion Proof — 2026-05-03 |
| [docs/evidence/SPEC94_95_LATENCY_RESPONSE_PROOF_2026-05-03.md](../evidence/SPEC94_95_LATENCY_RESPONSE_PROOF_2026-05-03.md) | Spec94/95 Latency, Response Size, and RSS Proof — 2026-05-03 |
| [docs/evidence/SPEC94_95_RUNTIME_GATES_2026-05-03.md](../evidence/SPEC94_95_RUNTIME_GATES_2026-05-03.md) | Spec94/95 Runtime Gates — 2026-05-03 |
| [docs/evidence/SPEC94_95_SECOND_SKEPTICAL_AUDIT_2026-05-03.md](../evidence/SPEC94_95_SECOND_SKEPTICAL_AUDIT_2026-05-03.md) | Spec94/95 Second Skeptical Audit — 2026-05-03 |
| [docs/evidence/SPEC94_95_SKEPTICAL_GAP_AUDIT_2026-05-03.md](../evidence/SPEC94_95_SKEPTICAL_GAP_AUDIT_2026-05-03.md) | Spec94/95 Skeptical Gap Audit — 2026-05-03 |
| [docs/evidence/SPEC94_H1_HEAP_PROFILING_ENVIRONMENT_LIMITATION_2026-05-04.md](../evidence/SPEC94_H1_HEAP_PROFILING_ENVIRONMENT_LIMITATION_2026-05-04.md) | SPEC94 H1 Environment Limitation: Heap Profiling Tools |
| [docs/evidence/SPEC96_CRITICAL_GAP_AUDIT_2026-05-21.md](../evidence/SPEC96_CRITICAL_GAP_AUDIT_2026-05-21.md) | Spec96 Critical Implementation Gap Audit — 2026-05-21 |
| [docs/evidence/SPEC96_CRITICAL_GAP_AUDIT_RECHECK_2026-05-21.md](../evidence/SPEC96_CRITICAL_GAP_AUDIT_RECHECK_2026-05-21.md) | Spec96 Critical Gap Audit Recheck — 2026-05-21 post-compaction |
| [docs/evidence/SPEC96_FOCUS_SLICE_AND_WORK_LOOP_ROUTE_REPAIR_2026-05-21.md](../evidence/SPEC96_FOCUS_SLICE_AND_WORK_LOOP_ROUTE_REPAIR_2026-05-21.md) | Spec96 Focus Slice + Work-loop Route Repair Proof — 2026-05-21 |
| [docs/evidence/SPEC96_HARDENING_GAPS_CLOSED_2026-05-21.md](../evidence/SPEC96_HARDENING_GAPS_CLOSED_2026-05-21.md) | Spec96 Hardening Gaps Closed — 2026-05-21 |
| [docs/evidence/SPEC96_LOWMEM_SURGICAL_AGENT_STRESS.md](../evidence/SPEC96_LOWMEM_SURGICAL_AGENT_STRESS.md) | Spec96 LowMem Surgical-Agent Stress |
| [docs/evidence/SPEC96_TRAJECTORY_AGENT_GOLDEN_EVALS.md](../evidence/SPEC96_TRAJECTORY_AGENT_GOLDEN_EVALS.md) | Spec96 Trajectory Agent Golden Evals |
| [docs/evidence/SPEC96_TRAVERSAL_BUDGET_GOLDEN_EVALS.md](../evidence/SPEC96_TRAVERSAL_BUDGET_GOLDEN_EVALS.md) | Spec96 Traversal Budget Golden Evals |
| [docs/evidence/SPEC96_TRAVERSE_RESUME_V2_GOLDEN_EVALS.md](../evidence/SPEC96_TRAVERSE_RESUME_V2_GOLDEN_EVALS.md) | Spec96 Traverse + Resume Packet v2 Golden Evals |
| [docs/evidence/SPEC97_REFLEX_DIRECT_API_LIVE_PROOF_2026-05-25.md](../evidence/SPEC97_REFLEX_DIRECT_API_LIVE_PROOF_2026-05-25.md) | SPEC97 Reflex Direct API Live Proof — 2026-05-25 |
| [docs/evidence/SPEC_INTENT_VS_ACTUAL_CODE_RUNTIME_GAP_AUDIT_2026-04-23.md](../evidence/SPEC_INTENT_VS_ACTUAL_CODE_RUNTIME_GAP_AUDIT_2026-04-23.md) | Spec Intent vs Actual Code/Runtime Gap Audit — 2026-04-23 |
| [docs/evidence/STRICT_SPEC_GATE_CI_PROOF_2026-05-25.md](../evidence/STRICT_SPEC_GATE_CI_PROOF_2026-05-25.md) | Strict Spec Gate CI Proof — 2026-05-25 |
| [docs/evidence/doc78-production-runtime-series-2026-04-18-run1/series-summary.md](../evidence/doc78-production-runtime-series-2026-04-18-run1/series-summary.md) | Doc78 Production Runtime Series Summary |
| [docs/evidence/doc78-production-runtime-series-2026-04-18-run3/series-summary.md](../evidence/doc78-production-runtime-series-2026-04-18-run3/series-summary.md) | Doc78 Production Runtime Series Summary |
| [docs/evidence/doc78-production-runtime-series-2026-04-18-run4/series-summary.md](../evidence/doc78-production-runtime-series-2026-04-18-run4/series-summary.md) | Doc78 Production Runtime Series Summary |
| [docs/evidence/doc78-production-runtime-series-latest/series-summary.md](../evidence/doc78-production-runtime-series-latest/series-summary.md) | Doc78 Production Runtime Series Summary |

### Audits, gap maps, decomposition, status

| Doc | Title / role |
| --- | --- |
| [docs/AGENT_AUDIT_SPEC.md](../AGENT_AUDIT_SPEC.md) | Agent Audit Spec — Live Cognitive Dashboard Upgrade |
| [docs/API_SPEC_COMPLIANCE_AUDIT_2026-02-28.md](../API_SPEC_COMPLIANCE_AUDIT_2026-02-28.md) | Focusa API Spec Compliance Audit — 2026-02-28 (Post-remediation) |
| [docs/DECOMPOSITION_ARTIFACT_INDEX_2026-04-13.md](../DECOMPOSITION_ARTIFACT_INDEX_2026-04-13.md) | Decomposition Artifact Index — 2026-04-13 |
| [docs/DECOMPOSITION_COMPLETENESS_CHECKPOINT_2026-04-13.md](../DECOMPOSITION_COMPLETENESS_CHECKPOINT_2026-04-13.md) | Decomposition Completeness Checkpoint — 2026-04-13 |
| [docs/DECOMPOSITION_OVERLAP_AND_GAP_REVIEW_2026-04-13.md](../DECOMPOSITION_OVERLAP_AND_GAP_REVIEW_2026-04-13.md) | Decomposition Overlap and Gap Review — 2026-04-13 |
| [docs/DOC78_AUTONOMY_OVERLAP_REVIEW_2026-04-13.md](../DOC78_AUTONOMY_OVERLAP_REVIEW_2026-04-13.md) | Doc 78 Autonomy Overlap Review — 2026-04-13 |
| [docs/DOC78_SECONDARY_COGNITION_CALLSITE_AUDIT_2026-04-13.md](../DOC78_SECONDARY_COGNITION_CALLSITE_AUDIT_2026-04-13.md) | Doc 78 Secondary/Background Cognition Call-Site Audit — 2026-04-13 |
| [docs/FOCUSA_EXHAUSTIVE_SPEC_AUDIT.md](../FOCUSA_EXHAUSTIVE_SPEC_AUDIT.md) | Focusa Exhaustive Spec Audit |
| [docs/FOCUSA_TOOL_SURFACE_COMPLIANCE_MATRIX_2026-04-15.md](../FOCUSA_TOOL_SURFACE_COMPLIANCE_MATRIX_2026-04-15.md) | Focusa Tool Surface Compliance Matrix — 2026-04-15 |
| [docs/IMPLEMENTATION_CUTOFF_AUDIT_2026-04-13.md](../IMPLEMENTATION_CUTOFF_AUDIT_2026-04-13.md) | Implementation Cutoff Audit — 2026-04-13 |
| [docs/IMPLEMENTATION_STATUS_MATRIX_2026-04-13.md](../IMPLEMENTATION_STATUS_MATRIX_2026-04-13.md) | Implementation Status Matrix — Docs 51-78 |
| [docs/LEGACY_BEAD_RECONCILIATION_2026-04-13.md](../LEGACY_BEAD_RECONCILIATION_2026-04-13.md) | Legacy Bead Reconciliation — 2026-04-13 |
| [docs/ONTOLOGY_ADDENDA_IMPLEMENTATION_MATRIX_2026-04-12.md](../ONTOLOGY_ADDENDA_IMPLEMENTATION_MATRIX_2026-04-12.md) | Ontology Addenda Implementation Matrix — 2026-04-12 |
| [docs/ONTOLOGY_SPEC_CODE_GAP_AUDIT_2026-04-16.md](../ONTOLOGY_SPEC_CODE_GAP_AUDIT_2026-04-16.md) | Ontology Spec ↔ Code Gap Audit (Direct Read, No Summary Proxy) |
| [docs/POST_CUTOFF_DECOMPOSITION_PLAN_2026-04-13.md](../POST_CUTOFF_DECOMPOSITION_PLAN_2026-04-13.md) | Post-Cutoff Decomposition Plan — 2026-04-13 |
| [docs/PRE78_ONTOLOGY_PARITY_STATUS_2026-04-18.md](../PRE78_ONTOLOGY_PARITY_STATUS_2026-04-18.md) | Pre-78 Ontology Parity Status — 2026-04-18 (updated) |
| [docs/PROOF_SURFACE_REQUIREMENTS_2026-04-13.md](../PROOF_SURFACE_REQUIREMENTS_2026-04-13.md) | Proof Surface Requirements — 2026-04-13 |
| [docs/REBASELINE_SINGLE_WRITER_SUMMARY_2026-04-13.md](../REBASELINE_SINGLE_WRITER_SUMMARY_2026-04-13.md) | Rebaseline Single-Writer Summary — 2026-04-13 |
| [docs/SHARED_SUBSTRATE_CONSUMER_PROOF_EXPANSION_2026-04-13.md](../SHARED_SUBSTRATE_CONSUMER_PROOF_EXPANSION_2026-04-13.md) | Shared-Substrate Consumer/Proof Expansion — 2026-04-13 |
| [docs/SPEC88_IMPLEMENTATION_DECOMPOSITION_2026-04-28.md](../SPEC88_IMPLEMENTATION_DECOMPOSITION_2026-04-28.md) | Spec 88 Implementation Decomposition — Ontology-backed Workpoint Continuity |
| [docs/TRUST_RESTORATION_AUDIT_2026-04-12.md](../TRUST_RESTORATION_AUDIT_2026-04-12.md) | Trust Restoration Audit — 2026-04-12 |

### Gen1 detail specs

| Doc | Title / role |
| --- | --- |
| [docs/G1-07-ascc.md](../G1-07-ascc.md) | docs/07-ascc.md — Anchored Structured Context Checkpointing (ASCC) (MVP) |
| [docs/G1-09-memory.md](../G1-09-memory.md) | docs/09-memory.md — Minimal Memory (Semantic + Procedural) (MVP) |
| [docs/G1-10-workers.md](../G1-10-workers.md) | docs/10-workers.md — Background Workers & Async Cognition (MVP) |
| [docs/G1-12-api.md](../G1-12-api.md) | docs/12-api.md — Local API (HTTP) Specification (MVP) |
| [docs/G1-13-cli.md](../G1-13-cli.md) | docs/13-cli.md — Focusa CLI Contract (MVP) |
| [docs/G1-14-reflection-loop.md](../G1-14-reflection-loop.md) | docs/G1-14-reflection-loop.md — Reflection Loop Overlay (Policy-Safe Meta-Cognition) |
| [docs/G1-16-testing.md](../G1-16-testing.md) | docs/16-testing.md — Testing & Acceptance (MVP) |
| [docs/G1-detail-00-doc-suite-readme.md](../G1-detail-00-doc-suite-readme.md) | docs/00-README.md — Focusa MVP Documentation Suite |
| [docs/G1-detail-03-runtime-daemon.md](../G1-detail-03-runtime-daemon.md) | docs/03-runtime-daemon.md — Daemon Runtime, State, and Persistence |
| [docs/G1-detail-04-proxy-adapter.md](../G1-detail-04-proxy-adapter.md) | docs/04-proxy-adapter.md — Proxy Mode & Harness-Agnostic Integration |
| [docs/G1-detail-05-focus-stack-hec.md](../G1-detail-05-focus-stack-hec.md) | docs/05-focus-stack-hec.md — Focus Stack (HEC) Specification (MVP) |
| [docs/G1-detail-06-focus-gate.md](../G1-detail-06-focus-gate.md) | docs/06-focus-gate.md — Focus Gate (RAS-Inspired) Specification (MVP) |
| [docs/G1-detail-08-ecs.md](../G1-detail-08-ecs.md) | docs/08-ecs.md — Externalized Context Store (ECS) & Handles (MVP) |
| [docs/G1-detail-11-prompt-assembly.md](../G1-detail-11-prompt-assembly.md) | docs/11-prompt-assembly.md — Prompt Assembly & Token Discipline (MVP) |
| [docs/G1-detail-15-events-observability.md](../G1-detail-15-events-observability.md) | docs/15-events-observability.md — Event Schema, Logging, and Tracing |
| [docs/G1-detail-PRD-gen2-intermediate.md](../G1-detail-PRD-gen2-intermediate.md) | PRD.md — Focusa Product Requirements Snapshot |

### Other supporting docs

| Doc | Title / role |
| --- | --- |
| [docs/BRANCH_ACCEPTANCE_CRITERIA_2026-04-13.md](../BRANCH_ACCEPTANCE_CRITERIA_2026-04-13.md) | Branch Acceptance Criteria — 2026-04-13 |
| [docs/DOC61_TRUTHFUL_CORE_COGNITION_SUBSTRATE_FRONTIER_2026-04-16.md](../DOC61_TRUTHFUL_CORE_COGNITION_SUBSTRATE_FRONTIER_2026-04-16.md) | Doc 61 Truthful Core Cognition Substrate Frontier — 2026-04-16 |
| [docs/DOC78_F1_F5_CLOSURE_SCORECARD_2026-04-17.md](../DOC78_F1_F5_CLOSURE_SCORECARD_2026-04-17.md) | Doc 78 F1-F5 Closure Scorecard — 2026-04-17 |
| [docs/DOC78_REMAINING_IMPLEMENTATION_FRONTIER_2026-04-16.md](../DOC78_REMAINING_IMPLEMENTATION_FRONTIER_2026-04-16.md) | Doc 78 Remaining Implementation Frontier — 2026-04-16 |
| [docs/FIRST_CONSUMER_CANDIDATES_2026-04-13.md](../FIRST_CONSUMER_CANDIDATES_2026-04-13.md) | First Consumer Candidates — 2026-04-13 |
| [docs/GPT5_4_UNIFIED_INTELLIGENCE_INTEGRATION_SPEC.md](../GPT5_4_UNIFIED_INTELLIGENCE_INTEGRATION_SPEC.md) | GPT5.4 Unified Intelligence Integration Spec |
| [docs/INTEGRATION_SPEC.md](../INTEGRATION_SPEC.md) | Wirebot Unified Intelligence System — Integration Spec |
| [docs/MEMORY_EXTRACTION_PIPELINE_SPEC.md](../MEMORY_EXTRACTION_PIPELINE_SPEC.md) | Memory Extraction Pipeline Spec — Go Code Fixes |
| [docs/MEMORY_SYNCD_INTEGRATION_SPEC.md](../MEMORY_SYNCD_INTEGRATION_SPEC.md) | Memory-Syncd Integration Spec — Focusa Bridge + SOUL Watch |
| [docs/POST_CUTOFF_CODE_LOCUS_MAP_2026-04-13.md](../POST_CUTOFF_CODE_LOCUS_MAP_2026-04-13.md) | Post-Cutoff Code Locus Map — 2026-04-13 |
| [docs/POST_CUTOFF_DOC_TO_BEAD_MAP_2026-04-13.md](../POST_CUTOFF_DOC_TO_BEAD_MAP_2026-04-13.md) | Post-Cutoff Doc → Bead Map — 2026-04-13 |
| [docs/POST_CUTOFF_SPARSE_LOCUS_NOTES_2026-04-13.md](../POST_CUTOFF_SPARSE_LOCUS_NOTES_2026-04-13.md) | Post-Cutoff Sparse Locus Notes — 2026-04-13 |
| [docs/PREREQUISITE_AND_BLOCKING_MAP_2026-04-13.md](../PREREQUISITE_AND_BLOCKING_MAP_2026-04-13.md) | Prerequisite and Blocking Map — 2026-04-13 |
| [docs/SPEC89_HARDENED_FOCUSA_TOOL_OPERATOR_GUIDE_2026-04-28.md](../SPEC89_HARDENED_FOCUSA_TOOL_OPERATOR_GUIDE_2026-04-28.md) | Spec89 Hardened Focusa Tool Operator Guide — 2026-04-28 |
| [docs/THIN_DOC_REFINEMENT_PASS_2026-04-13.md](../THIN_DOC_REFINEMENT_PASS_2026-04-13.md) | Thin-Doc Refinement Pass — 2026-04-13 |
| [docs/UNDER_SPECIFIED_DOC_CHECK_2026-04-13.md](../UNDER_SPECIFIED_DOC_CHECK_2026-04-13.md) | Under-Specified Doc Check — 2026-04-13 |
| [docs/UNIFIED_ORGANISM_SPEC.md](../UNIFIED_ORGANISM_SPEC.md) | Wirebot Unified Organism Spec |
| [docs/WIKI_AGENT_SPEC.md](../WIKI_AGENT_SPEC.md) | Wiki-Agent Spec — Autonomous Knowledge Graph Maintenance |
| [docs/WIKI_ENRICH_NIGHTLY_SPEC.md](../WIKI_ENRICH_NIGHTLY_SPEC.md) | Wiki Enrichment Nightly Spec — Upgrade |
| [docs/WORK_MODE_NOTE_2026-04-13.md](../WORK_MODE_NOTE_2026-04-13.md) | Work Mode Note — 2026-04-13 |
| [docs/addendum-gaps.md](../addendum-gaps.md) | Focusa Spec Audit — ADDENDUM SPECS (45-57) |
| [docs/bootstrap-prompt-rust.md](../bootstrap-prompt-rust.md) | Rust-First Engineer Agent Bootstrap Prompt — Focusa MVP |
| [docs/bootstrap-prompt.md](../bootstrap-prompt.md) | Engineer Agent Bootstrap Prompt — Focusa MVP |
| [docs/core-reducer.md](../core-reducer.md) | Focusa-Core Reducer — Canonical Pseudocode Spec (AUTHORITATIVE) |
| [docs/gaps.md](../gaps.md) | Focusa Spec Audit — PARTIAL / REVALIDATED |

