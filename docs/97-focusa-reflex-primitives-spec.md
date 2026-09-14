# 97 — Focusa Reflex Primitives Spec

**Date:** 2026-05-25  
**Status:** implemented / dependency-audited / live-validated
**Priority:** high  
**Owner:** Focusa + Pi integration  
**Source:** Operator framing from the "agent cerebellum" model: routine cognition should become reliable reflex, not repeated high-cognition planning.

---

## 1) Why this spec exists

Recent Focusa specs improved tool contracts, Workpoint continuity, live proof, agent-first polish, non-Pi awareness, memory/RPC efficiency, low-latency ontology projection, and Trajectory Projection. Those specs collectively define many strong pieces, but they still describe most behavior by subsystem.

The next useful abstraction is a universal primitive layer:

```text
repeating operational burden -> typed Focusa reflex -> bounded evidence + escalation
```

This spec names and organizes that layer as **Reflex Primitives**.

A Reflex Primitive is a small, typed, inspectable, context-fed routine that handles recurring operational cognition before the model spends reasoning tokens. It is not a new authority system. It composes existing Focusa primitives — Ontology, Workpoint, Focus State, Trajectory, Focus Gate, Evidence, Work-loop, Prediction, Metacognition, ResourceMode, and tool envelopes — into reusable agent reflexes.

The core product insight:

> Focusa should offload boring, stateful, safety-relevant, proof-bearing cognition into local reflexes so the model can reserve high cognition for novel judgment.

---

## 2) Lineage position

Spec97 extends the 90s lineage:

| Prior spec | Contribution this spec depends on |
|---|---|
| Spec90 | Ontology-backed tool/action contracts and parity. |
| Spec91 | Live tool contract proof harness. |
| Spec92 | Agent-first polish, hook coverage, recovery envelopes, predictive power. |
| Spec93 | Non-Pi awareness and cross-agent usability. |
| Spec94 | Intent-preserving memory/RPC optimization and hot/cold route discipline. |
| Spec95 | Ontology low-latency intelligence enhancement. |
| Spec96 | Trajectory Projection, project/session scope, daemon stability, LowMem hardening. |

Spec97 does not replace these. It creates a cross-cutting vocabulary and acceptance model for the repeatable routines they imply.

### 2.1 Dependency audit — 2026-05-25

Audit rule: a dependency counts as functional only when code/routes/tests or validation output prove it; bead closure alone is not proof.

| Dependency | Required for Spec97 | Verified functional evidence | Audit status |
|---|---|---|---|
| Spec90 tool/action contracts | Primitive registry must map to stable tool contracts, ontology actions, docs, result envelopes, and parity metadata. | `docs/current/focusa-tool-contracts.json`, `apps/pi-extension/src/tool-contracts.ts`, `GET /v1/ontology/tool-contracts` in `crates/focusa-api/src/routes/ontology.rs`, `node scripts/validate-focusa-tool-contracts.mjs --json` returned `failures=0`, `tools=59`, `contracts=59`. | Functional in repo/static validation. |
| Spec91 live proof harness | Reflex acceptance needs live/static parity proof before claiming runtime release readiness. | `scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures --json` passed after rebuild/restart with `static_count=59`, `live_count=59`, and `payload_equal=true`. | Functional in live daemon proof. |
| Spec92 agent-first polish and prediction | Recovery reflexes and learning reflexes need `tool_result_v1`, bounded failures, predictive record/evaluate surfaces, and token/cache telemetry. | Pi tool wrappers expose `tool_result_v1`; prediction routes/tools/docs are present; `tests/spec96_predict_evaluate_not_found_static_test.sh`, `tests/spec96_pi_retry_posture_contract_static_test.sh`, and contract validation cover current behavior. | Functionally available as substrate; Spec97 must only reference it through bounded primitive metadata. |
| Spec93 non-Pi awareness | Reflexes must be visible outside Pi through awareness cards and CLI/API entrypoints. | `/v1/awareness/card` is routed in `crates/focusa-api/src/routes/awareness.rs` and `server.rs`; CLI `focusa awareness card` exists; docs/current `NON_PI_AGENT_FOCUSA_USAGE.md` documents OpenClaw/Wirebot path. | Functional substrate. |
| Spec94 memory/payload/RPC discipline | Reflexes must stay summary-first, bounded, cursor/rehydrate-aware, and pressure-safe. | `tests/spec94_response_size_and_metadata_contract_test.sh` passed; ECS/memory/ontology/work-loop/telemetry bounded metadata and response-size instrumentation are present. | Functional substrate. |
| Spec95 ontology intelligence | Reflex routing needs ontology actions, risks, affordances, working sets, prompt-safe context, and retrieval governance. | `tests/spec95_ontology_intelligence_contract_test.sh` passed; `/v1/ontology/working-set`, `/context`, `/affordances`, `/retrieval-governor`, `/tool-choreography` are routed in `ontology.rs`. | Functional substrate. |
| Spec96 trajectory/project/scope/LowMem | Reflexes require verified ProjectIdentity, project_root+continuity scope, Workpoint v2, Trajectory view, traversal, ResourceMode, and LowMem degradation. | `tests/spec96_project_identity_quorum_static_test.sh`, `tests/spec96_trajectory_clarity_gate_static_test.sh`, `tests/spec96_traverse_schema_static_test.sh`, and `tests/spec96_resource_mode_envelope_static_test.sh` passed; bootstrap default gap closed in `crates/focusa-api/src/routes/trajectory.rs`. | Functional substrate. |

### 2.2 Implementation closure ledger from audit

All audited Spec97 gaps are closed; this ledger preserves proof and ongoing maintenance requirements:

| Gap id | Dependency/phase | Closure evidence | Ongoing maintenance requirement |
|---|---|---|---|
| `G97-live-contract-parity` | Spec91 live proof dependency | Closed after adding `focusa_reflex_primitives` to static/TS contracts, choreography, README docs, rebuilding/restarting daemon, and rerunning live proof: `status=passed`, `payload_equal=true`, `static_count=59`, `live_count=59`. | Keep contract registry, Pi tool registrations, choreography, docs, and live daemon payloads synchronized. |
| `G97-primitive-registry` | Phase A | Closed by `docs/current/focusa-reflex-primitives.json` plus `tests/spec97_reflex_primitive_registry_static_test.sh`; registry is read-only and covers all ten families. | Keep registry entries unique, bounded, read-only, and mapped to existing Focusa surfaces. |
| `G97-reflex-envelope-metadata` | Phase B | Closed for Pi tool envelopes and API-native envelopes: `tool_result_v1.reflex_suggestions` is emitted by Pi wrappers and core API failure/degraded envelopes for Focus State, Workpoint, Trajectory, and Traverse. | Keep primitive ids bounded and registry-backed; do not let suggestions override operator steering or retry posture. |
| `G97-ontology-reflex-routing` | Phase C | Closed for bounded traversal by `surface=reflex_primitives`, direct read-only `GET /v1/reflex/primitives`, and ontology reflex object/action classes. | Keep traversal, direct API, ontology classes, and registry metadata in parity. |
| `G97-golden-reflex-scenarios` | Phase D | Closed by `docs/current/spec97-reflex-golden-scenarios.json`, static scenario validation, and live runtime dogfood in `tests/spec97_reflex_runtime_dogfood_test.sh` for direct API, traverse routing, and degraded/reflex recovery suggestions. | Extend scenarios only when new primitive families are added. |
| `G97-utility-card-reflex-language` | Phase E | Closed by `crates/focusa-api/src/routes/awareness.rs` and `tests/spec97_reflex_utility_card_static_test.sh`; Utility Card now names reflex affordances only for blocked/degraded next-step routing. | Keep language concise and avoid metacognitive noise. |

---

## 3) Core thesis

Most agent frameworks over-index on the model's prefrontal cortex: planning, chain-of-thought, multi-step reasoning, and executive control.

Focusa should explicitly also build the agent cerebellum: routine coordination, state binding, safety invariants, proof capture, recovery, and context routing that should happen reliably without re-planning every time.

In Focusa terms:

```text
Prefrontal surfaces: Trajectory, Prediction, Metacognition, high-level agent reasoning.
Cerebellar/reflex surfaces: ProjectIdentity, Focus Gate, Minimal Slice, Workpoint, Evidence handles, Work-loop guards, ResourceMode, result envelopes.
```

The model should not have to repeatedly rediscover:

- which project is active,
- whether a Workpoint is stale,
- where proof should be linked,
- whether a tool result is canonical,
- whether context is safe to inject,
- whether a retry is safe,
- whether operator steering changed the task,
- whether CI evidence is final proof,
- whether a large payload should be traversed or summarized.

Those are Reflex Primitive candidates.

---

## 4) Reflex Primitive contract

Every Reflex Primitive must be expressible as:

```text
Trigger -> Context inputs -> Reflex action -> Evidence output -> Escalation boundary
```

### 4.1 Fields

| Field | Meaning |
|---|---|
| `primitive_id` | Stable snake_case id. |
| `family` | identity, scope, continuity, evidence, recovery, salience, execution, learning, resource, governance. |
| `trigger` | Event, tool result, operator phrase, state mismatch, threshold, schedule, or explicit command. |
| `context_inputs` | Required Focusa surfaces and ontology objects needed before action. |
| `reflex_action` | Narrow state read/write/suggestion/checkpoint/evidence operation. |
| `evidence_output` | Proof handle, trace event, Workpoint evidence ref, status envelope, or no-op reason. |
| `escalation_boundary` | Exact condition that requires model reasoning or operator input. |
| `authority_boundary` | Which subsystem remains canonical. |
| `hot_path_budget` | Whether the primitive must run hot, summary-first, async, or cold opt-in. |
| `failure_envelope` | Required `tool_result_v1`/error-empty-state recovery shape. |

### 4.2 Contract rules

1. Reflexes are **small**. A primitive handles one recurring invariant.
2. Reflexes are **typed**. Inputs/outputs use existing Focusa schemas or explicit new schemas.
3. Reflexes are **inspectable**. They emit bounded evidence, traces, or status.
4. Reflexes are **operator-governed**. They do not override operator steering.
5. Reflexes are **context-fed**. Ontology/Trajectory/Workpoint/Focus State provide context; primitives do not invent it.
6. Reflexes are **bounded by authority**. They never bypass the reducer, Workpoint scope, project identity, writer ownership, or evidence rules.
7. Reflexes are **degradable**. If blocked, they return exact recovery posture instead of vague failure.
8. Reflexes are **composable**. Higher-level workflows chain primitives rather than building opaque mega-actions.

---

## 5) Primitive families

### 5.1 Identity Reflexes

Purpose: solve "where am I?" before durable work.

Canonical context feeds:

- ProjectIdentity
- `.focusa-project.json`
- git remote
- beads prefix
- Pi session id
- continuity id
- project_root safety rules

Initial primitives:

| Primitive | Trigger | Reflex action | Escalation |
|---|---|---|---|
| `bind_project_root` | broad/unsafe cwd or session start | infer candidate roots, call project identity/verify, persist verified root | multiple plausible roots or low confidence |
| `reject_unsafe_root` | project_root `/root`, `/`, home dir, or broad scope | block Workpoint/evidence writes and suggest explicit root | operator explicitly confirms unusual safe root |
| `confirm_continuity_scope` | compaction/resume/model switch | verify project_root + continuity_id + session_id alignment | mismatch with no canonical packet |
| `detect_cross_project_packet` | Workpoint/Trajectory/evidence packet read | reject stale packet and request checkpoint/resume in correct root | operator chooses migration/supersession |

### 5.2 Scope Reflexes

Purpose: solve "what exact object/action is active?"

Context feeds:

- Ontology objects/links
- active object refs
- Workpoint target objects
- current ask/scope classification
- Focus Gate signals

Initial primitives:

| Primitive | Trigger | Reflex action | Escalation |
|---|---|---|---|
| `detect_semantic_project_scope_conflict` | operator project correction or project alias/path conflicts with saved Workpoint before API `scope_mismatch` | derive `CurrentScopeVerdict`, set `action_authority_for_current_ask=false`, and route verify/rebind | unknown root or migration/supersession choice |
| `resolve_active_object` | evidence/action target ambiguous | run active object resolution with current Workpoint + hint | multiple incompatible targets |
| `enforce_do_not_drift` | next action crosses Workpoint boundary | surface blocker/drift warning | operator intentionally changes mission |
| `steering_reset_slice` | operator changes subject | suppress stale Focus Slice and rebuild relevant slice | conflicting operator instructions |
| `guard_stale_focus_state` | stale decisions/constraints appear irrelevant | demote to advisory or ask state hygiene plan | destructive hygiene requires approval |

### 5.3 Continuity Reflexes

Purpose: solve "what was I doing, and how do I resume safely?"

Context feeds:

- Workpoint records
- Focus State bounded slots
- Trajectory view
- lineage/tree snapshots
- compaction artifacts

Initial primitives:

| Primitive | Trigger | Reflex action | Escalation |
|---|---|---|---|
| `checkpoint_before_compaction` | compaction/model switch/context pressure | create typed Workpoint checkpoint with next action | Workpoint unavailable/degraded after retry |
| `resume_from_canonical_workpoint` | session resume/uncertainty | fetch resume packet and continue only if canonical/scope matches | no canonical packet and operator ask unclear |
| `hydrate_sparse_summary` | compaction summary empty fields | fill from nearest canonical related sources, not raw transcript | no safe related source |
| `snapshot_before_risky_change` | risky edit/restore/compare | create tree snapshot and expose restore handle | destructive action needs operator confirmation |

### 5.4 Evidence Reflexes

Purpose: solve "what proof changed confidence?"

Context feeds:

- Workpoint verification records
- Evidence store/ECS handles
- CI/test IDs
- API/file refs
- trajectory required evidence

Initial primitives:

| Primitive | Trigger | Reflex action | Escalation |
|---|---|---|---|
| `capture_ci_proof` | GitHub Actions run completes | link run id/result to active Workpoint or evidence store | failing run requires diagnostic loop |
| `summarize_large_artifact` | tool output exceeds budget | store handle, inject bounded summary | raw artifact needed for legal/security review |
| `require_evidence_for_claim` | assistant claims completion/release | verify evidence refs before final handoff | no proof available |
| `link_file_api_test_proof` | test/API/file proof observed | attach target_ref/result/evidence_ref | scope mismatch or stale Workpoint |

### 5.5 Recovery Reflexes

Purpose: solve "what now when something is blocked/degraded?"

Context feeds:

- `tool_result_v1`
- error-empty-state envelope
- daemon health
- resource mode
- tool doctor
- retry posture

Initial primitives:

| Primitive | Trigger | Reflex action | Escalation |
|---|---|---|---|
| `route_noncanonical_result` | `canonical=false` or `degraded=true` | call narrow resume/doctor/read tool before acting | repeated degraded results |
| `retry_safe_pending` | `pending` + safe retry posture | wait/retry with bounded backoff | retry budget exceeded |
| `diagnose_scope_mismatch` | scope mismatch failure class | verify project identity, checkpoint/resume in correct root | migration required |
| `resource_mode_fallback` | resource_exhausted/cold timeout | activate/read LowMem, use traverse summary slices | hot route unhealthy |

### 5.6 Salience Reflexes

Purpose: solve "what deserves attention now?"

Context feeds:

- Focus Gate signals/candidates
- warnings/errors/repeated patterns
- active frame
- ontology tags
- operator steering traces

Initial primitives:

| Primitive | Trigger | Reflex action | Escalation |
|---|---|---|---|
| `surface_repeated_failure` | repeated error fingerprint | raise candidate with evidence refs | candidate conflicts with operator priority |
| `suppress_irrelevant_context` | Focus Slice relevance fails | omit context and trace omission reason | omission hides safety constraint |
| `pin_operator_priority` | explicit operator priority | pin/surface relevant frame/candidate | priority conflicts with policy/safety |
| `decay_resolved_candidate` | resolved/suppressed signal | lower pressure/archive candidate | unresolved risk remains |

### 5.7 Execution Reflexes

Purpose: solve "what boring work can continue safely?"

Context feeds:

- Work-loop status
- writer ownership
- beads/task state
- policy budgets
- verification requirements
- execution environment affordances

Initial primitives:

| Primitive | Trigger | Reflex action | Escalation |
|---|---|---|---|
| `preflight_writer_ownership` | work-loop mutation requested | check writer/status before pause/resume/stop | writer conflict |
| `select_next_ready_work` | current task complete/no blocker | select next ready bead/task | none ready or ambiguous priority |
| `enforce_verification_before_persist` | completion/write closure | require tests/evidence before close/push | verification impossible or destructive |
| `close_task_with_citations` | task done + evidence exists | close bead with code/spec/evidence citations | missing required citation |

### 5.8 Learning Reflexes

Purpose: solve "what should compound?"

Context feeds:

- Prediction records/outcomes
- Metacog captures/reflections/adjustments
- evidence refs
- quality gates
- lineage outcomes

Initial primitives:

| Primitive | Trigger | Reflex action | Escalation |
|---|---|---|---|
| `record_forecast_before_risk` | risky/uncertain action | record prediction with confidence/evidence | no meaningful evidence basis |
| `evaluate_prediction_after_outcome` | outcome observed | score prediction and capture learning | outcome ambiguous |
| `promote_evidence_backed_lesson` | repeated validated lesson | promote to retrieval memory | insufficient evidence |
| `decay_stale_learning` | old/low-utility signal | reduce retrieval prominence/archive | safety-critical lesson |

### 5.9 Resource Reflexes

Purpose: solve "how do we stay useful under pressure?"

Context feeds:

- ResourceMode
- LowMem state
- response size telemetry
- hot/cold route metadata
- traversal budgets

Initial primitives:

| Primitive | Trigger | Reflex action | Escalation |
|---|---|---|---|
| `prefer_summary_hot_path` | broad status/context requested | return summary route and rehydrate refs | operator explicitly requests cold full payload |
| `gate_full_payload` | full lineage/ontology requested | require explicit cold opt-in | hot path instability |
| `degrade_with_recovery` | route timeout/resource pressure | return bounded degraded envelope + next tool | daemon unhealthy |
| `preserve_tool_affordances_lowmem` | LowMem active | keep official tools callable and bounded | tool registry unavailable |

### 5.10 Governance Reflexes

Purpose: solve "what requires conscious/operator authority?"

Context feeds:

- operator steering
- policy gates
- approval headers
- destructive action classifier
- proposal governance
- work-loop policy

Initial primitives:

| Primitive | Trigger | Reflex action | Escalation |
|---|---|---|---|
| `require_destructive_confirmation` | destructive/system action | block and request explicit approval | none; operator required |
| `proposal_not_direct_write` | governance-changing action | submit proposal, await resolution | urgent operator override |
| `operator_steering_supersedes_loop` | operator redirects | update context and suppress stale continuation | ambiguous redirect |
| `approval_gate_state_hygiene` | hygiene apply requested | require explicit approved=true | no approval |

---

## 6) Ontology-fed cohesion

Reflex Primitives must not become isolated scripts. They are cohesive because each primitive reads and writes through shared Focusa context.

```text
Ontology: objects, links, valid actions, risks, affordances
Trajectory: desired state, current verified state, gap, Workpoint candidate
Workpoint: immediate mission/action/targets/evidence/next slice
Focus State: bounded current intent, constraints, failures, decisions, results
Focus Gate: salience pressure and candidate surfacing
Evidence/ECS: proof handles and rehydratable artifacts
Work-loop: governed continuation, writer, policy, budgets
Prediction/Metacog: forecast, outcome, reusable learning
ResourceMode/Traverse: bounded access under pressure
```

### 6.1 Ontology role

Ontology should supply typed context for primitives:

- object class (`file`, `endpoint`, `tool`, `bead`, `ci_run`, `workpoint`, `trajectory`, `operator_directive`),
- action class (`verify`, `checkpoint`, `link_evidence`, `close_task`, `retry`, `restore`, `suppress`, `escalate`),
- risk class (`scope_mismatch`, `destructive`, `stale_state`, `resource_pressure`, `governance_pending`),
- affordance class (`safe_local_edit_available`, `transport_attached`, `writer_owned`, `ci_available`, `as_user_required`),
- evidence requirement (`test_id`, `gh_run`, `file_line`, `api_response`, `artifact_handle`).

### 6.2 Primitive routing view

The current `/v1/reflex/primitives` route and `surface=reflex_primitives` traversal expose:

```json
{
  "primitive_id": "capture_ci_proof",
  "family": "evidence",
  "trigger": "ci_run_completed",
  "context_inputs": ["ProjectIdentity", "Workpoint", "Trajectory.required_evidence_refs", "Ontology.ci_run"],
  "recommended_action": "focusa_evidence_capture",
  "escalation_boundary": "ci_conclusion != success or target scope mismatch",
  "authority_boundary": "Evidence is linked only through Workpoint/evidence reducer path"
}
```

Current direct route: `GET /v1/reflex/primitives?family=<family>&query=<risk-or-object>&limit=<n>` returns bounded read-only primitive summaries from the registry. Use `include_payload=true` only for explicit cold/full inspection. Live proof: `docs/evidence/SPEC97_REFLEX_DIRECT_API_LIVE_PROOF_2026-05-25.md`.

This view should be read-only first. Mutation remains in existing tools until individual primitive APIs are justified.

---

## 7) Authority and non-goals

### 7.1 Authority boundaries

- ProjectIdentity remains project/scope authority.
- Workpoint remains immediate continuation authority.
- Trajectory remains advisory project navigation unless reducer-promoted.
- Focus State remains bounded current cognitive slots.
- Ontology remains typed object/action/risk/affordance context.
- Work-loop remains governed continuous execution and writer authority.
- Evidence remains proof authority.
- Operator steering remains highest instruction authority.

### 7.2 Non-goals

This spec does not:

- create a second scheduler,
- turn Focusa into an autonomous executor,
- bypass Beads/task authority,
- bypass project_root + continuity_id scope,
- make Focus Gate automatically mutate focus stack,
- inject all primitive context every turn,
- require every reflex to mutate state,
- replace model reasoning for novel judgment,
- remove operator confirmation for destructive/governance actions.

---

## 8) Implementation roadmap

### Phase A — Primitive registry (read-only)

- Add a static registry describing primitive families and mappings to existing tools/routes.
- Include authority boundary, context inputs, escalation boundary, and hot/cold posture.
- Expose via bounded docs, direct read-only API, and traverse projection.

Current registry: `docs/current/focusa-reflex-primitives.json` (`schema=focusa.reflex_primitives.v1`, `version=spec97.reflex_primitives.v1`).

Acceptance:

- Registry covers at least the ten families above.
- Every primitive maps to existing Focusa surfaces or is marked `planned`.
- No mutation routes are added in Phase A.

### Phase B — Reflex suggestions in tool envelopes

- Extend `tool_result_v1.next_tools`/recovery hints with primitive ids where useful.
- Add `reflex_suggestions` metadata for common failures: scope mismatch, pending, degraded, resource exhausted, missing evidence, stale Workpoint.

Acceptance:

- Existing tool contracts remain valid.
- Suggestions are bounded and do not override operator steering.
- CI contract validation covers primitive id consistency.

### Phase C — Ontology-fed routing

- Add ontology classes for reflex primitives, triggers, action classes, risks, and affordances.
- Allow `focusa_traverse` to return primitive summaries by family/object/risk.

Current traversal route: `POST /v1/traverse` with `surface=reflex_primitives`, `selector=family`, and `anchor=<family>` returns bounded registry-backed primitive summaries. Direct API route `GET /v1/reflex/primitives` exposes the same registry as read-only summaries for agents/tools that do not need full traversal. API-native degraded/error envelopes include bounded `reflex_suggestions` for common failure classes.

Acceptance:

- Active object + risk can retrieve relevant primitive candidates without full ontology payload.
- Cold payload gates remain explicit.
- Ontology object/action classes include reflex primitives, triggers, actions, risks, affordances, and registry routing actions.

### Phase D — Dogfood golden scenarios

Add golden evals for common boring workflows. Current machine-readable scenarios live at `docs/current/spec97-reflex-golden-scenarios.json` and are checked by `tests/spec97_reflex_golden_scenarios_static_test.sh`:

1. unsafe `/root` -> bind verified project root -> persist session scope;
2. compaction warning -> checkpoint -> resume canonical Workpoint;
3. scope mismatch -> verify project -> reject stale evidence link -> checkpoint correct scope;
4. resource pressure -> LowMem -> summary/traverse recovery;
5. writer/evidence/citation preflight -> close bead with citations.

Future runtime scenario extensions should add:

6. CI success -> capture evidence -> close bead with citations;
7. repeated failure -> Focus Gate candidate -> operator-resolved/suppressed;
8. prediction before risky change -> evaluate after CI outcome.

Acceptance:

- Each scenario proves `Trigger -> Context -> Reflex -> Evidence -> Escalation`.
- Passing degraded envelopes count only when recovery is explicit and actionable.

### Phase E — UI/agent presentation

- Add docs/Utility Card language that names reflexes only when useful.
- Avoid turning reflex metadata into visible noise.
- Present reflexes as concise next-action affordances, not cognitive prose.

Current Utility Card wording names reflex affordances only when blocked/degraded: follow `reflex_suggestions` or traverse `surface=reflex_primitives` for the smallest safe next step.

Acceptance:

- Minimal Slice rules remain satisfied.
- Operator input remains primary.

---

## 9) Acceptance criteria

Spec97 is initially accepted when:

1. A primitive registry exists and is documented.
2. Primitive families map to Focusa surfaces and authority boundaries.
3. At least five golden scenarios prove the reflex contract.
4. Tool envelopes can reference primitive ids without breaking existing clients.
5. Ontology/traverse can retrieve bounded primitive context by family/risk/object.
6. No primitive bypasses reducer, Workpoint scope, writer ownership, or operator approval.
7. Docs clearly distinguish Reflex Primitives from high-level reasoning surfaces.
8. CI validates primitive registry consistency with tool contracts.

---

## 10) Open design questions

1. Should primitive ids live in `docs/current/focusa-tool-contracts.json`, a new `focusa-reflex-primitives.json`, or ontology registry state?
2. Should primitives be exposed only through docs/traverse first, or should `/v1/reflex/primitives` be added?
3. Which primitives are safe to run automatically vs only suggest?
4. How should primitive success be measured: avoided drift, fewer retries, evidence coverage, CI pass rate, operator corrections, or calibration stats?
5. Should Work-loop select-next consume primitive suggestions, or remain strictly task-policy driven?
6. What is the minimum UI language that helps agents without leaking metacognitive/internal prose?

---

## 11) Summary

Spec97 makes a product-level claim:

> Focusa's next compounding layer is universal Reflex Primitives: local, typed, context-fed routines for identity, scope, continuity, evidence, recovery, salience, execution, learning, resource pressure, and governance.

This keeps Focusa aligned with its current architecture while making the system more cohesive: Ontology describes the world, Trajectory describes direction, Workpoint describes the current step, Evidence proves confidence changes, Focus Gate surfaces salience, Work-loop governs continuation, and Reflex Primitives make the boring operational cognition reliable enough that models can spend their reasoning budget on the hard parts.
