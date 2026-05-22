# Spec96 Focus Slice + Work-loop Route Repair Proof — 2026-05-21

## Scope

Source-level repairs after `SPEC96_CRITICAL_GAP_AUDIT_RECHECK_2026-05-21.md`.

## Fixed: Focus Slice ResourceMode + TOOL_AFFORDANCES injection

- `apps/pi-extension/src/turns.ts:182` builds LowMem/ResourceMode Focus Slice lines.
- `apps/pi-extension/src/turns.ts:206` builds compact `TOOL_AFFORDANCES` guidance.
- `apps/pi-extension/src/turns.ts:454` injects `resource_mode` into `sectionEntries`.
- `apps/pi-extension/src/turns.ts:457` injects `tool_affordances` into `sectionEntries`.
- `tests/spec96_lowmem_focus_slice_wpv2_static_test.sh:8` now fails if `resourceModeLines` is computed but not injected.
- `tests/spec96_focus_slice_runtime_injection_test.mts:130` runtime-mocks LowMem and asserts `RESOURCE_MODE`, `LOWMEM_BUDGET`, `TOOL_AFFORDANCES`, recovery, and do-not-use lines are present in the injected Focus Slice.

Verified:

```text
tests/spec96_lowmem_focus_slice_wpv2_static_test.sh PASS
tests/spec96_focus_slice_runtime_injection_test.sh PASS
apps/pi-extension: npx tsc --noEmit PASS
```

## Source patched: explicit Work-loop hot/deep routes

- `crates/focusa-api/src/routes/work_loop.rs:1798` adds hot `/v1/work-loop/health` handler.
- `crates/focusa-api/src/routes/work_loop.rs:1848` adds hot summary metadata: `route_tier`, `summary_only`, `deep_status_route`, and `cold_omitted`.
- `crates/focusa-api/src/routes/work_loop.rs:1944` adds cold deep metadata: `route_tier=cold`, `summary_only=false`, `cold_omitted=[]`.
- `crates/focusa-api/src/routes/work_loop.rs:2068` adds `status_deep` wrapper.
- `crates/focusa-api/src/routes/work_loop.rs:2970` registers `/v1/work-loop/health` and `/v1/work-loop/status/deep`.
- `tests/spec96_work_loop_route_split_static_test.sh` verifies source route registration and metadata.
- `tests/spec96_work_loop_route_split_runtime_test.sh` verifies live route contract after approved rebuild/restart.

Verified now:

```text
tests/spec96_work_loop_route_split_static_test.sh PASS
git diff --check PASS
```

Runtime route proof remains pending until an approved daemon rebuild/restart loads the changed Rust source.

## Source patched: compaction resume injects only safe WorkpointResumePacketV2

- `apps/pi-extension/src/state.ts` adds `normalizeWorkpointResumePacketEnvelope()` so `/workpoint/resume` v2 payloads stay attached to scoped Workpoint packets.
- `apps/pi-extension/src/session.ts` and `apps/pi-extension/src/compaction.ts` preserve `resume_packet_v2` when caching scoped packets.
- `apps/pi-extension/src/compaction.ts` refreshes the Workpoint resume packet at post-compact auto-resume time.
- `apps/pi-extension/src/compaction.ts` renders `## WorkpointResumePacketV2` only when the packet is safe for current `project_root + continuity_id` and canonical.
- `apps/pi-extension/src/compaction.ts` no longer falls back to raw/v1 `scopedPacket` JSON or `S.activeWorkpointSummary` in the auto-resume steer.
- `tests/spec96_compaction_resume_injection_v2_static_test.sh` now fails if auto-resume can still inject raw/v1 scoped packets.

Verified now:

```text
tests/spec96_compaction_resume_injection_v2_static_test.sh PASS
tests/spec96_workpoint_post_compaction_resume_static_test.sh PASS
apps/pi-extension: npx tsc --noEmit PASS
```

Runtime compaction handoff proof remains pending until an actual compaction occurs under the updated Pi extension.

## Source patched: focusa_traverse Spec96 schema/output parity

- `crates/focusa-api/src/routes/traverse.rs` accepts structured tag refs, `tag_mode`, `include_payload`, `include_rehydrate_refs`, `budget_tokens`, and `session_identity`.
- `crates/focusa-api/src/routes/traverse.rs` wraps response items as TraversedItem-style records with `anchor`, `ordinal`, `tag`, `surface_version`, `freshness`, `scope`, and `data`.
- `crates/focusa-api/src/routes/traverse.rs` adds traversal `caps`, `omitted`, `rehydrate_refs`, `verified_tags`, `stale_tags`, and tag-scheme metadata.
- `crates/focusa-api/src/routes/traverse.rs` normalizes unsupported surface/full payload classes to Spec96 taxonomy (`validation_rejected`, `resource_exhausted`, `read_model_lag`).
- `apps/pi-extension/src/tools.ts` advertises structured tags and schema aliases for `focusa_traverse`.
- `tests/spec96_traverse_schema_static_test.sh` and `tests/spec96_traverse_schema_runtime_test.sh` cover schema/output parity.

Verified now:

```text
tests/spec96_traverse_schema_static_test.sh PASS
tests/spec96_traverse_tool_static_test.sh PASS
tests/spec96_traverse_adapters_static_test.sh PASS
apps/pi-extension: npx tsc --noEmit PASS
```

Runtime traverse schema proof remains pending until an approved daemon rebuild/restart loads the changed Rust source.


## Source patched: shared ProjectIdentity/FocusaSessionIdentity envelope

- `crates/focusa-core/src/types.rs` adds shared `ProjectIdentityRecord`, `ProjectIdentitySignalRecord`, and `FocusaSessionIdentity` structs.
- `crates/focusa-api/src/routes/workpoint.rs` accepts `session_identity` on checkpoint, resume, and evidence link requests.
- `crates/focusa-api/src/routes/workpoint.rs` applies session identity defaults for project root, continuity id, and temporal session id before scope checks.
- `crates/focusa-api/src/routes/workpoint.rs` rejects evidence links when `session_identity` mismatches Workpoint project or continuity scope.
- `crates/focusa-api/src/routes/workpoint.rs` classifies accepted-but-not-visible evidence as `failure_class=read_model_lag` with safe retry posture.
- `crates/focusa-api/src/routes/trajectory.rs` accepts `session_identity` on all POST trajectory calls and scopes projections from the envelope when flat fields are absent.
- `crates/focusa-api/src/routes/trajectory.rs` now incorporates `/project/identity` quorum output into Trajectory view as `project_identity_api`, `quorum_status`, and `quorum_confidence`.
- `apps/pi-extension/src/state.ts` adds `buildFocusaSessionIdentity()` using `/project/identity` when the project root is safe.
- `apps/pi-extension/src/tools.ts` attaches `session_identity` to Workpoint checkpoint/resume/evidence and Trajectory define/assess/propose/checkpoint/resume payloads.
- `tests/spec96_session_identity_envelope_static_test.sh` verifies core/API/Pi identity envelope coverage and read-model-lag taxonomy.

Verified now:

```text
tests/spec96_session_identity_envelope_static_test.sh PASS
tests/spec96_trajectory_focus_slice_static_test.sh PASS
tests/spec96_trajectory_clarity_gate_static_test.sh PASS
tests/spec96_trajectory_agent_golden_eval_test.sh PASS
tests/spec96_trajectory_hierarchy_grouping_static_test.sh PASS
tests/spec96_trajectory_workpoint_handoff_static_test.sh PASS
apps/pi-extension: npx tsc --noEmit PASS
git diff --check PASS
```

Rust `cargo check -p focusa-api` is pending because the `wirebot` user has no accessible cargo binary in this environment; root-owned cargo under `/root/.cargo/bin` is permission denied for user-safe execution.

## Source patched: durable reducer-backed Trajectory lifecycle

- `crates/focusa-core/src/types.rs` adds reducer-owned Trajectory lifecycle records: `TrajectoryProjectionRecord`, milestones, goal provenance, Definition-of-Done, checkpoints, and state deltas.
- `crates/focusa-core/src/types.rs` adds `FocusaState.trajectory` plus `trajectory_goal_defined`, `trajectory_checkpoint_persisted`, and `trajectory_state_delta_recorded` events.
- `crates/focusa-core/src/reducer.rs` persists canonical Trajectory goals, supersedes same-scope prior trajectories, records checkpoints, and applies current-state/evidence deltas.
- `crates/focusa-api/src/routes/trajectory.rs` makes accepted `define-goal` reducer-backed (`canonical=true`, `persisted=true`, `mutates_canonical_state=true`) while preserving advisory/no-execution authority.
- `crates/focusa-api/src/routes/trajectory.rs` records checkpoint packets and assessment state deltas via reducer events.
- `crates/focusa-api/src/routes/trajectory.rs` exposes durable lifecycle metadata in view output: active trajectory id, checkpoint/delta counts, goal provenance, milestones, and Definition-of-Done.
- `tests/spec96_trajectory_reducer_lifecycle_static_test.sh` verifies durable lifecycle records, reducer events, API persistence hooks, and queryable lifecycle output.

Verified now:

```text
tests/spec96_trajectory_reducer_lifecycle_static_test.sh PASS
tests/spec96_trajectory_clarity_gate_static_test.sh PASS
tests/spec96_trajectory_focus_slice_static_test.sh PASS
tests/spec96_trajectory_workpoint_handoff_static_test.sh PASS
tests/spec96_trajectory_hierarchy_grouping_static_test.sh PASS
tests/spec96_trajectory_agent_golden_eval_test.sh PASS
apps/pi-extension: npx tsc --noEmit PASS
git diff --check PASS
```

Rust `cargo check -p focusa-api` remains pending because `wirebot` lacks an accessible cargo binary in this environment.

## Source patched: Trajectory clarity gate precondition on mutating Pi tools

- `apps/pi-extension/src/tools.ts` adds `enforceTrajectoryClarityPrecondition()` which refreshes `/trajectory/view?mode=summary` for the active project/continuity scope.
- `focusa_workpoint_checkpoint` blocks canonical checkpoint writes when clarity is `unclear`, `conflicted`, project identity is mismatched, or operator input is required.
- `focusa_evidence_capture` and `focusa_workpoint_link_evidence` enforce the clarity gate for scope conflicts while allowing evidence collection to resolve missing current-state facts.
- Mutating payloads include `trajectory_clarity_precondition` so downstream API/reducer logs can inspect the gate posture.
- `tests/spec96_trajectory_clarity_gate_static_test.sh` now verifies Pi Workpoint/evidence mutation tools enforce the clarity precondition.

Verified now:

```text
tests/spec96_trajectory_clarity_gate_static_test.sh PASS
tests/spec96_trajectory_reducer_lifecycle_static_test.sh PASS
tests/spec96_session_identity_envelope_static_test.sh PASS
apps/pi-extension: npx tsc --noEmit PASS
git diff --check PASS
```

## Source patched: CLI parity for Spec96 project/trajectory/traverse/resource domains

- `crates/focusa-cli/src/commands/project.rs` adds `focusa project identity` and `focusa project verify` mapped to `/v1/project/identity` and `/v1/project/verify`.
- `crates/focusa-cli/src/commands/trajectory.rs` adds `focusa trajectory view|define-goal|assess|propose-workpoint|checkpoint|resume` mapped to `/v1/trajectory/*`.
- `crates/focusa-cli/src/commands/traverse.rs` adds `focusa traverse read` and `focusa traverse verify-tags`, including Spec96 fields `tag_mode`, `include_payload`, `include_rehydrate_refs`, and `budget_tokens`.
- `crates/focusa-cli/src/commands/resource.rs` adds `focusa resource status|activate-lowmem|deactivate-lowmem|set-mode` mapped to `/v1/resource/mode`.
- `crates/focusa-cli/src/main.rs` and `commands/mod.rs` register the four new domains.
- `docs/current/CLI_REFERENCE_CURRENT.md` now advertises Spec96 CLI parity commands and examples.
- `tests/spec96_cli_parity_static_test.sh` verifies registration, API route mapping, schema flags, and docs coverage.

Verified now:

```text
tests/spec96_cli_parity_static_test.sh PASS
apps/pi-extension: npx tsc --noEmit PASS
git diff --check PASS
```

Rust compile remains pending because `wirebot` lacks an accessible cargo binary in this environment.

## Source patched: ResourceMode API preflight and tool_result envelope

- `crates/focusa-api/src/routes/resource.rs` accepts `preflight` and reports requested mode without mutating the runtime override.
- `crates/focusa-api/src/routes/resource.rs` adds `details.tool_result_v1` with `ok`, `status`, `failure_class`, `canonical`, `degraded`, retry posture, side effects, and next tools.
- `tests/spec96_resource_mode_envelope_static_test.sh` verifies ResourceMode API preflight/envelope support and Pi/CLI caller parity.

Verified now:

```text
tests/spec96_resource_mode_envelope_static_test.sh PASS
tests/spec96_cli_parity_static_test.sh PASS
tests/spec96_lowmem_focus_slice_wpv2_static_test.sh PASS
apps/pi-extension: npx tsc --noEmit PASS
git diff --check PASS
```

Runtime ResourceMode preflight proof remains pending until an approved daemon rebuild/restart loads the changed Rust route.

## Source patched: LowMem full-payload gate uses ResourceMode

- `crates/focusa-api/src/routes/bounded.rs` now blocks full payloads when `ResourceMode` is `lowmem` or `emergency`, even if raw `pressure_status.active` is false.
- `force_full_payload=true` remains the explicit override; non-full requests are never blocked.
- `tests/spec96_lowmem_full_payload_gate_static_test.sh` verifies the ResourceMode gate, unit coverage, and runtime stress metadata coverage.

Verified now:

```text
tests/spec96_lowmem_full_payload_gate_static_test.sh PASS
tests/spec96_lowmem_surgical_agent_static_test.sh PASS
tests/spec96_lowmem_focus_slice_wpv2_static_test.sh PASS
tests/spec96_lowmem_background_throttle_static_test.sh PASS
tests/spec96_resource_mode_envelope_static_test.sh PASS
apps/pi-extension: npx tsc --noEmit PASS
git diff --check PASS
```

Runtime proof remains pending until an approved daemon rebuild/restart loads the changed Rust route.

## Source patched: ResourceMode hysteresis and transition evidence

- `crates/focusa-api/src/routes/bounded.rs` now tracks `ResourceModeHysteresisRuntime` with immediate escalation and delayed recovery samples (`FOCUSA_RESOURCE_MODE_HYSTERESIS_RECOVERY_SAMPLES`, default 3).
- `ResourceModeStatus.hysteresis` exposes `hysteresis_v1` with raw/effective mode, action, recovery candidate/count, and sample threshold.
- Transition records keep `hysteresis_state`, `durability="pending"`, capped ring history, and omitted count; daemon background monitor continues to observe transitions without active session.
- `tests/spec96_resource_mode_hysteresis_static_test.sh` verifies hysteresis state, anti-flap policy, transition evidence, and daemon monitor wiring.

Verified now:

```text
tests/spec96_resource_mode_hysteresis_static_test.sh PASS
tests/spec96_lowmem_full_payload_gate_static_test.sh PASS
tests/spec96_lowmem_surgical_agent_static_test.sh PASS
tests/spec96_resource_mode_envelope_static_test.sh PASS
apps/pi-extension: npx tsc --noEmit PASS
git diff --check PASS
```

Runtime proof remains pending until an approved daemon rebuild/restart loads the changed Rust route.

## Source patched: Workpoint evidence link LowMem timeout

Observed operator error: `focusa_workpoint_link_evidence` returned `blocked: hot route timed out` while LowMem was active.

Root cause in source: `/v1/workpoint/evidence/link` dispatched the evidence event, then waited up to 240×50ms (12s) for read-model visibility; Pi aborts Focusa API calls after 5s, so accepted writes could be reported as hot-route timeouts.

Patch:

- `crates/focusa-api/src/routes/workpoint.rs` bounds evidence visibility polling by ResourceMode: emergency=1 attempt, lowmem=2, constrained=8, normal=40.
- Pending accepted links return HTTP 202 with `failure_class="read_model_lag"`, `retry_posture="safe_retry"`, `resource_mode`, and next tools instead of hanging to client timeout.
- LowMem accepted responses omit the full Workpoint packet (`summary_only=true`) to avoid returning large payloads on a write route.
- `tests/spec96_workpoint_evidence_lowmem_timeout_static_test.sh` verifies bounded wait, read-model-lag envelope, no full payload under LowMem, and no-link fallback availability.

Verified now:

```text
tests/spec96_workpoint_evidence_lowmem_timeout_static_test.sh PASS
tests/spec96_lowmem_surgical_agent_static_test.sh PASS
tests/spec96_resource_mode_hysteresis_static_test.sh PASS
tests/spec96_resource_mode_envelope_static_test.sh PASS
apps/pi-extension: npx tsc --noEmit PASS
git diff --check PASS
```

Live sessions need daemon rebuild/restart before this route behavior changes.

## Source patched: Runtime stress covers Workpoint evidence link timeout repair

- `tests/spec96_lowmem_surgical_agent_stress_test.sh` now creates a bounded Workpoint checkpoint under LowMem and links evidence with a 5s client budget.
- The runtime assertion accepts HTTP 200 `accepted` or HTTP 202 `pending` with `failure_class="read_model_lag"` and `retry_posture="safe_retry"`; timeout/000 or missing envelope fails.
- `crates/focusa-api/src/routes/workpoint.rs` reuses `workpoint_visibility_wait_attempts()` for checkpoint promotion and evidence visibility so both paths remain below Pi timeout under LowMem.

Verified now:

```text
tests/spec96_workpoint_evidence_lowmem_timeout_static_test.sh PASS
tests/spec96_lowmem_surgical_agent_static_test.sh PASS
tests/spec96_resource_mode_hysteresis_static_test.sh PASS
tests/spec96_lowmem_full_payload_gate_static_test.sh PASS
apps/pi-extension: npx tsc --noEmit PASS
git diff --check PASS
```

Live runtime stress still requires an approved daemon rebuild/restart to execute against the patched route implementation.

## Source patched: Tool Doctor surfaces ResourceMode transition posture

- `crates/focusa-api/src/routes/health.rs` `/v1/doctor` now includes `resource_mode` with mode/reason/pressure/budget/latest_transition/transition_omitted_count/hysteresis/tool policy/cold surfaces.
- `apps/pi-extension/src/tools.ts` `focusa_tool_doctor` now calls `/resource/mode` and prints ResourceMode plus latest transition in the operator-visible summary.
- `tests/spec96_tool_doctor_resource_mode_static_test.sh` verifies API and Pi tool doctor ResourceMode/transition exposure.

Verified now:

```text
tests/spec96_tool_doctor_resource_mode_static_test.sh PASS
tests/spec96_resource_mode_hysteresis_static_test.sh PASS
tests/spec96_workpoint_evidence_lowmem_timeout_static_test.sh PASS
tests/spec96_lowmem_surgical_agent_static_test.sh PASS
tests/spec96_resource_mode_envelope_static_test.sh PASS
apps/pi-extension: npx tsc --noEmit PASS
git diff --check PASS
```

## Source patched: ProjectIdentity quorum/tool-result parity

- `crates/focusa-api/src/routes/project.rs` ProjectIdentity responses now include full `details.tool_result_v1` fields: `ok`, `status`, `canonical`, `degraded`, `failure_class`, retry posture, `side_effects`, `evidence_refs`, and `next_tools`.
- `apps/pi-extension/src/tools.ts` ProjectIdentity/Verify tool details now surface `tool_result_v1` directly instead of only burying it in raw response.
- `tests/spec96_project_identity_quorum_static_test.sh` verifies marker/git/beads/workspace/daemon/operator quorum signals, unsafe/cwd-only degradation, API envelope, Pi/CLI parity, and docs.

Verified now:

```text
tests/spec96_project_identity_quorum_static_test.sh PASS
tests/spec96_cli_parity_static_test.sh PASS
tests/spec96_broad_root_scope_isolation_static_test.sh PASS
tests/spec96_session_identity_envelope_static_test.sh PASS
apps/pi-extension: npx tsc --noEmit PASS
git diff --check PASS
```

## Source patched: Trajectory tool output schemas

- `crates/focusa-api/src/routes/trajectory.rs` now wraps view/define-goal/assess/propose-workpoint/checkpoint/resume responses with `details.tool_result_v1`.
- Trajectory tool envelopes include status, canonical/degraded, failure class, retry posture, side effects, evidence refs, and next tools.
- Trajectory write endpoints report reducer side effects: `trajectory_goal_defined`, `trajectory_state_delta_recorded`, and `trajectory_checkpoint_persisted`.
- `apps/pi-extension/src/tools.ts` Trajectory tools now expose `tool_result_v1`, canonical/degraded/advisory fields, side effects, evidence refs, and structured candidate/checkpoint/resume payloads in details.
- `tests/spec96_trajectory_tool_output_schema_static_test.sh` verifies API, Pi, CLI, and docs schema parity.

Verified now:

```text
tests/spec96_trajectory_tool_output_schema_static_test.sh PASS
tests/spec96_trajectory_clarity_gate_static_test.sh PASS
tests/spec96_trajectory_reducer_lifecycle_static_test.sh PASS
tests/spec96_trajectory_workpoint_handoff_static_test.sh PASS
tests/spec96_cli_parity_static_test.sh PASS
apps/pi-extension: npx tsc --noEmit PASS
git diff --check PASS
```

