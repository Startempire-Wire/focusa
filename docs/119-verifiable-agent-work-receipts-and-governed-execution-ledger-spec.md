# Spec 119 — Verifiable Agent Work Receipts and Governed Execution Ledger

Status: Draft  
Owner: Verious Smith  
Created: 2026-07-05  
Scope: Focusa daemon HTTP API, CLI, Pi tools, Workpoint, Evidence, Context Authority, UIAI diagnostics intake, Spec111 Agent Context Bootstrap, public-safe cards, receipt schema package, local receipt ledger, event-chain verification, and future integration adapters.

---

## 0. Source Grounding

This spec formalizes the next Focusa product/architecture direction from current repository surfaces:

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
- `docs/107-spec-first-feature-lifecycle-and-claim-discipline-spec.md`
  - Defines actual/partial/surrogate/blocked/missing evidence classes.
  - Blocks completion claims when required evidence is missing or insufficient.
- `docs/109-agent-first-api-redesign-ax-spec.md`
  - Establishes the direction toward typed, bounded, discoverable, recoverable Agent Experience contracts.
- `docs/111-agent-context-bootstrap-and-delivery-spec.md`
  - Defines Focusa Agent Context Bootstrap & Delivery.
  - Defines AgentBootstrapPacket and AgentBootstrapReceipt.
  - States that Focusa Bootstrap turns a cold agent session into verified mission continuation.
- `docs/current/FOCUSA_ECOSYSTEM_INTERCONNECTEDNESS_AUDIT_2026-06-15.md`
  - Identifies the need for selection over dumping, shared AwarenessPacket substrate, UIAI proof bridge, tool-family selection, and public claim/evidence alignment.
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

---

## 1. Purpose

Focusa must make verifiable agent work a first-class product surface.

The next major product capability is a **Focusa Receipt**: a local-first, proof-backed, scope-bound artifact that records what an agent was asked to do, what scope it belonged to, what authority allowed or blocked action, what evidence supports the result, what remains unfinished, what context was delivered, and what the next safe action is.

This turns Focusa from a collection of agent memory/governance primitives into a visible work ledger that developers, teams, future agents, and public demo surfaces can trust.

---

## 2. Core Directive

Focusa MUST prioritize this outcome:

```text
Every meaningful agent work session should be resumable, governed, evidence-backed, receipt-producing, and locally verifiable.
```

Focusa should not add new terminology, tool families, API surfaces, or product claims unless they improve at least one of:

1. receipt quality;
2. proof quality;
3. authority accuracy;
4. recovery clarity;
5. integration portability;
6. outcome evaluation;
7. operator trust;
8. agent doability;
9. local verification;
10. bootstrap delivery proof.

---

## 3. Problem Statement

Focusa already has strong primitives:

- ProjectIdentity;
- Continuity ID;
- Trajectory ladder;
- Workpoint;
- Evidence Ref;
- Context Authority;
- Context Cognition;
- Session Transfer;
- Agent Context Bootstrap from Spec111;
- Prediction and Metacognition;
- DX/UX surfaces;
- UIAI diagnostics intake;
- generated tool contracts;
- local-first daemon/API/CLI/Pi integration;
- event-chain persistence.

The current gap is product consolidation.

The system can preserve state, enforce scope, capture proof, guide recovery, deliver bootstrap context, and hash-link events, but users and agents still need one canonical artifact that answers:

```text
What was the work?
What was the scope?
Was the action allowed?
Was bootstrap context delivered?
What changed?
What proves it?
What is unfinished?
What is the next safe step?
Can this record be locally verified?
```

Without this artifact:

- internal vocabulary can feel like friction;
- tool count can feel like complexity;
- proof remains distributed across logs, tests, Workpoints, UIAI diagnostics, bootstrap receipts, and docs;
- public demos lack a consistent evidence object;
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
- bootstrap delivery status when relevant;
- changed objects;
- authority posture;
- evidence refs;
- test/browser/API proof;
- blocked or missing proof;
- final claim status;
- local verification status;
- next safe action.

### 4.2 Agent Benefit

An agent using Focusa should receive:

- one canonical continuation anchor;
- one current authority posture;
- one proof status;
- one bootstrap/delivery status when relevant;
- one next safe tool/action;
- up to three recovery tools when blocked.

The agent should not need to scan the full Focusa tool surface to determine what to do next.

### 4.3 Team Benefit

A team should be able to require:

```text
No risky agent mutation without Context Authority preflight and a Focusa Receipt.
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
- bootstrap file writes.

### 4.4 Public Demo Benefit

A public-safe receipt can become the standard object for Arena demos:

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
- bootstrap delivery status;
- authority decision;
- tool sequence;
- evidence class;
- failure mode;
- recovery path;
- final outcome.

Local-first storage remains the default. Any aggregation, sharing, export, or training use must be explicit, redacted, and opt-in.

---

## 5. Technical Advantages

### 5.1 Workpoint Becomes the Execution Anchor

The Workpoint remains the immediate continuation authority.

Receipts must reference a canonical Workpoint when available and must mark the receipt degraded or blocked when no exact-scoped Workpoint exists.

### 5.2 Context Authority Becomes the Mutation Boundary

Receipts must include Context Authority verdicts for risky operations.

A receipt must never claim a risky mutation was safely completed unless the relevant preflight verdict is present, fresh, and compatible with the action.

### 5.3 Evidence Becomes Structural

Receipts must classify evidence as:

```text
actual | partial | surrogate | blocked | missing
```

Partial, surrogate, or blocked evidence may be useful, but must not support a completed claim unless the acceptance criteria allow it.

### 5.4 UIAI Becomes Product-Reality Proof

UIAI diagnostics and browser reliability reports should become first-class receipt evidence.

Browser proof must distinguish:

- actual browser proof;
- blocked browser proof;
- private URL guard proof;
- missing native proof;
- surrogate API/web proof.

### 5.5 Spec111 Bootstrap Becomes Receipt-Producing Delivery Proof

Spec111 Agent Context Bootstrap should not maintain a separate durable receipt system.

Instead:

```text
AgentBootstrapReceipt = specialized projection of a Focusa Receipt.
Focusa Receipt = canonical durable record committed through Spec119.
```

Bootstrap build/render/write/verify outcomes should map into `receipt_type = bootstrap_delivery` when persisted.

### 5.6 Tool Count Becomes Tool Selection

The receipt layer must use tool contract/choreography metadata to recommend:

- top exact next tool;
- up to three next tools;
- up to three recovery tools;
- relevant family hints only.

### 5.7 Existing Event Integrity Becomes Work-Level Verification

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
- replacing Spec111 Agent Context Bootstrap;
- making Focusa a general task runner;
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
- evidence classification;
- authority posture and authority freshness;
- local receipt verification;
- receipt events linked into the existing event hash chain;
- bootstrap delivery receipt mapping;
- public-safe redaction path for post-MVP;
- adapter-friendly schema design;
- portable JSON Schemas in the repository.

---

## 7. Design Principles

### 7.1 Artifact Over Terminology

Canonical Focusa terms remain, but the product experience should lead with useful artifacts.

A new user should understand the receipt before needing to fully understand every internal term.

### 7.2 One Safe Next Action

Every receipt should include `next_safe_action`.

When Focusa cannot determine a safe next action, it should say why and return recovery tools.

### 7.3 Scope Before Certainty

A receipt cannot be canonical unless `project_root + continuity_id` are verified.

If scope is missing, unsafe, stale, or mismatched, the receipt must be degraded or blocked.

### 7.4 Proof Before Completion

A final claim cannot be marked complete unless matching evidence exists.

Completion language must be blocked when evidence is partial, surrogate, blocked, or missing.

### 7.5 Preview Before Commit

Receipt generation must support preview mode before writing to the durable ledger.

Preview may aggregate read models. Commit must enter the daemon-owned write path or serialized writer path.

### 7.6 Local-First by Default

Receipts are stored locally by default.

Public export, Arena export, team export, or external telemetry must be explicit and redacted.

### 7.7 Selection Over Surface Area

Receipts and awareness cards should compress Focusa’s tool graph into relevant choices.

The default surface should never overwhelm the user or agent with the full tool list.

### 7.8 Integration-Ready, Not Integration-Dependent

Focusa Receipts should be useful through CLI/API/Pi on day one and later portable through adapters.

### 7.9 Hash-Linked at Commit

A committed receipt must have a receipt hash and a canonical receipt event linked into the existing Focusa event hash chain.

### 7.10 Fresh Authority for Risky Actions

Risky action authorization expires.

A stale or expired allow verdict cannot support a committed risky-mutation receipt.

---

## 8. Definitions

### Focusa Receipt

A local-first, scope-bound artifact summarizing agent work, authority posture, evidence, result, bootstrap delivery when relevant, verification status, and next safe action.

### Agent Work Ledger

The durable local store of committed Focusa Receipts.

### Receipt Preview

A generated receipt candidate that does not mutate the durable ledger.

### Receipt Commit

A persisted receipt written after scope, authority, evidence, idempotency, and integrity checks.

### Public-Safe Receipt

A redacted receipt suitable for public demo, Arena display, client review, or investor proof.

### Claim

A statement that the agent, tool, or operator wants to treat as true about work performed.

### Claim Status

One of:

```text
actual | partial | surrogate | blocked | missing
```

### Evidence Ref

A stable reference to proof such as test output, command output, browser diagnostics, screenshot path, API response, CI run, release artifact, bootstrap verification result, or log bundle.

### Governed Execution Boundary

The point where Focusa reconciles current ask, project scope, Workpoint, environment facts, risky action class, authority freshness, and Context Authority verdict before allowing or blocking mutation.

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

---

## 9. Receipt Relationship to Final Reports and Bootstrap Receipts

A Focusa Receipt does not replace every final report.

Instead:

```text
Receipt = structured source of truth
Final report = human-readable rendering of receipt + operator-facing explanation
AgentBootstrapReceipt = target-specific delivery projection of a receipt
Arena card = public-safe rendering of receipt
Agent handoff = continuation-focused rendering of receipt
CI summary = automation-focused rendering of receipt
```

Agents must treat the receipt as the canonical structured artifact when reporting completion, blockers, bootstrap delivery, or next steps.

---

## 10. MVP Receipt Types

Add top-level `receipt_type`.

Allowed MVP values:

```text
work_session
risky_mutation
final_report
blocked_claim
handoff
bootstrap_delivery
```

Definitions:

- `work_session`: summarizes a bounded work interval.
- `risky_mutation`: records a mutation attempt requiring Context Authority.
- `final_report`: supports or blocks a completion claim.
- `blocked_claim`: records why a claim cannot be treated as complete.
- `handoff`: captures state for another agent/session/operator.
- `bootstrap_delivery`: records Spec111 bootstrap packet build/render/write/verify status.

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
  "receipt_type": "work_session|risky_mutation|final_report|blocked_claim|handoff|bootstrap_delivery",
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
    "continuity_id": "string",
    "session_id": "string|null",
    "workstream_key": "string|null",
    "scope_verified": true
  },
  "operator_ask": {
    "text": "string",
    "source": "operator|agent|system|unknown",
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
    "mission": "string|null",
    "next_slice": "string|null",
    "active_object_refs": [],
    "canonical": true,
    "posture": "canonical|advisory|degraded|blocked|stale"
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
  "execution": {
    "summary": "string",
    "primary_actions": [],
    "touched_refs": [],
    "side_effects": [],
    "event_refs": [],
    "workpoint_refs": [],
    "trajectory_refs": [],
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
- `touched_refs` should contain compact references to files, routes, services, or objects.
- Full command logs, tool call logs, browser logs, bootstrap packet JSON, and CI logs should remain in their original stores and be linked through refs.
- Receipts should be compact enough for agents to read and reliable enough for humans to audit.

---

## 14. Evidence Ref Shape

Receipt evidence refs must use a consistent shape:

```json
{
  "evidence_ref": "string",
  "class": "actual|partial|surrogate|blocked|missing",
  "source": "test|cli|api|browser|uiai|ci|screenshot|log|operator|agent|bootstrap|unknown",
  "summary": "string",
  "supports_claim": true,
  "workpoint_id": "uuid|null",
  "artifact_path": "string|null",
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
  "receipt_type": "work_session|risky_mutation|final_report|blocked_claim|handoff|bootstrap_delivery",
  "project_root": "string",
  "continuity_id": "string",
  "workpoint_id": "uuid|null",
  "claim_status": "actual|partial|surrogate|blocked|missing",
  "outcome_status": "completed|partial|blocked|failed|in_progress|unknown",
  "authority_verdict": "allow|block|ask_operator|verify_first|diagnosis_only|planning_only|null",
  "bootstrap_delivery_status": "not_applicable|rendered|written|verified|failed|dry_run|blocked",
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

This summary is the preferred integration object for external agent tools, editor plugins, CI systems, and future handoff adapters.

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
schemas/receipt/examples/bootstrap_delivery.verified.json
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
- internal-only tool names beyond optional metadata fields.

The portable schema may reference Focusa concepts, but the minimum integration summary must be usable with:

```text
receipt_id
receipt_type
project_root
continuity_id
workpoint_id
claim_status
authority_verdict
bootstrap_delivery_status
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
- inspect current Workpoint;
- inspect trajectory;
- inspect Context Authority if action is risky;
- evaluate authority freshness when applicable;
- inspect bootstrap packet/delivery state when `receipt_type=bootstrap_delivery`;
- collect evidence refs;
- classify claim status;
- recommend next safe action;
- return degraded/blocked if required fields are missing.

### 17.2 Commit

`POST /v1/receipts/commit` persists a receipt.

Commit must reject when:

- `project_root` is missing or unsafe;
- `continuity_id` is missing for canonical receipt;
- Workpoint scope mismatches current ask;
- risky mutation lacks required Context Authority verdict;
- risky mutation has expired authority;
- bootstrap write evidence is claimed verified but verifier failed or was skipped;
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
- private evidence refs that cannot be shared.

### 17.6 Export

`POST /v1/receipts/{receipt_id}/export` returns one or more formats:

```text
json
markdown
arena_card
agent_handoff
ci_summary
bootstrap_projection
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
focusa receipt export <receipt_id> --format json|markdown|arena-card|agent-handoff|ci-summary|bootstrap-projection
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
focusa receipt export <receipt_id> --format json|markdown|arena-card|agent-handoff|ci-summary|bootstrap-projection
```

CLI requirements:

- `preview` must be safe by default.
- `commit` must require explicit confirmation or `--yes` when claim status is not `actual`.
- `commit` must block when `completion_allowed=false`.
- `show` must render compact human-readable summary by default.
- `verify` must show receipt hash and event-chain status.
- `--json` must return full schema.
- `export --format arena-card` must require redaction unless `--private` is explicitly supplied.

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
- If receipt preview blocks completion, the agent must report the missing evidence plainly.
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
claim_status
completion_allowed
bootstrap_delivery_status
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
claim_status
outcome_status
bootstrap_delivery_status
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
2. API validates request shape, scope, evidence class, authority freshness, bootstrap verifier state, and idempotency.
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
  "receipt_type": "work_session|risky_mutation|final_report|blocked_claim|handoff|bootstrap_delivery",
  "project_root": "string",
  "continuity_id": "string",
  "workpoint_id": "uuid|null",
  "claim_status": "actual|partial|surrogate|blocked|missing",
  "completion_allowed": false,
  "bootstrap_delivery_status": "not_applicable|rendered|written|verified|failed|dry_run|blocked",
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
authority
bootstrap
execution
evidence
claim
outcome
next_safe_action
privacy
```

---

## 25. Governed Execution Boundary

Receipt generation must integrate with Context Authority.

For risky operations, a receipt must include:

- action kind;
- target;
- current ask;
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
- preload/bootstrap file write.

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

## 27. Spec111 Bootstrap Integration

Spec111 Agent Context Bootstrap & Delivery should become a receipt-producing subsystem.

### 27.1 Relationship

```text
Spec111 Bootstrap builds/delivers/verifies startup context.
Spec119 Receipts record that delivery as verifiable work.
```

`AgentBootstrapReceipt` becomes a compact target-specific projection of `focusa.receipt.v1`, not a separate durable receipt system.

### 27.2 Required Mapping

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

### 27.3 Claim Status Rules

Bootstrap delivery receipts must classify claim status as:

```text
actual    = bootstrap packet was written/delivered and verified
partial   = bootstrap packet was rendered but not written or not verified
blocked   = bootstrap verification failed or required fields are missing
missing   = bootstrap packet is absent when required
surrogate = delivery proof comes from a different target/surface than required
```

### 27.4 Authority Rules

- `preload build` and `preload render` are read/compose by default and do not require mutation authority.
- `preload verify` is read-only and does not require mutation authority.
- `preload write` writes project files and therefore must be treated as a risky operation.
- `preload write` receipts must include Context Authority when the target writes files into a project root.
- Pi session-start bootstrap remains special: no file write by default; delivery happens through Pi session lifecycle and follow-up message/tool context.

### 27.5 Doc111 Update Required After Spec119 MVP

After Spec119 MVP is implemented, update `docs/111-agent-context-bootstrap-and-delivery-spec.md`:

- add Spec119 to the normative basis;
- state that `AgentBootstrapReceipt` is a specialized projection of `focusa.receipt.v1`;
- update preload write/verify routes to mention receipt preview/commit;
- add `focusa_receipt_preview` and `focusa_receipt_commit` as likely next tools after `focusa_preload_verify`;
- add receipt ledger consistency to preload doctor;
- update tests to assert bootstrap delivery receipts can be generated.

---

## 28. Claim Gate Integration

Spec107 claim discipline must become a hard pre-close path.

Rule:

```text
No final completion claim may be emitted by Focusa tooling when receipt preview returns completion_allowed=false.
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

## 29. Blocked and Degraded Agent View

When a receipt is degraded or blocked, the agent-facing response must include:

```text
status
posture
why_not_canonical
missing_scope
missing_evidence
authority_needed
bootstrap_status
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
  "missing_evidence": [],
  "authority_needed": null,
  "bootstrap_status": "not_applicable",
  "next_safe_action": {
    "summary": "Verify project identity and checkpoint a new Workpoint in the current project.",
    "tool": "focusa_project_identity",
    "requires_operator": false
  },
  "recovery_tools": [
    "focusa_project_identity",
    "focusa_workpoint_checkpoint",
    "focusa_receipt_preview"
  ]
}
```

---

## 30. UIAI Proof Bridge Requirements

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
- final report.

---

## 31. Integration Strategy

Focusa should not require every external system to adopt Focusa internals.

Instead, Focusa should provide portable receipt/workpoint/evidence/bootstrap payloads that can be consumed by:

- agent harnesses;
- CLI-based coding agents;
- editor extensions;
- CI/CD workflows;
- browser/product proof systems;
- public demo surfaces;
- future agent-to-agent handoff adapters.

Integration payloads should center on:

```text
receipt_id
project_root
continuity_id
workpoint_id
claim_status
bootstrap_delivery_status
evidence_refs
next_safe_action
```

Adapters should not expose the entire Focusa tool surface by default.

---

## 32. Public-Safe / Arena Card Requirements

Public-safe export is post-MVP.

A public-safe receipt should render as a compact card:

```text
Project: <public project name>
Work: <summary>
Scope: verified/degraded/blocked
Bootstrap: verified/not applicable/blocked
Authority: allowed/blocked/verify-first
Evidence: 3 actual, 1 blocked, 0 missing
Claim: actual/partial/blocked
Verification: hash-linked/unverified/broken
Next: <safe next action>
```

Arena export must never include private data unless explicitly marked public-safe.

Arena cards should be searchable by:

- project;
- task type;
- evidence class;
- outcome status;
- bootstrap delivery status;
- blocked reason;
- tool family;
- date;
- verification status.

---

## 33. Agent DX Requirements

Receipt surfaces must reduce agent confusion.

Every receipt response must include:

- `status`;
- `canonical`;
- `posture`;
- `claim.status`;
- `completion_allowed`;
- `bootstrap.delivery_status` when relevant;
- `next_safe_action`;
- `recovery_tools`;
- `evidence.counts`;
- `missing_evidence`;
- `verification` when committed.

Error states must be recoverable.

If a receipt cannot be generated, Focusa should return:

- why;
- what scope is missing;
- which evidence is missing;
- which authority verdict is needed;
- whether authority is expired;
- which bootstrap proof is missing when relevant;
- which tool to call next.

---

## 34. AwarenessPacket Integration

The shared AwarenessPacket substrate should use receipt state.

Input additions:

```text
latest_receipt
open_receipt_preview
claim_status
missing_evidence
authority_freshness
bootstrap_delivery_status
receipt_verification_status
public_export_state
```

Output additions:

```text
receipt_line
claim_status_line
proof_line
authority_freshness_line
bootstrap_line
verification_line
next_safe_action_line
```

Minimal awareness card should include receipt state only when useful:

- before final report;
- after risky mutation;
- after evidence capture;
- after UIAI diagnostics;
- after bootstrap write/verify;
- after blocked claim;
- after compaction/session transfer;
- when operator asks for status;
- when receipt verification fails.

---

## 35. Implementation Phases

### Phase 0 — Field Map and Fixtures

Deliverables:

```text
docs/current/FOCUSA_RECEIPT_FIELD_MAP.md
receipt schema fixture
example degraded receipt
example blocked claim receipt
example actual proof receipt
example bootstrap delivery receipt
```

Acceptance:

- Existing Focusa surfaces are mapped to receipt fields.
- Spec111 bootstrap fields are mapped to receipt fields.
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
- Aggregates project identity, continuity, Workpoint, trajectory, authority posture, authority freshness, bootstrap status, evidence summary, claim status, and next safe action.
- Blocks or degrades on scope mismatch.
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

### Phase 2.5 — Basic UIAI and Bootstrap Evidence Classification

Deliverables:

```text
UIAI diagnostics mapped into receipt evidence classes
browser proof shown in receipt preview
blocked browser proof shown as blocked evidence
Spec111 bootstrap verify result mapped into bootstrap_delivery receipts
```

Acceptance:

- Actual browser proof can support claims.
- Blocked browser proof cannot support completion.
- Surrogate proof is labeled as surrogate.
- Missing native/browser proof is labeled missing when required.
- Verified bootstrap delivery can support `bootstrap_delivery` actual claim status.
- Failed bootstrap verification blocks bootstrap delivery completion.

### Post-MVP Phase 3 — Public-Safe Export

Deferred until preview, commit, and verification are proven.

### Post-MVP Phase 4 — Arena Card

Deferred until public-safe export is proven.

### Post-MVP Phase 5 — External Adapter Payloads

Deferred until summary schema stabilizes.

### Post-MVP Phase 6 — External Checkpointing / Signing

Deferred until receipt UX and event-chain verification are proven.

---

## 36. MVP Acceptance Criteria

MVP is accepted when:

1. `docs/current/FOCUSA_RECEIPT_FIELD_MAP.md` exists.
2. Canonical example receipts exist.
3. JSON Schemas exist under `schemas/receipt/`.
4. Example receipts validate against schema.
5. Receipt preview works through API, CLI, and Pi.
6. Receipt preview is read-only.
7. Receipt preview returns `receipt_type` and `schema_version`.
8. Receipt preview uses existing Workpoint as the execution anchor.
9. Receipt preview degrades or blocks when `project_root + continuity_id` are missing or mismatched.
10. Receipt preview classifies claim status as actual/partial/surrogate/blocked/missing.
11. Receipt preview supports `bootstrap_delivery` receipt type.
12. Receipt preview returns one next safe action and up to three recovery tools.
13. Receipt commit persists locally through the Focusa write model.
14. Receipt commit emits receipt events.
15. Receipt commits produce `receipt_hash`.
16. Receipt commits create or link to a canonical receipt event.
17. Receipt events participate in the existing hash chain.
18. `focusa receipt verify <receipt_id>` verifies receipt hash and event-chain linkage.
19. Completion claims are blocked when `completion_allowed=false`.
20. Risky mutation receipts require Context Authority evidence.
21. Risky `allow` authority includes `issued_at`, `valid_until`, and `ttl_seconds`.
22. Expired authority blocks receipt commit.
23. Critical-risk actions require recheck at commit.
24. Basic UIAI diagnostics can appear as receipt evidence.
25. Basic Spec111 bootstrap verification can appear as receipt evidence.
26. The minimal receipt summary schema works without importing Focusa daemon internals.

---

## 37. Required MVP Tests

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
tests/spec119_receipt_summary_schema_static_test.sh
```

Post-MVP tests:

```text
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
2. Mac menubar API/web-only proof must classify as surrogate when native proof is required.
3. Risky deploy without Context Authority preflight must block completion.
4. Risky mutation with expired authority must block commit.
5. UIAI browser failure must classify as blocked evidence, not success.
6. Receipt query model mismatch with event chain must return degraded/broken verification.
7. Spec111 bootstrap verification failure must classify `bootstrap_delivery` as blocked.
8. Public export must remove private URLs and local absolute paths unless explicitly allowed.

---

## 38. Documentation Updates

Required docs:

```text
docs/current/FOCUSA_RECEIPT_CURRENT.md
docs/current/FOCUSA_RECEIPT_FIELD_MAP.md
docs/current/FOCUSA_RECEIPT_INTEGRITY.md
docs/current/FOCUSA_RECEIPT_SCHEMA_PACKAGE.md
docs/current/FOCUSA_RECEIPT_AUTHORITY_FRESHNESS.md
docs/current/FOCUSA_RECEIPT_BOOTSTRAP_INTEGRATION.md
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
- `docs/current/GOLDEN_WORKFLOW.md`;
- `docs/current/AUTHORITY_MODEL.md`;
- `docs/current/CONTEXT_AUTHORITY_CURRENT.md`;
- `docs/current/TAMPER_EVIDENT_EVENT_CHAIN.md`;
- `docs/107-spec-first-feature-lifecycle-and-claim-discipline-spec.md`;
- `docs/109-agent-first-api-redesign-ax-spec.md`;
- `docs/111-agent-context-bootstrap-and-delivery-spec.md`;
- generated tool surface summary;
- release docs;
- marketing copy.

Required documentation statement:

```text
Focusa Receipts are local-first, hash-linked to the Focusa event chain at commit, and verifiable through local CLI/API commands. Hash-chain verification detects ordinary database edits or deletions, but does not replace external signing, backups, access controls, or future out-of-band checkpoint publication.
```

---

## 39. Required Future Update to Spec111

After Spec119 MVP lands, update `docs/111-agent-context-bootstrap-and-delivery-spec.md` surgically:

1. Add Spec119 to normative basis.
2. Replace standalone durable interpretation of `AgentBootstrapReceipt` with:

```text
AgentBootstrapReceipt is a target-specific projection of a canonical Focusa Receipt with receipt_type=bootstrap_delivery.
```

3. Add receipt preview/commit to preload write and verify flows.
4. Add `focusa_receipt_preview` and `focusa_receipt_commit` as likely next tools after successful `focusa_preload_verify`.
5. Add receipt ledger consistency to preload doctor.
6. Add tests proving bootstrap delivery receipts can be generated.

This update is intentionally deferred so Spec119 can be added without rewriting Doc111 in the same change.

---

## 40. Success Criteria

This work is successful when Focusa can reliably answer:

```text
What did the agent do?
Was it allowed?
Was the authority fresh?
Was bootstrap context delivered?
What proves it?
What remains?
Can the record be locally verified?
What should happen next?
```

The default experience should become:

```text
One resumable mission.
One governed action path.
One bootstrap delivery proof when relevant.
One proof trail.
One local receipt.
One verification status.
One next safe action.
```

---

## 41. Closure Policy

Do not close Spec119 MVP implementation work until:

- receipt field map exists;
- portable JSON Schemas exist;
- schema examples validate;
- receipt preview exists across API/CLI/Pi;
- receipt commit persists locally;
- receipt commit creates a canonical event;
- receipt commit links into existing `event_hash_chain`;
- receipt verification works through CLI/API;
- claim gate blocks unsupported completion;
- risky mutation receipts include Context Authority;
- authority freshness is enforced;
- expired authority blocks commit;
- UIAI diagnostics can become receipt evidence;
- Spec111 bootstrap verification can become receipt evidence;
- tests prove scope mismatch and surrogate evidence behavior;
- tests prove receipt hash and event-chain linkage;
- docs explain the receipt workflow in beginner and advanced modes.

Partial receipt surfaces may ship behind preview labels, but public docs must not claim the receipt system is complete until all MVP acceptance criteria are met.

Public-safe export, Arena cards, external schema packages, out-of-band checkpoints, signing, and Doc111 surgical updates remain post-MVP unless explicitly accepted by operator steering.
