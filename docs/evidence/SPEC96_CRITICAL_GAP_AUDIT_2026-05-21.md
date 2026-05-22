# Spec96 Critical Implementation Gap Audit — 2026-05-21

Operator directive: missing/partial/improperly implemented Spec96 items must be called out explicitly.

## Method

- Restarted `focusa-daemon` because live `/v1/ontology/tool-contracts` was stale and omitted `focusa_traverse`.
- Rebuilt current source, installed `target/release/focusa-daemon`, restarted `focusa-daemon.service`, then verified live registry parity.
- Read `docs/96-trajectory-projection-and-daemon-stability-spec.md` fully: 2,158 lines, all sections 1–16 and appended eval notes.
- Compared spec requirements against API routes, Pi tools, CLI, runtime responses, tests, docs, and live daemon behavior.

## Freshness proof

```text
/v1/health ok after restart, uptime reset
/v1/ontology/tool-contracts live_count=56
focusa_traverse live index present
node scripts/prove-focusa-tool-contracts-live.mjs --json --safe-fixtures => status=passed, payload_equal=true
```

## Summary verdict

Spec96 is **not fully implemented**. Several high-value slices are present and validated, but the implementation is still a partial/prototype composition layer in major areas. The largest misses are ProjectIdentity, durable Trajectory lifecycle, full Workpoint Resume Packet v2 schema, `focusa_traverse` contract fidelity, work-loop route split completeness, CLI parity, Tool Affordances injection, and LowMem Focus Slice injection.

## What is meaningfully implemented

- `/v1/status` hot summary and `/v1/status/deep` cold split exists (`crates/focusa-api/src/routes/session.rs:133`, `:342`).
- `/v1/resource/mode` exists and runtime LowMem activation works (`crates/focusa-api/src/routes/resource.rs:52`, `:61`).
- Pi tool registry and live daemon registry now agree at 56 contracts after restart.
- Trajectory API routes exist (`crates/focusa-api/src/routes/trajectory.rs:991-998`).
- Pi trajectory tools exist (`apps/pi-extension/src/tools.ts:1490+`).
- `focusa_traverse` API and Pi tool exist (`crates/focusa-api/src/routes/traverse.rs:792-795`, `apps/pi-extension/src/tools.ts:2774+`).
- Workpoint Resume Packet v2 scaffold exists (`crates/focusa-api/src/routes/workpoint.rs:1126-1168`).
- Route latency guardrail passed live on 8787: hot health/status/resource/work-loop/workpoint stayed fast, cold probe did not break health.

## P0 gaps — core Spec96 requirements missing or materially wrong

### P0-1 — ProjectIdentity model/API/tools are missing

Spec96 §6 requires `.focusa-project.json`, multi-signal ProjectIdentity discovery, quorum/fingerprint, and these APIs/tools:

- `GET /v1/project/identity?cwd=...`
- `POST /v1/project/verify`
- `focusa_project_identity`
- `focusa_project_verify`

Current state:

- Routes return 404: `/v1/project/identity`, `/v1/project/verify`.
- No `focusa_project_*` Pi tools or docs exist.
- No `.focusa-project.json` marker exists in repo root.
- Trajectory has an ad-hoc fingerprint based only on `project_root|session/continuity` (`crates/focusa-api/src/routes/trajectory.rs:128-141`).
- Trajectory identity signals are only query + workpoint (`trajectory.rs:586-588`), not marker/git/beads/workspace/cwd/daemon/session/operator quorum.
- Confidence can become `high` only from query+workpoint match (`trajectory.rs:417-425`), not two independent project signals as required.

Impact: Spec96 scope safety is not actually backed by canonical multi-source ProjectIdentity. Current ProjectIdentity is a projection label, not the required identity system.

### P0-2 — FocusaSessionIdentity envelope is not implemented

Spec96 §7.4 requires a shared `FocusaSessionIdentity` with `session_frame_key`, `session_incarnation_id`, `project_root`, `cwd`, `workspace_id`, `process_id`, `started_at`, and `resume_source`, used by checkpoint/resume/evidence/trajectory calls.

Current state:

- Workpoint and trajectory request structs carry flat optional `session_id`, `continuity_id`, and `project_root` fields (`workpoint.rs:31-64`, `trajectory.rs:19-75`).
- No `FocusaSessionIdentity` type exists in `focusa-core` or API code.
- Evidence link request has no project/session identity envelope (`workpoint.rs:73-77`).
- Pi checkpoint/resume builds flat fields, not a shared envelope (`apps/pi-extension/src/tools.ts:1705+`, `:1816+`).

Impact: checkpoint/resume parity is improved but still not the Spec96 identity envelope. Cross-agent and cross-session guarantees remain incomplete.

### P0-3 — Trajectory Projection is not reducer-backed or durable

Spec96 §7–§8 requires TrajectoryProjection metadata, provenance, supersession, milestones, Definition of Done, clarity lifecycle, and reducer-approved projection metadata.

Current state:

- No core `TrajectoryProjection`, `TrajectoryMilestone`, `TrajectoryDefinitionOfDone`, `ProjectIdentity`, or `FocusaSessionIdentity` types exist (`rg` only finds docs/contracts and route-local request structs).
- `focusa_trajectory_define_goal` returns `canonical=false`, `mutates_canonical_state=false` (`trajectory.rs:660-666`).
- `focusa_trajectory_checkpoint` returns `persisted=false` and says reducer-backed metadata does not exist yet (`trajectory.rs:checkpoint_payload`, lines around 860-940).
- No reducer events for accepted trajectory metadata/supersession were found.
- `long_term_goal`, `desired_end_state`, and `current_state` are inferred from Focus State/frame/workpoint strings (`trajectory.rs:428-441`), not durable goal records or required evidence-bound current state.
- Milestones are not implemented beyond doc schemas.
- Definition-of-Done contract is not implemented in API payloads.
- Smartness metrics are not recorded as runtime metrics.

Impact: Trajectory exists as a bounded advisory view, not as the durable per-project trajectory lifecycle described by Spec96.

### P0-4 — Continuous trajectory clarity gate is not enforced globally

Spec96 §7.7 and §7.10 require clarity gate refresh at session start/resume, after compaction, steering changes, Workpoint transitions, evidence updates, failures, degradation, and before nontrivial actions.

Current state:

- `trajectory_clarity_gate_payload` exists and returns status/recommended action (`trajectory.rs:200-244`).
- It is only calculated inside trajectory view/derived APIs.
- No enforcement gate found before general tool execution, Pi nontrivial actions, work-loop actions, evidence updates, or Workpoint transitions.
- Operator confirmation prompt path is represented as text only, not a runtime workflow.

Impact: models can act without the mandatory active clarity record/posture. The gate is informational, not governing.

### P0-5 — LowMem Focus Slice injection is falsely passing static tests but not actually injected

Spec96 §10.5.1.6 requires active LowMem Focus Slice lines: `RESOURCE_MODE`, `LOWMEM_BUDGET`, `CONTEXT_POSTURE`, `BEST_NEXT_TOOLS`, `DO_NOT_USE_BY_DEFAULT`, `PRUNED_COUNTS`, `REHYDRATE_REFS`.

Current state:

- `getResourceModeFocusSliceLines()` is implemented (`apps/pi-extension/src/turns.ts:160-193`).
- `resourceModeLines` is computed (`turns.ts:395`) but never added to `sectionEntries` or the final Focus Slice.
- `tests/spec96_lowmem_focus_slice_wpv2_static_test.sh` only greps for the function/strings; it does not verify the lines are included in injection.

Impact: LowMem prompt posture can be absent exactly when the agent needs it. Existing static test is a false positive.

### P0-6 — Work-loop route split is incomplete

Spec96 §10.3 requires:

- `GET /v1/work-loop/health`
- `GET /v1/work-loop/status?summary_only=true`
- `GET /v1/work-loop/status/deep`
- replay closure routes

Current state:

- Router has `/v1/work-loop`, `/v1/work-loop/status`, and replay routes (`work_loop.rs:2912-2923`).
- `/v1/work-loop/health` returns 404 live.
- `/v1/work-loop/status/deep` returns 404 live.
- Summary payload contains `bounds.summary_only=true`, but no top-level `route_tier`/`summary_only` markers like `/v1/status`.

Impact: Work-loop hot/cold split is only query-based partial behavior, not the explicit route contract.

### P0-7 — Focus State stale-frame recovery still fails in live Pi tool use

Spec96 appended “Stale active-frame validation” says Pi Focus State tools refresh scoped frame identity, retry once, and mirror failed writes to scratchpad fallback.

Observed during this audit:

- `focusa_constraint` failed and auto-saved to scratchpad.
- `focusa_current_focus` failed with `target_frame_is_not_active` and did not record current focus.

Impact: stale frame handling is inconsistent across Focus State tools. Current behavior still leaks tool failure into normal workflow rather than uniformly recovering.

## P1 gaps — implemented partially or with contract mismatch

### P1-1 — `focusa_traverse` input schema diverges from Spec96

Spec96 §9.7.2 requires `tag_mode`, structured `tags: TraverseTagRef[]`, `include_payload`, `include_rehydrate_refs`, `budget_tokens`, and `session_identity`.

Current API request struct has only:

```rust
surface, selector, anchor, query, cursor, limit, depth, radius, fields, tags: Vec<String>, include_full_payload, force_full_payload
```

Evidence: `crates/focusa-api/src/routes/traverse.rs:22-39`.

Observed behavior:

- Structured tag object request to `/v1/traverse/verify-tags` returns HTTP 422.
- Spec field `include_payload=true` is ignored because the implementation expects `include_full_payload`.

Impact: models using the Spec96 schema will fail or silently get summary behavior.

### P1-2 — `focusa_traverse` output schema diverges from Spec96

Spec96 §9.7.3 requires `items` as `TraversedItem` with `anchor`, `tag`, `freshness`, `scope`, and traversal metadata with `caps`, `omitted`, `rehydrate_refs`, `stale_tags`, `verified_tags` inside traversal.

Current output:

- `items` are projected raw records without embedded `anchor`/`tag` fields.
- Tags are top-level `tags`, not attached per item (`traverse.rs:748-772`).
- `traversal` has `limit`, `depth`, `radius`, `metadata`, but no explicit `caps`, `omitted`, `rehydrate_refs` fields as specified (`traverse.rs:724-739`).
- No top-level `do_not_use`, `summary`, or `project_identity`.
- `tag_scheme` lacks spec fields `algorithm`, `length`, `includes_anchor`, `includes_surface_version` (`traverse.rs:759-768`).

Impact: traversal works as a bounded window, but it is not the Spec96 CAS/traversal contract.

### P1-3 — `focusa_traverse` failure taxonomy is off-spec

Spec96 allowed failure classes include `validation_rejected`, `scope_mismatch`, `read_model_lag`, `resource_exhausted`, `cold_path_timeout`, `noncanonical_fallback`.

Current unsupported surface returns `failure_class="unsupported_surface"` (`traverse.rs:697-708`), which is not in Spec96 §9.7.8 or §11.

Impact: recovery matrix/tool-result consumers cannot rely on the defined taxonomy.

### P1-4 — `focusa_traverse` surface coverage is shallow/stubbed

Examples:

- `tool_registry` returns one summary stub, not a searchable/cursorable registry. Live `surface=tool_registry selector=search query=trajectory` returned 0 items.
- `predictions` returns a telemetry summary stub, not prediction records (`traverse.rs:388-396`).
- `snapshots` returns current head summary, not snapshot index/diff metadata (`traverse.rs:398-406`).
- `ontology` selectors do not implement required `working_set`, `path`, typed neighborhood semantics; many selectors fall through to generic object list (`traverse.rs:540-558`).
- No ProjectIdentity/session scope validation for traversal tags.

Impact: major surfaces are present enough to satisfy static greps but not enough to satisfy the full surgical traversal substrate.

### P1-5 — Trajectory tool outputs are missing required fields

Spec96 §9.4 says `focusa_trajectory_propose_workpoint` must include `why_this_next`, `goal_link`, `current_state_delta`, and `completion_evidence_required`.

Current proposal includes mission/action_intent/verification_hooks/blockers/do_not_drift/checkpoint_required, but not those explicit fields (`trajectory.rs:800-858`).

Spec96 §9.3 says assess outputs `blockers`, `recommended_milestones`, `uncertainty_register`, and optional `next_workpoint_candidate`.

Current assess returns gaps/evidence/context/clarity/recommended_action, but not the full schema (`trajectory.rs:750-766`).

Impact: trajectory helps orientation, but not enough to explain why-next and proof obligations as specified.

### P1-6 — Workpoint Resume Packet v2 scaffold is incomplete

Spec96 §7.4.1 requires packet fields including `packet_id`, `generated_at`, `resume_source`, `degraded`, `confidence`, `project_identity`, `session_identity`, rich `resume_summary`, `tool_affordances.recovery`, full `api_provenance` with canonical/freshness/failure classes, and traversal slices with tags/rehydrate refs.

Current v2 packet has schema_version/canonical/failure_class/rendered_summary/resume_summary/workpoint/identity_axes/trajectory/traversal_slices/resource_mode/tool_affordances/api_provenance/session_continuity/identity_confidence/next_tools (`workpoint.rs:1126-1168`).

Missing or partial:

- no `packet_id`, `generated_at`, `resume_source`, top-level `degraded`, `confidence`.
- `project_identity` and `session_identity` are not top-level Spec96 structures; only identity axes/session_continuity.
- `resume_summary` lacks long-term goal, desired end state, current verified state, gap, why_this_next, context_sufficiency, warnings, do_not_use.
- `api_provenance` entries are static/projection claims, not actual recorded route calls/freshness.
- `traversal_slices` are candidate descriptors, not actual traversal results with tags.

Impact: v2 packet is useful but not enough for a fresh model to satisfy the full Spec96 continuation contract.

### P1-7 — ResourceMode API is missing API-level `preflight` and tool_result envelope

Spec96 §9.8 input includes `preflight?: boolean`; results include `tool_result_v1`, `side_effects`, `next_tools`, `resource_mode`, and failure taxonomy.

Current API body lacks `preflight` (`resource.rs:17-22`). Pi implements preflight client-side by GET, but the route contract does not.

Current GET returns status/resource_mode/transition_history/next_tools only (`resource.rs:52-58`), no `details.tool_result_v1`.

Impact: Pi path mostly works, but HTTP/API parity with Spec96 is incomplete.

### P1-8 — LowMem mode hysteresis and durable transition logging are partial

Spec96 §10.5.1.1 requires hysteresis to prevent flapping and transition record persistence or hot ring with durability state.

Current code:

- Records transitions in an in-memory ring (`bounded.rs:412-427`).
- Every transition record has `durability="pending"` (`bounded.rs:455-469`).
- Hysteresis is a note saying future implementation should add multi-sample counters (`bounded.rs:559-562`).

Impact: background resource monitor exists, but Spec96 hysteresis/durability is not implemented.

### P1-9 — Full payload degradation is pressure-gated, not LowMem-mode-gated

Spec96 LowMem acceptance requires full payload requests degrade explicitly under LowMem/tiny budgets.

Current `full_payload_blocked_by_pressure()` blocks only when `pressure_status().active`, not when `resource_mode_status().mode == lowmem` (`bounded.rs:679+`). Forced LowMem alone can still allow full payload if pressure threshold is inactive.

Impact: LowMem does not consistently enforce cold/full payload degradation; tests mostly check metadata presence, not actual forced-LowMem blocking.

### P1-10 — Tool Affordance Catalog is incomplete and not injected into Focus Slice

Spec96 §13.6–§13.7 requires model-facing affordance catalog with when-to-use, when-not-to-use, inputs/defaults, side effects, safety, failure recovery, example, expected result, next tools, and compact `TOOL_AFFORDANCES` Focus Slice section.

Current state:

- Static contract registry has name/label/purpose/family/api/cli/doc/side_effect (`docs/current/focusa-tool-contracts.json`), but not the full affordance catalog.
- Focus Slice injection has no `TOOL_AFFORDANCES` section; search found only compaction usage and no turn-slice injection.
- Safe audit after restart warns `focusa_silent_sessions` doc lacks Expected result and failure_class recovery.

Impact: tool selection is improved but not the Spec96 advertised workflow-power layer.

### P1-11 — CLI parity is missing for Spec96 tools

Spec96 requires CLI routes/commands for trajectory and traverse. Tool contracts claim commands such as `focusa trajectory view`.

Current state:

- `rg` under `crates/focusa-cli/src` found no trajectory, traverse, resource-mode, or project identity commands.
- Existing CLI still calls `/v1/work-loop/status` without summary query in some agent/doctor paths (`crates/focusa-cli/src/main.rs:290`, `commands/doctor.rs:96`).

Impact: API/Pi surfaces exist, but CLI/API parity promised by docs/contracts is false or incomplete.

### P1-12 — Tests include static false positives

Examples:

- LowMem Focus Slice test passes because strings/function exist, even though `resourceModeLines` is unused and not injected.
- Trajectory/traverse golden evals are mostly static string/regex comparisons, not fresh-agent behavioral end-to-end evals.
- `spec96_status_hot_deep_split_runtime_test.sh` requires `cargo`, but `wirebot` cannot access cargo; normal as-user runtime cannot run it.

Impact: green Spec96 tests overstate implementation completeness.

## P2 gaps — smaller but still real

### P2-1 — Status route split is implemented, but hot status still takes the shared Focusa read lock

Spec96 only makes `/v1/health` pure zero/near-zero lock; `/v1/status` hot is allowed bounded state. Still, extra-critical note: status hot clones session/frame summaries under read lock (`session.rs:36-94`). This is likely acceptable but not pure lock-free.

### P2-2 — `focusa_traverse` uses SHA-256 24-hex tags, not configurable short CRC/xxhash/sha1 scheme

Spec96 allows different algorithms, but requires tag scheme fields and collision configurability. Current scheme is fixed SHA-256 24 hex (`traverse.rs:116-130`, `:759-768`).

### P2-3 — Project root mismatch is currently common in live state

Live trajectory view for `/home/wirebot/focusa` degraded because active Workpoint project_root was `/root`. This is correctly detected, but it shows current Workpoint/Trajectory continuity can be polluted by prior session scope until ProjectIdentity is implemented.

### P2-4 — Contracts list CLI commands that do not exist

Example: `focusa_trajectory_view` contract lists `focusa trajectory view`, but CLI has no matching command. This is documentation/contract drift even after live registry parity.

## Section-by-section status map

| Spec96 section | Status | Gap summary |
|---|---|---|
| §1–§4 thesis/design laws | Partial | Documented/advisory; enforcement incomplete for ProjectIdentity, Trajectory authority boundaries, and Focus Slice affordances. |
| §5 conflicts | Partial | C1/C2 mostly fixed; C3 partial; C4 partial; C5 partial; C6 missing; C7 partial. |
| §6 ProjectIdentity | Missing | Dedicated APIs/tools/quorum/marker absent. |
| §7 Trajectory Projection | Partial | Ad-hoc derived view exists; durable schema/lifecycle/milestones/DOD/reducer events absent. |
| §8 Intelligence view | Partial | Context sufficiency exists; relevance rationale/current-state delta/learning_refs/prediction_refs/ask_operator_if/smartness metrics mostly absent. |
| §9 API/tool surface | Partial | Trajectory/resource/traverse API+Pi exist; project APIs/tools and CLI parity missing; schemas mismatched. |
| §10 Stability/LowMem | Partial | Status split and latency good; work-loop explicit health/deep missing; LowMem Focus Slice bug; hysteresis/durable transitions partial. |
| §11 Tool taxonomy | Partial | Pi has taxonomy helpers; API/tool results inconsistent and off-spec classes exist. |
| §12 Pi injection | Partial | Trajectory injected; LowMem and TOOL_AFFORDANCES not actually injected; ProjectIdentity is trajectory-derived only. |
| §13 Tool docs/affordances | Partial | Docs generated broadly; affordance catalog incomplete; safe audit warnings remain. |
| §14 Implementation sequence | Partial | Steps 2–4, 9–11, 15 CLI parity, 17, and parts of 19 remain open. |
| §15 Acceptance | Partial | Surgical traversal and stability partly pass; ProjectIdentity/session/Trajectory/model intelligence/tool usability not complete. |
| §16 Success condition | Not yet | Agents get better orientation, but full long-term/short-term/evidence binding across projects/sessions is not guaranteed. |

## Required next implementation beads

1. Implement ProjectIdentity marker/discovery/quorum plus `/v1/project/identity`, `/v1/project/verify`, `focusa_project_identity`, `focusa_project_verify`.
2. Add shared `FocusaSessionIdentity` type and require it across Workpoint, Trajectory, evidence, and Pi calls.
3. Add reducer-backed Trajectory metadata: accepted goals, provenance, supersession, milestones, Definition-of-Done, and state deltas.
4. Make trajectory clarity gate an enforced session/action precondition where Spec96 requires it.
5. Fix Pi Focus Slice to actually include `resourceModeLines`; upgrade test to assert section inclusion, not string existence.
6. Add `/v1/work-loop/health` and `/v1/work-loop/status/deep`; add route_tier/summary markers to work-loop summary.
7. Bring `focusa_traverse` input/output schemas into Spec96 parity, including structured tags, tag_mode, include_payload alias, rehydrate refs, caps, scope, and tag_scheme fields.
8. Replace traverse stubs with real tool_registry, predictions, snapshots, ontology working_set/path, and metacog/prediction surfaces.
9. Add CLI commands for trajectory, traverse, resource mode, and project identity or remove false CLI claims from contracts.
10. Add API-level `preflight` and `tool_result_v1` envelopes to resource and trajectory HTTP responses.
11. Implement LowMem hysteresis and durable/pending transition persistence, not just an in-memory ring with a future-work note.
12. Gate full payloads by LowMem/emergency mode, not just explicit pressure threshold.
13. Add full Tool Affordance Catalog and `TOOL_AFFORDANCES` Focus Slice injection.
14. Fix Focus State stale-frame retry/fallback consistency across all slot tools.
15. Replace static-only Spec96 false-positive tests with runtime/behavioral assertions for injection, schema, and fresh-agent flows.

