# Context Cognition Spec

Status: planning
Scope: Focusa project intelligence, context curation, ontology framing, surface contracts, and eval-driven optimization
Authority: advisory by default; promotion requires existing Focusa Workpoint, Evidence, Trajectory, or reducer-backed paths


## 0. Normative basis

This spec is based on the corrected architecture in Spec 98 and Spec 99, not the current implementation state.

Spec 98/99 corrections that govern Context Cognition:

- project scope is bounded by `project_root + continuity_id`,
- Workpoint remains immediate action authority,
- Trajectory supplies project/workstream goal and gap context,
- HLT is durable north-star context while MLG/STG/waypoints are adaptive advisory context,
- Ontology is semantic structure, not proof by itself,
- Evidence refs are the proof boundary,
- UIAI/browser packets are proposal-only until Focusa capture/link succeeds,
- Focus State stores bounded meaning, not raw transcript or scratch notes,
- telemetry/resource pressure is operational, not cognition authority,
- generated packets must use shared canonical/advisory/degraded/stale envelope semantics,
- packet generation must not mutate canonical cognition state.

Any mismatch between this spec and current code should be treated as an implementation gap, not as a reason to weaken this spec.

## 1. Purpose

Context Cognition compiles scoped project context, ontology, evidence, and optimized reasoning guidance into bounded advisory packets for safer, smarter Focusa work.

It improves how agents understand a project before action without creating a competing memory system, task scheduler, or authority source.

## 2. Core thesis

RepoPrompt-style context curation and DSPy-style optimization are complementary:

- context curation finds the right material,
- ontology explains what the material means,
- evidence marks what is proven,
- Workpoint preserves immediate action authority,
- Trajectory preserves goal and gap context,
- optimization improves how planner and judge prompts use the packet.

Short form:

> Context Cognition finds the right context, structures its meaning, and optimizes how Focusa reasons over it.

## 3. Non-goals

Context Cognition is not:

- LoRA or model fine-tuning,
- durable memory authority,
- a vector database product,
- a RepoPrompt clone UI,
- an autonomous task scheduler,
- a replacement for Workpoint, Trajectory, Ontology, Evidence, ProjectIdentity, or Focus State,
- a prompt stuffing mechanism,
- a new source of canonical truth.

## 4. Authority boundaries

Context Cognition output is advisory unless promoted through existing Focusa systems.

Canonical authority remains:

| Authority area | Source |
|---|---|
| Project scope | ProjectIdentity |
| Immediate next action | Workpoint |
| Goal and gap context | Trajectory |
| Structured meaning | Ontology |
| Proof | Evidence refs |
| Operator direction | current operator ask |
| Durable cognition updates | reducer-backed Focusa events |

Context Cognition may recommend:

- relevant files, docs, diffs, snippets, and codemaps,
- active object candidates,
- relation candidates,
- missing evidence,
- contradiction flags,
- stale-context warnings,
- next tool candidates,
- prompt/module improvements after eval.

Context Cognition must not directly:

- mutate Focus State,
- supersede HLT,
- close beads,
- promote evidence,
- override Workpoint authority,
- override operator steering,
- mark advisory inference as canonical.

## 5. Inputs

Required inputs:

- `project_root`
- `continuity_id`
- ProjectIdentity result
- Workpoint resume packet
- Trajectory view/resume packet
- Ontology active object set
- Evidence refs
- Focus State summary
- Beads state
- git status/diff summary
- relevant docs/files/codemaps

Optional inputs:

- UIAI diagnostics packets
- browser research evidence
- prediction records
- metacog lessons
- prior Context Cognition packets
- external MCP results
- work-loop status summary
- resource mode summary

## 6. Primary output

The primary artifact is a bounded packet.

```yaml
ContextCognitionPacket:
  schema_version: focusa.context_cognition_packet.v1
  status: completed | degraded | stale | blocked
  advisory: true
  canonical: false
  scope_status: matched | missing | partial | mismatch
  freshness:
    generated_at:
    stale:
    source_snapshot:
  scope:
    project_root:
    continuity_id:
    session_id:
    workpoint_id:
    trajectory_id:
  authority:
    action_authority: workpoint
    goal_context: trajectory
    semantic_context: ontology
    proof_context: evidence
    canonical_mutation_allowed: false
  selected_context:
    files: []
    diffs: []
    docs: []
    codemaps: []
    snippets: []
    excluded_context: []
  ontology_frame:
    active_objects: []
    relations: []
    affordances: []
    risks: []
    valid_next_actions: []
  evidence_frame:
    proven: []
    unproven: []
    stale: []
    missing: []
  reasoning_frame:
    likely_goal:
    active_gap:
    confidence:
    contradiction_flags: []
    drift_risks: []
  optimization_frame:
    module_name:
    prompt_artifact_ref:
    eval_score:
    baseline_score:
    promoted: false
  route_frame:
    next_tools: []
    recovery_tools: []
    do_not_use_by_default: []
  side_effects: []
  evidence_refs: []
  recommended_packet_use:
    include_in_prompt: []
    exclude_from_prompt: []
    next_tools: []
    do_not_drift: []
```

## 7. Ontology as semantic spine

Ontology is central. Context Cognition must not reduce project understanding to file selection.

Ontology maps:

- files to components,
- endpoints to features,
- beads to work items,
- evidence to proof targets,
- failures to risks,
- actions to valid next moves,
- relations to `depends_on`, `proves`, `blocks`, `supersedes`, `implements`, `regresses`, or `explains`.

This lets Focusa distinguish:

- a file that is merely nearby,
- a file that defines the active object,
- a file that proves the current state,
- a file that creates risk,
- a file that should be excluded from prompt context.

## 8. Surface interaction model

Context Cognition is a coordinator across existing Focusa surfaces. It should not make any surface subordinate to a new authority source.

| Surface | Interaction | Authority posture | Output into packet |
|---|---|---|---|
| Daemon core | Builds packet from bounded read models and scoped inputs | advisory read/composition | packet status, freshness, side effects |
| API | Exposes packet creation, preview, evaluate, and render routes | read-only/advisory unless separate promotion route exists | JSON envelope |
| CLI | Prints compact packet, proof commands, and diff/context summaries | read-only/advisory | terminal summary / JSON |
| Pi extension | Injects bounded packet into Focus Slice or compaction resume context | advisory support for agent prompt | compact section with do-not-drift |
| Focusa tools | Adds callable tool wrappers for packet creation and evidence linking | tool result envelope | `focusa_context_cognition_*` outputs |
| ProjectIdentity | Supplies project scope gate | canonical scope input | `scope_status` |
| Workpoint | Supplies immediate next-action authority | continuation authority | action target, next slice, do-not-drift |
| Trajectory | Supplies HLT/MLG/STG/gap context | goal context; HLT north-star, lower ladder advisory | goal frame and active gap |
| Ontology | Supplies active objects, relations, affordances, and valid actions | semantic advisory unless promoted | ontology frame |
| Evidence | Supplies proof handles and missing-proof map | proof boundary | evidence frame |
| Focus State | Supplies bounded meaning slots | canonical meaning input; no raw transcript | concise intent/focus/decisions/constraints |
| Beads | Supplies work item state | task tracker input | work item relevance and blockers |
| Git/files/docs | Supplies raw project signals | source material only | selected context and exclusions |
| UIAI | Supplies browser diagnostics/research packets | proposal-only until Focusa capture/link | external diagnostics/evidence candidates |
| Prediction | Supplies bounded forecasts | advisory | risk and expected-outcome hints |
| Metacog | Supplies reusable lessons | advisory until adopted | strategy hints |
| Work-loop | Supplies governed execution status | mutation guarded by writer/preflight | loop status and writer caution |
| Menubar | Displays packet status and handoff controls | display only; route-backed actions | human-readable packet card |
| Proof bundle | Verifies changed surfaces and packet quality | release gate | proof refs and required checks |

## 9. Daemon behavior

The daemon should treat Context Cognition as a bounded composition read path.

Responsibilities:

- gather scoped read-model slices,
- enforce `project_root + continuity_id` gate,
- refuse broad or missing project roots for canonical-looking packet renders,
- label stale/degraded/mismatch states,
- attach source refs and rehydrate refs,
- cap traversal depth, file count, byte count, and total packet size,
- record telemetry for packet latency and omitted context counts,
- never mutate Focus State, Workpoint, Trajectory, Evidence, or Beads from packet generation alone.

Daemon state policy:

- packet generation may write telemetry or runtime cache only,
- durable promotion must use existing reducer-backed routes,
- cached packets require freshness metadata and cannot be treated as canonical truth.

## 10. API contract

Potential routes:

| Route | Method | Purpose | Side effects |
|---|---|---|---|
| `/v1/context-cognition/packet` | POST | Build scoped packet from current Workpoint/Trajectory/Ontology/Evidence inputs | runtime cache/telemetry only |
| `/v1/context-cognition/preview` | POST | Preview selected context and omissions without cache write | read-only |
| `/v1/context-cognition/evaluate` | POST | Score packet against eval cases or provided acceptance criteria | telemetry/eval artifact only |
| `/v1/context-cognition/render` | POST | Render packet as compact prompt/CLI/menubar text | read-only |
| `/v1/context-cognition/proof` | POST | Map packet surfaces to proof commands | read-only or proof artifact |

All API responses should use the shared Focusa result envelope:

- `ok`
- `status`
- `canonical=false`
- `advisory=true`
- `degraded`
- `stale`
- `scope_status`
- `failure_class`
- `side_effects`
- `evidence_refs`
- `next_tools`
- `recovery_hint`
- `misuse_hint`

## 11. CLI contract

Proposed commands:

```text
focusa context-cognition packet --project-root ... --continuity-id ...
focusa context-cognition preview --query ... --budget-tokens ...
focusa context-cognition evaluate --case ...
focusa context-cognition render --format compact|json|markdown
focusa context-cognition proof --surface pi|api|cli|uiai|menubar
```

CLI behavior:

- default output is compact and advisory-labeled,
- `--json` exposes full packet,
- no CLI command promotes packet output by default,
- missing scope returns degraded output plus recovery commands,
- proof command output maps to existing Focusa test/proof surfaces.

## 12. Pi extension contract

The Pi extension should consume Context Cognition as prompt support, not action authority.

Possible injection points:

- Focus Slice minimal context,
- Workpoint resume support section,
- Trajectory review support section,
- compaction complete steer,
- model switch continuity packet,
- final report proof summary,
- UIAI diagnostics follow-up.

Pi render requirements:

- first line states `ContextCognitionPacket: advisory`,
- always shows `scope_status`, `stale`, `degraded`, and `source_refs`,
- names Workpoint as action authority,
- names Ontology as semantic context,
- names Evidence as proof context,
- lists excluded context count and reason classes,
- includes do-not-drift boundaries,
- keeps raw files/snippets bounded.

Pi must not:

- replace WorkpointResumePacket,
- replace TrajectoryResumePacket,
- treat packet suggestions as operator authorization,
- promote UIAI/browser research without evidence capture,
- inject large raw snippets under context pressure.

## 13. Focusa tool wrappers

Proposed tools:

- `focusa_context_cognition_packet`
- `focusa_context_cognition_preview`
- `focusa_context_cognition_evaluate`
- `focusa_context_cognition_render`
- `focusa_context_cognition_proof`

Tool behavior:

- read-only or advisory by default,
- explicit `project_root` recommended,
- `continuity_id` required for canonical-scope matching,
- attach evidence only through `focusa_evidence_capture` or `focusa_workpoint_link_evidence`,
- report `next_tools` instead of silently mutating other surfaces.

## 14. Context Curator

The Context Curator is the RepoPrompt-style capability adapted to Focusa authority rules.

Responsibilities:

- select context under a token budget,
- combine files, docs, diffs, snippets, codemaps, and evidence handles,
- preserve `project_root + continuity_id` scope,
- prefer bounded handles over raw blobs,
- label excluded context and why it was excluded,
- expose reviewable packet sections,
- avoid transcript-tail authority.

Selection criteria:

- Workpoint target relevance,
- Trajectory goal/gap relevance,
- Ontology active-object relevance,
- evidence/proof relevance,
- recent diff relevance,
- risk/contradiction relevance,
- token cost,
- staleness.

Codemap policy:

- codemaps are structural summaries, not proof,
- tree-sitter or existing ontology extraction may produce codemaps,
- codemaps should include symbol refs and file ranges,
- selected full snippets require a reason and budget cost.

## 15. Cognition Optimizer

The Cognition Optimizer is the DSPy-style capability.

Responsibilities:

- optimize planner and judge prompts/modules against eval cases,
- compare optimized output to baseline Focusa behavior,
- produce versioned prompt/module artifacts,
- record eval scores and failure classes,
- promote only artifacts that measurably improve outcomes.

Candidate optimization targets:

- Workpoint next-action validation,
- Trajectory gap assessment,
- evidence sufficiency judgment,
- stale-context detection,
- project scope mismatch detection,
- active object resolution,
- contradiction classification,
- compact packet rendering.

Runtime rule:

- optimization runs offline or in CI first,
- runtime consumes approved artifacts only,
- no always-on optimizer dependency in hot paths.

Promotion rule:

- optimized artifacts are config/prompt artifacts, not canonical state,
- promotion requires eval score improvement, rollback path, and proof refs,
- operator steering overrides optimized behavior.

## 16. UIAI interaction

UIAI browser/search diagnostics can feed Context Cognition only as proposal material until captured.

Required status mapping:

| UIAI state | Context Cognition treatment |
|---|---|
| raw browser read | external source candidate |
| diagnostics packet | advisory diagnostics candidate |
| Focusa intake succeeded | evidence candidate with capture ref |
| evidence linked to Workpoint | proof context |
| scope mismatch | degraded; exclude from authority-bearing prompt section |

Context Cognition should display UIAI packet status as:

- `proposal_only`,
- `capture_pending`,
- `captured`,
- `linked`,
- `scope_mismatch`,
- `stale`.

## 17. Menubar interaction

Menubar should display the same packet envelope as API/CLI/Pi.

Required UI card fields:

- packet status,
- scope status,
- project name/root,
- Workpoint id,
- Trajectory id,
- active objects,
- missing evidence count,
- selected context count,
- excluded context count,
- stale/degraded warnings,
- next tools,
- copy/export handoff action.

Menubar controls must call the same scoped API routes. UI selection alone does not create authority.

## 18. Work-loop interaction

Context Cognition may read bounded work-loop status, but it must not mutate work-loop state.

Allowed:

- show active work item,
- show writer owner if available,
- warn when continuous work-loop writer conflict exists,
- suggest `focusa_work_loop_writer_status` or preflight commands.

Not allowed:

- pause/resume/stop/select work-loop items,
- infer operator authorization from packet content,
- schedule background work from advisory context.

## 19. Eval requirements

No Context Cognition improvement is accepted without measurable proof.

Required eval categories:

- wrong project scope,
- stale Workpoint packet,
- missing evidence,
- over-broad context selection,
- under-selected critical file,
- ontology relation error,
- bad Trajectory gap,
- invalid next action,
- prompt token waste,
- compaction recovery failure,
- UIAI scope mismatch,
- menubar display mismatch,
- CLI/API/Pi envelope drift.

Minimum metrics:

- precision of selected context,
- recall of required context,
- contradiction detection accuracy,
- evidence sufficiency accuracy,
- next-action validity,
- token budget savings,
- operator correction rate,
- compaction recovery success,
- packet render parity across API/CLI/Pi/menubar.

## 20. Performance constraints

Context Cognition must be hot-path safe.

Constraints:

- bounded packet size,
- bounded traversal depth,
- bounded file/snippet count,
- no GPU requirement,
- no mandatory external service,
- no raw blob injection by default,
- low-memory mode degrades advisory context first,
- cached packet reuse allowed only with explicit freshness metadata,
- daemon packet route has timeout budget,
- Pi injection has token budget,
- CLI render has explicit `--full` opt-in,
- UIAI diagnostics are summarized before inclusion.

## 21. Proof and release requirements

Minimum proof bundle:

- schema validation test,
- API route envelope parity test,
- CLI render parity test,
- Pi compact render static test,
- menubar card parity test when menubar is wired,
- UIAI capture-status test,
- ontology active-object alignment test,
- evidence missing/proven/stale classification test,
- low-memory degradation test,
- compaction packet-size and latency smoke,
- eval harness baseline report.

## 22. Safety rules

Context Cognition must always display:

- `status`,
- `scope_status`,
- `canonical=false` unless promoted elsewhere,
- `advisory=true`,
- `stale` when applicable,
- `degraded` when applicable,
- `source_refs`,
- `evidence_refs`,
- `next_tools`,
- `do_not_drift`,
- `side_effects`,
- `misuse_hint`.

## 23. Implementation phases

### Phase 1 — Packet schema

Define `ContextCognitionPacket` and static validation.

Deliverables:

- schema spec,
- required fields,
- authority labels,
- packet render examples,
- static tests.

### Phase 2 — Surface contracts

Define API, CLI, Pi, tool, UIAI, menubar, and proof contract stubs.

Deliverables:

- route list,
- CLI command list,
- Pi render contract,
- tool wrapper contract,
- menubar card contract,
- UIAI status mapping.

### Phase 3 — Context Curator

Build scoped token-budgeted context selection.

Deliverables:

- context candidate scoring,
- codemap/snippet/file selection,
- exclusion reasons,
- ontology/evidence alignment.

### Phase 4 — Eval harness

Create failure-driven evals.

Deliverables:

- eval case format,
- baseline Focusa scores,
- metrics dashboard or report,
- proof bundle integration.

### Phase 5 — Cognition Optimizer

Use DSPy-style optimization for selected planner/judge modules.

Deliverables:

- optimized prompt/module artifacts,
- eval comparison reports,
- promotion criteria,
- rollback path.

### Phase 6 — Runtime consumption

Expose approved packets and optimized artifacts in Focusa surfaces.

Deliverables:

- Pi/CLI/menubar render,
- Workpoint/Trajectory support sections,
- low-memory behavior,
- stale/degraded fallback behavior.

## 24. Open design questions

1. Should packet generation live under `focusa_traverse`, a new `focusa_context_cognition` route, or both?
2. Should codemaps come from tree-sitter, existing ontology extraction, or a dedicated project index?
3. What is the first eval target: Workpoint next-action validation, evidence sufficiency, active object resolution, or packet render parity?
4. Which optimized artifacts may enter runtime, and what score threshold is required?
5. How should packet freshness be represented across compaction and model switches?
6. Should Context Cognition packets be stored as runtime cache, evidence artifacts, or both?
7. What is the maximum hot-path packet latency for Pi injection and compaction?

## 25. Recommended first bead

Create a planning/implementation bead for:

> Define `ContextCognitionPacket` schema and cross-surface envelope contract.

Acceptance criteria:

- packet schema includes scope, authority, selected context, ontology frame, evidence frame, reasoning frame, optimization frame, route frame, side effects, and packet-use guidance,
- packet is advisory by default,
- Workpoint remains immediate action authority,
- Ontology remains semantic spine,
- evidence handles remain proof boundary,
- API/CLI/Pi/tool render contracts use same envelope vocabulary,
- static test rejects missing authority/scope/status fields.

## 26. One-line definition

Context Cognition compiles scoped project context, ontology, evidence, and optimized reasoning guidance into bounded advisory packets for safer, smarter Focusa work.
