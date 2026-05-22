# Spec96 Critical Gap Audit Recheck — 2026-05-21 post-compaction

Operator directive: reread all of Spec96 fully, compare thoroughly with current code/runtime, identify missing, partial, or improper implementations.

## Method

- Full spec reread: `docs/96-trajectory-projection-and-daemon-stability-spec.md`, lines 1–2158.
- Repo status checked as `wirebot`; no edits made before this artifact.
- Daemon health checked on `127.0.0.1:8787`; no restart needed for this pass.
- Source compared across API, Pi extension, CLI, docs/contracts, tests, and live routes.
- Live probes used bounded/hot routes where possible because ResourceMode was oscillating between `emergency`, `lowmem`, and `normal` during audit.

## Freshness proof

```text
/v1/health => ok=true, sub-ms
/v1/status?summary_only=true => route_tier=hot summary_only=true, cold_omitted present
/v1/project/identity?cwd=/home/wirebot/focusa => canonical=true confidence=high
/v1/project/identity?cwd=/root&project_root=/root => canonical=false degraded=true, unsafe broad root mismatch
node scripts/prove-focusa-tool-contracts-live.mjs --json --safe-fixtures => passed, static_count=58, live_count=58, payload_equal=true
static tests spec96_lowmem_focus_slice_wpv2/static, spec96_traverse_tool/static, spec96_workpoint_resume_packet_v2/static => pass despite runtime/schema gaps below
```

## Updated verdict

Spec96 is still **not fully implemented**. Some items from the earlier audit have moved forward: ProjectIdentity API routes and Pi source registrations now exist, live registry parity is 58 tools, `/v1/status` hot/deep split works, and Workpoint Resume Packet v2 has a scaffold with ResourceMode fields. The remaining gaps are still material: shared identity is not a core/session envelope, Trajectory is not durable/reducer-backed, Focus Slice misses LowMem/tool-affordance injection, `focusa_traverse` diverges from the spec schema and semantics, explicit work-loop health/deep routes are missing, CLI parity is absent, and current post-compaction/native tool exposure still drifts.

## Meaningfully implemented now

- ProjectIdentity API exists: `GET /v1/project/identity`, `POST /v1/project/verify` (`crates/focusa-api/src/routes/project.rs:493`).
- ProjectIdentity discovery uses cwd, operator scope, marker, git root/remote, beads root, workspace file, daemon cwd, quorum, fingerprint, and unsafe-root rejection (`project.rs:210`, `project.rs:420`).
- Pi extension source registers `focusa_project_identity`, `focusa_project_verify`, and `focusa_traverse` (`apps/pi-extension/src/tools.ts:1490`, `:1528`, `:2864`).
- `/v1/status` hot and `/v1/status/deep` cold routes exist (`crates/focusa-api/src/routes/session.rs:138`, `:343`).
- ResourceMode API and background monitor exist (`crates/focusa-api/src/routes/resource.rs:52`, `:61`; `crates/focusa-api/src/server.rs:70`).
- Workpoint Resume Packet v2 scaffold exists and includes ResourceMode payload (`crates/focusa-api/src/routes/workpoint.rs:1193`, `:1065`).
- `focusa_traverse` API routes exist (`crates/focusa-api/src/routes/traverse.rs:23`, `:792`).
- External mutation epoch replaced prior whole-state JSON comparison in daemon reconciliation (`crates/focusa-core/src/runtime/daemon.rs:4695`).

## P0 gaps — core contract failures

### P0-1 — Current compaction/resume still delivered an unsafe `/root` v1 packet as canonical

Spec96 §4.7, §7.4.1, §12.1 require broad runtime roots to be quarantined and post-compaction resume to use canonical v2 packets from hot APIs.

Observed post-compaction packet in this session was `canonical=true` with `project_root=/root`; immediately after compaction, `focusa_workpoint_resume` rejected it as `rejected_unsafe_project_root`. That means the actual model handoff can still violate the spec even though safe Workpoint v2 rendering now exists for `/home/wirebot/focusa`.

Source concern: compaction auto-resume uses stored/scoped packet state and falls back to `S.activeWorkpointSummary`/raw packet if `resume_packet_v2` is absent (`apps/pi-extension/src/compaction.ts:373-392`). It does not guarantee a fresh safe ProjectIdentity + Workpoint v2 packet is rendered into the post-compact message.

Impact: a fresh model can still receive stale/broad-root canonical-looking carryover, exactly what Spec96 says must not happen.

### P0-2 — Shared `ProjectIdentity` and `FocusaSessionIdentity` are not core/session types

Spec96 §6–§7.4 requires ProjectIdentity and FocusaSessionIdentity to be shared envelopes across Workpoint, Trajectory, evidence, Work-loop, Focus Slice, and tool scope.

Current code has route-local ProjectIdentity discovery only; `rg` found no core `ProjectIdentity`, `FocusaSessionIdentity`, `TrajectoryProjection`, `TrajectoryMilestone`, or `TrajectoryDefinitionOfDone` structs/events in `focusa-core`.

Workpoint and trajectory requests still use flat optional fields (`workpoint.rs:38-64`, `trajectory.rs:27-75`). Evidence link has no project/session identity envelope (`workpoint.rs:88-92`).

Impact: ProjectIdentity is an API projection, not yet the canonical identity substrate required by Spec96.

### P0-3 — ProjectIdentity is not integrated into Trajectory/Workpoint scope authority

Spec96 §6.6 requires ProjectIdentity to feed Workpoint, Trajectory, Work-loop, Focus Slice, evidence, and tool scopes.

Trajectory view still computes identity from only query + active Workpoint (`trajectory.rs:356-425`) and uses `stable_project_fingerprint(project_root, session_id)` (`trajectory.rs:128`), not `/v1/project/identity` multi-signal quorum. Live `/v1/trajectory/view?project_root=/home/wirebot/focusa` returned `canonical=true`, but its `project_identity.signals` were only `query` and `workpoint`, not marker/git/beads/workspace.

Impact: Trajectory can call itself high-confidence while bypassing the stronger ProjectIdentity API that Spec96 introduced.

### P0-4 — Durable Trajectory Projection lifecycle is still absent

Spec96 §7–§8 requires durable/reducer-approved trajectory metadata: accepted goals, provenance, supersession, milestones, Definition of Done, state deltas, and smartness metrics.

Current state:

- `focusa_trajectory_define_goal` returns `canonical=false`, `mutates_canonical_state=false` (`trajectory.rs:665`).
- `focusa_trajectory_checkpoint` returns `persisted=false` (`trajectory.rs:911`).
- No reducer-backed trajectory events/types were found.
- Trajectory view derives goals/current state from Focus State, frame, and Workpoint strings (`trajectory.rs:356-581`).

Impact: Trajectory is an advisory derived view only, not the durable per-project trajectory lifecycle described in Spec96.

### P0-5 — Trajectory clarity gate is informational, not mandatory session/action gating

Spec96 §7.7 and §7.10 require clarity refresh before nontrivial actions and after session start/resume, compaction, steering, evidence, failures, and degradation.

Current code builds `trajectory_clarity_gate_payload` only inside Trajectory APIs (`trajectory.rs:200-244`). No enforcement was found before general tool execution, Workpoint transition, evidence link, work-loop mutation, or Pi nontrivial actions.

Impact: a model can still act while trajectory is stale/unclear/conflicted; posture is advisory only.

### P0-6 — LowMem Focus Slice lines are computed but not injected

Spec96 §10.5.1.6 requires `RESOURCE_MODE`, `LOWMEM_BUDGET`, `CONTEXT_POSTURE`, `BEST_NEXT_TOOLS`, `DO_NOT_USE_BY_DEFAULT`, pruned counts, and rehydrate guidance in the Focus Slice when constrained.

Source:

- `getResourceModeFocusSliceLines()` exists and returns the required lines (`apps/pi-extension/src/turns.ts:182-200`).
- `resourceModeLines` is computed (`turns.ts:408`).
- `sectionEntries` starts at `turns.ts:410` but never includes `resourceModeLines`.

Live symptom: the operator-provided Focus Slice omitted `RESOURCE_MODE` even when ResourceMode was `emergency/rss_hard_exceeded` earlier in this audit.

Impact: the model may not see the most important LowMem operating constraint.

### P0-7 — `TOOL_AFFORDANCES` Focus Slice injection is missing

Spec96 §13.6–§13.7 requires a model-facing affordance catalog and compact Focus Slice `TOOL_AFFORDANCES` section.

Current state:

- Workpoint v2 has a minimal `tool_affordances` object (`workpoint.rs:1150`).
- Trajectory view has a short `tool_affordances` array inside `intelligence_view` (`trajectory.rs:580-586`).
- Pi Focus Slice has no `TOOL_AFFORDANCES` section in `turns.ts:410-439`.

Impact: tool selection remains mostly inferred from prompt/tool names, not from the advertised workflow-power catalog required by Spec96.

### P0-8 — `focusa_traverse` is not spec-faithful and has incorrect `current` semantics

Spec96 §9.7 and §10.7 require bounded traversal with per-item anchors/tags, structured tag verification, caps/omitted/rehydrate metadata, scope, and correct surface selectors.

Current schema and behavior gaps:

- Input accepts `tags: Vec<String>`, `include_full_payload`, `force_full_payload`; it lacks structured `TraverseTagRef[]`, `tag_mode`, `include_payload`, `include_rehydrate_refs`, `budget_tokens`, `session_identity` (`traverse.rs:23-39`).
- Structured tag verification request returns HTTP 422.
- Output returns projected raw items without embedded `anchor`/`tag`; tags are separate top-level records (`traverse.rs:748-772`).
- `traversal` lacks the spec `caps`, `omitted`, `rehydrate_refs`, `stale_tags`, and `verified_tags` shape (`traverse.rs:711-739`).
- Failure classes include non-spec values `unsupported_surface` and `cold_full_payload_blocked_by_pressure` (`traverse.rs:702`, `:752`).
- `selector=current` is generic and returns the first item when no anchor is provided (`traverse.rs:436-448`), not the active/current Workpoint. Live `surface=workpoints selector=current` returned an old LowMem Workpoint, not active Workpoint `019e4c8c...`.
- Major surfaces are shallow/stubbed; `tool_registry` returns a single summary stub (`traverse.rs:581-585`).

Impact: central surgical traversal acceptance is not met; models following Spec96 schema will fail or receive misleading slices.

### P0-9 — Explicit work-loop health/deep routes are missing

Spec96 §10.3 requires `GET /v1/work-loop/health` and `GET /v1/work-loop/status/deep`.

Live probes:

```text
/v1/work-loop/health => HTTP 404
/v1/work-loop/status/deep => HTTP 404
```

Router only registers `/v1/work-loop`, `/v1/work-loop/status`, replay routes, and control routes (`crates/focusa-api/src/routes/work_loop.rs:2912-2939`).

Impact: route split is still query-based partial behavior, not the explicit hot/cold contract.

### P0-10 — CLI parity for Spec96 tools is absent despite contracts claiming commands

Spec96 §9.7.7 and §9.8 require CLI surfaces for traverse/resource/trajectory, and contracts list commands such as `focusa project identity` and `focusa trajectory view`.

Current CLI has no project, trajectory, traverse, or resource-mode command domains (`crates/focusa-cli/src/main.rs:120-170`; `rg trajectory|traverse|resource|project crates/focusa-cli/src` finds no command implementations beyond unrelated docs/root strings).

Impact: API/Pi parity exists partly, but CLI parity promised by docs/contracts is false.

### P0-11 — Current model-visible Pi tool surface is stale compared with live registry/source

Live `/v1/ontology/tool-contracts` advertises `focusa_project_identity`, `focusa_project_verify`, and `focusa_traverse`, and source registers them (`tools.ts:1490`, `:1528`, `:2864`). The current Pi operator tool list available to this model still does not include these native tools.

Impact: the spec requirement that official tools stay advertised/callable to the agent is not satisfied in the current post-compaction session, even though the daemon registry is fresh.

## P1 gaps — partial/mismatched implementations

### P1-1 — Workpoint Resume Packet v2 is a scaffold, not the full schema

Spec96 §7.4.1 requires `packet_id`, `generated_at`, `resume_source`, `degraded`, `confidence`, top-level `project_identity`, top-level `session_identity`, rich resume summary, traversal tags/rehydrate refs, and actual API provenance.

Current v2 keys: `schema_version`, `canonical`, `failure_class`, `rendered_summary`, `resume_summary`, `workpoint`, `identity_axes`, `trajectory`, `traversal_slices`, `resource_mode`, `tool_affordances`, `api_provenance`, `session_continuity`, `identity_confidence`, `next_tools` (`workpoint.rs:1193-1234`).

Missing/partial:

- no `packet_id`, `generated_at`, `resume_source`, top-level `degraded`, top-level `confidence`.
- no top-level Spec96 `project_identity` or `session_identity` objects.
- `resume_summary` lacks long-term goal, desired end state, current verified state, gap, why-this-next, context sufficiency, warnings, and do-not-use (`workpoint.rs:1213-1222`).
- `traversal_slices` are descriptors/candidates, not actual traverse outputs with tags/rehydrate refs (`workpoint.rs:1121-1148`).
- `api_provenance` entries for trajectory/traverse are projected/candidate claims, not actual call results/freshness/failure classes (`workpoint.rs:1227-1231`).

Impact: v2 helps, but a fresh model still lacks the full Spec96 continuation contract.

### P1-2 — ResourceMode API is missing route-level `preflight` and uniform `tool_result_v1`

Spec96 §9.8 requires input `preflight?: boolean` and results with `tool_result_v1`, side effects, next tools, resource mode, and failure taxonomy.

Current API body has only `action`, `mode`, `reason` (`resource.rs:17-22`). GET returns status/resource_mode/transition history/next_tools, no `tool_result_v1` (`resource.rs:52-58`). Pi implements preflight client-side, not API parity.

Impact: HTTP/API agents cannot perform spec preflight safely or rely on a uniform result envelope.

### P1-3 — ResourceMode hysteresis/durability is explicitly future work and flapping was observed

Spec96 §10.5.1.1 requires hysteresis and transition durability before auto mode changes.

Current `resource_mode_status()` embeds a note that future implementation should add multi-sample hysteresis counters (`bounded.rs:561`). Transitions are in-memory ring records with `durability="pending"` (`bounded.rs:412-469`). During this audit, ResourceMode moved `emergency -> lowmem -> normal` within minutes as RSS changed.

Impact: auto fallback works, but the no-flap/reliable transition contract is not implemented.

### P1-4 — Full payload gating is pressure-based, not mode-based

Spec96 LowMem acceptance requires cold/full payload requests to degrade under LowMem/emergency budgets unless explicitly safe.

Current helper blocks only when `pressure_status().active` and ignores `resource_mode_status().mode` (`bounded.rs:679-682`). Forced LowMem/emergency mode alone does not block full payloads.

Impact: LowMem can fail to enforce surgical behavior if pressure flag is inactive or mode has just flapped.

### P1-5 — Trajectory output schemas miss required intelligence/proof fields

Spec96 §8.2 requires relevance rationale, current state delta, learning refs, prediction refs, ask-operator-if, and negative context. Current `intelligence_view` has context sufficiency, similarity group, clarity gate, do_not_use, next_workpoint_candidate, tool_affordances, recent results/decisions/constraints (`trajectory.rs:566-591`).

Spec96 §9.3 assess requires blockers, recommended milestones, uncertainty register, optional Workpoint candidate. Current assess returns gaps/evidence/context/clarity/recommended_action but not full schema (`trajectory.rs:693-773`).

Spec96 §9.4 proposal requires top-level `next_action`, `verification_required`, `why_this_next`, `goal_link`, `current_state_delta`, and `completion_evidence_required`. Current proposal returns an advisory candidate with hooks/blockers/do-not-drift/checkpoint policy but lacks those fields (`trajectory.rs:816-883`).

Impact: Trajectory orientation is useful but still under-specified for proof-based why-next execution.

### P1-6 — Tool result taxonomy remains inconsistent

Spec96 §11 enumerates valid failure classes. Current code still emits off-spec classes such as `unsupported_surface` and `cold_full_payload_blocked_by_pressure` from traverse (`traverse.rs:702`, `:752`). Some route errors return prose/status without retry posture or `tool_result_v1`.

Impact: recovery matrix consumers cannot uniformly branch by spec failure class.

### P1-7 — Workpoint/evidence identity and read-model lag handling are incomplete

Spec96 requires checkpoint/resume/evidence paths to share identity envelopes and classify accepted-but-not-visible evidence as `read_model_lag`.

Current evidence link request has no project root/session/continuity identity (`workpoint.rs:88-92`). Pending evidence response says status `pending` but lacks `failure_class=read_model_lag` and retry posture (`workpoint.rs:1384-1394`).

Impact: evidence can still link across ambiguous scope and pending visibility is not classified per spec.

### P1-8 — ProjectIdentity route exists but docs/status details drift

Docs say unsafe broad roots return `status=unsafe_scope`; live `/v1/project/identity?cwd=/root&project_root=/root` returns top-level `status=degraded` and `project_identity.status=mismatch`, with unsafe reason in mismatches.

Also, the Focusa repo currently has no `.focusa-project.json` marker; high confidence comes from git+beads+workspace, which is allowed but leaves the marker schema unexercised in this project.

Impact: behavior is safe, but docs/contracts and live field values are not aligned.

### P1-9 — Safe/static tests overstate completeness

The following tests pass while the gaps above remain:

```text
tests/spec96_lowmem_focus_slice_wpv2_static_test.sh
tests/spec96_traverse_tool_static_test.sh
tests/spec96_workpoint_resume_packet_v2_static_test.sh
```

Examples:

- LowMem static test greps for `getResourceModeFocusSliceLines()` strings, but does not assert the lines are included in `sectionEntries` or emitted Focus Slice.
- Traverse static test checks route/tool existence and tag-policy strings, but not Spec96 structured tag schema or correct `current` semantics.
- Workpoint v2 static test checks scaffold strings, not full v2 schema or actual traversal tag/provenance payloads.

Impact: green Spec96 tests can hide material runtime/schema gaps.

## P2 gaps — smaller but still real

- `focusa_traverse` tag scheme is fixed SHA-256 24-hex URI tags; spec asks for returned algorithm/length/includes-anchor/includes-surface-version and configurable collision strategy (`traverse.rs:116-130`, `:759-768`).
- Tool registry contracts still list CLI commands for project/trajectory tools while `parity_status="domain"` and CLI implementations are absent (`apps/pi-extension/src/tool-contracts.ts:31-177`).
- Tool Affordance Catalog fields are sparse: registry has label/purpose/routes/docs/side effects, but not full when-to-use/when-not-to-use/default inputs/examples/recovery/result-shape workflow guide for every tool.
- Workpoint v2 uses `identity_axes` rather than the exact top-level `project_identity`/`session_identity` structures required by Spec96.
- Trajectory view can be `canonical=true` without durable goal source or evidence refs if Focus State/Workpoint strings align; Spec96 wants long-term/desired/current-state proof with provenance.
- `/v1/status` hot split is good, but many safe proof fixtures still do not cover new Spec96 families like project, trajectory, traverse, and resource mode.

## Section status map

| Spec96 section | Recheck status | Notes |
|---|---|---|
| §1–§4 design laws | Partial | Intent preserved; broad-root guard exists but compaction/session still emitted stale `/root` canonical packet. |
| §5 conflicts | Partial | C1/C2 mostly fixed; C3/C4/C6/C7 still partial. |
| §6 ProjectIdentity | Partial | API/tool source exists; not core type/envelope/integrated everywhere; marker absent in repo. |
| §7 Trajectory Projection | Partial | Derived view only; durable lifecycle, DOD, milestones, reducer metadata missing. |
| §8 Intelligence view | Partial | Sufficiency exists; relevance/current-delta/learning/prediction/ask-operator fields missing. |
| §9 API/tool surface | Partial | Project/resource/trajectory/traverse API+Pi source exist; schemas/CLI/model-visible exposure incomplete. |
| §10 Stability/LowMem | Partial | Status split works; work-loop explicit health/deep missing; LowMem injection/hysteresis/full-payload gating incomplete. |
| §11 Taxonomy | Partial | Off-spec failure classes and missing envelopes remain. |
| §12 Pi injection | Partial | Trajectory lines exist; ResourceMode and TOOL_AFFORDANCES missing from Focus Slice; post-compaction v2 not guaranteed. |
| §13 Tool docs/affordances | Partial | Docs/contracts broad but sparse; workflow affordance catalog incomplete. |
| §14 Implementation sequence | Partial | Steps 2–5 partly done; 7–18 mostly partial; 19 tests are static-heavy. |
| §15 Acceptance | Not met | Surgical traversal, packets, stability, identity, trajectory, model intelligence, and usability all have open gaps. |
| §16 Success condition | Not yet | Agents get better orientation but cannot reliably see full long-term/short-term/proof binding across compaction/resources/tool exposure. |

## Highest-leverage next repairs

1. Make compaction resume always render a fresh safe Workpoint Resume Packet v2 using ProjectIdentity + Workpoint + Trajectory + bounded traverse; never inject `/root` packets as canonical.
2. Promote ProjectIdentity and FocusaSessionIdentity into shared core/API/Pi types; require them on Workpoint checkpoint/resume/evidence and Trajectory calls.
3. Wire ProjectIdentity API results into Trajectory, Workpoint scope, Focus Slice, and Tool Doctor instead of query+workpoint-only identity.
4. Implement durable reducer-backed Trajectory metadata: accepted goals, provenance, supersession, milestones, Definition of Done, current-state deltas.
5. Inject ResourceMode and TOOL_AFFORDANCES into Pi Focus Slice; fix tests to assert actual emitted sections.
6. Bring `focusa_traverse` to Spec96 schema/output semantics and fix surface selectors, especially `workpoints/current`.
7. Add `/v1/work-loop/health` and `/v1/work-loop/status/deep` explicit routes.
8. Add CLI commands or remove false CLI claims from contracts/docs.
9. Add API preflight + `tool_result_v1` to ResourceMode and normalize failure classes across routes.
10. Replace static-only Spec96 proof with runtime schema/behavior tests for compaction packet, Focus Slice, traverse tags, LowMem, and fresh-agent tool exposure.
