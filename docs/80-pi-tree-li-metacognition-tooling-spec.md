# 80 — Pi `/tree` × Focusa LI Metacognition Tooling Spec (Sharpened)

Status: Planned (design-only; no implementation)
Owner: Focusa + Pi integration
Date: 2026-04-21
Supersedes: prior draft of this doc (same file)

---

## 0) Why this sharpened revision exists

Operator feedback: previous draft was directionally right but not sharp enough around ontology-layer integration and machine-grade acceptance criteria.

This revision tightens the original spec by:
- explicitly tying tool design to ontology layers,
- adding strict invariants and failure gates,
- defining measurable compounding outcomes,
- preserving the same planning-only constraint (no code before approval).

---

## 1) Authority + source alignment (unchanged intent, stricter mapping)

### Pi sources used
- `README.md`
- `docs/extensions.md`
- `docs/sdk.md`
- `docs/tui.md`
- `docs/skills.md`
- `docs/keybindings.md` (`/tree` and session tree interactions)

### Focusa sources used
- `docs/44-pi-focusa-integration-spec.md`
  - §36.5 missing `/fork` + `/tree` branch-aware state behavior
  - §37.2 tool-first metacognition
  - §37.2A work-loop bridge tools
- `docs/24-capabilities-cli.md`
  - CLI parity with API, no hidden mutation, machine-usable output
- `docs/gaps.md`
  - handler existence vs full branch-aware semantics closure

---

## 2) Problem statement (connected to original draft, tightened)

Current state:
1. CLI command surface exists but completion/parity is uneven for tooling ambitions.
2. `focusa export` execution remains stubbed for core dataset modes.
3. `/tree` and `/fork` branch semantics are under-specified for deterministic Focus State restore.
4. Metacognition is partially captured, but compounding learning loop is not yet enforced.

Required state:
- branch-correct cognition under tree navigation,
- tool-first metacognitive capture/retrieval/regulation,
- CLI/API parity for all tooling-critical domains,
- measurable improvement deltas over baseline.

---

## 3) Scope (same scope as original, now with explicit exclusion)

### In scope
- Pi `/tree` ↔ Focusa lineage/LI tool contracts.
- Metacognitive tool contracts for compounding loop.
- CLI completion backlog required for reliable tool wrapping.
- Outcome contract + metrics + acceptance gates.

### Out of scope
- implementation code,
- ontology domain expansion unrelated to lineage/metacognition integration,
- broad UI redesign beyond required operational widgets/prompts.

---

## 4) Ontology layer model (NEW: machine integration requirement)

All tooling in this spec must explicitly reference these layers:

1. Lexical (terms/synonyms/aliases)
2. Schema (object/link/action contracts)
3. Structural world graph (objects + links)
4. Dynamic action semantics (preconditions/effects)
5. Lifecycle/status semantics
6. Epistemic evidence/confidence semantics
7. Temporal freshness/decay semantics
8. Lineage/branch semantics (CLT + `/tree` + `/fork`)
9. Identity/authority semantics
10. Governance/versioning semantics
11. Metacognitive policy semantics
12. Outcome/impact semantics

### Layer rule
No tool spec can be approved if it mutates or interprets state without naming the ontology layers it touches.

---

## 5) Architecture decisions (sharpened from previous §5)

1. **Tool-first cognition capture**
   - decisions/constraints/failures/adjustments captured as structured tool calls, not marker text.
2. **Branch-keyed state model**
   - Focus State snapshots keyed by CLT node/branch lineage, with deterministic restore rules.
3. **API authority, CLI parity**
   - tools call `/v1/*`; CLI remains mandatory parity surface + fallback.
4. **Compounding is mandatory, not optional**
   - each cycle must include capture, retrieval, adaptation, and outcome evaluation.
5. **No hidden writes**
   - every mutation observable via events + citations + command path.

---

## 6) Tool contracts (replaces and sharpens original §6)

## 6.1 Tree/lineage bridge tools
1. `focusa_tree_head`
   - Reads current Pi session branch head and mapped CLT head.
   - Layers: 8, 12.
2. `focusa_tree_path`
   - Returns lineage path + branch point metadata + divergence summary.
   - Layers: 8, 7, 12.
3. `focusa_tree_snapshot_state`
   - Snapshots Focus State bound to CLT node.
   - Layers: 8, 5, 11.
4. `focusa_tree_restore_state`
   - Restores branch-correct state for target node.
   - Layers: 8, 5, 11, 9.
5. `focusa_tree_diff_context`
   - Diffs decisions/constraints/failures/open questions across branches.
   - Layers: 8, 11, 12.

## 6.2 Metacognition compounding tools
1. `focusa_metacog_capture`
   - Structured capture: decision, rationale, alternatives, confidence, strategy class.
   - Layers: 6, 11, 12.
2. `focusa_metacog_retrieve`
   - Retrieve prior successful/failed strategy packets by context similarity.
   - Layers: 11, 7, 12.
3. `focusa_metacog_reflect`
   - Generate reflection packet with explicit error classes and strategy updates.
   - Layers: 11, 6, 12.
4. `focusa_metacog_plan_adjust`
   - Apply reflection into next-step policy changes with constraints.
   - Layers: 11, 9, 5.
5. `focusa_metacog_evaluate_outcome`
   - Compare expected vs observed impact; produce learning delta candidate.
   - Layers: 12, 6, 7, 11.

## 6.3 Existing bridge tool set retained
- `focusa_work_loop_status`
- `focusa_work_loop_control`
- `focusa_work_loop_context`
- `focusa_work_loop_checkpoint`
- `focusa_work_loop_select_next`

---

## 7) CLI completion backlog (sharpened from original §7)

Priority A (blocking this initiative)
1. Export execution completion:
   - `focusa export sft|preference|contrastive|long-horizon` must execute, not stub.
2. Lineage CLI parity verification:
   - `lineage head|tree|node|path|children|summaries` fully implemented and schema-stable.
3. Machine-stable output contracts:
   - JSON schema for each command consumed by tools.

Priority B (hardening)
4. Structured error taxonomy for tool wrappers.
5. Capability introspection command for affordance routing.

---

## 8) Outcome contracts (sharpened from original §8)

Required measurable deltas vs baseline (rolling 14-day window; baseline = prior 14-day median):

1. Self-regulation
   - Metric: `strategy_adjusted_turn_rate = turns_with_checkpoint_or_plan_adjust / total_turns`
   - Gate: `+20%` relative improvement.

2. Outcome quality
   - Metric A: `failed_turn_ratio = failed_turns / total_turns`
   - Metric B: `rework_loop_rate = turns_marked_rework / total_turns`
   - Gate: both metrics improve by `>=15%`.

3. Transfer
   - Metric: `novel_context_strategy_reuse_rate`
   - Gate: `+15%` relative improvement in contexts tagged novel.

4. Motivation/ownership
   - Metric: `setback_recovery_rate = loops_continued_after_failure / loops_with_failure`
   - Gate: `+15%` relative improvement.

5. Social/perspective quality
   - Metric: `perspective_constraint_density = perspective_aware_constraints / total_constraints`
   - Gate: `+10%` relative improvement.

6. Instructor/operator regulation
   - Metric A: `steering_uptake_rate`
   - Metric B: `forced_pause_rate_after_steering`
   - Gate: A improves by `>=20%`; B does not regress.

Scoring rule:
- Gate D passes when at least 4 of 6 contracts meet thresholds with no critical regression in Outcome quality.

---

## 9) Non-negotiable invariants (NEW)

1. Every mutation must map to explicit command/tool/event path.
2. Every closure-worthy claim must include code/spec/evidence citations.
3. `/tree` navigation cannot leak post-fork state into prior branch context.
4. Compaction must preserve branch-keyed metacognitive artifacts.
5. No hidden prompt mutation outside declared policy layers.

---

## 10) Acceptance gates (replaces and tightens original §9)

Gate A — Spec completeness
- schemas, endpoint mappings, error model, fallback behavior documented.

Gate B — CLI readiness
- Priority A backlog complete; stubs removed for required domains.

Gate C — Branch correctness
- deterministic snapshot/restore across `/fork` + `/tree` replay tests.

Gate D — Compounding evidence
- positive deltas on >=4/6 outcome contracts over baseline.

Gate E — Governance integrity
- no hidden writes; full auditable mutation path.

---

## 11) Phased execution (same structure, tighter outputs)

Phase 0 — Design lock
- finalize tool JSON schemas + ontology layer mapping + invariants.

Phase 1 — CLI readiness
- close Priority A CLI gaps.

Phase 2 — Tree/LI integration
- snapshot/restore/diff semantics + replay tests.

Phase 3 — Metacognition compounding
- capture/retrieve/reflect/adjust/evaluate chain.

Phase 4 — Evaluation/hardening
- baseline comparison + gate validation + threshold tuning.

---

## 12) Risks + mitigations (tightened)

1. Tool proliferation without use
- Mitigation: affordance router + default minimal tool activation policy.

2. Branch-state corruption
- Mitigation: CLT-node keyed snapshots, immutable lineage references, replay checks.

3. Latency growth
- Mitigation: bounded retrieval windows, asynchronous reflection, token budgets.

4. Logging without learning
- Mitigation: mandatory outcome-eval and plan-adjust steps per cycle.

---

## 13) Immediate planning-only next actions

1. Finalize JSON schemas in Appendix A for all §6 tools.
2. Finalize endpoint/CLI fallback mapping in Appendix B.
3. Finalize baseline and scoring spec in Appendix C.
4. Finalize replay pack in Appendix D.
5. Operator review + explicit approval before any implementation work.

---

## 14) Appendix A — Tool contract catalog (planning contracts)

All tools must provide stable JSON input/output, explicit error codes, and listed ontology layers.

### 14.1 Tree/lineage bridge

| Tool | Required input | Success output | Error codes | Layers |
|---|---|---|---|---|
| `focusa_tree_head` | `session_id` (optional; defaults active) | `{ session_id, pi_tree_node, clt_head, branch_id }` | `TREE_HEAD_UNAVAILABLE`, `SESSION_NOT_FOUND` | 8,12 |
| `focusa_tree_path` | `clt_node_id?`, `to_root=true|false` | `{ head, path:[...], branch_point, depth }` | `CLT_NODE_NOT_FOUND` | 8,7,12 |
| `focusa_tree_snapshot_state` | `clt_node_id`, `snapshot_reason` | `{ snapshot_id, clt_node_id, created_at, checksum }` | `SNAPSHOT_WRITE_DENIED`, `SNAPSHOT_CONFLICT` | 8,5,11 |
| `focusa_tree_restore_state` | `clt_node_id`, `restore_mode=exact|merge` | `{ restored:true, snapshot_id, clt_node_id, conflicts:[...] }` | `SNAPSHOT_NOT_FOUND`, `RESTORE_CONFLICT`, `AUTHORITY_DENIED` | 8,5,11,9 |
| `focusa_tree_diff_context` | `from_clt_node_id`, `to_clt_node_id` | `{ decisions_delta, constraints_delta, failures_delta, open_questions_delta }` | `DIFF_INPUT_INVALID`, `CLT_NODE_NOT_FOUND` | 8,11,12 |

### 14.2 Metacognitive compounding

| Tool | Required input | Success output | Error codes | Layers |
|---|---|---|---|---|
| `focusa_metacog_capture` | `kind`, `content`, `rationale?`, `confidence?`, `strategy_class?` | `{ capture_id, stored:true, linked_turn_id }` | `CAPTURE_SCHEMA_INVALID` | 6,11,12 |
| `focusa_metacog_retrieve` | `current_ask`, `scope_tags[]`, `k` | `{ candidates:[...], ranked_by, retrieval_budget }` | `RETRIEVE_UNAVAILABLE`, `RETRIEVE_BUDGET_EXCEEDED` | 11,7,12 |
| `focusa_metacog_reflect` | `turn_range`, `failure_classes[]?` | `{ reflection_id, hypotheses:[...], strategy_updates:[...] }` | `REFLECT_INPUT_INVALID` | 11,6,12 |
| `focusa_metacog_plan_adjust` | `reflection_id`, `selected_updates[]` | `{ adjustment_id, next_step_policy, expected_deltas }` | `ADJUST_POLICY_CONFLICT` | 11,9,5 |
| `focusa_metacog_evaluate_outcome` | `adjustment_id`, `observed_metrics` | `{ evaluation_id, delta_scorecard, promote_learning:true|false }` | `EVAL_INPUT_INVALID` | 12,6,7,11 |

---

## 15) Appendix B — Endpoint + CLI binding matrix (with code-reality status)

Primary path is `/v1/*`; CLI is operator fallback only.

| Tool | Primary API path | API in current code | CLI fallback | CLI in current code | Required permission |
|---|---|---|---|---|---|
| `focusa_tree_head` | `GET /v1/lineage/head` | ✅ exists | `focusa lineage head --json` | ✅ exists | `lineage:read` |
| `focusa_tree_path` | `GET /v1/lineage/path/{clt_node_id}` | ✅ exists | `focusa lineage path <id> --json` | ✅ exists | `lineage:read` |
| `focusa_tree_snapshot_state` | `POST /v1/focus/snapshots` | ❌ planned | `focusa state snapshot create --json` | ❌ planned | `state:write` |
| `focusa_tree_restore_state` | `POST /v1/focus/snapshots/restore` | ❌ planned | `focusa state snapshot restore --json` | ❌ planned | `state:write` |
| `focusa_tree_diff_context` | `POST /v1/focus/snapshots/diff` | ❌ planned | `focusa state snapshot diff --json` | ❌ planned | `lineage:read` |
| `focusa_metacog_capture` | `POST /v1/metacognition/capture` | ❌ planned | `focusa metacognition capture --json` | ⚠️ stub exists (not endpoint-backed) | `metacognition:write` |
| `focusa_metacog_retrieve` | `POST /v1/metacognition/retrieve` | ❌ planned | `focusa metacognition retrieve --json` | ⚠️ stub exists (not endpoint-backed) | `metacognition:read` |
| `focusa_metacog_reflect` | `POST /v1/metacognition/reflect` | ❌ planned (`/v1/reflect/*` exists) | `focusa metacognition reflect --json` | ⚠️ stub exists (`focusa reflect ...` endpoint-backed) | `metacognition:write` |
| `focusa_metacog_plan_adjust` | `POST /v1/metacognition/adjust` | ❌ planned | `focusa metacognition adjust --json` | ⚠️ stub exists (not endpoint-backed) | `metacognition:write` |
| `focusa_metacog_evaluate_outcome` | `POST /v1/metacognition/evaluate` | ❌ planned | `focusa metacognition evaluate --json` | ⚠️ stub exists (not endpoint-backed) | `metacognition:write` |

Code-reality rule:
- This matrix is authoritative for decomposition. Any row marked ❌ must become explicit implementation beads before dependent tool rollout.

---

## 16) Appendix C — Baseline and scoring protocol

1. Baseline window: previous 14 days before feature enablement.
2. Evaluation window: rolling 14 days after enablement.
3. Minimum sample sizes:
   - `>=200` turns overall,
   - `>=30` turns in novel-context bucket,
   - `>=20` loops with failures for setback metrics.
4. Regression rule:
   - If failed-turn ratio worsens by `>5%`, Gate D fails even if 4/6 contracts pass.
5. Reporting cadence:
   - daily internal snapshot,
   - weekly operator report,
   - gate decision every 14 days.

---

## 17) Appendix D — Replay test pack for branch correctness

Required deterministic replay scenarios:

1. Fork snapshot integrity
   - create frame A -> decision D1 -> fork at T -> branch B adds D2.
   - restore pre-fork branch; D2 must not appear.

2. Tree navigation restore
   - navigate branch B1 -> B2 -> B1 repeatedly.
   - state checksum must match saved snapshot for each branch.

3. Merge-mode conflict visibility
   - restore with `restore_mode=merge` where conflicting constraints exist.
   - response must include explicit `conflicts[]` and no silent overwrite.

4. Compaction survival
   - compact after snapshot creation.
   - snapshot restore must produce same canonical decision/constraint sets.

Gate C passes only when all scenarios pass with stable checksums and zero silent mutation events.

---

## 18) Appendix E — Agent practice + observations form (compounding input contract)

Question addressed: do we have a standardized form for agent practice + observations that feeds compounding effect and planned tools?

Current answer:
- We have reflection outputs with `observations[]` (see `docs/G1-14-reflection-loop.md`),
- but we do not yet have a single canonical **practice+observation form contract** tied to the toolchain in this spec.

This appendix defines that contract.

### 18.1 Form purpose

Create one structured unit per meaningful work cycle that captures:
1. what the agent attempted,
2. what was observed,
3. what changed in strategy,
4. whether outcome improved,
5. what reusable learning should compound.

### 18.2 Canonical form schema (v1)

```json
{
  "form_version": "practice_observation_v1",
  "session_id": "...",
  "clt_node_id": "...",
  "work_item_id": "...",
  "timestamp": "ISO-8601",
  "ontology_alignment": {
    "working_set_type": "active_mission|debugging|refactor|regression|architecture",
    "membership_class": "pinned|deterministic|verified|inferred|provisional",
    "status": "active|speculative|blocked|verified|stale|deprecated|canonical|experimental",
    "lifecycle_stage": "proposed|candidate|canonical|verified|deprecated|retired",
    "provenance_class": "parser_derived|tool_derived|user_asserted|model_inferred|reducer_promoted|verification_confirmed",
    "verification_state": "unverified|verified|rejected",
    "reducer_event_refs": ["ontology_object_upsert_proposed", "ontology_verification_applied"]
  },
  "practice": {
    "goal": "string",
    "strategy_class": "decomposition|verification-first|risk-first|recovery|other",
    "planned_steps": ["..."],
    "selected_tools": ["..."],
    "expected_outcome": "string",
    "expected_metrics": {
      "failed_turn_ratio_target": "number?",
      "rework_loop_rate_target": "number?",
      "other": {}
    }
  },
  "observations": {
    "signals": [
      {
        "type": "success|failure|risk|constraint|drift|latency|quality",
        "source": "event|test|operator|tool",
        "summary": "string",
        "evidence_refs": ["docs/...", "tests/...", "crates/...", "/v1/..."],
        "link_type_refs": ["depends_on", "verifies"],
        "action_type_refs": ["verify_progress", "record_scope_failure"],
        "confidence": 0.0
      }
    ],
    "unexpected_findings": ["..."],
    "branch_context": {
      "forked": false,
      "from_clt_node": "...",
      "restored_snapshot_id": "..."
    }
  },
  "adaptation": {
    "decision": "string",
    "rationale": "string",
    "strategy_changes": ["..."],
    "constraints_added": ["..."],
    "failures_recorded": ["..."]
  },
  "outcome": {
    "result": "improved|unchanged|regressed|inconclusive",
    "observed_metrics": {
      "failed_turn_ratio": "number?",
      "rework_loop_rate": "number?",
      "setback_recovery_rate": "number?",
      "steering_uptake_rate": "number?"
    },
    "delta_summary": "string"
  },
  "compound_candidate": {
    "promote": true,
    "learning_statement": "string",
    "applicability": ["context tags"],
    "expiry_policy": "string"
  }
}
```

### 18.3 Required quality rules

1. `observations.signals[*].evidence_refs` must be non-empty.
2. `adaptation.decision` and `adaptation.rationale` must both exist.
3. `outcome.result` cannot be `improved` unless at least one observed metric exists.
4. `compound_candidate.promote=true` requires `learning_statement` + `applicability`.
5. If branch context indicates fork/tree movement, `clt_node_id` must match restored head.
6. `ontology_alignment.status` and `ontology_alignment.provenance_class` must be present for every form.
7. `reducer_event_refs` must include at least one emitted ontology reducer event name.

### 18.4 Tool mapping (planned)

- `focusa_metacog_capture` populates `practice` + initial `observations`.
- `focusa_metacog_reflect` enriches `observations` and `adaptation`.
- `focusa_metacog_plan_adjust` writes `adaptation.strategy_changes`.
- `focusa_metacog_evaluate_outcome` writes `outcome` + `compound_candidate`.
- `focusa_tree_snapshot_state` / `focusa_tree_restore_state` populate `branch_context`.

### 18.5 Gate integration

Appendix E is mandatory evidence input for:
- Gate C (branch correctness): validates branch context integrity.
- Gate D (compounding evidence): validates learning delta claims from structured forms.

Minimum volume requirement for Gate D decision:
- at least 50 valid forms in evaluation window,
- at least 20 forms in novel-context bucket.

### 18.6 Decomposition guidance for BD

When decomposing this spec into beads, create explicit tasks for:
1. form schema implementation + validation,
2. per-tool field population contracts,
3. storage/indexing for retrieval,
4. gate-report generation from form aggregates.

---

## 19) Appendix F — Ontology weave verification (docs + code)

This appendix prevents false weaving by explicitly separating:
- what is already authoritative in docs,
- what is implemented in code now,
- what remains planned.

### 19.1 Ontology docs reviewed for this weave check

Reviewed ontology suite and integration docs:
- `docs/45-ontology-overview.md`
- `docs/46-ontology-core-primitives.md`
- `docs/47-ontology-software-world.md`
- `docs/48-ontology-links-actions.md`
- `docs/49-working-sets-and-slices.md`
- `docs/50-ontology-classification-and-reducer.md`
- `docs/51-ontology-expression-and-proxy.md`
- `docs/58-65` visual/UI ontology docs
- `docs/61-domain-general-cognition-core.md`
- `docs/66-77` domain ontology docs
- `docs/44-pi-focusa-integration-spec.md`
- `docs/24-capabilities-cli.md`

### 19.2 Code reality checkpoints used

- Lineage routes present in code:
  - `crates/focusa-api/src/routes/capabilities.rs` routes for `/v1/lineage/head|tree|node|path|children|summaries`.
- Reflection routes present in code:
  - `crates/focusa-api/src/routes/reflection.rs` routes for `/v1/reflect/run|history|status`.
- Not present in code (planned):
  - `/v1/focus/snapshots*`
  - `/v1/metacognition/*`
- CLI has first-class `lineage` command domain and a first-class `metacognition` command domain in stub mode; metacognition commands are not endpoint-backed yet.

### 19.3 Anti-false-weaving rule

Any decomposition or implementation task must cite one of three labels:
1. `implemented-now` — confirmed by code path and testable endpoint/command.
2. `documented-authority` — required by authoritative docs, not yet implemented.
3. `planned-extension` — introduced by this spec as planned architecture.

No task may claim `implemented-now` without code citation.

---

## 20) Gap, performance, and full-utilization closure matrix (authoritative)

This section is the canonical decomposition source for the question:
- what is still missing,
- what must be performance-tuned,
- what blocks full Focusa utilization.

### 20.1 Functional gap matrix

| Area | Current state | Label | Closure target |
|---|---|---|---|
| Branch snapshot/restore API | `/v1/focus/snapshots*` missing in code | `planned-extension` | Implement snapshot create/restore/diff APIs + tests |
| Metacognition API domain | `/v1/metacognition/*` missing in code | `planned-extension` | Implement capture/retrieve/reflect/adjust/evaluate APIs + tests |
| CLI lineage parity | No first-class `focusa lineage ...` domain in current CLI | `documented-authority` | Add lineage command domain with schema-stable JSON output |
| CLI metacognition parity | No first-class `focusa metacognition ...` domain in current CLI | `planned-extension` | Add metacognition command domain mirroring API |
| Export execution pipeline | non-dry-run dataset export paths are not implemented | `implemented-now` (gap observed in code behavior) | Implement SFT/preference/contrastive/long-horizon execution paths |

### 20.2 Performance tuning matrix

| Path | Current risk | Label | Required gate |
|---|---|---|---|
| Reflection + metacog loop | latency inflation if retrieval/analysis unbounded | `planned-extension` | p95 added latency <= 12% vs baseline at equal workload |
| Snapshot/restore on `/tree` | branch switch stalls if state payload unbounded | `planned-extension` | restore p95 <= 400ms on standard workload |
| CLI tool wrappers | slow failover if API/CLI fallback behavior ambiguous | `documented-authority` | deterministic timeout + typed error envelope |
| Compaction with branch artifacts | excessive serialization cost under long sessions | `documented-authority` | compaction p95 <= 1.5x pre-branch baseline |

### 20.3 Full Focusa utilization criteria

Full utilization is reached only when all criteria below are true:

1. Tree/lineage correctness
- `/fork` and `/tree` produce branch-correct Focus State restore (Appendix D pass).

2. Tool-first metacognition loop
- capture -> retrieve -> reflect -> plan_adjust -> evaluate_outcome all available and used in production path.

3. CLI/API parity for required domains
- lineage + metacognition + export commands are machine-stable and test-covered.

4. Outcome compounding evidence
- Gate D passes (>=4/6 contracts) with no critical regression in outcome quality.

5. Governance integrity
- all mutations auditable; anti-false-weaving labels enforced in decomposition.

### 20.4 Decomposition directives for BD

Create beads in this order:
1. Functional gaps (20.1) blocking APIs and CLI domains.
2. Performance gates (20.2) with benchmark harness.
3. Utilization criteria proof pack (20.3) as final closure epic.

Each bead must include:
- label from §19.3,
- code citation (or planned endpoint citation for planned-extension),
- gate criterion it satisfies.
