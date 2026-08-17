# Spec141 Focusa Agent Capability Reference

Registry digest: `sha256:84e9b49b79c7109b6e47bc2ec2fd25f0fdc11d11928648609152f855ae8922d0`

This file is generated. Use the descriptor registry for complete strict schemas and machine metadata.

## Operator alignment contract

- refresh preferred address, timezone, local time, goals, constraints, desired pace, and canonical operator state before meaningful work or after long gaps
- treat cwd as launch location only; never infer project identity, binding consent, or new-user status from cwd, missing trajectory, or a missing marker
- consider legacy Focusa projects through git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting project creation
- use progressive disclosure and plain language; keep packet ids, hierarchy labels, tool routes, and internal recovery mechanics private unless requested
- never invent deadlines or urgency; ground consequential time claims in temporal authority and express uncertainty as a range
- for meaningful tasks record wall-clock start, predict human-readable delivery, observe actual duration, evaluate the prediction, and retain reusable timing lessons
- use Focusa capabilities to achieve the operator's desired outcome within operator constraints rather than making Focusa itself the center of conversation

## focusa_active_object_resolve

Resolve likely active object references from the current Workpoint and optional hint without inventing canonical refs. Use it when Resolve likely active object references from the current Workpoint and optional hint without inventing canonical refs. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.active.object.resolve`
- Family: `workpoint`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_workpoint_checkpoint`, `focusa_evidence_capture`, `focusa_traverse`
- Documentation: `docs/focusa-tools/tools/focusa_active_object_resolve.md`

## focusa_agent_artifact_delivery

Commit verified agent artifacts with explicit operator confirmation and a durable Receipt reference. Use it when Operate the Spec 140 agent artifact delivery surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.agent.artifact.delivery`
- Family: `agent_runtime`
- Side effects: `confirmed_receipted_artifact_delivery`, `confirmed_receipted_artifact_delivery`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Dependencies/next: `focusa_agent_artifact_verify`, `focusa_instruction_integrity_status`
- Documentation: `docs/focusa-tools/tools/focusa_agent_artifact_delivery.md`

## focusa_agent_artifact_preview

Preview a Spec 140 artifact delivery manifest; never writes files. Use it when Operate the Spec 140 agent artifact preview surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.agent.artifact.preview`
- Family: `agent_runtime`
- Side effects: `read_or_preview_only`, `read_or_preview_only`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Dependencies/next: `focusa_agent_artifact_delivery`, `focusa_agent_artifact_verify`
- Documentation: `docs/focusa-tools/tools/focusa_agent_artifact_preview.md`

## focusa_agent_artifact_verify

Verify content hashes and evidence for a Runtime Constitution delivery manifest. Use it when Operate the Spec 140 agent artifact verify surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.agent.artifact.verify`
- Family: `agent_runtime`
- Side effects: `read_or_preview_only`, `read_or_preview_only`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Dependencies/next: `focusa_agent_runtime_effective`, `focusa_agent_runtime_doctor`
- Documentation: `docs/focusa-tools/tools/focusa_agent_artifact_verify.md`

## focusa_agent_card

Read a compact, versioned Focusa Agent Card for cross-harness discovery. Returns interfaces, auth methods, progressive-discovery entry points, capability families, registry digest guidance, and extended-card routes without loading full schemas. Use it when Read compact cross-harness interfaces, auth, capabilities, families, and discovery entry points. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.agent.card`
- Family: `awareness`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-agent-bootstrap`
- Dependencies/next: `focusa_tool_search`, `focusa_tool_bundle`, `focusa_project_identity`
- Documentation: `docs/focusa-tools/tools/focusa_agent_card.md`

## focusa_agent_prompt

Read canonical Pi guidance; prefer focusa_* tools over raw daemon calls. Use it when Retrieve the Pi-aware daemon reminder and canonical tool-layer guidance to prevent raw curl/fetch drift. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.agent.prompt`
- Family: `focus_state`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_tool_doctor`, `focusa_trajectory_view`, `focusa_project_identity`
- Documentation: `docs/focusa-tools/tools/focusa_agent_prompt.md`

## focusa_agent_runtime_doctor

Diagnose Runtime Constitution compiler defaults, replacement gates, and delivery readiness. Use it when Operate the Spec 140 agent runtime doctor surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.agent.runtime.doctor`
- Family: `agent_runtime`
- Side effects: `read_or_preview_only`, `read_or_preview_only`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Dependencies/next: `focusa_agent_runtime_effective`, `focusa_instruction_integrity_status`
- Documentation: `docs/focusa-tools/tools/focusa_agent_runtime_doctor.md`

## focusa_agent_runtime_effective

Read effective project instruction claims and unresolved conflicts under Spec 140. Use it when Operate the Spec 140 agent runtime effective surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.agent.runtime.effective`
- Family: `agent_runtime`
- Side effects: `read_or_preview_only`, `read_or_preview_only`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Dependencies/next: `focusa_instruction_sources`, `focusa_instruction_conflicts`, `focusa_instruction_integrity_status`
- Documentation: `docs/focusa-tools/tools/focusa_agent_runtime_effective.md`

## focusa_agent_runtime_headless_verify

Verify foundational runtime capability parity without Mission Canvas or generated UI availability. Use it when Operate the Spec 140 agent runtime headless verify surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.agent.runtime.headless.verify`
- Family: `agent_runtime`
- Side effects: `confirmed_receipted_artifact_delivery`, `confirmed_receipted_artifact_delivery`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Dependencies/next: `focusa_instruction_integrity_status`, `focusa_agent_runtime_doctor`
- Documentation: `docs/focusa-tools/tools/focusa_agent_runtime_headless_verify.md`

## focusa_awareness_packet

Render a surface-aware AwarenessPacket with DVS-scored visible lines, suppressed lines, metadata, next_tools, and recovery_tools, including Spec 111 preload status surfaces. Use it when Render a surface-aware AwarenessPacket with DVS-scored visible lines, suppressed lines, metadata, next_tools, and recovery_tools. Use on reload, post-compaction, tool guidance, warning, or UIAI bridge surfaces. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.awareness.packet`
- Family: `awareness`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-agent-bootstrap`
- Dependencies/next: `focusa_workpoint_resume`, `focusa_trajectory_view`, `focusa_tool_doctor`
- Documentation: `docs/focusa-tools/tools/focusa_awareness_packet.md`

## focusa_bg_run

Run a terminal-blocking command in the background as a first-class Focusa job. The daemon records the job durably; on completion the agent's front terminal receives the completion notification with a bounded output tail (no polling). Canonical TBQ dispatch primitive — use instead of raw setsid/nohup shells whenever the Focusa daemon is up. Use it when Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.bg.run`
- Family: `background_job`
- Side effects: `durable_dispatch`, `durable_dispatch`
- Skills: `skill:focusa`, `skill:focusa-silent-sessions`
- Dependencies/next: `focusa_bg_status`, `focusa_workpoint_checkpoint`
- Documentation: `docs/focusa-tools/tools/focusa_bg_run.md`

## focusa_bg_run_many

Dispatch multiple terminal-blocking jobs in parallel as first-class Focusa jobs. Each job completes independently and delivers its completion notification (with bounded output tail) to the agent front terminal via SSE — the orchestration primitive for parallel builds, test shards, and multi-step pipelines. Returns the job ledger immediately; never blocks. Use it when Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.bg.run.many`
- Family: `background_job`
- Side effects: `durable_dispatch`, `durable_dispatch`
- Skills: `skill:focusa`, `skill:focusa-silent-sessions`
- Dependencies/next: `focusa_bg_status`, `focusa_workpoint_checkpoint`
- Documentation: `docs/focusa-tools/tools/focusa_bg_run_many.md`

## focusa_bg_status

Instant single-query status for Focusa background jobs (bg list / bg status). Use for at-a-glance state; the completion notification is the primary delivery path. Never use in a polling loop. Use it when Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.bg.status`
- Family: `background_job`
- Side effects: `read_status`, `read_status`
- Skills: `skill:focusa`, `skill:focusa-silent-sessions`
- Dependencies/next: `focusa_bg_run`, `focusa_workpoint_resume`
- Documentation: `docs/focusa-tools/tools/focusa_bg_status.md`

## focusa_bloatgaurd_domain

Spec 101 — read one Bloatgaurd budget domain and its checks/findings. Use it when Spec 101 — read one Bloatgaurd budget domain and its checks/findings. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.bloatgaurd.domain`
- Family: `diagnostics_hygiene`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`, `skill:focusa-resource-performance`
- Dependencies/next: `focusa_bloatgaurd_report`, `focusa_traverse`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_bloatgaurd_domain.md`

## focusa_bloatgaurd_gate_mode

Spec 101 — read one Bloatgaurd gate mode by code/name. Use it when Spec 101 — read one Bloatgaurd gate mode by code/name. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.bloatgaurd.gate.mode`
- Family: `diagnostics_hygiene`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`, `skill:focusa-resource-performance`
- Dependencies/next: `focusa_bloatgaurd_gate_modes`, `focusa_traverse`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_bloatgaurd_gate_mode.md`

## focusa_bloatgaurd_gate_modes

Spec 101 — read gate modes A/B/C thresholds, allowlist, and report schema. Use it when Spec 101 — read gate modes A/B/C thresholds, allowlist, and report schema. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.bloatgaurd.gate.modes`
- Family: `diagnostics_hygiene`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`, `skill:focusa-resource-performance`
- Dependencies/next: `focusa_bloatgaurd_gate_mode`, `focusa_bloatgaurd_report`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_bloatgaurd_gate_modes.md`

## focusa_bloatgaurd_profile

Spec 101 — read one profile preset by name. Use it when Spec 101 — read one profile preset by name. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.bloatgaurd.profile`
- Family: `diagnostics_hygiene`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`, `skill:focusa-resource-performance`
- Dependencies/next: `focusa_bloatgaurd_profiles`, `focusa_bloatgaurd_routines`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_bloatgaurd_profile.md`

## focusa_bloatgaurd_profiles

Spec 101 — read profile presets and operator switches. Use it when Spec 101 — read profile presets and operator switches. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.bloatgaurd.profiles`
- Family: `diagnostics_hygiene`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`, `skill:focusa-resource-performance`
- Dependencies/next: `focusa_bloatgaurd_profile`, `focusa_bloatgaurd_routines`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_bloatgaurd_profiles.md`

## focusa_bloatgaurd_report

Spec 101 — read the compact Bloatgaurd budget report for domains 5.1-5.8. Use it when Spec 101 — read the compact Bloatgaurd budget report for domains 5.1-5.8. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.bloatgaurd.report`
- Family: `diagnostics_hygiene`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`, `skill:focusa-resource-performance`
- Dependencies/next: `focusa_bloatgaurd_domain`, `focusa_context_cognition_render`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_bloatgaurd_report.md`

## focusa_bloatgaurd_rollout

Spec 101 — read rollout phases, acceptance checks, and proof commands. Use it when Spec 101 — read rollout phases, acceptance checks, and proof commands. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.bloatgaurd.rollout`
- Family: `diagnostics_hygiene`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`, `skill:focusa-resource-performance`
- Dependencies/next: `focusa_bloatgaurd_profiles`, `focusa_bloatgaurd_routines`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_bloatgaurd_rollout.md`

## focusa_bloatgaurd_routine

Spec 101 — read one named routine by name. Use it when Spec 101 — read one named routine by name. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.bloatgaurd.routine`
- Family: `diagnostics_hygiene`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`, `skill:focusa-resource-performance`
- Dependencies/next: `focusa_bloatgaurd_routines`, `focusa_bloatgaurd_profiles`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_bloatgaurd_routine.md`

## focusa_bloatgaurd_routines

Spec 101 — read named routines and automation matrix. Use it when Spec 101 — read named routines and automation matrix. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.bloatgaurd.routines`
- Family: `diagnostics_hygiene`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`, `skill:focusa-resource-performance`
- Dependencies/next: `focusa_bloatgaurd_routine`, `focusa_bloatgaurd_profiles`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_bloatgaurd_routines.md`

## focusa_bloatgaurd_tokenbloat_domain

Spec 101 — read one Tokenbloat Control domain and its prompt-visible fields/boundaries. Use it when Spec 101 — read one Tokenbloat Control domain and its prompt-visible fields/boundaries. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.bloatgaurd.tokenbloat.domain`
- Family: `diagnostics_hygiene`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`, `skill:focusa-resource-performance`
- Dependencies/next: `focusa_bloatgaurd_tokenbloat_report`, `focusa_traverse`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_bloatgaurd_tokenbloat_domain.md`

## focusa_bloatgaurd_tokenbloat_report

Spec 101 — read Tokenbloat Control report for domains 5.9-5.10. Use it when Spec 101 — read Tokenbloat Control report for domains 5.9-5.10. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.bloatgaurd.tokenbloat.report`
- Family: `diagnostics_hygiene`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`, `skill:focusa-resource-performance`
- Dependencies/next: `focusa_bloatgaurd_tokenbloat_domain`, `focusa_bloatgaurd_report`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_bloatgaurd_tokenbloat_report.md`

## focusa_browser_capabilities_intake

Validate and govern a UIAI or WebMCP browser capability manifest. Binds page tools to one session and origin, treats page safety annotations as untrusted, requires confirmation/evidence for mutation, and returns Focusa browser capability descriptors without executing them. Use it when Validate and session/origin-bind UIAI or WebMCP page capabilities under Focusa governance. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.browser.capabilities.intake`
- Family: `diagnostics_hygiene`
- Side effects: `write_browser_capability_evidence`, `write_browser_capability_evidence`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`, `skill:focusa-browser-uiai`
- Dependencies/next: `focusa_browser_workflow_plan`, `focusa_browser_diagnostics_intake`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_browser_capabilities_intake.md`

## focusa_browser_diagnostics_intake

Turn UIAI/browser diagnostics JSON into bounded Focusa evidence, active-object hints, a prediction candidate, and a metacog candidate. Use it when Turn UIAI/browser diagnostics JSON into bounded Workpoint evidence, active-object hints, prediction context, and optional metacog learning. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.browser.diagnostics.intake`
- Family: `workpoint`
- Side effects: `composite_evidence_prediction_optional_metacog`, `composite_evidence_prediction_optional_metacog`
- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-browser-uiai`
- Dependencies/next: `focusa_active_object_resolve`, `focusa_evidence_capture`, `focusa_predict_record`, `focusa_metacog_capture`
- Documentation: `docs/focusa-tools/tools/focusa_browser_diagnostics_intake.md`

## focusa_browser_workflow_plan

Build the governed UIAI/WebMCP sequence for one browser operation before action. Returns health, read/source, diagnostics, snapshot refs, mutation confirmation, bound execution, evidence intake, Workpoint linkage, and session cleanup steps. Use it when Plan a governed UIAI/WebMCP read, action, diagnostics, evidence, and cleanup sequence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.browser.workflow.plan`
- Family: `diagnostics_hygiene`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`, `skill:focusa-browser-uiai`
- Dependencies/next: `focusa_browser_capabilities_intake`, `focusa_browser_diagnostics_intake`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_browser_workflow_plan.md`

## focusa_call_stack_design

Write a typed, append-only Call Stack Design for a feature before implementation. Returns the standard Focusa call stack scaffold (entry → handlers → services → adapters → storage → output) that the operator/agent fills in for the specific feature. Per Spec 103. Use it when Write a typed, append-only Call Stack Design for a feature before implementation. Returns the standard Focusa call stack scaffold. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.call.stack.design`
- Family: `workpoint`
- Side effects: `write_call_stack_design`, `write_call_stack_design`
- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-spec-implementation`
- Dependencies/next: `focusa_call_stack_verify`, `focusa_workpoint_link_evidence`, `focusa_trajectory_assess`
- Documentation: `docs/focusa-tools/tools/focusa_call_stack_design.md`

## focusa_call_stack_verify

Verify a Call Stack Design against bounded implementation surfaces and report drift: entry surface, handlers, services, adapters, storage, output envelope, evidence, and Workpoint/STG alignment. Advisory only. Use it when Verify a Call Stack Design against bounded implementation surfaces and report drift without mutating Focus State. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.call.stack.verify`
- Family: `workpoint`
- Side effects: `read_call_stack_design_verify_drift`, `read_call_stack_design_verify_drift`
- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-spec-implementation`
- Dependencies/next: `focusa_call_stack_design`, `focusa_workpoint_link_evidence`, `focusa_trajectory_assess`
- Documentation: `docs/focusa-tools/tools/focusa_call_stack_verify.md`

## focusa_callgraph_observe

Observe a CallGraph run: ledger row, dispatches, paths, and the deterministic replay frontier. Read-only. Use it when Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.callgraph.observe`
- Family: `callgraph`
- Side effects: `read_observation`, `read_observation`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Dependencies/next: `focusa_trajectory_view`, `focusa_workpoint_resume`
- Documentation: `docs/focusa-tools/tools/focusa_callgraph_observe.md`

## focusa_callgraph_validate

Validate a CallGraph definition against the Spec 155 structural rules (identity, endpoints, entries, joins, compensation, per-cycle policy). Pure + deterministic. Use it when Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.callgraph.validate`
- Family: `callgraph`
- Side effects: `read_validation`, `read_validation`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Dependencies/next: `focusa_callgraph_observe`, `focusa_tool_describe`
- Documentation: `docs/focusa-tools/tools/focusa_callgraph_validate.md`

## focusa_canonical_instruction_amendment_activate

Activate a separately operator-approved amendment only after its official documentation sweep is complete. Use it when Operate the Spec 140 canonical instruction amendment activate surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.canonical.instruction.amendment.activate`
- Family: `agent_runtime`
- Side effects: `confirmed_receipted_artifact_delivery`, `confirmed_receipted_artifact_delivery`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`, `skill:focusa-security-auth-licensing`
- Dependencies/next: `focusa_instruction_integrity_evaluate`, `focusa_agent_runtime_effective`
- Documentation: `docs/focusa-tools/tools/focusa_canonical_instruction_amendment_activate.md`

## focusa_canonical_instruction_amendment_propose

Record an operator-originated canonical instruction amendment proposal without activating it. Use it when Operate the Spec 140 canonical instruction amendment propose surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.canonical.instruction.amendment.propose`
- Family: `agent_runtime`
- Side effects: `confirmed_receipted_artifact_delivery`, `confirmed_receipted_artifact_delivery`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`, `skill:focusa-security-auth-licensing`
- Dependencies/next: `focusa_canonical_instruction_amendment_activate`, `focusa_instruction_integrity_evaluate`
- Documentation: `docs/focusa-tools/tools/focusa_canonical_instruction_amendment_propose.md`

## focusa_cockpit_projection

Read the whole flywheel in one bounded payload: workset summaries, open CallGraph run frontiers, direction steers, and the background-job board with ETAs. Read-only, ledger-backed; the hand-in-glove operator view. Use it when Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.cockpit.projection`
- Family: `cockpit`
- Side effects: `read_projection`, `read_projection`
- Skills: `skill:focusa`, `skill:focusa-work-loop`
- Dependencies/next: `focusa_workset_projection`, `focusa_bg_status`
- Documentation: `docs/focusa-tools/tools/focusa_cockpit_projection.md`

## focusa_constraint

Record a DISCOVERED REQUIREMENT in Focus State. Constraints are hard boundaries from environment/architecture — NOT self-imposed tasks. Max 200 chars. Use it when Record a DISCOVERED REQUIREMENT in Focus State. Constraints are hard boundaries from environment/architecture — NOT self-imposed tasks. Max 200 chars. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.constraint`
- Family: `focus_state`
- Side effects: `write_state`, `write_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_project_identity`, `focusa_trajectory_view`, `focusa_workpoint_resume`
- Documentation: `docs/focusa-tools/tools/focusa_constraint.md`

## focusa_context_cognition

Build the bounded, advisory Spec 100 ContextCognitionPacket for the current project. Returns a typed packet describing scope, authority, freshness, selected context, ontology frame, evidence frame, reasoning frame, optimization frame, and route frame. Never mutates state. Use it when Build the bounded, advisory Spec 100 ContextCognitionPacket for the current project. Never mutates state. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.context.cognition`
- Family: `trajectory`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-agent-bootstrap`
- Dependencies/next: `focusa_active_object_resolve`, `focusa_workpoint_checkpoint`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_context_cognition.md`

## focusa_context_cognition_curate

Spec 100 Phase 3 — token-budgeted context selection. Takes candidates (files/docs/diffs/snippets/codemaps/evidence) and selects the highest-scoring subset under a token budget. Returns selected_context + excluded_context (with reasons). Use it when Spec 100 Phase 3 — token-budgeted context selection. Ranks candidates by workpoint target + evidence overlap and selects the highest-scoring subset under a token budget. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.context.cognition.curate`
- Family: `trajectory`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-agent-bootstrap`
- Dependencies/next: `focusa_context_cognition`, `focusa_context_cognition_render`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_context_cognition_curate.md`

## focusa_context_cognition_curate_eval

Spec 100 Phase 4 — run a curator eval case. Computes precision/recall/F1 vs. expected_selected_paths. Appends to curator-eval-ledger/{hash}/eval-runs.jsonl. Returns run_id, eval_ref, scores, and promoted flag (F1 > baseline_f1 AND F1 >= score_threshold). Use it when Spec 100 Phase 4 — run a curator eval case, compute precision/recall/F1, append to curator-eval-ledger JSONL. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.context.cognition.curate.eval`
- Family: `trajectory`
- Side effects: `write_curator_eval`, `write_curator_eval`
- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-agent-bootstrap`
- Dependencies/next: `focusa_context_cognition_curate_optimize`, `focusa_metacog_capture`, `focusa_predict_record`
- Documentation: `docs/focusa-tools/tools/focusa_context_cognition_curate_eval.md`

## focusa_context_cognition_curate_optimize

Spec 100 Phase 5 — submit a Cognition Optimizer artifact and get the promote/rollback decision. Returns the decision per the §15 promotion rule (eval_score > baseline_score AND eval_score >= score_threshold). Appends to cognition-optimizer-artifacts/{hash}/artifacts.jsonl. Use it when Spec 100 Phase 5 — submit a Cognition Optimizer artifact and get the promote/rollback decision per §15 promotion rule. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.context.cognition.curate.optimize`
- Family: `trajectory`
- Side effects: `write_cognition_optimizer_artifact`, `write_cognition_optimizer_artifact`
- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-agent-bootstrap`
- Dependencies/next: `focusa_context_cognition_optimizer_artifacts`, `focusa_predict_record`, `focusa_metacog_capture`
- Documentation: `docs/focusa-tools/tools/focusa_context_cognition_curate_optimize.md`

## focusa_context_cognition_optimizer_artifacts

Spec 100 Phase 5 — list Cognition Optimizer artifacts (versioned JSONL) for a project+module. Returns the recent artifact list and the latest promoted artifact (if any). Use it when Spec 100 Phase 5 — list Cognition Optimizer artifacts (versioned JSONL) for a project+module. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.context.cognition.optimizer.artifacts`
- Family: `trajectory`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-agent-bootstrap`
- Dependencies/next: `focusa_context_cognition_curate_optimize`
- Documentation: `docs/focusa-tools/tools/focusa_context_cognition_optimizer_artifacts.md`

## focusa_context_cognition_proof

Map Spec 100 ContextCognitionPacket surfaces to proof commands (curl + focusa + audits). Returns bounded command list. Read-only. Use it when Map Spec 100 ContextCognitionPacket surfaces to proof commands (curl + focusa + audits). It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.context.cognition.proof`
- Family: `trajectory`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-agent-bootstrap`
- Dependencies/next: `focusa_context_cognition`, `focusa_context_cognition_render`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_context_cognition_proof.md`

## focusa_context_cognition_render

Render the Spec 100 ContextCognitionPacket as compact text (for prompt/CLI/menubar). Returns bounded lines + the packet's workpoint_id, trajectory_id, and rehydrate_id. Advisory only. Use it when Render the Spec 100 ContextCognitionPacket as compact text (for prompt/CLI/menubar). It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.context.cognition.render`
- Family: `trajectory`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-agent-bootstrap`
- Dependencies/next: `focusa_context_cognition`, `focusa_context_cognition_proof`
- Documentation: `docs/focusa-tools/tools/focusa_context_cognition_render.md`

## focusa_credentials_verify

Ask the Credential Authority whether a requirement is satisfied by the given grants — secret-free: the verdict and reasons only, never secret values. Use before touching any provider seam. Use it when Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.credentials.verify`
- Family: `credential`
- Side effects: `read_verdict`, `read_verdict`
- Skills: `skill:focusa`, `skill:focusa-security-auth-licensing`
- Dependencies/next: `focusa_credentials_verify`, `focusa_tool_doctor`
- Documentation: `docs/focusa-tools/tools/focusa_credentials_verify.md`

## focusa_current_focus

Update current focus — what you are actively working on right now (1-3 sentences, max 300 chars). Use it when Update current focus — what you are actively working on right now (1-3 sentences, max 300 chars). It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.current.focus`
- Family: `focus_state`
- Side effects: `write_state`, `write_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_trajectory_view`, `focusa_workpoint_checkpoint`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_current_focus.md`

## focusa_daemon_routing_status

Resolve one explicit project/worktree/continuity/native-session scope against a supplied daemon registry. Never infers a global or foreign daemon. Use it when Resolve one explicit project/worktree/continuity/native-session scope against a supplied daemon registry. Never infers a global or foreign daemon. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.daemon.routing.status`
- Family: `project_identity`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-project-scope`
- Dependencies/next: `focusa_project_identity`, `focusa_tool_doctor`, `focusa_workpoint_resume`
- Documentation: `docs/focusa-tools/tools/focusa_daemon_routing_status.md`

## focusa_decide

Record a crystallized architectural decision in Focus State. Use focusa_scratch for working notes first. Decisions are ONE sentence (<=160 chars) — architectural choices only, not task lists. Use it when Record a crystallized architectural decision in Focus State. Use focusa_scratch for working notes first. Decisions are ONE sentence (<=160 chars) — architectural choices only, not task lists. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.decide`
- Family: `focus_state`
- Side effects: `write_state`, `write_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_project_identity`, `focusa_trajectory_view`, `focusa_workpoint_resume`
- Documentation: `docs/focusa-tools/tools/focusa_decide.md`

## focusa_device_pair_complete

Complete a pending pairing (run on the VPS side; returns the long-lived token). Idempotent: re-running with the same code returns the original token. Use it when Mac menubar OAuth-like device pairing (focusa-ui0y). Run on the VPS side; returns the long-lived token. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.device.pair.complete`
- Family: `session_transfer`
- Side effects: `write_device_pair_complete`, `write_device_pair_complete`
- Skills: `skill:focusa`, `skill:focusa-session-recovery`, `skill:focusa-security-auth-licensing`
- Dependencies/next: `focusa_device_pair_status`, `focusa_device_pair_list`
- Documentation: `docs/focusa-tools/tools/focusa_device_pair_complete.md`

## focusa_device_pair_list

List paired devices for a host (append-only JSONL ledger, scope-bounded). Returns the recent device list with name, scopes, paired_at, last_seen_at, revoked. Use it when Mac menubar OAuth-like device pairing (focusa-ui0y). List paired devices for a host (append-only JSONL ledger). It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.device.pair.list`
- Family: `session_transfer`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-session-recovery`, `skill:focusa-security-auth-licensing`
- Dependencies/next: `focusa_device_pair_revoke`, `focusa_session_transfer`
- Documentation: `docs/focusa-tools/tools/focusa_device_pair_list.md`

## focusa_device_pair_qr

Mac menubar OAuth-like device pairing with QR handoff (Spec focusa-ui0y, Mode B). Calls /v1/device/pair/start and returns pair_url + pair_url_qr_payload prominently so the Mac menubar can render a QR the operator's phone can scan. Use it when Mac menubar OAuth-like device pairing with QR handoff (focusa-ui0y, Mode B). Same as pair_start but surfaces pair_url for QR rendering (Telegram/Discord-style). It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.device.pair.qr`
- Family: `session_transfer`
- Side effects: `write_device_pair`, `write_device_pair`
- Skills: `skill:focusa`, `skill:focusa-session-recovery`, `skill:focusa-security-auth-licensing`
- Dependencies/next: `focusa_device_pair_status`, `focusa_device_pair_list`
- Documentation: `docs/focusa-tools/tools/focusa_device_pair_qr.md`

## focusa_device_pair_revoke

Revoke a paired device. Appends a new entry with revoked=true to the append-only JSONL ledger and removes the in-memory token. The next call from the device will be rejected with status=revoked. Use it when Mac menubar OAuth-like device pairing (focusa-ui0y). Revoke a paired device; appends revoked=true to ledger. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.device.pair.revoke`
- Family: `session_transfer`
- Side effects: `write_device_pair_revoke`, `write_device_pair_revoke`
- Skills: `skill:focusa`, `skill:focusa-session-recovery`, `skill:focusa-security-auth-licensing`
- Dependencies/next: `focusa_device_pair_list`
- Documentation: `docs/focusa-tools/tools/focusa_device_pair_revoke.md`

## focusa_device_pair_start

Mac menubar OAuth-like device pairing (Spec focusa-ui0y). Generate an 8-char pairing code (FOCUS-XXXX-XXXX, 5 min TTL). The operator runs `focusa device pair-complete <code>` on their VPS, then the Mac app polls focusa_device_pair_status to retrieve the long-lived token (30 day TTL). Use it when Mac menubar OAuth-like device pairing (focusa-ui0y). Generate an 8-char code + pair_url for VPS-side completion via CLI, QR+phone, or QR+VPS browser. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.device.pair.start`
- Family: `session_transfer`
- Side effects: `write_device_pair`, `write_device_pair`
- Skills: `skill:focusa`, `skill:focusa-session-recovery`, `skill:focusa-security-auth-licensing`
- Dependencies/next: `focusa_device_pair_status`, `focusa_device_pair_list`, `focusa_device_pair_qr`
- Documentation: `docs/focusa-tools/tools/focusa_device_pair_start.md`

## focusa_device_pair_status

Check the status of a pending or completed pairing by code OR by device_id. Returns the token (when completed) + status + scopes + expires_at. Use it when Mac menubar OAuth-like device pairing (focusa-ui0y). Check pairing status by code or device_id. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.device.pair.status`
- Family: `session_transfer`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-session-recovery`, `skill:focusa-security-auth-licensing`
- Dependencies/next: `focusa_device_pair_list`, `focusa_device_pair_revoke`
- Documentation: `docs/focusa-tools/tools/focusa_device_pair_status.md`

## focusa_dxux_digest

Spec105 — read compact continuation/doability digest. Use it when Spec105 — read compact continuation/doability digest. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.dxux.digest`
- Family: `diagnostics_hygiene`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`
- Dependencies/next: `focusa_workpoint_resume`, `focusa_dxux_report`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_dxux_digest.md`

## focusa_dxux_explain

Spec105 — explain a failure and return recovery commands. Use it when Spec105 — explain a failure and return recovery commands. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.dxux.explain`
- Family: `diagnostics_hygiene`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`
- Dependencies/next: `focusa_dxux_report`, `focusa_tool_doctor`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_dxux_explain.md`

## focusa_dxux_report

Spec105 — read implementation report for DXUX-001..012. Use it when Spec105 — read implementation report for DXUX-001..012. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.dxux.report`
- Family: `diagnostics_hygiene`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`
- Dependencies/next: `focusa_dxux_requirement`, `focusa_dxux_digest`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_dxux_report.md`

## focusa_dxux_requirement

Spec105 — read one DXUX requirement by id. Use it when Spec105 — read one DXUX requirement by id. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.dxux.requirement`
- Family: `diagnostics_hygiene`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`
- Dependencies/next: `focusa_dxux_report`, `focusa_dxux_digest`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_dxux_requirement.md`

## focusa_epistemic_operation

Invoke one exact generated Spec 138/138A operation through durable typed API authority; the client never settles authority locally. Use it when Invoke one exact generated Spec 138/138A operation through durable typed API authority. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.epistemic.operation`
- Family: `metacognition`
- Side effects: `typed_read_or_canonical_epistemic_mutation`, `typed_read_or_canonical_epistemic_mutation`
- Skills: `skill:focusa`, `skill:focusa-metacognition`
- Dependencies/next: `focusa_prediction_authority`, `focusa_metacog_retrieve`, `focusa_trajectory_view`
- Documentation: `docs/focusa-tools/tools/focusa_epistemic_operation.md`

## focusa_evidence_capture

Capture a bounded evidence ref/result and optionally link it to the active Workpoint. Use it when Capture a bounded evidence ref/result and optionally link it to the active Workpoint. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.evidence.capture`
- Family: `workpoint`
- Side effects: `evidence_link`, `evidence_link`
- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-evidence-outcomes`
- Dependencies/next: `focusa_workpoint_link_evidence`, `focusa_trajectory_assess`, `focusa_recent_result`
- Documentation: `docs/focusa-tools/tools/focusa_evidence_capture.md`

## focusa_failure

Record a specific failure with diagnosis in Focus State. Must identify WHAT failed and WHY (or suspected why). Max 300 chars. Use it when Record a specific failure with diagnosis in Focus State. Must identify WHAT failed and WHY (or suspected why). Max 300 chars. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.failure`
- Family: `focus_state`
- Side effects: `write_state`, `write_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_tool_doctor`, `focusa_workpoint_resume`, `focusa_metacog_capture`
- Documentation: `docs/focusa-tools/tools/focusa_failure.md`

## focusa_fast_forward

Fast-forward session completion by multiplying parallel workloop-bound silent sessions (2x/4x/6x/8x...). Compiles the deterministic FanoutPlan — round-robin task division across lanes with per-lane policy budgets — then returns the plan; each lane executes as one silent session bound to its work items (docs/168, #312). Use it when Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.fast.forward`
- Family: `session_fanout`
- Side effects: `durable_dispatch`, `durable_dispatch`
- Skills: `skill:focusa`, `skill:focusa-silent-sessions`
- Dependencies/next: `focusa_bg_status`, `focusa_workpoint_checkpoint`
- Documentation: `docs/focusa-tools/tools/focusa_fast_forward.md`

## focusa_hlt_history

Read append-only HLT ledger entries with session filters, fallback candidates, and generic HLT tracking. Spec 125 §7.2-7.6. Use it when Read append-only HLT change history with session filters, fallback candidates, and generic HLT tracking. Spec 125 §7.2-7.6. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.hlt.history`
- Family: `trajectory`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_trajectory_view`, `focusa_trajectory_define_goal`, `focusa_project_verify`
- Documentation: `docs/focusa-tools/tools/focusa_hlt_history.md`

## focusa_instruction_conflicts

Read deterministic instruction conflicts; unresolved equal-authority claims remain blocked. Use it when Operate the Spec 140 instruction conflicts surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.instruction.conflicts`
- Family: `agent_runtime`
- Side effects: `read_or_preview_only`, `read_or_preview_only`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Dependencies/next: `focusa_instruction_explain`, `focusa_instruction_simulate`, `focusa_instruction_integrity_evaluate`
- Documentation: `docs/focusa-tools/tools/focusa_instruction_conflicts.md`

## focusa_instruction_explain

Explain one instruction claim from the current bounded source inventory. Use it when Operate the Spec 140 instruction explain surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.instruction.explain`
- Family: `agent_runtime`
- Side effects: `read_or_preview_only`, `read_or_preview_only`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Dependencies/next: `focusa_instruction_simulate`, `focusa_agent_runtime_effective`
- Documentation: `docs/focusa-tools/tools/focusa_instruction_explain.md`

## focusa_instruction_integrity_evaluate

Evaluate the foundational headless InstructionIntegrityGuard and durably record its fail-closed decision. Use it when Operate the Spec 140 instruction integrity evaluate surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.instruction.integrity.evaluate`
- Family: `agent_runtime`
- Side effects: `confirmed_receipted_artifact_delivery`, `confirmed_receipted_artifact_delivery`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`, `skill:focusa-security-auth-licensing`
- Dependencies/next: `focusa_instruction_integrity_status`, `focusa_agent_runtime_headless_verify`
- Documentation: `docs/focusa-tools/tools/focusa_instruction_integrity_evaluate.md`

## focusa_instruction_integrity_status

Read foundational guard availability, amendment authority, and outage posture. Use it when Operate the Spec 140 instruction integrity status surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.instruction.integrity.status`
- Family: `agent_runtime`
- Side effects: `read_or_preview_only`, `read_or_preview_only`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`, `skill:focusa-security-auth-licensing`
- Dependencies/next: `focusa_instruction_integrity_evaluate`, `focusa_agent_runtime_doctor`
- Documentation: `docs/focusa-tools/tools/focusa_instruction_integrity_status.md`

## focusa_instruction_simulate

Preview path/profile/target-specific instruction behavior without committing changes. Use it when Operate the Spec 140 instruction simulate surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.instruction.simulate`
- Family: `agent_runtime`
- Side effects: `read_or_preview_only`, `read_or_preview_only`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Dependencies/next: `focusa_runtime_constitution_preview`, `focusa_instruction_integrity_evaluate`
- Documentation: `docs/focusa-tools/tools/focusa_instruction_simulate.md`

## focusa_instruction_sources

Discover bounded, registered project instruction sources with trust and authority metadata. Use it when Operate the Spec 140 instruction sources surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.instruction.sources`
- Family: `agent_runtime`
- Side effects: `read_or_preview_only`, `read_or_preview_only`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Dependencies/next: `focusa_instruction_conflicts`, `focusa_instruction_explain`, `focusa_agent_runtime_effective`
- Documentation: `docs/focusa-tools/tools/focusa_instruction_sources.md`

## focusa_intent

Set the frame intent — what this session is trying to achieve (1-3 sentences, max 500 chars). Use it when Set the frame intent — what this session is trying to achieve (1-3 sentences, max 500 chars). It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.intent`
- Family: `focus_state`
- Side effects: `write_state`, `write_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_project_identity`, `focusa_trajectory_view`, `focusa_workpoint_checkpoint`
- Documentation: `docs/focusa-tools/tools/focusa_intent.md`

## focusa_li_tree_extract

Extract decision/constraint/risk signals and reflection trigger from lineage tree for metacognitive compounding. Use it when Extract decision/constraint/risk signals and reflection trigger from lineage tree for metacognitive compounding. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.li.tree.extract`
- Family: `tree_lineage`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-session-recovery`
- Dependencies/next: `focusa_metacog_capture`, `focusa_metacog_reflect`, `focusa_tree_snapshot_state`
- Documentation: `docs/focusa-tools/tools/focusa_li_tree_extract.md`

## focusa_lineage_tree

Fetch a bounded Focusa lineage window for /tree-aware reasoning. Full tree requires explicit cold opt-in. Use it when Fetch Focusa lineage tree for /tree-aware reasoning and LI addon workflows. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.lineage.tree`
- Family: `tree_lineage`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-session-recovery`
- Dependencies/next: `focusa_li_tree_extract`, `focusa_tree_path`, `focusa_traverse`
- Documentation: `docs/focusa-tools/tools/focusa_lineage_tree.md`

## focusa_metacog_capture

Store a reusable learning signal so future reasoning can retrieve it instead of rediscovering the same lesson. Use it when Store a reusable learning signal so future reasoning can retrieve it instead of rediscovering the same lesson. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.metacog.capture`
- Family: `metacognition`
- Side effects: `write_state`, `write_state`
- Skills: `skill:focusa`, `skill:focusa-metacognition`
- Dependencies/next: `focusa_metacog_retrieve`, `focusa_metacog_reflect`, `focusa_workpoint_checkpoint`
- Documentation: `docs/focusa-tools/tools/focusa_metacog_capture.md`

## focusa_metacog_doctor

Diagnose signal quality and retrieval usefulness in one move. Best safe diagnostic tool when deciding whether more capture or reflection work is needed. Use it when Diagnose signal quality and retrieval usefulness in one move. Best safe diagnostic tool when deciding whether more capture or reflection work is needed. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.metacog.doctor`
- Family: `metacognition`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-metacognition`
- Dependencies/next: `focusa_metacog_retrieve`, `focusa_metacog_recent_reflections`, `focusa_tool_doctor`
- Documentation: `docs/focusa-tools/tools/focusa_metacog_doctor.md`

## focusa_metacog_evaluate_outcome

Judge whether an adjustment improved results and whether the learning should be promoted. Use it when Judge whether an adjustment improved results and whether the learning should be promoted. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.metacog.evaluate.outcome`
- Family: `metacognition`
- Side effects: `write_state`, `write_state`
- Skills: `skill:focusa`, `skill:focusa-metacognition`, `skill:focusa-evidence-outcomes`
- Dependencies/next: `focusa_metacog_capture`, `focusa_predict_stats`, `focusa_decide`
- Documentation: `docs/focusa-tools/tools/focusa_metacog_evaluate_outcome.md`

## focusa_metacog_loop_run

Run capture -> retrieve -> reflect -> adjust -> evaluate in one move. Best composite tool when you want learning workflow compression instead of manual chaining. Use it when Run capture -> retrieve -> reflect -> adjust -> evaluate in one move. Best composite tool when you want learning workflow compression instead of manual chaining. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.metacog.loop.run`
- Family: `metacognition`
- Side effects: `write_state`, `write_state`
- Skills: `skill:focusa`, `skill:focusa-metacognition`
- Dependencies/next: `focusa_predict_stats`, `focusa_workpoint_checkpoint`, `focusa_metacog_doctor`
- Documentation: `docs/focusa-tools/tools/focusa_metacog_loop_run.md`

## focusa_metacog_plan_adjust

Turn a reflection into a tracked adjustment artifact that can later be evaluated for real improvement. Use it when Turn a reflection into a tracked adjustment artifact that can later be evaluated for real improvement. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.metacog.plan.adjust`
- Family: `metacognition`
- Side effects: `write_state`, `write_state`
- Skills: `skill:focusa`, `skill:focusa-metacognition`
- Dependencies/next: `focusa_metacog_evaluate_outcome`, `focusa_predict_record`, `focusa_workpoint_checkpoint`
- Documentation: `docs/focusa-tools/tools/focusa_metacog_plan_adjust.md`

## focusa_metacog_recent_adjustments

Best safe helper for finding recent adjustment ids before evaluation or promotion decisions. Use it when Best safe helper for finding recent adjustment ids before evaluation or promotion decisions. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.metacog.recent.adjustments`
- Family: `metacognition`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-metacognition`
- Dependencies/next: `focusa_metacog_evaluate_outcome`, `focusa_metacog_doctor`, `focusa_metacog_reflect`
- Documentation: `docs/focusa-tools/tools/focusa_metacog_recent_adjustments.md`

## focusa_metacog_recent_reflections

Best safe helper for finding recent reflection ids and update sets before adjust or promote work. Use it when Best safe helper for finding recent reflection ids and update sets before adjust or promote work. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.metacog.recent.reflections`
- Family: `metacognition`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-metacognition`
- Dependencies/next: `focusa_metacog_plan_adjust`, `focusa_metacog_doctor`, `focusa_metacog_reflect`
- Documentation: `docs/focusa-tools/tools/focusa_metacog_recent_reflections.md`

## focusa_metacog_reflect

Generate reusable hypotheses and strategy updates from recent turns when you need learning from past outcomes. Use it when Generate reusable hypotheses and strategy updates from recent turns when you need learning from past outcomes. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.metacog.reflect`
- Family: `metacognition`
- Side effects: `write_state`, `write_state`
- Skills: `skill:focusa`, `skill:focusa-metacognition`
- Dependencies/next: `focusa_metacog_plan_adjust`, `focusa_metacog_capture`, `focusa_workpoint_checkpoint`
- Documentation: `docs/focusa-tools/tools/focusa_metacog_reflect.md`

## focusa_metacog_retrieve

Best safe search tool for past learning signals relevant to the current ask. Use this before planning or reflection. Use it when Best safe search tool for past learning signals relevant to the current ask. Use this before planning or reflection. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.metacog.retrieve`
- Family: `metacognition`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-metacognition`
- Dependencies/next: `focusa_metacog_capture`, `focusa_metacog_reflect`, `focusa_predict_record`
- Documentation: `docs/focusa-tools/tools/focusa_metacog_retrieve.md`

## focusa_next_step

Record what you plan to do next (max 160 chars). Use it when Record what you plan to do next (max 160 chars). It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.next.step`
- Family: `focus_state`
- Side effects: `write_state`, `write_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_workpoint_checkpoint`, `focusa_active_object_resolve`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_next_step.md`

## focusa_north_star_gate

Inspect the current verified Project → HLT → MLG → STG → waypoint → gap → Workpoint → frontier chain before meaningful action. Read-only and fail-closed. Use it when Inspect the fail-closed Project → HLT → MLG → STG → Workpoint → frontier authority chain. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.north.star.gate`
- Family: `trajectory`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_project_identity`, `focusa_trajectory_view`, `focusa_workpoint_resume`
- Documentation: `docs/focusa-tools/tools/focusa_north_star_gate.md`

## focusa_note

Miscellaneous note (max 180 chars). Bounded at 20, oldest decay first. Use it when Miscellaneous note (max 180 chars). Bounded at 20, oldest decay first. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.note`
- Family: `focus_state`
- Side effects: `write_state`, `write_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_project_identity`, `focusa_trajectory_view`, `focusa_workpoint_resume`
- Documentation: `docs/focusa-tools/tools/focusa_note.md`

## focusa_ontology_scope_migration

Dry-run, apply, inspect, or roll back granular legacy ontology scope migration. Apply/rollback require explicit confirmation and per-record evidence; ownership is never inferred. Use it when Dry-run, apply, inspect, and roll back granular evidence-backed migration of quarantined legacy ontology records into one verified workstream. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.ontology.scope.migration`
- Family: `diagnostics_hygiene`
- Side effects: `confirmed_append_only_scope_migration`, `confirmed_append_only_scope_migration`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`
- Dependencies/next: `focusa_project_verify`, `focusa_evidence_capture`, `focusa_workpoint_link_evidence`
- Documentation: `docs/focusa-tools/tools/focusa_ontology_scope_migration.md`

## focusa_open_question

Record an open question that needs to be answered (max 180 chars). Use it when Record an open question that needs to be answered (max 180 chars). It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.open.question`
- Family: `focus_state`
- Side effects: `write_state`, `write_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_trajectory_assess`, `focusa_traverse`, `focusa_metacog_retrieve`
- Documentation: `docs/focusa-tools/tools/focusa_open_question.md`

## focusa_predict_evaluate

Evaluate a prediction inside its exact typed project/workstream scope. Use it when Evaluate a Focusa prediction against an actual outcome and optional score; required before final task completion when relevant predictions exist. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.predict.evaluate`
- Family: `metacognition`
- Side effects: `write_prediction_evaluation`, `write_prediction_evaluation`
- Skills: `skill:focusa`, `skill:focusa-metacognition`, `skill:predictive-power`
- Dependencies/next: `focusa_metacog_capture`, `focusa_metacog_reflect`, `focusa_predict_stats`
- Documentation: `docs/focusa-tools/tools/focusa_predict_evaluate.md`

## focusa_predict_recent

List recent predictions from one typed project/workstream scope. Use it when List recent bounded Focusa prediction records. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.predict.recent`
- Family: `metacognition`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-metacognition`, `skill:predictive-power`
- Dependencies/next: `focusa_predict_stats`, `focusa_predict_evaluate`, `focusa_metacog_retrieve`
- Documentation: `docs/focusa-tools/tools/focusa_predict_recent.md`

## focusa_predict_record

Record a bounded, inspectable Focusa prediction. Predictions guide decisions; they never override operator steering. Use it when Record a bounded, inspectable Focusa prediction; core at task start, trajectory review, compaction review, and end-of-task reports. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.predict.record`
- Family: `metacognition`
- Side effects: `write_prediction`, `write_prediction`
- Skills: `skill:focusa`, `skill:focusa-metacognition`, `skill:predictive-power`
- Dependencies/next: `focusa_evidence_capture`, `focusa_predict_evaluate`, `focusa_metacog_capture`
- Documentation: `docs/focusa-tools/tools/focusa_predict_record.md`

## focusa_predict_stats

Report prediction calibration for one typed project/workstream scope. Use it when Report Focusa prediction accuracy/calibration stats for compaction cards, trajectory reviews, and work reports. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.predict.stats`
- Family: `metacognition`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-metacognition`, `skill:predictive-power`
- Dependencies/next: `focusa_predict_recent`, `focusa_metacog_doctor`, `focusa_tool_doctor`
- Documentation: `docs/focusa-tools/tools/focusa_predict_stats.md`

## focusa_prediction_authority

Append or project immutable Spec 138 prediction/outcome/learning/transfer authority in one typed project/workstream scope. Use it when Append or project immutable Spec 138 authority in typed scope. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.prediction.authority`
- Family: `metacognition`
- Side effects: `write_or_read_prediction_authority`, `write_or_read_prediction_authority`
- Skills: `skill:focusa`, `skill:focusa-metacognition`, `skill:predictive-power`
- Dependencies/next: `focusa_predict_recent`, `focusa_evidence_capture`, `focusa_metacog_capture`
- Documentation: `docs/focusa-tools/tools/focusa_prediction_authority.md`

## focusa_preload_build

Build Preload Packet through the scoped Spec 111 preload API. Use it when Build a scoped agent bootstrap packet without writing it. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.preload.build`
- Family: `preload`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-agent-bootstrap`
- Dependencies/next: `focusa_preload_render`, `focusa_preload_verify`
- Documentation: `docs/focusa-tools/tools/focusa_preload_build.md`

## focusa_preload_doctor

Doctor Preload Scope through the scoped Spec 111 preload API. Use it when Diagnose bootstrap delivery readiness and recovery steps. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.preload.doctor`
- Family: `preload`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-agent-bootstrap`
- Dependencies/next: `focusa_preload_profiles`, `focusa_preload_build`
- Documentation: `docs/focusa-tools/tools/focusa_preload_doctor.md`

## focusa_preload_profiles

List bounded Spec 111 agent bootstrap profiles. Use it when List bounded agent bootstrap profiles. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.preload.profiles`
- Family: `preload`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-agent-bootstrap`
- Dependencies/next: `focusa_preload_build`, `focusa_preload_doctor`
- Documentation: `docs/focusa-tools/tools/focusa_preload_profiles.md`

## focusa_preload_receipt_commit

Commit an idempotent Spec 111 bootstrap delivery receipt. Use it when Commit an idempotent bootstrap delivery receipt. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.preload.receipt.commit`
- Family: `preload`
- Side effects: `write_receipt`, `write_receipt`
- Skills: `skill:focusa`, `skill:focusa-agent-bootstrap`
- Dependencies/next: `focusa_preload_verify`, `focusa_preload_doctor`
- Documentation: `docs/focusa-tools/tools/focusa_preload_receipt_commit.md`

## focusa_preload_receipt_preview

Preview a Spec 111 bootstrap delivery receipt without committing it. Use it when Preview a bootstrap delivery receipt without committing it. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.preload.receipt.preview`
- Family: `preload`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-agent-bootstrap`
- Dependencies/next: `focusa_preload_receipt_commit`, `focusa_preload_verify`
- Documentation: `docs/focusa-tools/tools/focusa_preload_receipt_preview.md`

## focusa_preload_render

Render Preload Packet through the scoped Spec 111 preload API. Use it when Render a scoped agent bootstrap packet. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.preload.render`
- Family: `preload`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-agent-bootstrap`
- Dependencies/next: `focusa_preload_verify`, `focusa_preload_write`
- Documentation: `docs/focusa-tools/tools/focusa_preload_render.md`

## focusa_preload_verify

Verify Preload Packet through the scoped Spec 111 preload API. Use it when Verify bootstrap packet scope and integrity. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.preload.verify`
- Family: `preload`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-agent-bootstrap`
- Dependencies/next: `focusa_preload_write`, `focusa_preload_doctor`
- Documentation: `docs/focusa-tools/tools/focusa_preload_verify.md`

## focusa_preload_write

Write a Spec 111 preload packet to an allowlisted target with an idempotency key. Use it when Write an agent bootstrap packet to an allowlisted target. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.preload.write`
- Family: `preload`
- Side effects: `write_project_files`, `write_project_files`
- Skills: `skill:focusa`, `skill:focusa-agent-bootstrap`
- Dependencies/next: `focusa_preload_receipt_preview`, `focusa_preload_verify`
- Documentation: `docs/focusa-tools/tools/focusa_preload_write.md`

## focusa_project_bootstrap

Preview, apply, inspect, or repair the idempotent local project-discipline baseline before Project Genesis. Use it when Preview, apply, inspect, or repair an idempotent local project-discipline baseline with explicit Git/task choices, receipts, rollback, and Project Genesis handoff. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.project.bootstrap`
- Family: `project_identity`
- Side effects: `preview_read_or_confirmed_local_bootstrap_repair`, `preview_read_or_confirmed_local_bootstrap_repair`
- Skills: `skill:focusa`, `skill:focusa-project-scope`
- Dependencies/next: `focusa_project_genesis`, `focusa_project_verify`, `focusa_workpoint_resume`
- Documentation: `docs/focusa-tools/tools/focusa_project_bootstrap.md`

## focusa_project_card

Build an advisory project-intelligence card from ProjectIdentity, ontology, trajectory, Workpoint/evidence, prediction, and metacog signals. Use it when Build an advisory project-intelligence card from ProjectIdentity, ontology, trajectory, Workpoint/evidence, prediction, and metacog signals for bootstrap/re-bootstrap. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.project.card`
- Family: `project_identity`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-project-scope`
- Dependencies/next: `focusa_project_card_outcome`, `focusa_traverse`, `focusa_trajectory_view`, `focusa_metacog_retrieve`
- Documentation: `docs/focusa-tools/tools/focusa_project_card.md`

## focusa_project_card_outcome

Attach a final outcome/result to a specific project-card algorithm_run_id and update learned project-card weights. Use it when Attach a verified result to a project-card algorithm_run_id so project-card learning weights and future bootstrap/sequence planning can improve. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.project.card.outcome`
- Family: `project_identity`
- Side effects: `write_project_card_outcome`, `write_project_card_outcome`
- Skills: `skill:focusa`, `skill:focusa-project-scope`, `skill:focusa-evidence-outcomes`
- Dependencies/next: `focusa_project_card`, `focusa_predict_record`, `focusa_metacog_capture`
- Documentation: `docs/focusa-tools/tools/focusa_project_card_outcome.md`

## focusa_project_genesis

Start, resume, inspect, or atomically commit the Project Genesis chain from verified identity and HLT through the first Workpoint. Use it when Stage, resume, inspect, or atomically commit the verified project journey from HLT and specification through tasks, first Workpoint, coordination, and readiness receipt. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.project.genesis`
- Family: `project_identity`
- Side effects: `start_resume_read_or_confirmed_atomic_commit`, `start_resume_read_or_confirmed_atomic_commit`
- Skills: `skill:focusa`, `skill:focusa-project-scope`
- Dependencies/next: `focusa_workpoint_resume`, `focusa_trajectory_view`, `focusa_project_verify`
- Documentation: `docs/focusa-tools/tools/focusa_project_genesis.md`

## focusa_project_identity

Resolve bounded ProjectIdentity from cwd/project_root using marker, git, beads, workspace, daemon, and operator project signals. Use it when Rank explicit, active-worktree, canonical-parent, marker/Beads, persisted-session, and bounded parent-directory project candidates; fail closed on ambiguity before trusting project-bound context. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.project.identity`
- Family: `project_identity`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-project-scope`
- Dependencies/next: `focusa_project_card`, `focusa_project_verify`, `focusa_trajectory_view`, `focusa_workpoint_resume`
- Documentation: `docs/focusa-tools/tools/focusa_project_identity.md`

## focusa_project_verify

Verify active project folder against expected ProjectIdentity fields and report mismatches without mutating state. Use it when Verify expected project identity fields against ranked worktree/session candidates and surface ambiguity or project/continuity mismatches without mutating Focusa state. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.project.verify`
- Family: `project_identity`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-project-scope`
- Dependencies/next: `focusa_project_bootstrap`, `focusa_project_genesis`, `focusa_trajectory_view`, `focusa_workpoint_resume`, `focusa_tool_doctor`
- Documentation: `docs/focusa-tools/tools/focusa_project_verify.md`

## focusa_prompt_variant_diff

Compare two caller-supplied prompt variant projections without mutating Focusa state. Use it when Operate the Spec 140 prompt variant diff surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.prompt.variant.diff`
- Family: `agent_runtime`
- Side effects: `read_or_preview_only`, `read_or_preview_only`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Dependencies/next: `focusa_prompt_variant_preview`, `focusa_agent_runtime_doctor`
- Documentation: `docs/focusa-tools/tools/focusa_prompt_variant_diff.md`

## focusa_prompt_variant_preview

Compile and preview a target prompt variant without activation. Use it when Operate the Spec 140 prompt variant preview surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.prompt.variant.preview`
- Family: `agent_runtime`
- Side effects: `read_or_preview_only`, `read_or_preview_only`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Dependencies/next: `focusa_prompt_variant_diff`, `focusa_agent_artifact_preview`
- Documentation: `docs/focusa-tools/tools/focusa_prompt_variant_preview.md`

## focusa_recent_result

Record a completed result, output, or reference (max 180 chars). Use it when Record a completed result, output, or reference (max 180 chars). It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.recent.result`
- Family: `focus_state`
- Side effects: `write_state`, `write_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_evidence_capture`, `focusa_trajectory_assess`, `focusa_workpoint_checkpoint`
- Documentation: `docs/focusa-tools/tools/focusa_recent_result.md`

## focusa_reflex_primitives

List bounded Spec97 Reflex Primitive summaries by family/query; read-only routing metadata, never mutation authority. Use it when Read bounded Spec97 Reflex Primitive summaries by family/query from the read-only registry; advisory routing metadata only, never mutation authority. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.reflex.primitives`
- Family: `traversal`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-tool-discovery`
- Dependencies/next: `focusa_traverse`, `focusa_tool_doctor`, `focusa_workpoint_resume`
- Documentation: `docs/focusa-tools/tools/focusa_reflex_primitives.md`

## focusa_resource_mode

Read or control Focusa resource mode, including activating/deactivating LowMem mode when resources are constrained. Use it when Read or control Focusa ResourceMode, including activating or deactivating LowMem mode when resources are constrained. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.resource.mode`
- Family: `diagnostics_hygiene`
- Side effects: `control_state`, `control_state`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`, `skill:focusa-resource-performance`
- Dependencies/next: `focusa_traverse`, `focusa_trajectory_view`, `focusa_workpoint_resume`
- Documentation: `docs/focusa-tools/tools/focusa_resource_mode.md`

## focusa_runtime_constitution_preview

Preview a compiled Runtime Constitution without activation or artifact delivery. Use it when Operate the Spec 140 runtime constitution preview surface with typed scope and evidence. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.runtime.constitution.preview`
- Family: `agent_runtime`
- Side effects: `read_or_preview_only`, `read_or_preview_only`
- Skills: `skill:focusa`, `skill:focusa-spec-implementation`
- Dependencies/next: `focusa_prompt_variant_preview`, `focusa_agent_artifact_preview`
- Documentation: `docs/focusa-tools/tools/focusa_runtime_constitution_preview.md`

## focusa_scratch

Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done. Use it when Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.scratch`
- Family: `focus_state`
- Side effects: `local_note`, `local_note`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_project_identity`, `focusa_trajectory_view`, `focusa_workpoint_resume`
- Documentation: `docs/focusa-tools/tools/focusa_scratch.md`

## focusa_session_transfer

Typed save/continue/rollover wrapper for moving long work between Pi sessions without forking or continuity-id fingerprint fallback. Use it when Save, continue, or Spec130-roll over a long Focusa/Pi work session with explicit source_scope/target_scope or target_continuity_id, source/target session ids, checkpoint/packet refs, and rollover action. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.session.transfer`
- Family: `workpoint`
- Side effects: `save_may_checkpoint_workpoint`, `save_may_checkpoint_workpoint`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_workpoint_resume`, `focusa_project_card`, `focusa_trajectory_view`
- Documentation: `docs/focusa-tools/tools/focusa_session_transfer.md`

## focusa_silent_sessions

Daemon-native Spec133 Silent Session client for status, observation, steering, controls, config, receipts, capabilities, and legacy action compatibility; process-control failures return failure_class=process_control_failed with receipt-backed recovery. Use it when Thin daemon-native Spec133 API client for exact session/run status, bounded observation, steering, controls, config, receipts, capabilities, and legacy action compatibility; process-control failures return failure_class=process_control_failed with receipt-backed recovery. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.silent.sessions`
- Family: `work_loop`
- Side effects: `daemon_api_control`, `daemon_api_control`
- Skills: `skill:focusa`, `skill:focusa-work-loop`, `skill:focusa-silent-sessions`
- Dependencies/next: `focusa_work_loop_status`, `focusa_work_loop_checkpoint`, `focusa_resource_mode`
- Documentation: `docs/focusa-tools/tools/focusa_silent_sessions.md`

## focusa_state_hygiene_apply

Approval-gated, non-destructive hygiene apply; records an auditable Focus State note via reducer-backed /focus/update. Use it when Approval-gated, non-destructive hygiene apply; records an auditable Focus State note via reducer-backed /focus/update. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.state.hygiene.apply`
- Family: `diagnostics_hygiene`
- Side effects: `write_focus_state_note`, `write_focus_state_note`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`
- Dependencies/next: `focusa_state_hygiene_doctor`, `focusa_workpoint_resume`, `focusa_tool_doctor`
- Documentation: `docs/focusa-tools/tools/focusa_state_hygiene_apply.md`

## focusa_state_hygiene_doctor

Diagnose stale or duplicate Focus State signals without mutating state. Use it when Diagnose stale or duplicate Focus State signals without mutating state. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.state.hygiene.doctor`
- Family: `diagnostics_hygiene`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`
- Dependencies/next: `focusa_state_hygiene_plan`, `focusa_tool_doctor`, `focusa_scratch`
- Documentation: `docs/focusa-tools/tools/focusa_state_hygiene_doctor.md`

## focusa_state_hygiene_plan

Create a proposal-style hygiene plan; does not mutate Focus State. Use it when Create a proposal-style hygiene plan; does not mutate Focus State. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.state.hygiene.plan`
- Family: `diagnostics_hygiene`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`
- Dependencies/next: `focusa_state_hygiene_apply`, `focusa_state_hygiene_doctor`, `focusa_tool_doctor`
- Documentation: `docs/focusa-tools/tools/focusa_state_hygiene_plan.md`

## focusa_temporal_authority

Read, commit, revise, observe, forecast, or preflight project-scoped temporal claims without fabricating deadlines or urgency. Use it when Read, commit, revise, observe, forecast, or preflight scoped temporal claims with evidence, confidence, uncertainty, freshness, and no fabricated urgency. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.temporal.authority`
- Family: `trajectory`
- Side effects: `status_preflight_read_or_confirmed_claim_write_or_observation`, `status_preflight_read_or_confirmed_claim_write_or_observation`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_trajectory_view`, `focusa_workpoint_resume`, `focusa_project_verify`
- Documentation: `docs/focusa-tools/tools/focusa_temporal_authority.md`

## focusa_tool_bundle

Load a bounded family bundle of capability metadata and optionally strict schemas. Use after search or graph traversal when one workflow needs several related tools; avoid broad all-tool prompt injection. Use it when Load one bounded capability family with schemas deferred by default. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.tool.bundle`
- Family: `traversal`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-tool-discovery`
- Dependencies/next: `focusa_tool_describe`, `focusa_tool_graph`, `focusa_tool_search`
- Documentation: `docs/focusa-tools/tools/focusa_tool_bundle.md`

## focusa_tool_describe

Cold-load one complete runtime Focusa tool definition after search. Returns strict input/output schemas, operational guidance, authority, side effects, failures, recovery, dependencies, skills, docs, and protocol bindings without loading unrelated tools. Use it when Cold-load one complete capability contract including strict schemas and recovery. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.tool.describe`
- Family: `traversal`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-tool-discovery`
- Dependencies/next: `focusa_tool_graph`, `focusa_agent_card`, `focusa_tool_search`
- Documentation: `docs/focusa-tools/tools/focusa_tool_describe.md`

## focusa_tool_doctor

Diagnose Focusa tool-suite readiness, active Workpoint continuity, daemon health, and likely next repair action. Use it when Diagnose Focusa tool-suite readiness, active Workpoint continuity, daemon health, and likely next repair action. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.tool.doctor`
- Family: `diagnostics_hygiene`
- Side effects: `diagnostic`, `diagnostic`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`
- Dependencies/next: `focusa_resource_mode`, `focusa_project_identity`, `focusa_workpoint_resume`
- Documentation: `docs/focusa-tools/tools/focusa_tool_doctor.md`

## focusa_tool_graph

Traverse the bounded capability dependency and likely-next graph from one tool or family. Use it to plan a valid workflow sequence without loading the complete registry or inventing dependencies. Use it when Traverse bounded capability dependencies and likely-next workflow edges. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.tool.graph`
- Family: `traversal`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-tool-discovery`
- Dependencies/next: `focusa_tool_describe`, `focusa_tool_bundle`, `focusa_tool_search`
- Documentation: `docs/focusa-tools/tools/focusa_tool_graph.md`

## focusa_tool_search

Search the bounded Focusa capability catalog before loading full schemas. Returns ranked metadata, scope, side-effect, skill, documentation, and discovery refs so agents can select the narrowest tool under token budget. Use it when Search bounded capability metadata before cold-loading full schemas. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.tool.search`
- Family: `traversal`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-tool-discovery`
- Dependencies/next: `focusa_tool_describe`, `focusa_tool_bundle`, `focusa_tool_graph`
- Documentation: `docs/focusa-tools/tools/focusa_tool_search.md`

## focusa_trajectory_assess

Assess current project state against the desired Trajectory end state and return gaps/recommended action. Use it when Assess project current state against desired Trajectory end state and return gaps/recommended action; task-boundary reviews should cross-check predictions and metacog lessons. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.trajectory.assess`
- Family: `trajectory`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_trajectory_propose_workpoint`, `focusa_workpoint_checkpoint`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_trajectory_assess.md`

## focusa_trajectory_checkpoint

Create an advisory Trajectory checkpoint packet before compaction/model switch; pair with Workpoint checkpoint for canonical continuation. Use it when Create an advisory Trajectory checkpoint packet before compaction/model switch; pair with Workpoint checkpoint for canonical continuation. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.trajectory.checkpoint`
- Family: `trajectory`
- Side effects: `advisory_checkpoint`, `advisory_checkpoint`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_workpoint_checkpoint`, `focusa_trajectory_resume`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_trajectory_checkpoint.md`

## focusa_trajectory_define_goal

Create an advisory per-project Trajectory goal candidate without changing task/execution authority. Use it when Create an advisory per-project Trajectory goal candidate, including HLT/MLG/STG/Waypoints, without changing task or execution authority. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.trajectory.define.goal`
- Family: `trajectory`
- Side effects: `advisory_projection`, `advisory_projection`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_trajectory_assess`, `focusa_trajectory_propose_workpoint`, `focusa_trajectory_checkpoint`
- Documentation: `docs/focusa-tools/tools/focusa_trajectory_define_goal.md`

## focusa_trajectory_propose_workpoint

Propose an advisory Workpoint candidate from the active per-project Trajectory gap; does not promote or execute it. Use it when Propose an advisory Workpoint candidate from the active per-project Trajectory gap; does not promote or execute it. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.trajectory.propose.workpoint`
- Family: `trajectory`
- Side effects: `advisory_projection`, `advisory_projection`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_workpoint_checkpoint`, `focusa_active_object_resolve`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_trajectory_propose_workpoint.md`

## focusa_trajectory_resume

Resume per-project Trajectory orientation plus Workpoint handoff context after compaction/model switch/session resume. Use it when Resume per-project Trajectory orientation plus Workpoint handoff context after compaction/model switch/session resume, including prediction/metacog review prompts. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.trajectory.resume`
- Family: `trajectory`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_workpoint_resume`, `focusa_trajectory_view`, `focusa_tool_doctor`
- Documentation: `docs/focusa-tools/tools/focusa_trajectory_resume.md`

## focusa_trajectory_view

Read the per-project Trajectory Intelligence view: project identity, goal/state/gap/evidence/drift, and next Workpoint candidate. Use it when Read the per-project Trajectory Intelligence view before acting: project identity, goal/state/gap/evidence/drift, next Workpoint candidate, and learning-loop context for task closure. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.trajectory.view`
- Family: `trajectory`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_temporal_authority`, `focusa_trajectory_assess`, `focusa_trajectory_define_goal`, `focusa_workpoint_resume`
- Documentation: `docs/focusa-tools/tools/focusa_trajectory_view.md`

## focusa_traverse

Read-only surgical traversal across large Focusa surfaces. Use for bounded lineage, ontology, evidence, telemetry, Workpoint, and registry slices instead of full payloads. Use it when Read-only surgical traversal across large Focusa surfaces using bounded selectors, cursors, field projection, tags, and cold full-payload guards. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.traverse`
- Family: `traversal`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-tool-discovery`
- Dependencies/next: `focusa_active_object_resolve`, `focusa_evidence_capture`, `focusa_workpoint_resume`
- Documentation: `docs/focusa-tools/tools/focusa_traverse.md`

## focusa_tree_diff_context

Best safe compare tool for snapshots. Use this instead of guessing what changed across checkpoints. Use it when Best safe compare tool for snapshots. Use this instead of guessing what changed across checkpoints. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.tree.diff.context`
- Family: `tree_lineage`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-session-recovery`
- Dependencies/next: `focusa_tree_restore_state`, `focusa_tree_path`, `focusa_metacog_capture`
- Documentation: `docs/focusa-tools/tools/focusa_tree_diff_context.md`

## focusa_tree_head

Best safe starting point for lineage work. Use first when you need current branch/head context before path, snapshot, diff, or restore work. Use it when Best safe starting point for lineage work. Use first when you need current branch/head context before path, snapshot, diff, or restore work. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.tree.head`
- Family: `tree_lineage`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-session-recovery`
- Dependencies/next: `focusa_tree_path`, `focusa_tree_snapshot_state`, `focusa_lineage_tree`
- Documentation: `docs/focusa-tools/tools/focusa_tree_head.md`

## focusa_tree_path

Safe ancestry lookup. Use when branch position or lineage depth matters and you do not want to infer it from prior turns. Use it when Safe ancestry lookup. Use when branch position or lineage depth matters and you do not want to infer it from prior turns. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.tree.path`
- Family: `tree_lineage`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-session-recovery`
- Dependencies/next: `focusa_tree_snapshot_state`, `focusa_tree_diff_context`, `focusa_traverse`
- Documentation: `docs/focusa-tools/tools/focusa_tree_path.md`

## focusa_tree_recent_snapshots

Best safe helper for finding recent snapshot ids. Use this before diff or restore when you do not already know the right snapshot id. Use it when Best safe helper for finding recent snapshot ids. Use this before diff or restore when you do not already know the right snapshot id. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.tree.recent.snapshots`
- Family: `tree_lineage`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-session-recovery`
- Dependencies/next: `focusa_tree_diff_context`, `focusa_tree_snapshot_compare_latest`, `focusa_tree_snapshot_state`
- Documentation: `docs/focusa-tools/tools/focusa_tree_recent_snapshots.md`

## focusa_tree_restore_state

Restore a saved checkpoint when you need rollback or exact/merge recovery. State-changing tool. Use it when Restore a saved checkpoint when you need rollback or exact/merge recovery. State-changing tool. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.tree.restore.state`
- Family: `tree_lineage`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-session-recovery`
- Dependencies/next: `focusa_tree_head`, `focusa_tree_path`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_tree_restore_state.md`

## focusa_tree_snapshot_compare_latest

Create a fresh snapshot and compare it to the latest prior snapshot in one move. Best tool when you want checkpoint + diff without manual id hunting. Use it when Create a fresh snapshot and compare it to the latest prior snapshot in one move. Best tool when you want checkpoint + diff without manual id hunting. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.tree.snapshot.compare.latest`
- Family: `tree_lineage`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-session-recovery`
- Dependencies/next: `focusa_tree_diff_context`, `focusa_tree_restore_state`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_tree_snapshot_compare_latest.md`

## focusa_tree_snapshot_state

Create a recoverable checkpoint before risky work or comparisons. Best write tool for saving current state with a reason. Use it when Create a recoverable checkpoint before risky work or comparisons. Best write tool for saving current state with a reason. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.tree.snapshot.state`
- Family: `tree_lineage`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-session-recovery`
- Dependencies/next: `focusa_tree_recent_snapshots`, `focusa_tree_diff_context`, `focusa_tree_restore_state`
- Documentation: `docs/focusa-tools/tools/focusa_tree_snapshot_state.md`

## focusa_utility_card

Read compact bootstrap, post-compaction, recovery, and brevity guidance. Use it when Read compact bootstrap, post-compaction, recovery, and brevity guidance. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.utility.card`
- Family: `diagnostics_hygiene`
- Side effects: `read_state`, `read_state`
- Skills: `skill:focusa`, `skill:focusa-troubleshooting`
- Dependencies/next: `focusa_agent_prompt`, `focusa_workpoint_resume`, `focusa_trajectory_view`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_utility_card.md`

## focusa_work_loop_checkpoint

Create a manual continuous-loop checkpoint. Use it when Create a manual continuous-loop checkpoint. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.work.loop.checkpoint`
- Family: `work_loop`
- Side effects: `checkpoint`, `checkpoint`
- Skills: `skill:focusa`, `skill:focusa-work-loop`
- Dependencies/next: `focusa_work_loop_select_next`, `focusa_workpoint_checkpoint`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_work_loop_checkpoint.md`

## focusa_work_loop_context

Update continuation decision context (current ask/scope/steering). Use it when Update continuation decision context (current ask/scope/steering). It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.work.loop.context`
- Family: `work_loop`
- Side effects: `write_context`, `write_context`
- Skills: `skill:focusa`, `skill:focusa-work-loop`
- Dependencies/next: `focusa_work_loop_checkpoint`, `focusa_work_loop_status`, `focusa_workpoint_checkpoint`
- Documentation: `docs/focusa-tools/tools/focusa_work_loop_context.md`

## focusa_work_loop_control

Control continuous work loop: on, pause, resume, stop. Use it when Control continuous work loop: on, pause, resume, stop. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.work.loop.control`
- Family: `work_loop`
- Side effects: `control_state`, `control_state`
- Skills: `skill:focusa`, `skill:focusa-work-loop`
- Dependencies/next: `focusa_work_loop_writer_status`, `focusa_work_loop_status`, `focusa_work_loop_checkpoint`
- Documentation: `docs/focusa-tools/tools/focusa_work_loop_control.md`

## focusa_work_loop_select_next

Ask daemon to defer blocked work and select next ready work item. Use it when Ask daemon to defer blocked work and select next ready work item. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.work.loop.select.next`
- Family: `work_loop`
- Side effects: `select_next_work`, `select_next_work`
- Skills: `skill:focusa`, `skill:focusa-work-loop`
- Dependencies/next: `focusa_workpoint_checkpoint`, `focusa_work_loop_context`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_work_loop_select_next.md`

## focusa_work_loop_status

Get current continuous work-loop state and budgets. Use it when Get current continuous work-loop state and budgets. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.work.loop.status`
- Family: `work_loop`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-work-loop`
- Dependencies/next: `focusa_work_loop_writer_status`, `focusa_work_loop_context`, `focusa_work_loop_select_next`
- Documentation: `docs/focusa-tools/tools/focusa_work_loop_status.md`

## focusa_work_loop_writer_status

Read current work-loop writer ownership and mutation preflight guidance without mutating state. Use it when Read current work-loop writer ownership and mutation preflight guidance without mutating state. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.work.loop.writer.status`
- Family: `work_loop`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-work-loop`
- Dependencies/next: `focusa_work_loop_status`, `focusa_work_loop_context`, `focusa_work_loop_checkpoint`
- Documentation: `docs/focusa-tools/tools/focusa_work_loop_writer_status.md`

## focusa_workpoint_checkpoint

Create a typed Focusa Workpoint checkpoint before compaction, resume, context overflow, model switch, or risky continuation. Use this instead of trusting raw transcript memory; Focusa becomes the canonical continuation source and returns an explicit next-step hint. Use it when Create a typed Focusa Workpoint checkpoint before compaction, resume, context overflow, model switch, or risky continuation. Use this instead of trusting raw transcript memory; Focusa becomes the canonical continuation source and returns an explicit next-step hint. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.workpoint.checkpoint`
- Family: `workpoint`
- Side effects: `checkpoint`, `checkpoint`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_workpoint_resume`, `focusa_active_object_resolve`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_workpoint_checkpoint.md`

## focusa_workpoint_link_evidence

Attach a stable evidence reference or verification result to the active canonical Workpoint. Use it when Attach a stable evidence reference or verification result to the active canonical Workpoint. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.workpoint.link.evidence`
- Family: `workpoint`
- Side effects: `evidence_link`, `evidence_link`
- Skills: `skill:focusa`, `skill:focusa-workpoint`, `skill:focusa-evidence-outcomes`
- Dependencies/next: `focusa_trajectory_assess`, `focusa_workpoint_resume`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_workpoint_link_evidence.md`

## focusa_workpoint_resume

Fetch the active Focusa WorkpointResumePacket after compaction, resume, context overflow, model switch, or uncertainty. Use this instead of guessing from transcript tail; output includes canonical/degraded status, warnings, and the exact next action. Use it when Fetch the active Focusa WorkpointResumePacket after compaction, resume, context overflow, model switch, or uncertainty. Use this instead of guessing from transcript tail; output includes canonical/degraded status, warnings, and the exact next action. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.workpoint.resume`
- Family: `workpoint`
- Side effects: `read_only`, `read_only`
- Skills: `skill:focusa`, `skill:focusa-workpoint`
- Dependencies/next: `focusa_trajectory_view`, `focusa_active_object_resolve`, `focusa_evidence_capture`
- Documentation: `docs/focusa-tools/tools/focusa_workpoint_resume.md`

## focusa_workset_projection

Read a Spec 149 Workset: the deterministic replay projection (membership, requirement dispositions, settlement) from the append-only ledger. Read-only; execution lives in CallGraph. Use it when Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done. It returns a typed Focusa result with bounded recovery and likely next capabilities.

- Capability: `focusa.workset.projection`
- Family: `workset`
- Side effects: `read_projection`, `read_projection`
- Skills: `skill:focusa`, `skill:focusa-work-loop`
- Dependencies/next: `focusa_workpoint_resume`, `focusa_callgraph_validate`
- Documentation: `docs/focusa-tools/tools/focusa_workset_projection.md`
