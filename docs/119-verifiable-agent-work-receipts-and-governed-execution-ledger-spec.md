# Spec 119 — Verifiable Agent Work Receipts and Governed Execution Ledger

Status: Draft  
Owner: Verious Smith  
Created: 2026-07-05  
Scope: Focusa daemon HTTP API, CLI, Pi tools, Workpoint, Evidence, Context Authority, UIAI diagnostics intake, Spec88 Workpoint continuity, Spec100 Context Cognition, Spec111 Agent Context Bootstrap, Spec112 install verification, Spec113 Eval Ledger, Spec114 public proof surfaces, Spec115 cloud projections, Spec116 provider-neutral work-item closure, public-safe cards, receipt schema package, local receipt ledger, event-chain verification, and future integration adapters.

---

## 0. Source Grounding

This spec formalizes a unifying receipt layer for current and planned Focusa surfaces.

### 0.1 Current implemented / current-doc surfaces

- `README.md`
  - Focusa is a local-first mission cohesion layer for AI coding agents.
  - Focusa preserves ProjectIdentity, Continuity ID, HLT/MLG/STG, Waypoints, Workpoints, Evidence Refs, Context Cognition, Context Authority, and proof-backed continuation.
  - Current runtime includes Rust daemon, HTTP API, CLI, TUI, Pi extension, and menubar proof surfaces.
- `docs/current/generated/tool-surface-summary.md`
  - Current tool surface includes 97 tool contracts, 11 families, API/CLI/Pi parity, and full docs coverage.
- `docs/current/AUTHORITY_MODEL.md`
  - Operator steering wins.
  - No canonical read/write without verified `project_root + continuity_id`.
  - Transcript tail is never authority.
  - Results must expose canonical/advisory/degraded/blocked/stale posture.
- `docs/current/GOLDEN_WORKFLOW.md`
  - Defines the canonical happy path from ProjectIdentity → Trajectory → Workpoint → Context Cognition → implementation → Evidence → session transfer → final proof report.
- `docs/current/CONTEXT_AUTHORITY_CURRENT.md`
  - Requires preflight before risky mutations including deploy, release, git push, destructive file operations, migrations, broad refactors, config changes, live service actions, and install/update ambiguity.
- `docs/current/TAMPER_EVIDENT_EVENT_CHAIN.md`
  - Documents the current SQLite `event_hash_chain` behavior.
  - Event chain rows include `event_id`, `chain_index`, `previous_hash`, `payload_sha256`, `event_hash`, and `created_at`.
  - Hash-chain verification detects ordinary database edits or deletions, but does not replace external signing, backups, access controls, or out-of-band checkpoint publication.
- `crates/focusa-api/src/routes/workpoint.rs`
  - Existing Workpoint routes already carry `project_root`, `continuity_id`, `mission`, `next_slice`, `active_object_refs`, action intent, evidence linkage, idempotency, preview mode, and scope rejection behavior.
- `crates/focusa-api/src/server.rs`
  - Existing API router contains the core route families and applies JSON guard, mutation rate limit, route-scope enforcement, auth, and error-envelope middleware.
- `crates/focusa-core/src/runtime/persistence_sqlite.rs`
  - SQLite persistence is the canonical store for append-only events and versioned state snapshots.
  - Existing `append_event` writes canonical events and links them into `event_hash_chain`.

### 0.2 Prior and planned specs this spec must preserve

- `docs/88-ontology-backed-workpoint-continuity.md`
  - Defines the ontology-backed Workpoint as the typed continuation contract, not raw transcript tail.
  - Defines WorkpointCheckpoint, WorkpointResumePacket, ActiveMissionSet, CurrentActionIntent, VerificationRecords, blockers, and drift detection.
  - Requires Pi compaction, context overflow, and model switch/fork to preserve/resume from Workpoint packets.
- `docs/100-context-cognition-spec.md`
  - Defines Context Cognition as bounded advisory context curation and reasoning guidance.
  - Requires `project_root + continuity_id` scope, advisory/canonical=false posture, stale/degraded labeling, selected/excluded context, evidence frames, and no canonical mutation from packet generation.
- `docs/107-spec-first-feature-lifecycle-and-claim-discipline-spec.md`
  - Defines actual/partial/surrogate/blocked/missing evidence classes.
  - Blocks completion claims when required evidence is missing or insufficient.
- `docs/109-agent-first-api-redesign-ax-spec.md`
  - Establishes typed, bounded, discoverable, recoverable Agent Experience contracts.
- `docs/111-agent-context-bootstrap-and-delivery-spec.md`
  - Defines Focusa Agent Context Bootstrap & Delivery.
  - Defines AgentBootstrapPacket and AgentBootstrapReceipt.
  - States that Focusa Bootstrap turns a cold agent session into verified mission continuation.
- `docs/112-install-binary-architecture-spec.md`
  - Defines system detection, release asset selection, checksum/signature verification, atomic install/rollback, license validation, daemon install, and AX-correct install recovery.
- `docs/113-agent-benchmark-spec.md`
  - Defines Focusa-vs-No-Focusa benchmark runs, Eval Ledger boundaries, evidence capture, grounded claims tasks, and measured release comparisons.
- `docs/114-public-benchmark-flywheel-spec.md`
  - Defines `bench.focusa.dev`, `evals.focusa.dev`, and `proof.focusa.dev`.
  - Defines public-safe proof snapshots and the rule that public APIs serve generated/redacted artifacts only.
- `docs/115-focusa-cloud-control-plane-tool-gateway-master-spec.md`
  - Defines cloud as hosted coordination while local nodes retain Workpoints, Focus State, Evidence refs, Context Authority, Eval Ledger, private diagnostics, and project files.
  - Core rule preserved by this spec: cloud coordinates, node decides, receipts prove, private state stays local.
- `docs/116-provider-neutral-work-item-closure-authority-spec.md`
  - Defines Focusa as the provider-neutral closure authority.
  - Defines WorkItem, WorkItemRef, WorkItemProvider, ClosureClaim, ClosurePolicy, ClosureValidationResult, ProviderAdapter, and ProviderCapabilities.
  - Defines the closure lifecycle: prepare → validate → authorize → submit → reconcile → audit.
  - Defines close-time blocking and bypass detection for provider systems such as bd, Linear, Asana, and GitHub Issues.

---

## 1. Purpose

Focusa must make verifiable agent work a first-class product surface.

The major product capability is a **Focusa Receipt**: a local-first, proof-backed, scope-bound artifact that records what an agent was asked to do, what scope it belonged to, which Workpoint checkpoint/revision anchored continuation, what advisory context was supplied, what authority allowed or blocked action, what evidence supports the result, what remains unfinished, what context was delivered, what work item/provider closure state exists, and what the next safe action is.

This turns Focusa’s Workpoint continuity, Context Cognition, authority, evidence, bootstrap, install, eval, cloud, and closure primitives into one visible work ledger that developers, teams, future agents, public-safe proof surfaces, and provider integrations can trust.

---

## 2. Core Directive

Focusa MUST prioritize this outcome:

```text
Every meaningful agent work session should be resumable, governed, evidence-backed, receipt-producing, and locally verifiable.
```

Focusa should not add new terminology, tool families, API surfaces, public claims, or provider integrations unless they improve at least one of:

1. receipt quality;
2. proof quality;
3. authority accuracy;
4. closure truth;
5. recovery clarity;
6. integration portability;
7. outcome evaluation;
8. operator trust;
9. agent doability;
10. local verification;
11. typed continuation provenance;
12. advisory-context clarity;
13. bootstrap delivery proof;
14. install/license proof;
15. public-safe proof projection.

---

## 3. Problem Statement

Focusa already has strong primitives:

- ProjectIdentity;
- Continuity ID;
- Trajectory ladder;
- ontology-backed Workpoint continuity from Spec88;
- WorkpointCheckpoint and WorkpointResumePacket;
- ActiveMissionSet and CurrentActionIntent;
- Workpoint drift detection;
- Context Cognition from Spec100;
- ContextCognitionPacket, curator, optimizer, selected/excluded context, and evidence frame;
- Evidence Ref;
- Context Authority;
- Context Cognition;
- Session Transfer;
- Agent Context Bootstrap from Spec111;
- install/license verification direction from Spec112;
- Eval Ledger and benchmark direction from Spec113;
- public proof/benchmark direction from Spec114;
- local/cloud boundary direction from Spec115;
- provider-neutral closure authority from Spec116;
- Prediction and Metacognition;
- DX/UX surfaces;
- UIAI diagnostics intake;
- generated tool contracts;
- local-first daemon/API/CLI/Pi integration;
- event-chain persistence.

The current gap is product consolidation.

The system can preserve state, enforce scope, capture proof, guide recovery, deliver bootstrap context, validate work-item closure truth, curate advisory context, detect drift, and hash-link events, but users and agents still need one canonical artifact that answers:

```text
What was the work?
What was the scope?
Which Workpoint checkpoint/revision anchored it?
Did the work drift from the active ActionIntent?
What advisory context was supplied or excluded?
Was the action allowed?
Was bootstrap context delivered?
Was install/license state verified?
Was a work item closure claim valid?
Which provider mutation happened or was blocked?
What changed?
What proves it?
What is advisory only?
What is unfinished?
What is the next safe step?
Can this record be locally verified?
```

Without this artifact:

- internal vocabulary can feel like friction;
- tool count can feel like complexity;
- Workpoint provenance can be reduced to a bare id without revision/checkpoint/drift context;
- advisory Context Cognition can be mistaken for proof or authority;
- proof remains distributed across logs, tests, Workpoints, UIAI diagnostics, bootstrap receipts, provider task systems, eval ledgers, and docs;
- public demos lack a consistent evidence object;
- closure systems can drift into provider-specific truth;
- team workflows lack a simple audit boundary;
- future integration adapters have no portable payload to exchange;
- event-chain integrity is not directly visible at the work-summary level.

---

## 4. Product Benefit

### 4.1 Developer Benefit

A developer using Focusa should be able to ask:

```text
What did the agent actually do?
```

and receive a compact receipt with:

- task summary;
- scoped project identity;
- Workpoint continuity;
- Workpoint checkpoint/revision/provenance;
- active object/action/evidence provenance;
- Context Cognition advisory context status;
- bootstrap delivery status when relevant;
- work item closure status when relevant;
- install/license verification status when relevant;
- changed objects;
- authority posture;
- evidence refs;
- test/browser/API/CI/provider proof;
- blocked or missing proof;
- final claim status;
- local verification status;
- next safe action.

### 4.2 Agent Benefit

An agent using Focusa should receive:

- one canonical continuation anchor;
- one Workpoint checkpoint/revision anchor;
- one current authority posture;
- one proof status;
- one advisory-context status;
- one bootstrap/delivery status when relevant;
- one work-item closure status when relevant;
- one next safe tool/action;
- up to three recovery tools when blocked.

The agent should not need to scan the full Focusa tool surface to determine what to do next.

### 4.3 Team Benefit

A team should be able to require:

```text
No risky agent mutation, provider close, or public claim without a Focusa Receipt.
```

This creates a governed execution boundary for:

- git pushes;
- deploys;
- release publication;
- database migrations;
- destructive file operations;
- broad refactors;
- secret/config changes;
- live service operations;
- binary replacement;
- daemon restart;
- generated-code overwrite;
- cross-project file edits;
- bootstrap file writes;
- work-item close/status mutations;
- install/license activation claims;
- benchmark/public proof claims.

### 4.4 Public Demo Benefit

A public-safe receipt can become the standard object for Arena, proof, and benchmark surfaces:

```text
Here is the work.
Here is the proof.
Here is what was blocked.
Here is what remains.
```

### 4.5 Future Data Benefit

Receipts create structured outcome records:

- task type;
- plan shape;
- Workpoint checkpoint/revision;
- action intent;
- drift status;
- selected/excluded context summary;
- bootstrap delivery status;
- authority decision;
- closure validation status;
- provider mutation status;
- tool sequence;
- evidence class;
- failure mode;
- recovery path;
- eval/benchmark relation;
- final outcome.

Local-first storage remains the default. Any aggregation, sharing, export, public snapshot, cloud publication, or training use must be explicit, redacted, and opt-in.

---

## 5. Technical Advantages

### 5.1 Spec88 Workpoint Becomes the Typed Continuation Anchor

The Workpoint remains the immediate continuation authority.

Receipts must reference a canonical Workpoint when available and must mark the receipt degraded or blocked when no exact-scoped Workpoint exists.

Receipts must not reduce Workpoint continuity to `workpoint_id` alone. They should preserve:

- Workpoint id;
- revision;
- status;
- source checkpoint id;
- checkpoint reason;
- ActiveMissionSet / active object set;
- CurrentActionIntent;
- VerificationRecords;
- blockers/open loops;
- next slice;
- drift status.

This preserves Spec88’s design law: meaning lives in the typed Workpoint, not in the transcript.

### 5.2 Spec100 Context Cognition Becomes Advisory Context Frame

Context Cognition is advisory by default and must not become proof or authority inside receipts.

Receipts may summarize ContextCognitionPacket state, but must preserve:

- `canonical=false`;
- `advisory=true`;
- scope status;
- stale/degraded/block status;
- selected context summary;
- excluded context summary;
- evidence frame;
- contradiction/drift risks;
- source refs;
- next/recovery tool suggestions.

Context Cognition can support reasoning, but Evidence refs remain the proof boundary and Workpoint remains action authority.

### 5.3 Context Authority Becomes the Mutation Boundary

Receipts must include Context Authority verdicts for risky operations.

A receipt must never claim a risky mutation was safely completed unless the relevant preflight verdict is present, fresh, and compatible with the action.

### 5.4 Evidence Becomes Structural

Receipts must classify evidence as:

```text
actual | partial | surrogate | blocked | missing
```

Partial, surrogate, or blocked evidence may be useful, but must not support a completed claim unless the acceptance criteria allow it.

### 5.5 UIAI Becomes Product-Reality Proof

UIAI diagnostics and browser reliability reports should become first-class receipt evidence.

Browser proof must distinguish:

- actual browser proof;
- blocked browser proof;
- private URL guard proof;
- missing native proof;
- surrogate API/web proof.

### 5.6 Spec111 Bootstrap Becomes Receipt-Producing Delivery Proof

Spec111 Agent Context Bootstrap should not maintain a separate durable receipt system.

Instead:

```text
AgentBootstrapReceipt = specialized projection of a Focusa Receipt.
Focusa Receipt = canonical durable record committed through Spec119.
```

Bootstrap build/render/write/verify outcomes should map into `receipt_type = bootstrap_delivery` when persisted.

### 5.7 Spec112 Install Verification Becomes Receipt-Producing Setup Proof

Install and upgrade flows should be able to produce `install_verification` receipts that record:

- detected OS/arch/libc/init system;
- selected release asset;
- checksum/signature verification;
- license validation state;
- daemon install/start result;
- rollback state;
- doctor result;
- recovery hints.

### 5.8 Spec113 Eval Ledger Becomes Receipt-Producing Measurement Proof

Eval Ledger events remain append-only measurement records. Receipts do not replace them.

Instead:

```text
Eval Ledger = task/run event truth.
Receipt = summarized, evidence-linked proof object for a run, comparison, or benchmark claim.
```

### 5.9 Spec114 Public Proof Uses Receipt Projections

`proof.focusa.dev` and `bench.focusa.dev` should consume public-safe receipt projections, not raw daemon state.

### 5.10 Spec115 Cloud Hosts Projections, Not Authority

The local node owns canonical receipts. Cloud may host redacted projections, indexes, and published snapshots only after explicit export/publish.

### 5.11 Spec116 Closure Authority Becomes Receipt-Producing Closure Truth

Spec116’s ClosureClaim and provider mutation lifecycle must map into `work_item_closure` receipts.

The rule is:

```text
Focusa validates closure truth.
Providers store and display the closure.
Receipts prove the closure validation and provider mutation result.
```

### 5.12 Tool Count Becomes Tool Selection

The receipt layer must use tool contract/choreography metadata to recommend:

- top exact next tool;
- up to three next tools;
- up to three recovery tools;
- relevant family hints only.

### 5.13 Existing Event Integrity Becomes Work-Level Verification

Receipt commits must reuse Focusa’s existing event hash chain.

The receipt query model may exist for fast reads, but the canonical integrity path is:

```text
Receipt commit → ReceiptCommitted event → events table → event_hash_chain
```

---

## 6. Non-Goals

This spec does not require:

- replacing existing agent frameworks;
- replacing external agent/tool protocols;
- replacing Spec88 Workpoint continuity;
- replacing Spec100 Context Cognition;
- replacing Spec111 Agent Context Bootstrap;
- replacing Spec113 Eval Ledger;
- replacing Spec116 provider adapters;
- making Focusa a general task manager;
- making Focusa a generic vector memory system;
- exposing all Focusa tools to users by default;
- weakening canonical Focusa vocabulary;
- claiming regulatory certification;
- adding cloud sync;
- creating team/multi-user permissions in this slice;
- making public sharing the default;
- aggregating user data by default;
- implementing public-key signing in the MVP;
- publishing standalone packages before the schema stabilizes.

This spec does require:

- a local-first receipt artifact;
- API/CLI/Pi access;
- scope-bound persistence;
- Workpoint checkpoint/revision/drift provenance;
- Context Cognition advisory-context summary;
- evidence classification;
- authority posture and authority freshness;
- local receipt verification;
- receipt events linked into the existing event hash chain;
- bootstrap delivery receipt mapping;
- install verification receipt mapping;
- eval/benchmark receipt mapping;
- provider-neutral closure receipt mapping;
- public-safe redaction path for post-MVP;
- adapter-friendly schema design;
- portable JSON Schemas in the repository.

---

## 7. Design Principles

### 7.1 Artifact Over Terminology

Canonical Focusa terms remain, but the product experience should lead with useful artifacts.

A new user should understand the receipt before needing to fully understand every internal term.

### 7.2 Typed Workpoint Over Transcript Tail

Receipt continuation must be anchored in Workpoint checkpoint/revision state, not raw transcript summary.

Transcript tail may be evidence/context only when captured and labeled; it cannot override a canonical Workpoint.

### 7.3 Advisory Context Is Not Proof

Context Cognition may help choose context and guide reasoning, but receipts must label it as advisory.

Selected context, codemaps, snippets, and optimization hints do not prove completion unless linked through Evidence refs.

### 7.4 One Safe Next Action

Every receipt should include `next_safe_action`.

When Focusa cannot determine a safe next action, it should say why and return recovery tools.

### 7.5 Scope Before Certainty

A receipt cannot be canonical unless `project_root + continuity_id` are verified.

If scope is missing, unsafe, stale, or mismatched, the receipt must be degraded or blocked.

### 7.6 Proof Before Completion

A final claim cannot be marked complete unless matching evidence exists.

Completion language must be blocked when evidence is partial, surrogate, blocked, or missing.

### 7.7 Closure Truth Before Provider Mutation

A work item may not be closed through Focusa unless a `work_item_closure` receipt preview returns `completion_allowed=true` and the ClosureClaim is valid, fresh, provider-scoped, and evidence-backed.

### 7.8 Preview Before Commit

Receipt generation must support preview mode before writing to the durable ledger.

Preview may aggregate read models. Commit must enter the daemon-owned write path or serialized writer path.

### 7.9 Local-First by Default

Receipts are stored locally by default.

Public export, Arena export, team export, cloud projection, or external telemetry must be explicit and redacted.

### 7.10 Selection Over Surface Area

Receipts and awareness cards should compress Focusa’s tool graph into relevant choices.

The default surface should never overwhelm the user or agent with the full tool list.

### 7.11 Integration-Ready, Not Integration-Dependent

Focusa Receipts should be useful through CLI/API/Pi on day one and later portable through adapters.

### 7.12 Hash-Linked at Commit

A committed receipt must have a receipt hash and a canonical receipt event linked into the existing Focusa event hash chain.

### 7.13 Fresh Authority for Risky Actions

Risky action authorization expires.

A stale or expired allow verdict cannot support a committed risky-mutation receipt.

### 7.14 Public and Cloud Surfaces Are Projections

Local receipt state is canonical. Public, cloud, proof, bench, and Arena surfaces consume redacted projections.

---

## 8. Definitions

### Focusa Receipt

A local-first, scope-bound artifact summarizing agent work, Workpoint provenance, advisory context, authority posture, evidence, result, bootstrap delivery, install verification, closure status, benchmark relation, verification status, and next safe action.

### Agent Work Ledger

The durable local store of committed Focusa Receipts.

### Receipt Preview

A generated receipt candidate that does not mutate the durable ledger.

### Receipt Commit

A persisted receipt written after scope, Workpoint, authority, evidence, advisory-context, closure, idempotency, and integrity checks.

### Public-Safe Receipt

A redacted receipt suitable for public demo, Arena display, proof viewer, benchmark display, client review, or investor proof.

### Claim

A statement that the agent, tool, provider adapter, or operator wants to treat as true about work performed.

### Claim Status

One of:

```text
actual | partial | surrogate | blocked | missing
```

### Evidence Ref

A stable reference to proof such as test output, command output, browser diagnostics, screenshot path, API response, CI run, release artifact, bootstrap verification result, provider closure result, eval run report, install verification result, or log bundle.

### Workpoint Provenance

The receipt-visible Workpoint checkpoint, revision, ActiveMissionSet, CurrentActionIntent, VerificationRecords, blockers, next slice, and drift status that anchor continuation.

### Context Cognition Frame

A receipt-visible advisory summary of ContextCognitionPacket status, selected/excluded context, evidence frame, contradiction/drift risks, source refs, stale/degraded posture, and next/recovery tools.

### Governed Execution Boundary

The point where Focusa reconciles current ask, project scope, Workpoint, advisory context, environment facts, risky action class, authority freshness, Context Authority verdict, and closure policy before allowing or blocking mutation.

### Receipt Hash

A SHA-256 hash over the canonicalized receipt JSON payload.

### Event Chain Hash

A SHA-256 checkpoint created by Focusa’s existing `event_hash_chain`.

### Query Model

Receipt tables optimized for listing and retrieval. The query model is not the integrity source.

### Integrity Ledger

The canonical event path: `events + event_hash_chain`.

### AgentBootstrapReceipt

A target-specific Spec111 delivery/verification projection generated from a canonical Focusa Receipt. It is not a separate durable receipt system after Spec119.

### ClosureClaim

A Spec116 provider-neutral claim that a WorkItem is eligible for provider closure because Focusa has validated scope, policy, evidence, and mutation plan.

### Cloud Projection

A redacted, published, or indexed representation of a local receipt. It is not the canonical receipt.

---

## 9. Receipt Relationship to Reports, Workpoints, Context Cognition, Bootstrap, Closure, Evals, Public Proof, and Cloud

A Focusa Receipt does not replace every report or ledger.

Instead:

```text
Receipt = structured source of truth for work proof
Workpoint = typed continuation authority and next-action anchor
ContextCognitionPacket = advisory context/reasoning support
Final report = human-readable rendering of receipt + operator-facing explanation
AgentBootstrapReceipt = target-specific delivery projection of a receipt
ClosureClaim = provider-neutral close claim validated before closure receipt commit
Eval Ledger = append-only event truth for benchmark runs
Benchmark report = measured aggregate derived from Eval Ledger + receipts
Arena/proof/bench card = public-safe rendering of receipt
Agent handoff = continuation-focused rendering of receipt
Cloud projection = redacted hosted index/snapshot of local receipt
CI summary = automation-focused rendering of receipt
```

Agents must treat the receipt as the canonical structured artifact when reporting completion, blockers, Workpoint provenance, advisory context, bootstrap delivery, closure state, install verification, eval evidence, or next steps.

---

## 10. Receipt Types

### 10.1 MVP receipt types

```text
work_session
risky_mutation
final_report
blocked_claim
handoff
bootstrap_delivery
work_item_closure
```

Definitions:

- `work_session`: summarizes a bounded work interval.
- `risky_mutation`: records a mutation attempt requiring Context Authority.
- `final_report`: supports or blocks a completion claim.
- `blocked_claim`: records why a claim cannot be treated as complete.
- `handoff`: captures state for another agent/session/operator.
- `bootstrap_delivery`: records Spec111 bootstrap packet build/render/write/verify status.
- `work_item_closure`: records Spec116 ClosureClaim validation, provider mutation, reconcile, bypass, or block status.

### 10.2 Post-MVP receipt types

```text
install_verification
eval_run
benchmark_result
public_proof_snapshot
cloud_projection
support_bundle
```

Definitions:

- `install_verification`: records Spec112 install/upgrade/license verification.
- `eval_run`: summarizes one Spec113 Eval Ledger run.
- `benchmark_result`: summarizes one measured benchmark comparison or release result.
- `public_proof_snapshot`: records a Spec114 public-safe proof export.
- `cloud_projection`: records a Spec115 redacted cloud-hosted projection.
- `support_bundle`: records operator-approved support bundle generation and redaction state.

---

## 11. Schema Versioning

Receipt schema uses explicit top-level version fields:

```json
{
  "schema": "focusa.receipt.v1",
  "schema_version": "1.0.0",
  "receipt_type": "work_session",
  "receipt_id": "uuid"
}
```

Rules:

- `schema` is the stable machine contract family.
- `schema_version` is the precise version.
- Breaking schema changes require a new major version.
- Additive fields may increment minor version.
- Renderers must tolerate unknown fields.

---

## 12. Receipt Schema v1

Canonical schema:

```text
focusa.receipt.v1
```

Minimum shape:

```json
{
  "schema": "focusa.receipt.v1",
  "schema_version": "1.0.0",
  "receipt_type": "work_session|risky_mutation|final_report|blocked_claim|handoff|bootstrap_delivery|work_item_closure|install_verification|eval_run|benchmark_result|public_proof_snapshot|cloud_projection|support_bundle",
  "receipt_id": "uuid",
  "created_at": "iso8601",
  "project_identity": {
    "project_root": "string",
    "project_id": "string|null",
    "project_name": "string|null",
    "canonical": true,
    "posture": "canonical|advisory|degraded|blocked|stale"
  },
  "continuity": {
    "continuity_id": "string|null",
    "session_id": "string|null",
    "workstream_key": "string|null",
    "scope_verified": true
  },
  "operator_ask": {
    "text": "string|null",
    "source": "operator|agent|system|provider|installer|eval_runner|cloud|unknown",
    "captured_at": "iso8601|null"
  },
  "trajectory": {
    "hlt": "string|null",
    "mlg": "string|null",
    "stg": "string|null",
    "waypoints": [],
    "active_gap": "string|null",
    "posture": "canonical|advisory|degraded|blocked|stale"
  },
  "workpoint": {
    "workpoint_id": "uuid|null",
    "revision": null,
    "status": "active|superseded|degraded|blocked|unknown|null",
    "source_checkpoint_id": "string|null",
    "checkpoint_reason": "compaction|resume|context_overflow|operator_checkpoint|pre_surgery|model_switch|verification_complete|unknown|null",
    "work_item_id": "string|null",
    "mission": "string|null",
    "active_object_set_id": "string|null",
    "active_object_refs": [],
    "current_action_intent_id": "string|null",
    "current_action_summary": "string|null",
    "verification_record_ids": [],
    "blocker_object_ids": [],
    "next_slice": "string|null",
    "do_not_drift": [],
    "drift_status": "not_checked|aligned|drift_detected|superseded_by_operator|unknown",
    "drift_reason": "string|null",
    "canonical": true,
    "posture": "canonical|advisory|degraded|blocked|stale"
  },
  "context_cognition": {
    "packet_id": "string|null",
    "schema_version": "focusa.context_cognition_packet.v1|null",
    "status": "completed|degraded|stale|blocked|null",
    "scope_status": "matched|missing|partial|mismatch|null",
    "canonical": false,
    "advisory": true,
    "stale": false,
    "source_snapshot": "string|null",
    "selected_context_summary": [],
    "excluded_context_summary": [],
    "over_budget_summary": [],
    "ontology_frame_summary": {
      "active_objects": [],
      "relations": [],
      "risks": [],
      "valid_next_actions": []
    },
    "evidence_frame_summary": {
      "proven": [],
      "unproven": [],
      "stale": [],
      "missing": []
    },
    "reasoning_frame_summary": {
      "likely_goal": "string|null",
      "active_gap": "string|null",
      "confidence": "low|medium|high|unknown|null",
      "contradiction_flags": [],
      "drift_risks": []
    },
    "optimization_frame_summary": {
      "module_name": "string|null",
      "prompt_artifact_ref": "string|null",
      "eval_ref": "string|null",
      "promoted": false
    },
    "source_refs": [],
    "next_tools": [],
    "recovery_tools": []
  },
  "authority": {
    "required": false,
    "verdict": "allow|block|ask_operator|verify_first|diagnosis_only|planning_only|null",
    "risk_class": "none|low|medium|high|critical",
    "action_kind": "string|null",
    "target": "string|null",
    "issued_at": "iso8601|null",
    "valid_until": "iso8601|null",
    "ttl_seconds": 0,
    "freshness_status": "fresh|expired|missing|not_required",
    "requires_recheck": false,
    "conflicts": [],
    "safe_alternative": "string|null",
    "preflight_ref": "string|null"
  },
  "bootstrap": {
    "packet_id": "string|null",
    "target": "cursor|claude|codex|pi|opencode|generic|null",
    "mode": "session_start|post_compaction|session_transfer|recovery|tool_guidance|null",
    "delivery_status": "not_applicable|rendered|written|verified|failed|dry_run|blocked",
    "files_written": [],
    "files_skipped": [],
    "verifier_status": "pending|passed|failed|skipped|null",
    "missing_fields": [],
    "failed_checks": [],
    "fail_phrase": "FOCUSA_PRELOAD_FAIL|null"
  },
  "closure": {
    "claim_id": "string|null",
    "work_item": {
      "provider": "bd|linear|asana|github|unknown|null",
      "provider_item_id": "string|null",
      "external_url": "string|null"
    },
    "closure_kind": "code|docs|deploy|investigation|no_code|admin|null",
    "closure_policy": {
      "mode": "off|warn|block|null",
      "require_focusa_claim": true,
      "require_proof_refs": true,
      "require_spec_refs": true,
      "policy_version": "string|null"
    },
    "validation_status": "valid|blocked|expired|null",
    "provider_mutation_plan": {
      "provider": "bd|linear|asana|github|unknown|null",
      "action": "close|comment|status_update|close_with_comment|null",
      "target_status": "string|null",
      "idempotency_key": "string|null"
    },
    "provider_submit_status": "not_attempted|submitted|reconciled|partial_failure|bypassed|null",
    "bypass_detected": false
  },
  "install": {
    "install_id": "string|null",
    "os": "string|null",
    "arch": "string|null",
    "libc": "string|null",
    "init_system": "string|null",
    "asset": "string|null",
    "checksum_status": "not_applicable|passed|failed|missing",
    "signature_status": "not_applicable|passed|failed|missing",
    "license_status": "not_applicable|valid|invalid|expired|skipped_eval",
    "service_status": "not_applicable|installed|started|failed|rolled_back",
    "rollback_status": "not_applicable|not_needed|completed|failed"
  },
  "eval": {
    "suite_id": "string|null",
    "run_id": "string|null",
    "task_id": "string|null",
    "arm": "no_focusa|passive_focusa|tool_only_focusa|full_focusa|null",
    "resolved": null,
    "score": null,
    "report_ref": "string|null",
    "ledger_ref": "string|null"
  },
  "public_projection": {
    "projection_type": "none|arena_card|proof_snapshot|benchmark_snapshot|cloud_projection",
    "publish_allowed": false,
    "redaction_status": "not_applicable|pending|passed|failed",
    "public_url": "string|null",
    "projection_hash": "string|null"
  },
  "execution": {
    "summary": "string",
    "primary_actions": [],
    "touched_refs": [],
    "side_effects": [],
    "event_refs": [],
    "workpoint_refs": [],
    "trajectory_refs": [],
    "context_cognition_refs": [],
    "evidence_refs": []
  },
  "evidence": {
    "refs": [],
    "counts": {
      "actual": 0,
      "partial": 0,
      "surrogate": 0,
      "blocked": 0,
      "missing": 0
    }
  },
  "claim": {
    "text": "string",
    "status": "actual|partial|surrogate|blocked|missing",
    "completion_allowed": false,
    "missing_evidence": [],
    "overclaim_risks": []
  },
  "outcome": {
    "status": "completed|partial|blocked|failed|in_progress|unknown",
    "prediction_refs": [],
    "metacog_refs": [],
    "elapsed_ms": null,
    "token_estimate": null
  },
  "next_safe_action": {
    "summary": "string",
    "tool": "string|null",
    "reason": "string",
    "requires_operator": false,
    "recovery_tools": []
  },
  "privacy": {
    "public_safe": false,
    "redacted_fields": [],
    "private_refs": []
  },
  "verification": {
    "receipt_hash": "string|null",
    "receipt_event_id": "uuid|null",
    "event_chain_hash": "string|null",
    "previous_event_chain_hash": "string|null",
    "event_chain_index": null,
    "verified_at_commit": false,
    "signature": null
  }
}
```

---

## 13. Execution Summary Must Not Become a Full Audit Log

The `execution` block should summarize and link rather than duplicate every low-level event.

Guidance:

- `primary_actions` should contain only important actions.
- `touched_refs` should contain compact references to files, routes, services, providers, eval runs, or objects.
- Full command logs, tool call logs, browser logs, bootstrap packet JSON, provider API payloads, eval event streams, context cognition packets, full selected snippets, and CI logs should remain in their original stores and be linked through refs.
- Receipts should be compact enough for agents to read and reliable enough for humans to audit.

---

## 14. Evidence Ref Shape

Receipt evidence refs must use a consistent shape:

```json
{
  "evidence_ref": "string",
  "class": "actual|partial|surrogate|blocked|missing",
  "source": "test|cli|api|browser|uiai|ci|screenshot|log|operator|agent|workpoint|context_cognition|bootstrap|install|provider|eval|benchmark|cloud|unknown",
  "summary": "string",
  "supports_claim": true,
  "workpoint_id": "uuid|null",
  "workpoint_revision": null,
  "context_cognition_packet_id": "string|null",
  "work_item_ref": "string|null",
  "artifact_path": "string|null",
  "external_url": "string|null",
  "created_at": "iso8601|null",
  "public_safe": false,
  "redaction_required": true
}
```

---

## 15. Minimal Adapter Payload

Define a small export shape for external systems.

Schema:

```text
focusa.receipt.summary.v1
```

Shape:

```json
{
  "schema": "focusa.receipt.summary.v1",
  "schema_version": "1.0.0",
  "receipt_id": "uuid",
  "receipt_type": "work_session|risky_mutation|final_report|blocked_claim|handoff|bootstrap_delivery|work_item_closure|install_verification|eval_run|benchmark_result|public_proof_snapshot|cloud_projection|support_bundle",
  "project_root": "string",
  "continuity_id": "string|null",
  "workpoint_id": "uuid|null",
  "workpoint_revision": null,
  "current_action_intent_id": "string|null",
  "drift_status": "not_checked|aligned|drift_detected|superseded_by_operator|unknown",
  "context_cognition_status": "completed|degraded|stale|blocked|null",
  "claim_status": "actual|partial|surrogate|blocked|missing",
  "outcome_status": "completed|partial|blocked|failed|in_progress|unknown",
  "authority_verdict": "allow|block|ask_operator|verify_first|diagnosis_only|planning_only|null",
  "bootstrap_delivery_status": "not_applicable|rendered|written|verified|failed|dry_run|blocked",
  "closure_validation_status": "valid|blocked|expired|null",
  "provider_submit_status": "not_attempted|submitted|reconciled|partial_failure|bypassed|null",
  "install_verification_status": "not_applicable|passed|failed|partial",
  "public_projection_status": "none|pending|published|blocked",
  "evidence_summary": {
    "actual": 0,
    "partial": 0,
    "surrogate": 0,
    "blocked": 0,
    "missing": 0
  },
  "next_safe_action": {
    "summary": "string",
    "tool": "string|null",
    "requires_operator": false
  }
}
```

This summary is the preferred integration object for external agent tools, editor plugins, CI systems, provider adapters, cloud projections, public proof surfaces, and future handoff adapters.

External systems should not need to consume the full Focusa tool graph to benefit from Focusa receipts.

---

## 16. Portable Receipt Schema Package

Focusa should expose receipt schemas without requiring external tools to depend on the full daemon.

Add a schema package inside the repository first:

```text
schemas/receipt/focusa.receipt.v1.schema.json
schemas/receipt/focusa.receipt.summary.v1.schema.json
schemas/receipt/focusa.receipt_verification.v1.schema.json
schemas/receipt/examples/final_report.partial.json
schemas/receipt/examples/risky_mutation.blocked.json
schemas/receipt/examples/work_session.actual.json
schemas/receipt/examples/workpoint_drift.blocked.json
schemas/receipt/examples/context_cognition.stale.json
schemas/receipt/examples/bootstrap_delivery.verified.json
schemas/receipt/examples/work_item_closure.valid.json
schemas/receipt/examples/install_verification.partial.json
schemas/receipt/examples/benchmark_result.public_safe.json
```

Add generated language bindings later:

```text
packages/focusa-receipt-schema-js
crates/focusa-receipt-schema
```

MVP requirement:

- JSON Schema files exist.
- Examples validate against schemas.
- Focusa API/CLI/Pi use the same schema definitions or generated types.
- README/docs point external integrators to the schema files.

Post-MVP:

- publish standalone JS package;
- publish standalone Rust crate;
- provide minimal adapter docs.

### 16.1 Schema Independence Rule

Portable receipt schemas must not depend on:

- Focusa daemon runtime;
- Pi extension internals;
- Tauri menubar code;
- UIAI implementation details;
- provider-specific adapter internals;
- cloud control plane internals;
- internal-only tool names beyond optional metadata fields.

The portable schema may reference Focusa concepts, but the minimum integration summary must be usable with:

```text
receipt_id
receipt_type
project_root
continuity_id
workpoint_id
workpoint_revision
claim_status
authority_verdict
context_cognition_status
bootstrap_delivery_status
closure_validation_status
evidence_summary
next_safe_action
```

---

## 17. Required API Surfaces

Add receipt routes:

```http
POST /v1/receipts/preview
POST /v1/receipts/commit
GET  /v1/receipts/{receipt_id}
GET  /v1/receipts
GET  /v1/receipts/{receipt_id}/verify
GET  /v1/receipts/verify-chain
POST /v1/receipts/{receipt_id}/redact
POST /v1/receipts/{receipt_id}/export
```

MVP API routes:

```http
POST /v1/receipts/preview
POST /v1/receipts/commit
GET  /v1/receipts/{receipt_id}
GET  /v1/receipts
GET  /v1/receipts/{receipt_id}/verify
GET  /v1/receipts/verify-chain
```

Post-MVP API routes:

```http
POST /v1/receipts/{receipt_id}/redact
POST /v1/receipts/{receipt_id}/export
```

### 17.1 Preview

`POST /v1/receipts/preview` generates a receipt candidate without ledger mutation.

Required behavior:

- verify project scope;
- inspect current Workpoint id/revision/checkpoint;
- inspect ActiveMissionSet, CurrentActionIntent, VerificationRecords, blockers, next slice, and drift status when available;
- inspect current ContextCognitionPacket status when available;
- preserve Context Cognition as advisory and never as proof/authority;
- inspect trajectory;
- inspect Context Authority if action is risky;
- evaluate authority freshness when applicable;
- inspect bootstrap packet/delivery state when `receipt_type=bootstrap_delivery`;
- inspect closure claim/policy/provider state when `receipt_type=work_item_closure`;
- inspect install verification state when `receipt_type=install_verification`;
- inspect eval/benchmark references when `receipt_type=eval_run|benchmark_result`;
- inspect public/cloud projection status when `receipt_type=public_proof_snapshot|cloud_projection`;
- collect evidence refs;
- classify claim status;
- recommend next safe action;
- return degraded/blocked if required fields are missing.

### 17.2 Commit

`POST /v1/receipts/commit` persists a receipt.

Commit must reject when:

- `project_root` is missing or unsafe;
- `continuity_id` is missing for canonical project-bound receipt;
- Workpoint scope mismatches current ask;
- Workpoint drift is detected and not operator-superseded;
- Workpoint checkpoint/revision is required but missing;
- Context Cognition is used as proof without Evidence refs;
- risky mutation lacks required Context Authority verdict;
- risky mutation has expired authority;
- bootstrap write evidence is claimed verified but verifier failed or was skipped;
- closure claim is invalid, expired, or mismatched to provider/work item;
- provider mutation is claimed reconciled without provider proof;
- install verification is claimed complete without checksum/signature/license/service evidence when required;
- benchmark or public claim lacks eval/proof references;
- completion claim lacks actual evidence;
- supplied evidence refs do not exist or are private without redaction markers;
- idempotency conflicts with a prior receipt commit.

### 17.3 Verify

`GET /v1/receipts/{receipt_id}/verify` verifies one receipt.

Verification must:

- recompute `receipt_hash`;
- confirm receipt payload matches stored hash;
- confirm receipt event exists;
- confirm receipt event participates in `event_hash_chain`;
- confirm event chain continuity from previous hash to current hash;
- return degraded if legacy events cannot be verified;
- return blocked if receipt hash or event chain is broken.

### 17.4 Verify Chain

`GET /v1/receipts/verify-chain` verifies the receipt-visible event chain posture.

MVP behavior:

- verify receipt-related events;
- report first broken link if found;
- report latest chain index/hash;
- report legacy/unverifiable rows separately.

### 17.5 Redact

`POST /v1/receipts/{receipt_id}/redact` creates a public-safe projection.

Redaction must remove or mask:

- absolute private file paths when necessary;
- secrets;
- tokens;
- private URLs;
- private logs;
- customer/client names when flagged;
- private screenshots unless explicitly public-safe;
- private evidence refs that cannot be shared;
- provider tokens or private provider payloads;
- private eval holdout task bodies;
- raw selected context snippets unless explicitly marked public-safe.

### 17.6 Export

`POST /v1/receipts/{receipt_id}/export` returns one or more formats:

```text
json
markdown
arena_card
agent_handoff
ci_summary
workpoint_projection
context_cognition_projection
bootstrap_projection
closure_projection
install_projection
benchmark_projection
cloud_projection
```

---

## 18. Required CLI Surfaces

Add:

```bash
focusa receipt preview
focusa receipt commit
focusa receipt show <receipt_id>
focusa receipt list
focusa receipt verify <receipt_id>
focusa receipt verify-chain
focusa receipt redact <receipt_id>
focusa receipt export <receipt_id> --format json|markdown|arena-card|agent-handoff|ci-summary|workpoint-projection|context-cognition-projection|bootstrap-projection|closure-projection|install-projection|benchmark-projection|cloud-projection
```

MVP CLI:

```bash
focusa receipt preview
focusa receipt commit
focusa receipt show <receipt_id>
focusa receipt list
focusa receipt verify <receipt_id>
focusa receipt verify-chain
```

Post-MVP CLI:

```bash
focusa receipt redact <receipt_id>
focusa receipt export <receipt_id> --format json|markdown|arena-card|agent-handoff|ci-summary|workpoint-projection|context-cognition-projection|bootstrap-projection|closure-projection|install-projection|benchmark-projection|cloud-projection
```

CLI requirements:

- `preview` must be safe by default.
- `commit` must require explicit confirmation or `--yes` when claim status is not `actual`.
- `commit` must block when `completion_allowed=false`.
- `show` must render compact human-readable summary by default.
- `verify` must show receipt hash and event-chain status.
- `--json` must return full schema.
- `export --format arena-card|benchmark-projection|cloud-projection` must require redaction unless `--private` is explicitly supplied.

---

## 19. Required Pi Tools

Add Pi tools:

```text
focusa_receipt_preview
focusa_receipt_commit
focusa_receipt_show
focusa_receipt_verify
focusa_receipt_redact
focusa_receipt_export
```

MVP Pi tools:

```text
focusa_receipt_preview
focusa_receipt_commit
focusa_receipt_show
focusa_receipt_verify
```

Post-MVP Pi tools:

```text
focusa_receipt_redact
focusa_receipt_export
```

Pi tool behavior:

- Agents should call `focusa_receipt_preview` before final reports.
- Agents must not call `focusa_receipt_commit` when `completion_allowed=false`.
- Agents must not close provider work items unless closure receipt preview allows it.
- Agents must not treat Context Cognition packet suggestions as operator authorization or proof.
- If receipt preview blocks completion or closure, the agent must report the missing evidence plainly.
- Pi output should show one next safe action and up to three recovery tools.
- `focusa_receipt_verify` should be used when the operator asks whether a committed receipt is intact.

---

## 20. Ledger Persistence

Receipts must persist locally.

Preferred storage:

```text
SQLite tables backed by existing Focusa persistence patterns.
```

Minimum tables:

```text
agent_work_receipts
agent_work_receipt_evidence_refs
agent_work_receipt_actions
```

Required columns on `agent_work_receipts`:

```text
receipt_id
receipt_type
project_root
continuity_id
workpoint_id
workpoint_revision
source_checkpoint_id
current_action_intent_id
drift_status
context_cognition_packet_id
context_cognition_status
claim_status
completion_allowed
bootstrap_delivery_status
closure_validation_status
provider_submit_status
install_verification_status
eval_run_id
public_projection_status
receipt_json
receipt_hash
receipt_event_id
event_chain_index
event_chain_hash
previous_event_chain_hash
created_at
```

Required indexes:

```text
receipt_id
created_at
project_root
continuity_id
workpoint_id
workpoint_revision
source_checkpoint_id
current_action_intent_id
drift_status
context_cognition_packet_id
claim_status
outcome_status
bootstrap_delivery_status
closure_validation_status
provider_submit_status
install_verification_status
eval_run_id
public_projection_status
public_safe
receipt_hash
receipt_event_id
event_chain_index
```

Rule:

```text
agent_work_receipts is the query model.
events + event_hash_chain is the integrity ledger.
```

If the query model and event chain disagree, verification must prefer the event-chain-backed canonical record and report the query model as stale/degraded.

JSONL export may exist as a secondary artifact, but SQLite is the canonical local ledger.

---

## 21. Persistence Must Follow the Daemon-Owned State Model

Receipt commit must not bypass Focusa’s canonical write path.

Implementation rule:

```text
Receipt preview may aggregate read models.
Receipt commit must dispatch or serialize through the daemon-owned write path.
```

Acceptable MVP implementation:

1. API receives `POST /v1/receipts/commit`.
2. API validates request shape, scope, Workpoint provenance, advisory context labels, evidence class, authority freshness, bootstrap verifier state, closure state, and idempotency.
3. API dispatches a receipt commit action or enters the existing serialized writer path.
4. Core receipt evaluator produces the accepted receipt record.
5. Persistence writes the receipt query record.
6. Persistence appends a canonical receipt event.
7. Existing persistence appends hash-chain checkpoint for the event.
8. API returns the committed receipt envelope with verification fields.

---

## 22. Receipt Integrity

Focusa already has a tamper-evident event chain at the SQLite persistence layer.

Receipt commit MUST use that existing integrity path instead of creating an unrelated receipt-only chain.

Rule:

```text
A committed receipt is not fully accepted unless it is persisted and represented by one or more canonical receipt events that participate in the existing event_hash_chain.
```

MVP Phase 2 must include receipt event hashing by default.

Required-on-commit verification fields:

```json
{
  "verification": {
    "receipt_hash": "string",
    "receipt_event_id": "uuid",
    "event_chain_hash": "string",
    "previous_event_chain_hash": "string|null",
    "event_chain_index": 0,
    "verified_at_commit": true,
    "signature": null
  }
}
```

Notes:

- `receipt_hash` is the SHA-256 hash of the canonicalized receipt JSON payload.
- `receipt_event_id` links the receipt to the canonical persisted Focusa event.
- `event_chain_hash` is the hash checkpoint from Focusa’s existing event chain.
- `previous_event_chain_hash` links to the prior event hash checkpoint.
- `signature` remains optional and post-MVP.
- Hash chaining detects ordinary local row edits/deletions; it does not claim to prevent privileged machine-level tampering.

---

## 23. Receipt Commit Event Types

Add canonical receipt event types:

```text
ReceiptPreviewed
ReceiptCommitRequested
ReceiptCommitted
ReceiptRejected
ReceiptVerificationChecked
```

MVP-required:

```text
ReceiptCommitRequested
ReceiptCommitted
ReceiptRejected
```

Post-MVP:

```text
ReceiptPreviewed
ReceiptVerificationChecked
ReceiptRedacted
ReceiptExported
ReceiptPublished
```

`ReceiptCommitted` must include:

```json
{
  "type": "ReceiptCommitted",
  "receipt_id": "uuid",
  "receipt_type": "work_session|risky_mutation|final_report|blocked_claim|handoff|bootstrap_delivery|work_item_closure|install_verification|eval_run|benchmark_result|public_proof_snapshot|cloud_projection|support_bundle",
  "project_root": "string",
  "continuity_id": "string|null",
  "workpoint_id": "uuid|null",
  "workpoint_revision": null,
  "source_checkpoint_id": "string|null",
  "current_action_intent_id": "string|null",
  "drift_status": "not_checked|aligned|drift_detected|superseded_by_operator|unknown",
  "context_cognition_packet_id": "string|null",
  "context_cognition_status": "completed|degraded|stale|blocked|null",
  "claim_status": "actual|partial|surrogate|blocked|missing",
  "completion_allowed": false,
  "bootstrap_delivery_status": "not_applicable|rendered|written|verified|failed|dry_run|blocked",
  "closure_validation_status": "valid|blocked|expired|null",
  "provider_submit_status": "not_attempted|submitted|reconciled|partial_failure|bypassed|null",
  "receipt_hash": "string",
  "receipt_event_id": "uuid",
  "evidence_counts": {
    "actual": 0,
    "partial": 0,
    "surrogate": 0,
    "blocked": 0,
    "missing": 0
  }
}
```

---

## 24. Canonicalization Requirement

Receipt hashes require deterministic canonicalization.

MVP canonicalization rule:

```text
Canonical receipt JSON must be UTF-8 JSON with sorted object keys, no insignificant whitespace, stable timestamp strings, and no runtime-only fields.
```

Excluded from `receipt_hash`:

```text
verification.signature
verification.verified_at
rendered_markdown
transient API envelope fields
debug-only route metadata
```

Included in `receipt_hash`:

```text
schema
schema_version
receipt_type
receipt_id
created_at
project_identity
continuity
operator_ask
trajectory
workpoint
context_cognition
authority
bootstrap
closure
install
eval
public_projection
execution
evidence
claim
outcome
next_safe_action
privacy
```

---

## 25. Governed Execution Boundary

Receipt generation must integrate with Workpoint, Context Cognition, and Context Authority.

For risky operations, a receipt must include:

- action kind;
- target;
- current ask;
- Workpoint checkpoint/revision;
- CurrentActionIntent;
- drift status;
- Context Cognition advisory status when used;
- environment role when available;
- project root;
- repo/daemon/CLI version when relevant;
- Context Authority verdict;
- authority freshness;
- conflicts;
- safe alternative;
- preflight evidence ref.

Risky operations include:

- deploy;
- release publish;
- git push;
- destructive file operation;
- database migration;
- broad refactor;
- cross-project edit;
- generated-code overwrite;
- secret/config change;
- live service action;
- binary replacement;
- daemon restart;
- install/update ambiguity;
- preload/bootstrap file write;
- work-item provider close/status mutation.

If no preflight exists, receipt status must be `blocked` or `verify_first`.

---

## 26. Authority Freshness and Expiration

Risky action authorization must expire.

The `authority` block must include:

```json
{
  "authority": {
    "required": true,
    "verdict": "allow|block|ask_operator|verify_first|diagnosis_only|planning_only|null",
    "risk_class": "none|low|medium|high|critical",
    "action_kind": "string|null",
    "target": "string|null",
    "issued_at": "iso8601|null",
    "valid_until": "iso8601|null",
    "ttl_seconds": 0,
    "freshness_status": "fresh|expired|missing|not_required",
    "requires_recheck": false,
    "conflicts": [],
    "safe_alternative": "string|null",
    "preflight_ref": "string|null"
  }
}
```

Rules:

- `allow` verdicts for risky mutations must include `issued_at`, `valid_until`, and `ttl_seconds`.
- Expired authority cannot support a committed `risky_mutation` receipt.
- If authority is expired, receipt preview must return `verify_first` or `blocked`.
- `diagnosis_only` and `planning_only` verdicts cannot authorize mutation.
- `block` verdicts do not need TTL but should include `issued_at`.
- Missing freshness on a risky mutation is degraded at preview and blocked at commit.

### 26.1 Default TTL Policy

Default TTL policy:

```text
low risk: 30 minutes
medium risk: 15 minutes
high risk: 5 minutes
critical risk: 0 minutes; explicit recheck required at commit
```

Risk mapping:

```text
deploy: high
release publish: high
git push: medium or high depending on branch
destructive file operation: high
database migration: critical
secret/config change: critical
live service action: high
binary replacement: high
daemon restart: medium or high depending on host role
broad refactor: medium
cross-project edit: high
generated-code overwrite: medium
preload/bootstrap file write: medium
work-item provider close/status mutation: high
```

This policy may later become configurable, but MVP should hardcode safe defaults.

### 26.2 Authority Recheck at Commit

Receipt preview may show an allowed risky action when the preflight is fresh.

Receipt commit must re-evaluate freshness.

Commit must reject when:

```text
authority.required=true
AND authority.verdict=allow
AND now > valid_until
```

Blocked response:

```json
{
  "status": "blocked",
  "posture": "blocked",
  "failure_class": "authority_expired",
  "completion_allowed": false,
  "authority": {
    "verdict": "verify_first",
    "freshness_status": "expired",
    "requires_recheck": true
  },
  "next_safe_action": {
    "summary": "Run Context Authority preflight again before committing this receipt.",
    "tool": "focusa_action_preflight",
    "requires_operator": false
  },
  "recovery_tools": [
    "focusa_action_preflight",
    "focusa_receipt_preview",
    "focusa_receipt_commit"
  ]
}
```

---

## 27. Spec88 Workpoint Continuity Integration

Spec88 is a first-class receipt input.

### 27.1 Relationship

```text
Spec88 preserves typed continuation.
Spec119 records which typed continuation was used, whether the agent drifted, and what proof closed the loop.
```

### 27.2 Required Mapping

When a receipt is previewed or committed for work tied to an active Workpoint, map:

```text
Workpoint.workpoint_id              → receipt.workpoint.workpoint_id
Workpoint.revision                  → receipt.workpoint.revision
Workpoint.status                    → receipt.workpoint.status
Workpoint.source_checkpoint_id      → receipt.workpoint.source_checkpoint_id
Workpoint.checkpoint_reason         → receipt.workpoint.checkpoint_reason
Workpoint.work_item_id              → receipt.workpoint.work_item_id
Workpoint.active_object_set_id      → receipt.workpoint.active_object_set_id
Workpoint.current_action_intent_id  → receipt.workpoint.current_action_intent_id
Workpoint.verification_record_ids   → receipt.workpoint.verification_record_ids
Workpoint.blocker_object_ids        → receipt.workpoint.blocker_object_ids
Workpoint.next_slice                → receipt.workpoint.next_slice
Workpoint.do_not_drift              → receipt.workpoint.do_not_drift
WorkpointDriftDetected              → receipt.workpoint.drift_status + drift_reason
```

### 27.3 Rules

- A receipt may not use raw transcript tail as the continuation anchor when a scoped Workpoint exists.
- If Workpoint drift is detected, receipt preview must return blocked/degraded unless operator steering explicitly supersedes the Workpoint.
- If receipt completion depends on Workpoint action intent, the receipt must include CurrentActionIntent or mark itself degraded.
- If receipt proof depends on verification records, the receipt must include linked VerificationRecord ids or Evidence refs.
- A receipt generated during compaction, overflow recovery, model switch, or handoff must record the Workpoint checkpoint/revision used.

---

## 28. Spec100 Context Cognition Integration

Spec100 is a first-class advisory-context input.

### 28.1 Relationship

```text
Spec100 selects and frames context.
Spec119 records which context frame was used and keeps it advisory unless evidence promotes the claim.
```

### 28.2 Required Mapping

When Context Cognition participates in a receipt preview or commit, map:

```text
ContextCognitionPacket.packet_id                    → receipt.context_cognition.packet_id
ContextCognitionPacket.schema_version               → receipt.context_cognition.schema_version
ContextCognitionPacket.status                       → receipt.context_cognition.status
ContextCognitionPacket.scope_status                 → receipt.context_cognition.scope_status
ContextCognitionPacket.canonical/advisory           → receipt.context_cognition.canonical/advisory
ContextCognitionPacket.freshness.stale              → receipt.context_cognition.stale
ContextCognitionPacket.freshness.source_snapshot    → receipt.context_cognition.source_snapshot
selected_context                                    → receipt.context_cognition.selected_context_summary
excluded_context                                    → receipt.context_cognition.excluded_context_summary
over_budget                                         → receipt.context_cognition.over_budget_summary
ontology_frame                                      → receipt.context_cognition.ontology_frame_summary
evidence_frame                                      → receipt.context_cognition.evidence_frame_summary
reasoning_frame                                     → receipt.context_cognition.reasoning_frame_summary
optimization_frame                                  → receipt.context_cognition.optimization_frame_summary
source_refs                                         → receipt.context_cognition.source_refs
route_frame.next_tools                              → receipt.context_cognition.next_tools
route_frame.recovery_tools                          → receipt.context_cognition.recovery_tools
```

### 28.3 Rules

- Context Cognition output is always advisory unless separately promoted through existing Focusa systems.
- A receipt must never treat selected context, codemaps, snippets, relation candidates, or optimizer hints as proof.
- Context Cognition may contribute to `next_safe_action`, `recovery_tools`, overclaim risks, and missing evidence analysis.
- If Context Cognition is stale, degraded, or scope-mismatched, the receipt must surface that status.
- Public receipt projections must not expose raw selected snippets unless explicitly public-safe.

---

## 29. Spec111 Bootstrap Integration

Spec111 Agent Context Bootstrap & Delivery should become a receipt-producing subsystem.

### 29.1 Relationship

```text
Spec111 Bootstrap builds/delivers/verifies startup context.
Spec119 Receipts record that delivery as verifiable work.
```

`AgentBootstrapReceipt` becomes a compact target-specific projection of `focusa.receipt.v1`, not a separate durable receipt system.

### 29.2 Required Mapping

When `/v1/preload/build`, `/v1/preload/render`, `/v1/preload/write`, or `/v1/preload/verify` produces a delivery result, Focusa SHOULD be able to generate a receipt preview.

When a bootstrap delivery result is persisted, Focusa SHOULD commit:

```text
receipt_type = bootstrap_delivery
```

Mapping:

```text
AgentBootstrapPacket.packet_id        → receipt.bootstrap.packet_id
AgentBootstrapPacket.target           → receipt.bootstrap.target
AgentBootstrapPacket.mode             → receipt.bootstrap.mode
AgentBootstrapReceipt.status          → receipt.bootstrap.delivery_status
AgentBootstrapReceipt.files_written   → receipt.bootstrap.files_written
AgentBootstrapReceipt.files_skipped   → receipt.bootstrap.files_skipped
AgentBootstrapReceipt.verifier.status → receipt.bootstrap.verifier_status
missing_fields                        → receipt.bootstrap.missing_fields
failed_checks                         → receipt.bootstrap.failed_checks
FOCUSA_PRELOAD_FAIL                   → receipt.bootstrap.fail_phrase
```

### 29.3 Claim Status Rules

Bootstrap delivery receipts must classify claim status as:

```text
actual    = bootstrap packet was written/delivered and verified
partial   = bootstrap packet was rendered but not written or not verified
blocked   = bootstrap verification failed or required fields are missing
missing   = bootstrap packet is absent when required
surrogate = delivery proof comes from a different target/surface than required
```

### 29.4 Authority Rules

- `preload build` and `preload render` are read/compose by default and do not require mutation authority.
- `preload verify` is read-only and does not require mutation authority.
- `preload write` writes project files and therefore must be treated as a risky operation.
- `preload write` receipts must include Context Authority when the target writes files into a project root.
- Pi session-start bootstrap remains special: no file write by default; delivery happens through Pi session lifecycle and follow-up message/tool context.

---

## 30. Spec116 Work-Item Closure Integration

Spec116 provider-neutral closure authority is a first-class receipt producer.

### 30.1 Relationship

```text
Spec116 validates closure truth.
Spec119 records closure validation, provider mutation, reconcile, bypass, and proof status.
```

### 30.2 Required Mapping

When `focusa work-item closure prepare|validate|submit` or `focusa work-item close` produces a closure state, Focusa SHOULD generate a receipt preview.

When a closure is submitted or blocked under Focusa authority, Focusa SHOULD commit:

```text
receipt_type = work_item_closure
```

Mapping:

```text
ClosureClaim.claim_id                    → receipt.closure.claim_id
ClosureClaim.work_item.provider          → receipt.closure.work_item.provider
ClosureClaim.work_item.provider_item_id  → receipt.closure.work_item.provider_item_id
ClosureClaim.work_item.external_url      → receipt.closure.work_item.external_url
ClosureClaim.closure_kind                → receipt.closure.closure_kind
ClosureClaim.policy_version              → receipt.closure.closure_policy.policy_version
ClosureClaim.validation_status           → receipt.closure.validation_status
ProviderMutationPlan.provider            → receipt.closure.provider_mutation_plan.provider
ProviderMutationPlan.action              → receipt.closure.provider_mutation_plan.action
ProviderMutationPlan.target_status       → receipt.closure.provider_mutation_plan.target_status
ProviderMutationPlan.idempotency_key     → receipt.closure.provider_mutation_plan.idempotency_key
reconcile result                         → receipt.closure.provider_submit_status
bypass audit                             → receipt.closure.bypass_detected
```

### 30.3 Closure Claim Status Rules

Closure receipts must classify claim status as:

```text
actual    = ClosureClaim valid, provider mutation reconciled, evidence refs satisfy closure profile
partial   = ClosureClaim valid but provider mutation or reconcile is incomplete
blocked   = ClosureClaim invalid, expired, missing required proof/spec refs, or policy blocks close
missing   = no ClosureClaim exists when provider close is requested
surrogate = provider state changed but Focusa validation/audit evidence is indirect or out-of-band
```

### 30.4 Closure Authority Rules

- Raw provider close attempts guarded by Focusa should produce `work_item_closure` receipt previews or blocked receipt candidates.
- `focusa work-item close <id> --from-workpoint` should run receipt preview or equivalent closure validation before provider mutation.
- Provider mutation must not occur unless ClosureClaim is valid and receipt preview says `completion_allowed=true`.
- Break-glass overrides must create `work_item_closure` receipts with `claim.status=partial|blocked|surrogate` unless policy explicitly validates the override as actual.
- Bypassed provider-side closures must be recorded as `bypass_detected=true` and should block public/release claims until reconciled.

### 30.5 Closure Next Tools

Typical next-tool routing:

```text
blocked/missing ClosureClaim → focusa work-item closure prepare
invalid ClosureClaim         → focusa work-item closure validate
valid but not submitted      → focusa work-item closure submit
provider mismatch/bypass     → focusa doctor closure
receipt missing              → focusa_receipt_preview
```

---

## 31. Spec112 Install Verification Integration

Spec112 installer and upgrade flows should become receipt-producing setup proof.

When `focusa install`, install bootstrapper, update, or license activation produces a result, Focusa SHOULD be able to generate:

```text
receipt_type = install_verification
```

Claim status rules:

```text
actual    = asset selected, checksum/signature verified, license state resolved, service install/start proven
partial   = install succeeded but optional service/license/doctor proof is missing
blocked   = incompatible system, checksum/signature failure, license invalid, service start failed without rollback proof
missing   = install claim exists without install evidence
surrogate = installer output exists but no local binary/service proof exists
```

Install receipts must not expose license keys, auth tokens, private hostnames, or environment secrets.

---

## 32. Spec113 Eval Ledger and Benchmark Integration

Spec113 Eval Ledger events remain the source of measurement truth.

Receipts should summarize and prove:

```text
receipt_type = eval_run
receipt_type = benchmark_result
```

Rules:

- Do not duplicate full eval event streams inside receipts.
- Link to Eval Ledger run IDs, raw report artifacts, scoring commit, environment digest, and evidence refs.
- Public benchmark claims must come from measured artifacts, not predictions.
- Receipt `claim.status` must be `blocked` or `missing` if benchmark evidence is predicted, incomplete, cherry-picked, or lacks raw artifacts.

---

## 33. Spec114 Public Proof Integration

Spec114 public proof and benchmark surfaces consume public-safe projections.

Rules:

- `proof.focusa.dev` consumes `public_proof_snapshot` projections.
- `bench.focusa.dev` consumes `benchmark_result` projections.
- `arena.focusa.dev` consumes `arena_card` projections.
- Public projections must not expose the internal daemon or `/v1/evals/*` directly.
- Public projection requires redaction, secret scan, proof refs, and publish approval.

---

## 34. Spec115 Cloud Projection Integration

Spec115 cloud control plane may index, host, and publish redacted receipt projections.

Rules:

```text
Local receipt = canonical.
Cloud/proof/bench receipt = redacted projection.
Cloud must not become receipt authority.
```

Cloud may own:

- receipt projection index;
- public-safe proof hosting;
- benchmark snapshot hosting;
- support bundle intake;
- team visibility summaries.

Cloud must not own:

- canonical Workpoint authority;
- raw private Focus State;
- raw project files;
- unredacted diagnostics;
- private Eval Ledger events unless explicitly exported by policy.

---

## 35. Claim Gate Integration

Spec107 claim discipline must become a hard pre-close path.

Rule:

```text
No final completion claim, provider close, public claim, or benchmark claim may be emitted by Focusa tooling when receipt preview returns completion_allowed=false.
```

Required behavior:

- `focusa_receipt_preview` evaluates the claim.
- If `completion_allowed=false`, final report tools must render the claim as blocked/partial rather than complete.
- If a future `focusa_workpoint_complete` or equivalent closure tool exists, it must require a valid receipt or run receipt preview internally.
- CLI and Pi flows must show missing evidence and recovery tools when completion is blocked.

Minimum blocked response shape:

```json
{
  "status": "blocked",
  "canonical": false,
  "completion_allowed": false,
  "claim_status": "missing",
  "missing_evidence": [],
  "overclaim_risks": [],
  "next_safe_action": {
    "summary": "Capture actual proof before claiming completion.",
    "tool": "focusa_evidence_capture",
    "requires_operator": false
  },
  "recovery_tools": [
    "focusa_evidence_capture",
    "focusa_workpoint_link_evidence",
    "focusa_receipt_preview"
  ]
}
```

---

## 36. Blocked and Degraded Agent View

When a receipt is degraded or blocked, the agent-facing response must include:

```text
status
posture
why_not_canonical
missing_scope
missing_workpoint_provenance
missing_evidence
authority_needed
context_cognition_status
bootstrap_status
closure_status
install_status
projection_status
next_safe_action
recovery_tools
```

Example:

```json
{
  "status": "blocked",
  "posture": "blocked",
  "why_not_canonical": "workpoint project_root does not match current project_root",
  "missing_scope": ["verified project_root + continuity_id"],
  "missing_workpoint_provenance": ["source_checkpoint_id", "current_action_intent_id"],
  "missing_evidence": [],
  "authority_needed": null,
  "context_cognition_status": "stale",
  "bootstrap_status": "not_applicable",
  "closure_status": "not_applicable",
  "install_status": "not_applicable",
  "projection_status": "none",
  "next_safe_action": {
    "summary": "Verify project identity and checkpoint a new Workpoint in the current project.",
    "tool": "focusa_project_identity",
    "requires_operator": false
  },
  "recovery_tools": [
    "focusa_project_identity",
    "focusa_workpoint_checkpoint",
    "focusa_context_cognition_preview",
    "focusa_receipt_preview"
  ]
}
```

---

## 37. UIAI Proof Bridge Requirements

UIAI diagnostics intake must be receipt-aware.

When UIAI reports browser/product diagnostics, Focusa should be able to convert them into receipt evidence:

```text
UIAI diagnostics → Focusa evidence refs → receipt evidence block → claim support/blocker
```

Required classifications:

- actual browser proof;
- blocked browser proof;
- missing browser proof;
- private URL guard proof;
- surrogate proof;
- native/runtime proof missing.

UIAI evidence must be linkable to:

- Workpoint;
- active object;
- claim;
- receipt;
- final report;
- public-safe projection when explicitly redacted and allowed.

---

## 38. Integration Strategy

Focusa should not require every external system to adopt Focusa internals.

Instead, Focusa should provide portable receipt/workpoint/context/evidence/bootstrap/closure payloads that can be consumed by:

- agent harnesses;
- CLI-based coding agents;
- editor extensions;
- provider adapters;
- CI/CD workflows;
- browser/product proof systems;
- installer/update flows;
- eval and benchmark systems;
- cloud projection surfaces;
- public demo surfaces;
- future agent-to-agent handoff adapters.

Integration payloads should center on:

```text
receipt_id
project_root
continuity_id
workpoint_id
workpoint_revision
current_action_intent_id
claim_status
authority_verdict
context_cognition_status
bootstrap_delivery_status
closure_validation_status
provider_submit_status
evidence_refs
next_safe_action
```

Adapters should not expose the entire Focusa tool surface by default.

---

## 39. Public-Safe / Arena / Bench / Proof Card Requirements

Public-safe export is post-MVP.

A public-safe receipt should render as a compact card:

```text
Project: <public project name>
Work: <summary>
Scope: verified/degraded/blocked
Workpoint: <id>@<revision> drift=<status>
Context: advisory <completed/stale/degraded>
Bootstrap: verified/not applicable/blocked
Closure: valid/reconciled/blocked/not applicable
Authority: allowed/blocked/verify-first
Evidence: 3 actual, 1 blocked, 0 missing
Claim: actual/partial/blocked
Verification: hash-linked/unverified/broken
Next: <safe next action>
```

Public export must never include private data unless explicitly marked public-safe.

Cards should be searchable by:

- project;
- task type;
- evidence class;
- outcome status;
- workpoint id/revision;
- drift status;
- context cognition status;
- bootstrap delivery status;
- closure validation status;
- provider submit status;
- blocked reason;
- tool family;
- date;
- verification status.

---

## 40. Agent DX Requirements

Receipt surfaces must reduce agent confusion.

Every receipt response must include:

- `status`;
- `canonical`;
- `posture`;
- `workpoint.workpoint_id` and `workpoint.revision` when relevant;
- `workpoint.current_action_intent_id` when relevant;
- `workpoint.drift_status` when relevant;
- `context_cognition.status` and `context_cognition.advisory=true` when relevant;
- `claim.status`;
- `completion_allowed`;
- `bootstrap.delivery_status` when relevant;
- `closure.validation_status` when relevant;
- `install.license_status` and verification state when relevant;
- `next_safe_action`;
- `recovery_tools`;
- `evidence.counts`;
- `missing_evidence`;
- `verification` when committed.

Error states must be recoverable.

If a receipt cannot be generated, Focusa should return:

- why;
- what scope is missing;
- which Workpoint provenance is missing;
- whether Workpoint drift exists;
- which advisory context is stale/degraded;
- which evidence is missing;
- which authority verdict is needed;
- whether authority is expired;
- which bootstrap proof is missing when relevant;
- which closure proof/policy/provider status is missing when relevant;
- which tool to call next.

---

## 41. AwarenessPacket Integration

The shared AwarenessPacket substrate should use receipt state.

Input additions:

```text
latest_receipt
open_receipt_preview
workpoint_revision
drift_status
context_cognition_status
claim_status
missing_evidence
authority_freshness
bootstrap_delivery_status
closure_validation_status
provider_submit_status
install_verification_status
receipt_verification_status
public_export_state
```

Output additions:

```text
receipt_line
workpoint_line
drift_line
context_cognition_line
claim_status_line
proof_line
authority_freshness_line
bootstrap_line
closure_line
install_line
verification_line
next_safe_action_line
```

Minimal awareness card should include receipt state only when useful:

- before final report;
- before provider closure;
- after risky mutation;
- after Workpoint checkpoint/resume/drift detection;
- after Context Cognition render/curate/proof;
- after evidence capture;
- after UIAI diagnostics;
- after bootstrap write/verify;
- after closure validation/submit/reconcile;
- after blocked claim;
- after compaction/session transfer;
- when operator asks for status;
- when receipt verification fails.

---

## 42. Implementation Phases

### Phase 0 — Field Map and Fixtures

Deliverables:

```text
docs/current/FOCUSA_RECEIPT_FIELD_MAP.md
receipt schema fixture
example degraded receipt
example blocked claim receipt
example actual proof receipt
example workpoint drift receipt
example stale context cognition receipt
example bootstrap delivery receipt
example work item closure receipt
example install verification receipt
example benchmark result receipt
```

Acceptance:

- Existing Focusa surfaces are mapped to receipt fields.
- Spec88 Workpoint fields are mapped to receipt fields.
- Spec100 Context Cognition fields are mapped to receipt fields.
- Spec111 bootstrap fields are mapped to receipt fields.
- Spec112 install fields are mapped to receipt fields.
- Spec113/114 eval and public proof fields are mapped to receipt fields.
- Spec115 cloud projection fields are mapped to receipt fields.
- Spec116 closure fields are mapped to receipt fields.
- Example receipts are checked into docs or fixtures.
- Portable JSON Schema directory exists.
- Examples validate against schemas.
- No runtime implementation required.

### Phase 1 — Receipt Preview MVP

Deliverables:

```text
POST /v1/receipts/preview
focusa receipt preview
focusa_receipt_preview
```

Acceptance:

- Read-only.
- No ledger mutation.
- Aggregates project identity, continuity, Workpoint provenance, Context Cognition advisory status, trajectory, authority posture, authority freshness, bootstrap status, closure status, evidence summary, claim status, and next safe action.
- Blocks or degrades on scope mismatch.
- Blocks or degrades when required Workpoint provenance is missing.
- Blocks or degrades when drift is detected and not operator-superseded.
- Labels Context Cognition as advisory.
- Supports `receipt_type`.
- Supports `schema_version`.
- Returns one next safe action and up to three recovery tools.

### Phase 2 — Receipt Commit + Integrity MVP

Deliverables:

```text
POST /v1/receipts/commit
GET /v1/receipts/{receipt_id}
GET /v1/receipts
GET /v1/receipts/{receipt_id}/verify
GET /v1/receipts/verify-chain

focusa receipt commit
focusa receipt show
focusa receipt list
focusa receipt verify
focusa receipt verify-chain

focusa_receipt_commit
focusa_receipt_show
focusa_receipt_verify
```

Acceptance:

- Commit follows daemon-owned write path or serialized writer path.
- Dedicated receipt query table exists.
- Receipt commit emits a canonical receipt event.
- Receipt event participates in existing `event_hash_chain`.
- Receipt hash is computed at commit.
- Receipt can be verified.
- Event chain can be verified.
- Completion claims are blocked when `completion_allowed=false`.
- Risky mutation commits recheck authority freshness.
- Expired authority blocks commit.
- Critical-risk actions require recheck at commit.

### Phase 2.5 — Basic UIAI, Workpoint, Context Cognition, Bootstrap, and Closure Evidence Classification

Deliverables:

```text
UIAI diagnostics mapped into receipt evidence classes
browser proof shown in receipt preview
blocked browser proof shown as blocked evidence
Spec88 Workpoint provenance mapped into receipts
Spec100 Context Cognition advisory status mapped into receipts
Spec111 bootstrap verify result mapped into bootstrap_delivery receipts
Spec116 closure validation mapped into work_item_closure receipts
```

Acceptance:

- Actual browser proof can support claims.
- Blocked browser proof cannot support completion.
- Surrogate proof is labeled as surrogate.
- Missing native/browser proof is labeled missing when required.
- Workpoint checkpoint/revision/current action intent can anchor receipts.
- Workpoint drift can block or degrade completion.
- Context Cognition stale/degraded/advisory state is visible and never counted as proof.
- Verified bootstrap delivery can support `bootstrap_delivery` actual claim status.
- Failed bootstrap verification blocks bootstrap delivery completion.
- Valid ClosureClaim can support `work_item_closure` actual claim status only after provider reconcile evidence.
- Invalid, expired, or bypassed closure blocks completion or marks surrogate/partial as appropriate.

### Post-MVP Phase 3 — Install, Eval, Benchmark, and Public Proof Receipt Projections

Deferred until preview, commit, verification, Workpoint, Context Cognition, bootstrap, and closure mappings are proven.

Deliverables:

```text
install_verification receipt preview/commit
eval_run receipt projection
benchmark_result receipt projection
public_proof_snapshot projection
cloud_projection export boundary
```

### Post-MVP Phase 4 — Public-Safe Export

Deferred until public-safe projection fields and redaction tests are proven.

### Post-MVP Phase 5 — Arena/Bench/Proof Cards

Deferred until public-safe export is proven.

### Post-MVP Phase 6 — External Adapter Payloads

Deferred until summary schema stabilizes.

### Post-MVP Phase 7 — External Checkpointing / Signing

Deferred until receipt UX and event-chain verification are proven.

---

## 43. MVP Acceptance Criteria

MVP is accepted when:

1. `docs/current/FOCUSA_RECEIPT_FIELD_MAP.md` exists.
2. Canonical example receipts exist.
3. JSON Schemas exist under `schemas/receipt/`.
4. Example receipts validate against schema.
5. Receipt preview works through API, CLI, and Pi.
6. Receipt preview is read-only.
7. Receipt preview returns `receipt_type` and `schema_version`.
8. Receipt preview uses existing Workpoint as the execution anchor.
9. Receipt preview includes Workpoint checkpoint/revision/current action intent when available.
10. Receipt preview degrades or blocks when `project_root + continuity_id` are missing or mismatched.
11. Receipt preview classifies claim status as actual/partial/surrogate/blocked/missing.
12. Receipt preview labels Context Cognition as advisory when included.
13. Receipt preview blocks claims that rely on Context Cognition without Evidence refs.
14. Receipt preview supports `bootstrap_delivery` receipt type.
15. Receipt preview supports `work_item_closure` receipt type.
16. Receipt preview returns one next safe action and up to three recovery tools.
17. Receipt commit persists locally through the Focusa write model.
18. Receipt commit emits receipt events.
19. Receipt commits produce `receipt_hash`.
20. Receipt commits create or link to a canonical receipt event.
21. Receipt events participate in the existing hash chain.
22. `focusa receipt verify <receipt_id>` verifies receipt hash and event-chain linkage.
23. Completion claims are blocked when `completion_allowed=false`.
24. Provider closure claims are blocked when ClosureClaim is invalid, expired, or missing proof refs.
25. Risky mutation receipts require Context Authority evidence.
26. Risky `allow` authority includes `issued_at`, `valid_until`, and `ttl_seconds`.
27. Expired authority blocks receipt commit.
28. Critical-risk actions require recheck at commit.
29. Basic UIAI diagnostics can appear as receipt evidence.
30. Basic Spec88 Workpoint provenance can appear in receipts.
31. Basic Spec100 Context Cognition packet status can appear in receipts.
32. Basic Spec111 bootstrap verification can appear as receipt evidence.
33. Basic Spec116 closure validation can appear as receipt evidence.
34. The minimal receipt summary schema works without importing Focusa daemon internals.

---

## 44. Required MVP Tests

MVP tests:

```text
tests/spec119_receipt_field_map_static_test.sh
tests/spec119_receipt_schema_static_test.sh
tests/spec119_receipt_schema_package_static_test.sh
tests/spec119_receipt_examples_validate_test.sh
tests/spec119_receipt_preview_api_test.sh
tests/spec119_receipt_preview_cli_static_test.sh
tests/spec119_receipt_preview_pi_tool_static_test.sh
tests/spec119_receipt_scope_mismatch_test.sh
tests/spec119_receipt_workpoint_provenance_test.sh
tests/spec119_receipt_workpoint_drift_test.sh
tests/spec119_receipt_context_cognition_advisory_test.sh
tests/spec119_receipt_context_cognition_stale_test.sh
tests/spec119_receipt_claim_gate_test.sh
tests/spec119_receipt_commit_persistence_test.sh
tests/spec119_receipt_commit_hash_test.sh
tests/spec119_receipt_event_chain_link_test.sh
tests/spec119_receipt_verify_cli_test.sh
tests/spec119_receipt_context_authority_required_test.sh
tests/spec119_receipt_authority_ttl_test.sh
tests/spec119_receipt_authority_expired_commit_block_test.sh
tests/spec119_receipt_uiai_basic_evidence_test.sh
tests/spec119_receipt_bootstrap_delivery_test.sh
tests/spec119_receipt_work_item_closure_test.sh
tests/spec119_receipt_summary_schema_static_test.sh
```

Post-MVP tests:

```text
tests/spec119_receipt_install_verification_test.sh
tests/spec119_receipt_eval_run_projection_test.sh
tests/spec119_receipt_benchmark_result_test.sh
tests/spec119_receipt_public_projection_test.sh
tests/spec119_receipt_cloud_projection_test.sh
tests/spec119_receipt_redaction_test.sh
tests/spec119_receipt_export_static_test.sh
tests/spec119_receipt_arena_card_test.sh
tests/spec119_receipt_adapter_summary_test.sh
tests/spec119_receipt_external_checkpoint_test.sh
tests/spec119_receipt_signing_test.sh
```

Existing tamper-evident event-chain tests must remain required by CI.

Regression fixtures:

1. Cross-project Workpoint resume must block canonical receipt.
2. Missing Workpoint checkpoint/revision must degrade receipt when action continuity is claimed.
3. Workpoint drift must block receipt completion unless operator supersession is explicit.
4. Context Cognition selected context must not count as proof without Evidence refs.
5. Stale Context Cognition packet must degrade receipt context status.
6. Mac menubar API/web-only proof must classify as surrogate when native proof is required.
7. Risky deploy without Context Authority preflight must block completion.
8. Risky mutation with expired authority must block commit.
9. UIAI browser failure must classify as blocked evidence, not success.
10. Receipt query model mismatch with event chain must return degraded/broken verification.
11. Spec111 bootstrap verification failure must classify `bootstrap_delivery` as blocked.
12. Spec116 invalid ClosureClaim must classify `work_item_closure` as blocked.
13. Provider-side bypass without Focusa validation must classify closure evidence as surrogate or blocked until reconciled.
14. Public export must remove private URLs and local absolute paths unless explicitly allowed.

---

## 45. Documentation Updates

Required docs:

```text
docs/current/FOCUSA_RECEIPT_CURRENT.md
docs/current/FOCUSA_RECEIPT_FIELD_MAP.md
docs/current/FOCUSA_RECEIPT_INTEGRITY.md
docs/current/FOCUSA_RECEIPT_SCHEMA_PACKAGE.md
docs/current/FOCUSA_RECEIPT_WORKPOINT_CONTEXT_INTEGRATION.md
docs/current/FOCUSA_RECEIPT_AUTHORITY_FRESHNESS.md
docs/current/FOCUSA_RECEIPT_BOOTSTRAP_INTEGRATION.md
docs/current/FOCUSA_RECEIPT_CLOSURE_INTEGRATION.md
docs/current/FOCUSA_RECEIPT_INSTALL_INTEGRATION.md
docs/current/FOCUSA_RECEIPT_EVAL_BENCH_PROOF_INTEGRATION.md
docs/current/FOCUSA_RECEIPT_CLOUD_PROJECTION.md
docs/current/FOCUSA_RECEIPT_PUBLIC_EXPORT.md

docs/focusa-tools/tools/focusa_receipt_preview.md
docs/focusa-tools/tools/focusa_receipt_commit.md
docs/focusa-tools/tools/focusa_receipt_show.md
docs/focusa-tools/tools/focusa_receipt_verify.md
docs/focusa-tools/tools/focusa_receipt_redact.md
docs/focusa-tools/tools/focusa_receipt_export.md
```

Required links from:

- `README.md`;
- `docs/88-ontology-backed-workpoint-continuity.md`;
- `docs/100-context-cognition-spec.md`;
- `docs/current/GOLDEN_WORKFLOW.md`;
- `docs/current/AUTHORITY_MODEL.md`;
- `docs/current/CONTEXT_AUTHORITY_CURRENT.md`;
- `docs/current/TAMPER_EVIDENT_EVENT_CHAIN.md`;
- `docs/107-spec-first-feature-lifecycle-and-claim-discipline-spec.md`;
- `docs/109-agent-first-api-redesign-ax-spec.md`;
- `docs/111-agent-context-bootstrap-and-delivery-spec.md`;
- `docs/112-install-binary-architecture-spec.md`;
- `docs/113-agent-benchmark-spec.md`;
- `docs/114-public-benchmark-flywheel-spec.md`;
- `docs/115-focusa-cloud-control-plane-tool-gateway-master-spec.md`;
- `docs/116-provider-neutral-work-item-closure-authority-spec.md`;
- generated tool surface summary;
- release docs;
- marketing copy.

Required documentation statement:

```text
Focusa Receipts are local-first, Workpoint-anchored, Context-Cognition-aware, hash-linked to the Focusa event chain at commit, and verifiable through local CLI/API commands. Hash-chain verification detects ordinary database edits or deletions, but does not replace external signing, backups, access controls, or future out-of-band checkpoint publication.
```

---

## 46. Required Future Updates to Prior Specs

After Spec119 MVP lands, update these specs surgically.

### 46.1 Spec88 update

- Add Spec119 to the normative basis.
- State that Workpoint checkpoint/revision/drift provenance is receipt-visible.
- Add receipt preview/commit to compaction, overflow, model-switch, and handoff flows where durable proof is required.
- Add tests proving receipts preserve Workpoint checkpoint/revision and drift status.

### 46.2 Spec100 update

- Add Spec119 to the normative basis.
- State that ContextCognitionPacket may be summarized in receipts but remains advisory.
- Add tests proving Context Cognition cannot satisfy proof without Evidence refs.
- Add receipt projection fields for stale/degraded/selected/excluded context.

### 46.3 Spec111 update

- Add Spec119 to the normative basis.
- State that `AgentBootstrapReceipt` is a specialized projection of `focusa.receipt.v1`.
- Update preload write/verify routes to mention receipt preview/commit.
- Add `focusa_receipt_preview` and `focusa_receipt_commit` as likely next tools after successful `focusa_preload_verify`.
- Add receipt ledger consistency to preload doctor.
- Add tests proving bootstrap delivery receipts can be generated.

### 46.4 Spec112 update

- Add `install_verification` receipt type to install success/failure/rollback docs.
- Add receipt generation after install doctor.
- Ensure license keys and private host data are excluded from receipts.

### 46.5 Spec113 update

- Add receipt projection for Eval Ledger run summaries.
- Add `receipt_id` to benchmark run reports.
- Require public claims to cite benchmark_result receipt projection.

### 46.6 Spec114 update

- Define `proof.focusa.dev` snapshots as public-safe receipt projections.
- Define `bench.focusa.dev` benchmark claim cards as benchmark_result receipt projections.
- Keep public APIs serving generated/redacted artifacts only.

### 46.7 Spec115 update

- Define cloud-hosted proof receipts as projections of local canonical receipts.
- Ensure cloud cannot mutate local receipt truth.
- Add receipt projection index boundaries.

### 46.8 Spec116 update

- Add Spec119 to normative basis.
- State that valid provider closures should produce `work_item_closure` receipts.
- Add receipt preview/commit to prepare/validate/submit/reconcile lifecycle.
- Add `focusa_receipt_preview` and `focusa_receipt_commit` to likely next tools.
- Add receipt verification to closure doctor.

---

## 47. Success Criteria

This work is successful when Focusa can reliably answer:

```text
What did the agent do?
Which Workpoint checkpoint/revision anchored it?
Did it drift?
Which advisory context was supplied, stale, excluded, or degraded?
Was it allowed?
Was the authority fresh?
Was bootstrap context delivered?
Was install/license state verified?
Was work-item closure valid and reconciled?
What proves it?
What remains?
Can the record be locally verified?
What should happen next?
```

The default experience should become:

```text
One resumable mission.
One typed Workpoint continuation anchor.
One advisory context frame.
One governed action path.
One bootstrap delivery proof when relevant.
One closure truth record when relevant.
One proof trail.
One local receipt.
One verification status.
One next safe action.
```

---

## 48. Closure Policy

Do not close Spec119 MVP implementation work until:

- receipt field map exists;
- portable JSON Schemas exist;
- schema examples validate;
- receipt preview exists across API/CLI/Pi;
- receipt commit persists locally;
- receipt commit creates a canonical event;
- receipt commit links into existing `event_hash_chain`;
- receipt verification works through CLI/API;
- Workpoint checkpoint/revision/current-action provenance is receipt-visible;
- Workpoint drift can block/degrade receipt completion;
- Context Cognition is visible as advisory and cannot satisfy proof by itself;
- claim gate blocks unsupported completion;
- provider closure gate blocks invalid closure;
- risky mutation receipts include Context Authority;
- authority freshness is enforced;
- expired authority blocks commit;
- UIAI diagnostics can become receipt evidence;
- Spec88 Workpoint provenance can become receipt context;
- Spec100 Context Cognition state can become advisory receipt context;
- Spec111 bootstrap verification can become receipt evidence;
- Spec116 closure validation can become receipt evidence;
- tests prove scope mismatch and surrogate evidence behavior;
- tests prove receipt hash and event-chain linkage;
- docs explain the receipt workflow in beginner and advanced modes.

Partial receipt surfaces may ship behind preview labels, but public docs must not claim the receipt system is complete until all MVP acceptance criteria are met.

Public-safe export, Arena cards, benchmark projections, cloud projections, external schema packages, out-of-band checkpoints, signing, and prior-spec surgical updates remain post-MVP unless explicitly accepted by operator steering.
