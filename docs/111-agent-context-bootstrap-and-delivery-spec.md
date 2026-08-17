# docs/111-agent-context-bootstrap-and-delivery-spec.md — Focusa Agent Context Bootstrap & Delivery

Status: proposed / implementation-ready  
Scope: Focusa core, API, CLI, Pi extension, tool contracts, docs, static audits, and Spec119 receipt integration  
Authority: read/compose by default; explicit file writes only through `write` endpoints/CLI; no new cognition authority

---

## 0. Normative basis

This spec is a systemic follow-on to:

- Spec 88 — Workpoint continuity
- Spec 96 — ProjectIdentity / Trajectory / daemon stability
- Spec 98/99 — project_root + continuity_id authority correction
- Spec 100 — Context Cognition
- Spec 101 — Bloatgaurd / token budget posture
- Spec 105 — Agent DX/UX
- Spec 106 — Vision tightening
- Spec 107 — spec-first lifecycle and claim discipline
- Spec 108 — Awareness substrate / utility-card surfaces
- Spec 109 — Agent-First API / AX; claimed downstream work; do not reuse number
- Spec 110 — Pi tool-layer reminder behavior; claimed downstream work; do not reuse number
- Spec 119 — Verifiable Agent Work Receipts and Governed Execution Ledger

Focusa already has the required primitives:

- ProjectIdentity resolves project scope.
- WorkpointResumePacket carries immediate action authority.
- Trajectory supplies HLT/MLG/STG/gap context.
- Context Cognition selects bounded advisory context.
- Utility Card supplies startup/post-compaction guidance.
- Awareness renders surface-specific guidance.
- Session Transfer composes save/continue semantics.
- Pi session_start already performs a Pi-specific bootstrap.
- Spec119 defines the canonical durable Focusa Receipt ledger for proving delivery, closure, evidence, and completion state.

The gap is that these surfaces are not unified into one portable agent-context delivery system for Cursor, Claude Code, Codex, OpenCode/OpenClaw, Pi, and future adapters.

Any mismatch between this spec and current implementation is an implementation gap, not a reason to weaken this spec.

---

## 1. Purpose

Spec 111 adds a first-class Focusa **Agent Context Bootstrap & Delivery** layer.

The layer builds, renders, writes, and verifies bounded agent startup packets from existing Focusa authority surfaces so a new agent session can continue verified project work without relying on transcript tail, stale chat memory, or tool-specific prompt hacks.

Short form:

> Focusa Bootstrap turns a cold agent session into a verified mission continuation.

Spec119 integration clarifies that bootstrap delivery/verification receipts are not a separate durable system. A bootstrap delivery result becomes durable only when recorded as a Focusa Receipt with:

```text
receipt_type = bootstrap_delivery
```

---

## 2. Core thesis

Agent sessions fail in predictable ways:

- the agent starts in the wrong project,
- the agent treats transcript tail as authority,
- the agent forgets Workpoint/Trajectory context,
- proof is buried in prior chat,
- Cursor/Claude/Codex/Pi each need context in a different shape,
- handoffs depend on the operator remembering what to paste,
- bootstrap delivery can be claimed even when the target agent never received or verified the packet.

Focusa should not solve this by becoming a Cursor rules generator.

Focusa should solve this by becoming the **source-of-truth context compiler and delivery verifier**:

```text
ProjectIdentity
  → WorkpointResumePacket
  → TrajectoryView
  → ContextCognitionPacket
  → UtilityCard
  → AwarenessPacket
  → AgentBootstrapPacket
  → target-specific delivery artifacts
  → AgentBootstrapReceipt projection
  → Focusa Receipt (Spec119 canonical durable ledger)
```

The product promise is not “write better prompts.”

The product promise is:

> Before an AI coding agent acts, Focusa can prove which project, mission, Workpoint, next action, evidence refs, and drift boundaries it was given.

After Spec119, the durable proof object for that promise is the Focusa Receipt.

---

## 3. Non-goals

Spec 111 is not:

- a replacement for Workpoint,
- a replacement for Context Cognition,
- a replacement for Session Transfer,
- a replacement for Spec119 Receipts,
- a Cursor-only integration,
- a vector database,
- a prompt-stuffing mechanism,
- a new task scheduler,
- a new source of canonical truth,
- a hidden auto-mutation system,
- a route that edits arbitrary project files without explicit request,
- a guarantee that an external agent obeyed the packet after delivery,
- a standalone durable audit ledger separate from the Focusa Receipt ledger.

---

## 4. Authority boundaries

Canonical authority remains unchanged.

| Area | Authority |
|---|---|
| Project scope | ProjectIdentity |
| Immediate next action | Workpoint |
| Goal / gap context | Trajectory |
| Semantic context | Ontology / Context Cognition |
| Proof | Evidence refs |
| Operator steering | current operator ask |
| Durable cognition mutation | reducer-backed Focusa events |
| Delivery artifact generation | Spec 111 preload writer |
| Delivery verification | Spec 111 preload verifier |
| Durable delivery proof | Spec119 Focusa Receipt with `receipt_type=bootstrap_delivery` |

Spec 111 outputs are delivery artifacts, not cognition authority.

Generated files are prompt surfaces. They do not become canonical truth.

If a generated file conflicts with live Focusa state, live Focusa state wins.

Spec 111 may produce:

- target-specific context files,
- target-specific static rule files,
- prompt snippets,
- verification prompts,
- AgentBootstrapReceipt projections,
- diagnostic cards,
- Focusa Receipt previews or commits for bootstrap delivery when routed through Spec119.

Spec 111 must not:

- promote Workpoints,
- define or supersede Trajectories,
- capture Evidence directly,
- mutate FocusState,
- alter reducer-backed cognition,
- close beads,
- install hooks without explicit request,
- claim delivery completion without verifier evidence,
- bypass Spec119 when durable receipt persistence is required.

---

## 5. New vocabulary

### 5.1 AgentBootstrapPacket

A bounded packet generated from existing Focusa surfaces for one target agent surface.

Schema:

```yaml
AgentBootstrapPacket:
  schema_version: focusa.agent_bootstrap_packet.v1
  packet_id:
  generated_at:
  target:
  mode:
  status: completed | degraded | blocked
  canonical: false
  advisory: true
  project_root:
  continuity_id:
  workpoint_id:
  trajectory_id:
  project_identity:
  authority:
    project_scope: project_identity
    action_authority: workpoint
    goal_context: trajectory
    context_selection: context_cognition
    proof_boundary: evidence_refs
    mutation_allowed: false
  mission:
  exact_next_action:
  active_object_refs: []
  blockers: []
  do_not_drift: []
  evidence_refs: []
  proof_gaps: []
  selected_context:
    include: []
    exclude: []
    over_budget: []
  utility_guidance:
    startup: []
    post_compaction: []
    recovery: []
  awareness:
    visible_lines: []
    recovery_tools: []
  delivery:
    target:
    profile:
    files: []
    hooks: []
    prompt_snippet:
    fail_phrase: FOCUSA_PRELOAD_FAIL
  verification:
    required_fields: []
    acceptance_prompt:
    verifier_status: pending | passed | failed | skipped
  freshness:
    source_snapshot:
    rendered_at:
    stale: false
  side_effects: []
  next_tools: []
```

### 5.2 AgentBootstrapProfile

A target-specific delivery recipe.

Supported initial targets:

```text
cursor
claude
codex
pi
opencode
generic
```

Each profile declares:

```yaml
AgentBootstrapProfile:
  target:
  static_rule_paths: []
  dynamic_context_paths: []
  json_packet_paths: []
  hook_paths: []
  supports_hooks: bool
  supports_prompt_file: bool
  supports_followup_message: bool
  max_static_lines:
  max_dynamic_lines:
  default_mode:
  fail_phrase: FOCUSA_PRELOAD_FAIL
```

### 5.3 AgentBootstrapReceipt

A target-specific delivery/verification projection.

After Spec119, this projection is not the canonical durable record. The canonical durable record is a Focusa Receipt with `receipt_type=bootstrap_delivery`.

```yaml
AgentBootstrapReceipt:
  schema_version: focusa.agent_bootstrap_receipt.v1
  receipt_id:
  spec119_receipt_id:
  packet_id:
  target:
  project_root:
  continuity_id:
  status: written | verified | failed | dry_run | projected
  files_written: []
  files_skipped: []
  verifier:
    status:
    missing_fields: []
    failed_checks: []
    acceptance_prompt:
  generated_at:
  side_effects: []
  focusa_receipt:
    receipt_type: bootstrap_delivery
    claim_status: actual | partial | surrogate | blocked | missing
    completion_allowed: bool
    receipt_hash:
    receipt_event_id:
    event_chain_hash:
```

### 5.4 FOCUSA_PRELOAD_FAIL

A required fail-closed phrase used by external agents.

If the target session did not receive or cannot prove it received the required Focusa bootstrap context, it must respond:

```text
FOCUSA_PRELOAD_FAIL
```

It must not guess from transcript tail.

### 5.5 bootstrap_delivery Focusa Receipt

A Spec119 receipt type recording bootstrap delivery, verification, or failure.

Minimum mapping:

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

---

## 6. Required systemic change

Spec 111 extracts Pi’s session-start bootstrap pattern into a portable Focusa bootstrap layer.

Current Pi behavior remains valid, but Pi becomes one adapter among several.

The new architecture:

```text
crates/focusa-core/src/preload.rs
  pure builders, renderers, profiles, verification logic

crates/focusa-api/src/routes/preload.rs
  HTTP facade over build/render/write/verify/doctor

crates/focusa-cli/src/commands/preload.rs
  CLI facade over API routes

apps/pi-extension/src/session.ts
  keeps Pi session_start lifecycle
  calls/aligns with preload build where possible
  does not duplicate target-independent policy

apps/pi-extension/src/tool-contracts.ts
  registers preload tools/contracts/choreography

docs/focusa-tools/tools/
  documents focusa_preload_build/write/verify/doctor

Spec119 receipts
  records durable bootstrap delivery proof when requested or required

tests/
  adds Spec111 static and route/CLI/tool-contract audits
  adds Spec111 ↔ Spec119 receipt integration audits
```

---

## 7. Core module design

Add:

```text
crates/focusa-core/src/preload.rs
```

Export:

```rust
pub enum AgentBootstrapTarget {
    Cursor,
    Claude,
    Codex,
    Pi,
    OpenCode,
    Generic,
}

pub enum AgentBootstrapMode {
    SessionStart,
    PostCompaction,
    SessionTransfer,
    Recovery,
    ToolGuidance,
}

pub struct AgentBootstrapProfile { /* target delivery recipe */ }
pub struct AgentBootstrapInput { /* normalized Focusa surface inputs */ }
pub struct AgentBootstrapPacket { /* schema above */ }
pub struct AgentBootstrapReceipt { /* target-specific projection */ }
pub struct AgentBootstrapRenderedFiles { /* path + kind + content */ }
pub struct AgentBootstrapVerification { /* pass/fail + fields */ }
pub struct AgentBootstrapReceiptProjection { /* Spec119 bootstrap_delivery mapping */ }

pub fn profile_for_target(target: AgentBootstrapTarget) -> AgentBootstrapProfile;

pub fn build_agent_bootstrap_packet(input: AgentBootstrapInput) -> AgentBootstrapPacket;

pub fn render_agent_bootstrap_markdown(packet: &AgentBootstrapPacket) -> String;

pub fn render_agent_bootstrap_json(packet: &AgentBootstrapPacket) -> serde_json::Value;

pub fn render_target_files(packet: &AgentBootstrapPacket, profile: &AgentBootstrapProfile) -> AgentBootstrapRenderedFiles;

pub fn verify_agent_bootstrap_packet(packet: &AgentBootstrapPacket) -> AgentBootstrapVerification;

pub fn project_bootstrap_receipt(packet: &AgentBootstrapPacket, receipt: &AgentBootstrapReceipt) -> AgentBootstrapReceiptProjection;
```

Core rules:

- no filesystem writes in `focusa-core`,
- no HTTP calls in `focusa-core`,
- no UI logic in `focusa-core`,
- no reducer mutation,
- deterministic rendering for identical input,
- every output includes `schema_version`,
- every target profile includes `fail_phrase = "FOCUSA_PRELOAD_FAIL"`,
- Spec119 receipt projection logic may build serializable payloads but must not persist them directly from core.

---

## 8. API routes

Add:

```text
crates/focusa-api/src/routes/preload.rs
```

Wire into:

```text
crates/focusa-api/src/routes/mod.rs
crates/focusa-api/src/server.rs
```

Routes:

```text
GET  /v1/preload/profiles
POST /v1/preload/build
POST /v1/preload/render
POST /v1/preload/write
POST /v1/preload/verify
POST /v1/preload/doctor
POST /v1/preload/receipt-preview
POST /v1/preload/receipt-commit
```

### 8.1 GET /v1/preload/profiles

Purpose:

Return supported targets and their default delivery profiles.

Response:

```json
{
  "status": "completed",
  "schema": "focusa.agent_bootstrap_profiles.v1",
  "targets": ["cursor", "claude", "codex", "pi", "opencode", "generic"],
  "profiles": []
}
```

### 8.2 POST /v1/preload/build

Purpose:

Build an `AgentBootstrapPacket` without writing files.

Request:

```json
{
  "target": "cursor|claude|codex|pi|opencode|generic",
  "mode": "session_start|post_compaction|session_transfer|recovery|tool_guidance",
  "project_root": "/path/to/project",
  "continuity_id": "focusa-cont-...",
  "current_ask": "optional",
  "session_id": "optional",
  "include_context_cognition": true,
  "include_utility_card": true,
  "include_awareness": true
}
```

Composition order:

1. `project_identity_payload_for_scope`
2. Workpoint lookup/resume-equivalent packet projection
3. Trajectory view projection
4. Context Cognition packet/render where scoped
5. Utility Card
6. Awareness packet for selected mode/surface
7. AgentBootstrapPacket renderer

Response:

```json
{
  "status": "completed|degraded|blocked",
  "canonical": false,
  "advisory": true,
  "packet": {},
  "required_fields": [],
  "missing_fields": [],
  "next_tools": ["focusa_preload_render", "focusa_preload_write", "focusa_preload_verify", "focusa_receipt_preview"]
}
```

Hard blocks:

- missing project_root,
- unsafe project_root,
- missing continuity_id for canonical delivery,
- ProjectIdentity status `unsafe_project_root`,
- Workpoint unavailable when mode requires action authority and no degraded fallback is allowed.

Degraded but allowed:

- no Workpoint but SessionTransfer has inferred candidate,
- no Trajectory but Workpoint has exact next action,
- Context Cognition unavailable but Utility Card and Workpoint are available,
- evidence refs missing but proof gap is declared.

### 8.3 POST /v1/preload/render

Purpose:

Render a packet into target-specific content without writing files.

Response includes:

```json
{
  "status": "completed",
  "target": "cursor",
  "rendered": {
    "static_rules_markdown": "...",
    "dynamic_context_markdown": "...",
    "packet_json": {},
    "acceptance_prompt": "..."
  },
  "files": [
    {"path": ".cursor/rules/focusa-project.mdc", "kind": "static_rule"},
    {"path": ".project/session-context.md", "kind": "dynamic_context"},
    {"path": ".focusa/preload/session-context.json", "kind": "packet_json"}
  ],
  "next_tools": ["focusa_preload_verify", "focusa_receipt_preview"]
}
```

### 8.4 POST /v1/preload/write

Purpose:

Write generated target artifacts into the verified project root.

Request:

```json
{
  "target": "cursor",
  "mode": "session_start",
  "project_root": "/path/to/project",
  "continuity_id": "focusa-cont-...",
  "dry_run": false,
  "overwrite": true,
  "write_static_rules": true,
  "write_dynamic_context": true,
  "write_json_packet": true,
  "write_hooks": false,
  "receipt_preview": true,
  "receipt_commit": false
}
```

Rules:

- `project_root` must be ProjectIdentity-safe.
- Writes may only occur under `project_root`.
- Paths containing `..` are rejected.
- Symlinks are not followed for write targets.
- Default write paths are target profile paths.
- `dry_run=true` returns planned writes without side effects.
- Existing files may only be overwritten when `overwrite=true`.
- Every written file includes a generated header:
  `Generated by Focusa Spec111. Do not edit; source of truth is Focusa live state.`
- File writes are operational side effects, not Focusa cognition mutations.
- File writes are risky enough to be receipt-visible.
- If `receipt_preview=true`, response includes a Spec119 `bootstrap_delivery` receipt preview.
- If `receipt_commit=true`, write must pass receipt/authority requirements before durable receipt commit.

Default target files:

```yaml
cursor:
  static_rule: .cursor/rules/focusa-project.mdc
  dynamic_context: .project/session-context.md
  packet_json: .focusa/preload/session-context.json

claude:
  static_rule: CLAUDE.md.focusa.generated
  dynamic_context: .focusa/preload/session-context.md
  packet_json: .focusa/preload/session-context.json

codex:
  static_rule: AGENTS.md.focusa.generated
  dynamic_context: .focusa/preload/session-context.md
  packet_json: .focusa/preload/session-context.json

opencode:
  static_rule: .opencode/focusa.md
  dynamic_context: .focusa/preload/session-context.md
  packet_json: .focusa/preload/session-context.json

generic:
  static_rule: .focusa/preload/agent-rules.md
  dynamic_context: .focusa/preload/session-context.md
  packet_json: .focusa/preload/session-context.json

pi:
  no file write by default
  delivery uses Pi session lifecycle and follow-up message/tool context
```

### 8.5 POST /v1/preload/verify

Purpose:

Verify a packet or written files are sufficient for a target agent to start without re-teaching.

Checks:

```text
project_identity_present
project_root_present
project_root_safe
continuity_id_present
workpoint_or_degraded_fallback_present
mission_present
exact_next_action_present
authority_boundary_present
do_not_drift_present
evidence_refs_or_proof_gap_present
target_profile_present
acceptance_prompt_present
fail_phrase_present
spec119_receipt_projection_available
```

Response:

```json
{
  "status": "passed|failed",
  "canonical": false,
  "advisory": true,
  "verifier": {
    "required_checks": [],
    "passed": [],
    "failed": [],
    "missing_fields": [],
    "fail_phrase": "FOCUSA_PRELOAD_FAIL"
  },
  "acceptance_prompt": "...",
  "receipt_preview": {},
  "next_tools": ["focusa_receipt_preview", "focusa_preload_doctor"]
}
```

### 8.6 POST /v1/preload/doctor

Purpose:

Diagnose delivery, not content.

Doctor checks:

```text
profile_known
project_root_safe
project_identity_verified_or_degraded_known
workpoint_resume_available
context_cognition_available
utility_card_available
awareness_available
render_paths_valid
write_paths_under_project_root
written_files_exist
written_files_include_packet_id
acceptance_prompt_present
receipt_consistent
spec119_receipt_preview_available
spec119_receipt_commit_available
receipt_ledger_consistent
```

Doctor must never say “content is wrong” when the problem is delivery.

Failure classes:

```text
project_root_missing
scope_mismatch
continuity_id_missing
workpoint_missing
context_surface_unavailable
target_profile_unknown
render_failed
write_rejected
write_failed
verify_failed
receipt_mismatch
receipt_preview_failed
receipt_commit_failed
receipt_ledger_degraded
```

### 8.7 POST /v1/preload/receipt-preview

Purpose:

Convenience route that builds the Spec119 `bootstrap_delivery` receipt preview for the current preload state.

Equivalent to calling `/v1/receipts/preview` with:

```json
{
  "receipt_type": "bootstrap_delivery"
}
```

### 8.8 POST /v1/preload/receipt-commit

Purpose:

Convenience route that commits the Spec119 `bootstrap_delivery` receipt for verified preload delivery.

Equivalent to calling `/v1/receipts/commit` with:

```json
{
  "receipt_type": "bootstrap_delivery"
}
```

Commit must reject when:

- preload verification failed,
- required fields are missing,
- file write side effects lack required authority,
- receipt preview says `completion_allowed=false`,
- receipt integrity requirements cannot be met.

---

## 9. CLI

Add:

```text
crates/focusa-cli/src/commands/preload.rs
```

Wire into:

```text
crates/focusa-cli/src/commands/mod.rs
crates/focusa-cli/src/main.rs
```

Commands:

```bash
focusa preload profiles

focusa preload build \
  --target cursor \
  --project-root "$PWD" \
  --continuity-id "$FOCUSA_CONTINUITY_ID" \
  --mode session-start \
  --json

focusa preload render \
  --target cursor \
  --project-root "$PWD" \
  --continuity-id "$FOCUSA_CONTINUITY_ID"

focusa preload write \
  --target cursor \
  --project-root "$PWD" \
  --continuity-id "$FOCUSA_CONTINUITY_ID" \
  --overwrite

focusa preload verify \
  --target cursor \
  --project-root "$PWD" \
  --continuity-id "$FOCUSA_CONTINUITY_ID"

focusa preload doctor \
  --target cursor \
  --project-root "$PWD" \
  --continuity-id "$FOCUSA_CONTINUITY_ID"

focusa preload receipt-preview \
  --target cursor \
  --project-root "$PWD" \
  --continuity-id "$FOCUSA_CONTINUITY_ID"

focusa preload receipt-commit \
  --target cursor \
  --project-root "$PWD" \
  --continuity-id "$FOCUSA_CONTINUITY_ID"
```

Human output must be compact:

```text
FOCUSA PRELOAD
target: cursor
status: completed
project: /path/to/project
continuity: focusa-cont-...
workpoint: ...
next: ...
files:
- .cursor/rules/focusa-project.mdc
- .project/session-context.md
verify: focusa preload verify --target cursor --project-root ... --continuity-id ...
receipt: focusa preload receipt-preview --target cursor --project-root ... --continuity-id ...
```

JSON output returns full envelopes.

---

## 10. Pi extension integration

Pi currently has its own session_start bootstrap.

Spec 111 must not break it.

Required changes:

1. Keep existing Pi lifecycle behavior.
2. Add Pi tool wrappers:

```text
focusa_preload_profiles
focusa_preload_build
focusa_preload_render
focusa_preload_write
focusa_preload_verify
focusa_preload_doctor
focusa_preload_receipt_preview
focusa_preload_receipt_commit
```

3. On Pi session_start, after project identity and Workpoint refresh, optionally call `/v1/preload/build` with:

```json
{
  "target": "pi",
  "mode": "session_start",
  "project_root": "S.sessionCwd",
  "continuity_id": "S.continuityId",
  "session_id": "S.sessionFrameKey",
  "current_ask": "S.currentAsk?.text"
}
```

4. Store the resulting packet in session state:

```ts
S.activeBootstrapPacket
S.lastBootstrapReceipt
S.lastBootstrapUpdate
S.lastBootstrapReceiptId
```

5. Awareness rendering may use the packet for reload/post-compaction/tool-guidance surfaces.
6. Pi must not write Cursor/Claude/Codex files unless the operator explicitly calls preload write.
7. Pi must not claim bootstrap delivery is verified unless Spec111 verifier passed or Spec119 receipt preview says `claim.status=actual`.

---

## 11. Tool contracts

Update:

```text
apps/pi-extension/src/tool-contracts.ts
docs/current/focusa-tool-contracts.json
docs/current/focusa-tool-choreography.json
apps/pi-extension/src/tools.ts
```

New family:

```ts
| "preload"
```

New tools:

```text
focusa_preload_profiles
focusa_preload_build
focusa_preload_render
focusa_preload_write
focusa_preload_verify
focusa_preload_doctor
focusa_preload_receipt_preview
focusa_preload_receipt_commit
```

Contract expectations:

```yaml
focusa_preload_profiles:
  family: preload
  side_effect_profile: read_state
  api_routes: ["GET /v1/preload/profiles"]
  likely_next_tools: ["focusa_preload_build"]

focusa_preload_build:
  family: preload
  side_effect_profile: read_state
  api_routes: ["POST /v1/preload/build"]
  likely_next_tools: ["focusa_preload_render", "focusa_preload_write", "focusa_preload_verify", "focusa_preload_receipt_preview"]

focusa_preload_render:
  family: preload
  side_effect_profile: read_state
  api_routes: ["POST /v1/preload/render"]
  likely_next_tools: ["focusa_preload_write", "focusa_preload_verify", "focusa_preload_receipt_preview"]

focusa_preload_write:
  family: preload
  side_effect_profile: write_project_files
  api_routes: ["POST /v1/preload/write"]
  likely_next_tools: ["focusa_preload_verify", "focusa_preload_receipt_preview", "focusa_preload_doctor"]

focusa_preload_verify:
  family: preload
  side_effect_profile: read_state
  api_routes: ["POST /v1/preload/verify"]
  likely_next_tools: ["focusa_preload_receipt_preview", "focusa_preload_doctor", "focusa_workpoint_resume"]

focusa_preload_doctor:
  family: preload
  side_effect_profile: read_state
  api_routes: ["POST /v1/preload/doctor"]
  likely_next_tools: ["focusa_project_identity", "focusa_workpoint_resume", "focusa_context_cognition", "focusa_receipt_preview"]

focusa_preload_receipt_preview:
  family: preload
  side_effect_profile: read_state
  api_routes: ["POST /v1/preload/receipt-preview", "POST /v1/receipts/preview"]
  likely_next_tools: ["focusa_preload_receipt_commit", "focusa_preload_doctor"]

focusa_preload_receipt_commit:
  family: preload
  side_effect_profile: write_receipt
  api_routes: ["POST /v1/preload/receipt-commit", "POST /v1/receipts/commit"]
  likely_next_tools: ["focusa_receipt_verify", "focusa_preload_doctor"]
```

---

## 12. Target rendering requirements

### 12.1 Static rule render

Purpose:

Long-lived, low-churn guidance.

Must include:

```text
- Focusa is source of truth for mission continuity.
- ProjectIdentity controls project scope.
- Workpoint controls immediate next action.
- Trajectory is goal/gap context only.
- Context Cognition is advisory context selection only.
- Evidence refs are proof boundary.
- Bootstrap delivery proof is recorded through Focusa Receipts when persisted.
- Never treat transcript tail as authority.
- If bootstrap packet is missing, say FOCUSA_PRELOAD_FAIL.
```

Must not include:

```text
- large logs,
- full Workpoint packet JSON,
- full ontology graph,
- full telemetry,
- stale transcript summaries,
- secrets,
- environment values.
```

### 12.2 Dynamic context render

Purpose:

Per-session startup packet.

Must include:

```text
FOCUSA_BOOTSTRAP_PACKET
packet_id:
generated_at:
target:
project_root:
continuity_id:
workpoint_id:
trajectory_id:

AUTHORITY
project_scope: ProjectIdentity
action_authority: Workpoint
goal_context: Trajectory
proof_boundary: Evidence refs
durable_delivery_proof: Focusa Receipt receipt_type=bootstrap_delivery

MISSION
...

EXACT_NEXT_ACTION
...

ACTIVE_OBJECTS
...

DO_NOT_DRIFT
...

EVIDENCE_REFS_OR_PROOF_GAPS
...

RECEIPT
receipt_id: <if committed>
receipt_preview: <if previewed>
claim_status: <actual|partial|surrogate|blocked|missing>

RECOVERY
If this packet is missing or stale, say FOCUSA_PRELOAD_FAIL and run focusa preload doctor.
```

### 12.3 Acceptance prompt

Every packet must include:

```text
Without using file search, grep, glob, semantic search, or external tools, answer only from the Focusa bootstrap context:
1. What project_root is active?
2. What continuity_id is active?
3. What is the active Workpoint or degraded fallback?
4. What is the exact next action?
5. What evidence refs or proof gaps are known?
6. What must you not drift into?
7. Was bootstrap receipt proof included or missing?

If any required answer is missing, respond exactly:
FOCUSA_PRELOAD_FAIL
```

---

## 13. Context Cognition integration

Spec 111 must use Context Cognition as a bounded selector, not as authority.

`/v1/preload/build` should call or reuse equivalent logic from:

```text
/v1/context-cognition
/v1/context-cognition/render
/v1/context-cognition/curate
```

Rules:

- selected context may be included only as advisory,
- excluded context must be visible when relevant,
- over-budget context must be labeled, not silently dropped,
- evidence-linked context gets priority,
- Workpoint next action remains the selection target unless current_ask safely supersedes it,
- selected_context must fit target profile budgets,
- receipt previews may summarize selected/excluded context but must not duplicate the full context packet.

Initial budgets:

```yaml
cursor:
  static_max_lines: 80
  dynamic_max_lines: 160

claude:
  static_max_lines: 120
  dynamic_max_lines: 200

codex:
  static_max_lines: 100
  dynamic_max_lines: 180

pi:
  static_max_lines: 0
  dynamic_max_lines: 120

generic:
  static_max_lines: 80
  dynamic_max_lines: 120
```

---

## 14. Awareness integration

Add new awareness surfaces:

```rust
SURFACE_AGENT_PRELOAD
SURFACE_PRELOAD_FAIL
SURFACE_PRELOAD_REMEDIATION
SURFACE_PRELOAD_RECEIPT
```

The AwarenessPacket may summarize bootstrap status:

```text
- preload delivered
- preload degraded
- preload failed verification
- project scope missing
- Workpoint missing
- evidence gap declared
- receipt preview available
- receipt committed
- receipt verification failed
- recovery tool
```

Awareness must not duplicate the full packet or the full receipt.

Awareness is a visible/status layer, not the delivery artifact itself.

---

## 15. Session Transfer integration

Extend `focusa_session_transfer`.

New optional fields:

```json
{
  "write_preload": false,
  "preload_target": "cursor",
  "preload_mode": "session_transfer",
  "receipt_preview": true,
  "receipt_commit": false
}
```

Behavior:

- `action="save"` may include a suggested preload command in `operator_handoff`.
- `action="continue"` may build a preload packet from the latest prior save.
- It must not write files unless `write_preload=true`.
- If no prior save exists, return degraded status with `focusa_preload_build` as next tool.
- Receipt preview may be returned for session-transfer bootstrap delivery.
- Receipt commit remains explicit unless an operator-enabled policy says otherwise.

Operator handoff should become:

```json
{
  "command": "cd <project_root> && pi",
  "first_tool": "focusa_session_transfer action=\"continue\" ...",
  "preload": "focusa preload write --target cursor --project-root <root> --continuity-id <id>",
  "receipt_preview": "focusa preload receipt-preview --target cursor --project-root <root> --continuity-id <id>",
  "authority_boundary": "project_root_plus_continuity_id"
}
```

---

## 16. Security and safety

Spec 111 file writes are high-trust local operations.

Required protections:

- Reject unsafe/broad project_root.
- Reject paths outside project_root.
- Reject path traversal.
- Do not follow symlinks for write destinations.
- Do not render `.env` values.
- Do not include secrets in generated files.
- Do not include raw logs unless explicitly captured as evidence handles.
- Keep generated artifacts small.
- Mark generated files with a header.
- `dry_run=true` must be fully supported.
- `write_hooks=false` by default.
- Hook installation must be opt-in.
- Receipt preview/commit must not include secrets, tokens, raw private logs, or unredacted local paths unless explicitly public-safe or private-only.
- Receipt commit for preload write must satisfy Spec119 integrity rules.

---

## 17. Resource and bloat posture

Spec 111 must respect Bloatgaurd.

Rules:

- default output is compact,
- full packet JSON is written only to `.focusa/preload/session-context.json`,
- markdown files use summaries and refs,
- Context Cognition selected_context is budgeted,
- Utility Card content is summarized, not pasted wholesale,
- Awareness visible lines are capped,
- receipt previews summarize delivery/proof status only,
- LowMem mode forces dynamic context to “surgical summary only.”

LowMem dynamic packet must include only:

```text
project_root
continuity_id
workpoint_id
mission
exact_next_action
do_not_drift
evidence_refs_or_gap
receipt_status_or_gap
recovery command
```

---

## 18. Failure model

All routes must return `tool_result_v1` under `details`.

Failure classes:

```text
project_root_missing
scope_mismatch
continuity_id_missing
project_identity_unverified
workpoint_missing
trajectory_unavailable
context_cognition_unavailable
utility_card_unavailable
awareness_unavailable
target_profile_unknown
render_failed
write_rejected
write_failed
verify_failed
receipt_preview_failed
receipt_commit_failed
receipt_integrity_failed
doctor_failed
```

Retry posture:

```text
validation/scope/write rejection -> do_not_retry_unchanged
daemon/resource transient -> safe_retry
missing Workpoint -> create_or_resume_workpoint
missing continuity -> bind_continuity_id
receipt preview blocked -> follow next_safe_action
receipt commit blocked -> fix missing evidence/authority before retry
```

---

## 19. Tests

Add:

```text
tests/spec111_preload_schema_static_test.py
tests/spec111_preload_routes_static_test.py
tests/spec111_preload_cli_static_test.py
tests/spec111_preload_tool_contract_static_test.py
tests/spec111_preload_write_safety_static_test.py
tests/spec111_preload_render_snapshot_test.py
tests/spec111_preload_receipt_integration_static_test.py
```

Minimum assertions:

### 19.1 Schema static test

Must verify:

```text
AgentBootstrapPacket
AgentBootstrapProfile
AgentBootstrapReceipt
AgentBootstrapReceiptProjection
AgentBootstrapTarget
AgentBootstrapMode
FOCUSA_PRELOAD_FAIL
```

exist in core.

### 19.2 Route static test

Must verify:

```text
routes/preload.rs exists
routes::preload::router() merged in server.rs
/v1/preload/profiles
/v1/preload/build
/v1/preload/render
/v1/preload/write
/v1/preload/verify
/v1/preload/doctor
/v1/preload/receipt-preview
/v1/preload/receipt-commit
```

### 19.3 CLI static test

Must verify:

```text
Commands::Preload
commands::preload::PreloadCmd
profiles/build/render/write/verify/doctor/receipt-preview/receipt-commit subcommands
```

### 19.4 Tool contract static test

Must verify:

```text
focusa_preload_profiles
focusa_preload_build
focusa_preload_render
focusa_preload_write
focusa_preload_verify
focusa_preload_doctor
focusa_preload_receipt_preview
focusa_preload_receipt_commit
```

exist in:

```text
apps/pi-extension/src/tools.ts
apps/pi-extension/src/tool-contracts.ts
docs/current/focusa-tool-contracts.json
docs/current/focusa-tool-choreography.json
docs/focusa-tools/tools/
```

### 19.5 Write safety test

Must verify code contains protections for:

```text
project_root safety
path traversal rejection
under-project-root enforcement
dry_run
overwrite flag
no symlink follow
generated header
receipt preview/commit awareness
```

### 19.6 Render snapshot test

For a synthetic packet, verify rendered markdown includes:

```text
FOCUSA_BOOTSTRAP_PACKET
ProjectIdentity
Workpoint
Trajectory
Evidence refs
Focusa Receipt
FOCUSA_PRELOAD_FAIL
exact next action
do not drift
```

and excludes:

```text
.env secret values
full telemetry logs
full ontology graph
raw transcript tail
```

### 19.7 Receipt integration static test

Must verify:

```text
receipt_type=bootstrap_delivery
focusa.receipt.v1
focusa_receipt_preview
focusa_receipt_commit
receipt_consistent
receipt_ledger_consistent
```

appear in this spec and relevant docs/tool contracts once implemented.

---

## 20. Documentation

Add:

```text
docs/focusa-tools/tools/focusa_preload_profiles.md
docs/focusa-tools/tools/focusa_preload_build.md
docs/focusa-tools/tools/focusa_preload_render.md
docs/focusa-tools/tools/focusa_preload_write.md
docs/focusa-tools/tools/focusa_preload_verify.md
docs/focusa-tools/tools/focusa_preload_doctor.md
docs/focusa-tools/tools/focusa_preload_receipt_preview.md
docs/focusa-tools/tools/focusa_preload_receipt_commit.md
docs/current/PRELOAD_TARGETS_CURRENT.md
docs/current/PRELOAD_RECEIPT_INTEGRATION_CURRENT.md
```

Update:

```text
README.md
docs/current/API_REFERENCE_CURRENT.md
docs/current/CLI_REFERENCE_CURRENT.md
docs/current/focusa-tool-contracts.json
docs/current/focusa-tool-choreography.json
docs/119-verifiable-agent-work-receipts-and-governed-execution-ledger-spec.md
```

README addition should be short:

```text
Spec 111 — Agent Context Bootstrap & Delivery:
Focusa can build, render, write, and verify compact startup packets for Cursor, Claude Code, Codex, Pi, OpenCode, and generic agents. Packets compose ProjectIdentity, Workpoint, Trajectory, Context Cognition, Utility Card, and Awareness without creating a new authority source. Durable delivery proof is recorded through Spec119 Focusa Receipts when persisted.
```

---

## 21. Implementation order

### Slice 1 — spec and static contracts

- Add/update this spec.
- Add tool docs.
- Add static tests.
- Add placeholders in contracts/choreography.
- No runtime behavior yet.

### Slice 2 — core packet types/renderers

- Add `crates/focusa-core/src/preload.rs`.
- Add type exports.
- Add profile definitions.
- Add markdown/json renderers.
- Add receipt projection type.
- Add unit tests.

### Slice 3 — API build/render/verify/doctor

- Add route file.
- Wire router.
- Implement build from existing state.
- Implement render.
- Implement verify.
- Implement doctor.
- Add receipt preview projection.
- No file writes yet.

### Slice 4 — safe write

- Implement `/v1/preload/write`.
- Add dry-run.
- Add write safety checks.
- Add generated headers.
- Add target-specific receipt projection.

### Slice 5 — receipt preview/commit integration

- Implement `/v1/preload/receipt-preview`.
- Implement `/v1/preload/receipt-commit`.
- Route to Spec119 receipt preview/commit surfaces.
- Add receipt failure/recovery envelopes.

### Slice 6 — CLI

- Add `focusa preload`.
- Add human and JSON output.
- Add receipt preview/commit subcommands.
- Add tests.

### Slice 7 — Pi/tool integration

- Add Pi tools.
- Add optional Pi session_start packet storage.
- Keep old session_start flow intact.
- Add awareness lines for receipt status.

### Slice 8 — docs/reference/regression

- Update README/current docs/tool docs.
- Add render snapshots.
- Add receipt integration docs.
- Add static/live-safe tests.

---

## 22. Acceptance criteria

Spec111 is accepted when:

1. AgentBootstrapPacket, AgentBootstrapProfile, AgentBootstrapReceipt, and AgentBootstrapReceiptProjection exist in core.
2. `/v1/preload/profiles`, `/build`, `/render`, `/write`, `/verify`, and `/doctor` exist.
3. `/v1/preload/receipt-preview` and `/v1/preload/receipt-commit` exist or are explicitly deferred behind Spec119 implementation gates.
4. CLI supports preload profiles/build/render/write/verify/doctor.
5. CLI supports preload receipt-preview/receipt-commit or reports the Spec119 dependency clearly.
6. Pi tools exist and do not break existing session_start lifecycle.
7. Target renderers exist for cursor, claude, codex, pi, opencode, and generic.
8. File writes are bounded under verified project_root and pass safety tests.
9. Verification fails closed with `FOCUSA_PRELOAD_FAIL` when required fields are missing.
10. Static rule render is compact and stable.
11. Dynamic context render contains exact next action, do-not-drift, evidence refs/proof gaps, and receipt status/gap.
12. Context Cognition is advisory and budgeted.
13. Awareness can summarize preload status without dumping full packets.
14. Receipt projection maps AgentBootstrapReceipt to Spec119 `bootstrap_delivery` fields.
15. Receipt commit, when enabled, creates a canonical Spec119 receipt rather than a standalone durable bootstrap ledger.
16. Tests and docs prove no secrets, raw logs, full ontology, or transcript tail are included by default.

---

## 23. Closure policy

Do not close Spec111 implementation work until:

- all child beads are closed or explicitly blocked/deferred with operator-accepted rationale,
- route/CLI/Pi/tool-contract tests pass,
- render snapshot tests pass,
- write safety tests pass,
- receipt integration static tests pass,
- docs explain the relationship between AgentBootstrapReceipt and Spec119 Focusa Receipts,
- any remaining missing native/runtime proof is labeled partial/surrogate/blocked, not complete.

Partial bootstrap surfaces may ship behind preview labels, but public docs must not claim durable bootstrap delivery proof is complete until Spec119 receipt integration is implemented and verified.
