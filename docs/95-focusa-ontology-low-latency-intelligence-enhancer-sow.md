# 95 — Focusa Ontology Integration and Low-Latency Intelligence Enhancer SOW

**Date:** 2026-05-03  
**Status:** planned / evidence-backed SOW  
**Priority:** critical  
**Owner:** Focusa ontology + Pi integration  
**Relation:** complements Spec94; implements ontology-intelligence portions through Doc78 secondary-cognition guardrails; does not replace ontology core specs.  

---

## 1) Purpose

Strengthen Focusa ontology integrations and object connections so ontology becomes the low-latency intelligence enhancer originally intended: a typed, bounded, interruptible working world that improves agent action selection without becoming a lossy shortcut or second authority.

This SOW is not a memory-stripping plan. It is a connection-strengthening and intelligence-enhancement plan. It folds in Spec94 payload discipline, Spec95 ontology integration work, Doc78 secondary-cognition governance, and external agent-intelligence research patterns gathered through Brave Search.

---

## 2) Original ontology intent to preserve

The ontology is **additive in implementation** and **canonical in semantics**. It supplies the typed software-world model that Focusa runtime, proxy layer, and Pi extension consume.

The ontology should define:

- what exists;
- how things relate;
- what actions are valid;
- what currently matters;
- what has been verified;
- what remains uncertain.

The ontology owns:

- canonical software-world semantics;
- object identity and type rules;
- typed relations;
- typed actions;
- working sets;
- provenance;
- verification state;
- status and freshness.

The reducer remains the only canonical write boundary.

**Evidence:**

- `docs/45-ontology-overview.md:1-93`
- `docs/46-ontology-core-primitives.md:1-99`
- `docs/47-ontology-software-world.md:1-131`
- `docs/48-ontology-links-actions.md:1-64`
- `docs/50-ontology-classification-and-reducer.md:1-86`
- `docs/66-affordance-and-execution-environment-ontology.md:1-57`

---

## 3) Current implementation evidence

### 3.1 Strong foundations already present

- Ontology route file already projects software/work/mission/execution world surfaces.
- Combined projection includes mission, workspace, canonical ontology, visual, affordance/execution, identity, governance, reference resolution, and projection-view semantics.
- Workpoint integration already treats ontology-backed Workpoint as typed continuation.
- Tool contracts already map Pi tools to ontology actions and object kinds.

**Evidence:**

- `crates/focusa-api/src/routes/ontology.rs:4937-4985`
- `crates/focusa-api/src/routes/ontology.rs:5105-5165`
- `docs/88-ontology-backed-workpoint-continuity.md:28-113`
- `docs/90-ontology-backed-tool-contracts-parity-spec.md:35-100`

### 3.2 Integration gaps / latency friction

Live probes and code inspection show places where ontology can be more useful and lower latency without weakening semantics:

| Surface | Observation | Implication |
|---|---:|---|
| `/v1/ontology/world` | 4.29 MB, ~2.21s | full world is too heavy for per-turn intelligence |
| `/v1/ontology/slices?active_mission` | 1.7 KB but probe showed ~4.2s | slice output is small, but implementation still builds full combined projection |
| `/v1/ontology/tool-contracts` | 32 KB, ~2 ms | static/cached contract projection is already a good pattern |
| `/v1/ontology/contracts` | 220 KB, ~16 ms | larger but still fast due static-ish generation |
| `/v1/ontology/affordances` | 404 | affordance ontology exists in docs/code projection but no direct low-latency route |
| `/v1/ontology/working-set` | 404 | working-set concept exists but no direct route |
| `/v1/ontology/context` | 404 | no dedicated prompt-safe ontology context route |
| Pi extension | no direct `/ontology/slices` fetch in extension path | Pi relies mainly on Focus Slice/Workpoint state, not a dedicated ontology intelligence call |
| turn route | `active_mission_slice_summary` used as directive | ontology does enter prompt assembly, but via full projection path |

**Evidence:**

- live probe on 2026-05-03 local daemon;
- `crates/focusa-api/src/routes/ontology.rs:5105-5165`;
- `crates/focusa-api/src/routes/turn.rs:200-240`;
- `apps/pi-extension/src/turns.ts`, `apps/pi-extension/src/state.ts` searches for ontology fetch paths;
- `docs/66-affordance-and-execution-environment-ontology.md:1-57`.

---

## 4) External intelligence patterns to fold in

These patterns were gathered from Brave Search and mapped into Focusa without creating a second cognitive authority.

| Pattern | External signal | Focusa integration |
|---|---|---|
| GraphRAG | Microsoft GraphRAG combines knowledge graphs, retrieval, network analysis, and summarization for stronger Q&A over private/complex data. | Use Focusa ontology as the graph spine; add community summaries, link-path retrieval, and evidence-backed reasons. |
| Reflection/critic loops | ReAct, Reflexion, and LATS combine reasoning, acting, environment feedback, and reflection. | Use Doc78 secondary cognition as critic/proposal layer over Workpoint/action outcomes. |
| Generative-agent memory | Generative Agents store experiences, synthesize higher-level reflections, and retrieve dynamically for planning. | Formalize episodic -> semantic -> procedural memory promotion using events, metacog, tool contracts, and playbooks. |
| Memory retrieval controller | MemR3-style retrieval controllers decide whether to answer or issue refined retrieval queries. | Add retrieval governor that chooses Focus State, Workpoint, ontology context, metacog, ECS/evidence, or no retrieval. |
| Agent benchmarks | AgentBench/SWE-Bench-style work emphasizes planning, tool use, memory, and workflow evaluation. | Add intelligence usefulness evals: retrieval hit rate, drift prevention, tool-call reduction, completion quality. |
| Hybrid retrieval | Modern RAG practice combines graph, exact, semantic/vector, keyword, freshness, and reranking. | Combine target refs, ontology graph traversal, semantic memory, ECS handles, evidence strength, recency, and operator steering. |
| Uncertainty/calibration | Agent verification literature emphasizes evaluator/feedback and confidence rather than blind memory use. | Add confidence, stale/verified/degraded flags, contradiction checks, and evidence strength to retrieval outputs. |

**Guardrail:** these are implementation patterns, not authority shifts. Focusa ontology/reducer remains canonical; secondary models propose, critique, rank, and summarize.

---

## 5) Secondary model layer alignment

Doc78 already defines the correct home for many intelligence features: bounded secondary cognition and persistent autonomy.

Secondary cognition should operate as a closed-loop improvement system:

1. observe;
2. extract / propose;
3. verify / critique;
4. evaluate against fixed success criteria;
5. promote / retain-as-projection / reject / archive;
6. checkpoint and trace;
7. recover / resume / continue;
8. apply decay and retention policy;
9. repeat.

Doc78 guardrails remain mandatory:

- secondary cognition is subordinate;
- the operator newest explicit input wins;
- ontology-governed scope, role, permission, and verification boundaries are mandatory;
- reducer/governance paths own canonical truth changes;
- canonical truth, projection, and active relevance are distinct;
- verification precedes promotion;
- scope purity is intelligence;
- improvement claims require fixed evals.

**Evidence:**

- `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md:68-225`
- `docs/DOC78_SECONDARY_COGNITION_CALLSITE_AUDIT_2026-04-13.md:1-35`
- `docs/DOC78_REMAINING_IMPLEMENTATION_FRONTIER_2026-04-16.md:1-83`

### 5.1 Secondary cognition programs this SOW should implement

| Program | Role | Canonical boundary |
|---|---|---|
| Retrieval governor | Decide which Focusa substrate to query for a given ask and budget. | emits retrieval plan/projection only |
| Ontology critic | Detect missing links, stale working sets, weak provenance, and action/target mismatch. | emits ontology proposals only |
| Execution critic | Compare intended Workpoint action to actual tool results and propose recovery. | writes failure/evidence through existing governed routes |
| Reflection synthesizer | Convert traces into reusable metacog signals and procedural playbooks. | metacog capture/evaluate/promote gates apply |
| Uncertainty auditor | Label retrieved context as verified/stale/speculative/degraded/contradictory. | cannot promote truth without evidence/reducer |
| Plan evaluator | Score plan quality, decomposition, reversibility, and safety before execution. | advisory unless operator/reducer accepts |

---

## 6) Core design law for this SOW

Ontology latency optimization must strengthen connections, not erase them.

Allowed:

- cache indexes;
- cache derived read projections;
- add adjacency maps;
- add bounded context routes;
- add route-specific working-set builders;
- add provenance-preserving summaries;
- add deterministic connection scoring;
- add explicit rehydrate handles and cursors.

Forbidden:

- removing ontology object/link/action classes;
- flattening typed links into freeform text only;
- treating a slice as canonical truth;
- letting Pi own canonical ontology memory;
- silently dropping provenance, verification, uncertainty, freshness, or reducer status;
- model-derived canonical writes without reducer promotion.

---

## 7) Proposed workstreams

## A) Low-latency ontology adjacency index

### A1. Build a read-side adjacency index

Maintain an in-memory, reducer-fed read index keyed by ontology object id:

- outgoing links by type;
- incoming links by type;
- object type;
- status/freshness;
- provenance refs;
- verification refs;
- working-set memberships;
- action affordances;
- related evidence handles;
- related Workpoints/tasks/failures/decisions.

This index is a projection, not canonical truth. It is rebuilt or incrementally updated from reducer-approved ontology state and verified event streams.

**Why:** current slices are tiny but can still require building the full combined projection. An adjacency index lets Focusa answer “what matters next for this object/ask?” in milliseconds.

**Acceptance:**

- index exposes counts and last reducer event id;
- index rebuild parity test equals canonical full-world semantics;
- stale index responses are explicitly marked degraded/stale;
- no canonical write path uses the read index as authority.

---

## B) Direct low-latency working-set route

### B1. Add `/v1/ontology/working-set`

Create a route optimized for current cognition:

```text
GET /v1/ontology/working-set?frame_id=&ask=&target_ref=&limit=&include_reasons=true
```

Return bounded, typed members:

- object id/type/status;
- top relation reasons;
- link path snippets;
- provenance/verification handles;
- confidence/freshness;
- action affordance ids;
- continuation cursor/rehydrate handles for omitted detail.

**Why:** docs define ObjectSet and working-set construction as first-class, but live route probe returned 404 for `/v1/ontology/working-set`.

**Intent preservation:** the route is an expression of ObjectSet/SlicePolicy, not a substitute ontology.

---

## C) Prompt-safe ontology context route

### C1. Add `/v1/ontology/context`

Create a single low-latency route Pi and other adapters can call before prompt assembly:

```text
POST /v1/ontology/context
{
  "current_ask": "...",
  "frame_id": "...",
  "workpoint_id": "...",
  "target_refs": ["..."],
  "budget_tokens": 500,
  "view_profile": "pi_operator_view"
}
```

Return:

- active object set;
- relevant link paths;
- valid next ontology actions;
- blocked/unsafe affordances;
- evidence handles;
- uncertainty flags;
- rehydrate handles;
- canonical/degraded/stale flags.

**Why:** Pi extension currently has sophisticated Focus Slice construction, but does not directly fetch ontology slices/context. A dedicated endpoint would make ontology an intelligence enhancer at the exact moment prompt context is selected.

**Intent preservation:** Pi consumes this projection but does not canonize it.

---

## D) Affordance/action intelligence route

### D1. Add `/v1/ontology/affordances`

Expose the practical-possibility ontology described in Spec66:

```text
GET /v1/ontology/affordances?target_ref=&action_intent=&scope=current
```

Return:

- feasible actions;
- blocked actions;
- preconditions;
- permission/authority boundaries;
- estimated latency/cost/reliability/reversibility;
- required verification hooks;
- safest next tool/action candidates.

**Why:** original ontology intent includes valid actions and practical possibility; Spec66 explicitly calls out cost, latency, reliability, reversibility, authority, and blockers as first-class.

**Intent preservation:** route reads existing ontology/action contracts and environment evidence; it does not approve high-risk actions by itself.

---

## E) Connection-strengthening reducers and proposals

### E1. Tool result -> ontology connection proposal

When tools run, emit proposal-grade ontology deltas that connect:

- tool action -> target object;
- target object -> evidence ref;
- evidence ref -> verification record;
- failure -> affected object;
- decision/constraint -> scoped objects/actions;
- Workpoint -> active object set/action intent.

These deltas remain proposal-only unless reducer-promoted.

**Why:** ontology becomes more intelligent when evidence and actions continuously enrich object relationships.

**Acceptance:**

- tool_result envelopes include candidate ontology deltas or delta refs;
- reducer records promotion/rejection;
- no silent canonical mutation.

---

## F) Deterministic link extraction for code/docs/tests

### F1. Fast extractors for stable relations

Add or strengthen deterministic extractors for:

- file -> module/package;
- route -> handler;
- test -> code under test;
- docs/spec -> code surface;
- tool contract -> API/CLI/core surface;
- Workpoint target_ref -> object id;
- evidence handle -> object/ref/doc/test.

**Why:** docs assign deterministic classifiers responsibility for object typing, imports/calls, route/endpoint extraction, schema/migration linkage, diff-to-object mapping, and test/build result linkage.

**Intent preservation:** deterministic extraction may propose or update evidence-backed structure, but canonical writes still pass reducer.

---

## G) Link-path scoring for low-latency relevance

### G1. Relevance scoring from graph paths

Use the adjacency index to score bounded context candidates by:

- direct target match;
- distance from active Workpoint/action intent;
- verified link strength;
- freshness;
- failure/risk/blocker proximity;
- operator steering terms;
- active mission membership;
- affordance feasibility.

Return short reasons, not only raw ids.

**Why:** this turns ontology into a practical intelligence enhancer: the model sees why each object matters and which connections are actionable.

**Guardrail:** scoring changes projection order only; it does not mutate canonical truth.

---

## H) Cache tiers and latency budgets

### H1. Read-projection cache tiers

Define cache tiers:

1. **Static:** tool contracts, action schemas, primitive catalogs.
2. **Reducer-fed hot:** adjacency index, working-set memberships, object summaries.
3. **Per-turn ephemeral:** ask-specific context projection.
4. **Explicit full:** paginated full-world/export routes.

Each cache entry must include:

- source reducer version/event id;
- generated_at;
- ttl or invalidation rule;
- canonical/degraded/stale status;
- object/link/action counts.

### H2. Budgets

Target budgets:

- `/v1/ontology/context`: p95 under 50 ms after warm-up;
- `/v1/ontology/working-set`: p95 under 50 ms after warm-up;
- `/v1/ontology/affordances`: p95 under 75 ms after warm-up;
- `/v1/ontology/slices`: p95 under 50 ms after warm-up;
- full-world/export routes remain explicit, bounded, and measured separately.

---

## I) Pi integration: consume ontology intelligence at the right time

### I1. Pi pre-prompt ontology context fetch

Before assembling/injecting Focus Slice, Pi bridge should request `/v1/ontology/context` with:

- current ask;
- active Workpoint id;
- active object refs;
- target refs from tool/result context;
- token budget;
- operator steering signal.

Pi should then merge the result into the Focus Slice as bounded sections:

- ACTIVE_OBJECT_SET;
- RELEVANT_LINK_PATHS;
- VALID_NEXT_ACTIONS;
- BLOCKED_AFFORDANCES;
- EVIDENCE_HANDLES;
- UNCERTAINTY_FLAGS.

**Guardrail:** Pi consumes and renders; Focusa remains authority.

---

## J) Retrieval governor and hybrid retrieval

### J1. Add a retrieval governor

Create a bounded controller that chooses the minimum useful Focusa substrate for each current ask:

- Workpoint resume packet;
- ontology context;
- active working set;
- metacognition retrieval;
- ECS/evidence handles;
- semantic/procedural memory;
- tool contracts/affordances;
- no retrieval when current ask is self-contained.

Inputs:

- current ask kind;
- query scope;
- operator steering detection;
- active Workpoint/action intent;
- token budget;
- target refs;
- stale/degraded state;
- previous retrieval outcomes.

Outputs:

- selected retrieval plan;
- reason for each substrate;
- excluded-context reason;
- expected token/payload budget;
- degraded/stale flags.

**Intent preservation:** the governor routes projections; it never writes canonical truth.

### J2. Add hybrid retrieval and reranking

Combine:

- exact object/path/workpoint refs;
- ontology graph link traversal;
- semantic memory retrieval;
- ECS/evidence handle lookup;
- keyword/query-scope match;
- recency/freshness;
- verification/evidence strength;
- operator steering terms;
- secondary-model reranking when cheap and bounded.

Return scored items with explicit reasons and evidence handles.

---

## K) Secondary cognition critic/evaluator loop

### K1. Execution critic

After significant tool calls, compare:

- intended action intent;
- target objects;
- expected verification hooks;
- actual tool result envelope;
- side effects/evidence refs;
- Workpoint next action.

Emit:

- no-op when aligned;
- bounded failure diagnosis proposal when misaligned;
- recovery suggestion;
- candidate ontology deltas connecting result/evidence/failure to target objects.

### K2. Reflection synthesizer

Use Doc78 secondary cognition to synthesize:

- reusable metacog signals;
- procedural playbooks;
- anti-patterns/failure classes;
- decision alternatives and rejected paths;
- prediction calibration data.

All outputs remain proposal/evaluation artifacts until promoted through existing metacog/reducer paths.

### K3. Uncertainty auditor

Every intelligence projection should label:

- verified;
- evidence-linked;
- speculative;
- stale;
- degraded;
- contradictory;
- rehydrate-needed.

This should become part of ontology/context/metacog retrieval envelopes.

---

## L) Episodic -> semantic -> procedural memory pipeline

### L1. Promotion pipeline

Define a pipeline that turns repeated experience into reusable intelligence:

1. episodic event/tool trace captured;
2. evidence handle attached;
3. secondary cognition proposes summary/lesson;
4. evaluator checks usefulness against task/eval criteria;
5. metacog captures reusable semantic learning;
6. repeated validated learning promotes to procedural playbook/tool-contract hint;
7. decay/retention policy archives weak or stale lessons.

**Cross-feature mapping:** events -> ECS evidence -> metacognition -> ontology links -> tool contracts/procedural docs.

---

## M) Intelligence dashboard and evaluation harness

### M1. Usefulness metrics

Measure whether Focusa actually improves LLM performance:

- retrieval hit rate;
- irrelevant-context rate;
- stale-context rate;
- drift prevented;
- tool calls saved;
- failed tool calls predicted;
- Workpoint resume success;
- evidence-linked answer rate;
- task completion delta;
- latency/RSS overhead.

### M2. Fixed eval suites

Create fixture tasks for:

- compaction recovery;
- ontology context selection;
- action affordance selection;
- uncertainty labeling;
- secondary critic recovery;
- metacog reuse;
- cross-file code/docs/test linkage;
- operator steering override.

This satisfies Doc78 requirement that improvement claims require fixed evals.

---

## N) Validation and proof

### N1. Semantic preservation tests

For every bounded/intelligence route:

- prove primitive class coverage remains intact;
- prove object/link/action category counts expose omitted detail;
- prove rehydrate/cursor path exists;
- prove reducer-only canonical write authority;
- prove no Pi canonical memory ownership.

### N2. Latency tests

Measure:

- cold/warm latency;
- cache hit rate;
- p50/p95/p99;
- response size;
- RSS impact;
- projection stale/degraded rate.

### N3. Intelligence usefulness tests

For fixture tasks, verify the ontology context returns:

- correct active objects;
- at least one valid action when available;
- blockers when action is not feasible;
- evidence handles for assertions;
- link reasons that explain relevance;
- no hallucinated canonical links.

---

## 8) Implementation order

1. Keep Spec94 payload/memory guardrails as implementation constraints.
2. Build read-only adjacency index with parity checks.
3. Add `/v1/ontology/working-set` from adjacency index.
4. Add `/v1/ontology/context` for prompt-safe intelligence.
5. Add `/v1/ontology/affordances` using Spec66 concepts and tool contracts.
6. Add retrieval governor and hybrid retrieval/reranking.
7. Add tool-result candidate ontology deltas.
8. Add deterministic link extractors for docs/code/tests/routes/tool contracts.
9. Add secondary execution critic, uncertainty auditor, and reflection synthesizer as Doc78 programs.
10. Add episodic -> semantic -> procedural memory promotion pipeline.
11. Wire Pi pre-prompt context fetch with strict no-canonical-write boundary.
12. Add latency, semantic preservation, and intelligence-usefulness proof harness.

---

## 9) Definition of done

1. Ontology is used as a low-latency prompt and action-selection enhancer, not only a full-world dump.
2. Pi can consume bounded ontology context without becoming a second cognitive authority.
3. Working-set, context, and affordance routes return in low milliseconds after warm-up.
4. Every returned item includes typed identity, relation reason, provenance/verification status, uncertainty label, and rehydrate path where needed.
5. Tool results and evidence handles enrich ontology connections through reducer-governed proposals.
6. Deterministic extractors strengthen stable code/docs/tests/routes/tool-contract relations.
7. Retrieval governor chooses minimal useful substrate and records excluded-context reasons.
8. Hybrid retrieval combines exact refs, ontology graph, semantic memory, ECS evidence, freshness, and operator steering.
9. Secondary cognition critic/evaluator programs run under Doc78 guardrails and never promote canonical truth directly.
10. Episodic traces can become semantic lessons and procedural playbooks only through evidence/eval/promotion gates.
11. Intelligence dashboard shows retrieval usefulness, drift prevention, stale-context rate, and latency/RSS overhead.
12. Semantic preservation tests prove no ontology primitive/domain/action/relation/governance class was stripped.
13. Latency/RSS tests prove the enhancer improves speed without weakening original ontology intent.

<!-- SPEC137A_138A_144_ARCHITECTURE_CLOSURE:spec144-bounded-semantic-runtime -->
## Spec 144 bounded RDF/OWL/SHACL runtime integration

Spec 95 performance and bounded-context laws apply to RDF/OWL/SHACL compilation, obligation triggers, Verification Pack resolution, and semantic validation. Focusa MUST use bounded graphs, precompiled bundles, incremental validation, cache-safe versioning, and explicit degraded posture without dropping mandatory obligations or reporting an unavailable reasoner as a pass.
