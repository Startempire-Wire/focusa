# docs/111-agent-context-bootstrap-and-delivery-spec.md — Focusa Agent Context Bootstrap & Delivery

Status: proposed / implementation-ready
Scope: Focusa core, API, CLI, Pi extension, tool contracts, docs, static audits
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
- Spec 109 — claimed downstream work; do not reuse number
- Spec 110 — claimed downstream work; do not reuse number

Focusa already has the required primitives:

- ProjectIdentity resolves project scope.
- WorkpointResumePacket carries immediate action authority.
- Trajectory supplies HLT/MLG/STG/gap context.
- Context Cognition selects bounded advisory context.
- Utility Card supplies startup/post-compaction guidance.
- Awareness renders surface-specific guidance.
- Session Transfer composes save/continue semantics.
- Pi session_start already performs a Pi-specific bootstrap.

The gap is that these surfaces are not unified into one portable agent-context delivery system for Cursor, Claude Code, Codex, OpenCode/OpenClaw, Pi, and future adapters.

Any mismatch between this spec and current implementation is an implementation gap, not a reason to weaken this spec.

---

## 1. Purpose

Spec 111 adds a first-class Focusa **Agent Context Bootstrap & Delivery** layer.

The layer builds, renders, writes, and verifies bounded agent startup packets from existing Focusa authority surfaces so a new agent session can continue verified project work without relying on transcript tail, stale chat memory, or tool-specific prompt hacks.

Short form:

> Focusa Bootstrap turns a cold agent session into a verified mission continuation.

---

## 2. Core thesis

Agent sessions fail in predictable ways:

- the agent starts in the wrong project,
- the agent treats transcript tail as authority,
- the agent forgets Workpoint/Trajectory context,
- proof is buried in prior chat,
- Cursor/Claude/Codex/Pi each need context in a different shape,
- handoffs depend on the operator remembering what to paste.

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
  → delivery verification receipt
```

The product promise is not “write better prompts.”

The product promise is:

> Before an AI coding agent acts, Focusa can prove which project, mission, Workpoint, next action, evidence refs, and drift boundaries it was given.

---

## 3. Non-goals

Spec 111 is not:

- a replacement for Workpoint,
- a replacement for Context Cognition,
- a replacement for Session Transfer,
- a Cursor-only integration,
- a vector database,
- a prompt-stuffing mechanism,
- a new task scheduler,
- a new source of canonical truth,
- a hidden auto-mutation system,
- a route that edits arbitrary project files without explicit request,
- a guarantee that an external agent obeyed the packet after delivery.

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
| Delivery success | Spec 111 preload verifier / receipt |

Spec 111 outputs are delivery artifacts, not cognition authority.

Generated files are prompt surfaces. They do not become canonical truth.

If a generated file conflicts with live Focusa state, live Focusa state wins.

Spec 111 may produce:

- target-specific context files,
- target-specific static rule files,
- prompt snippets,
- verification prompts,
- receipts,
- diagnostic cards.

Spec 111 must not:

- promote Workpoints,
- define or supersede Trajectories,
- capture Evidence,
- mutate FocusState,
- alter reducer-backed cognition,
- close beads,
- install hooks without explicit request.

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

A delivery/verification receipt.

```yaml
AgentBootstrapReceipt:
  schema_version: focusa.agent_bootstrap_receipt.v1
  receipt_id:
  packet_id:
  target:
  project_root:
  continuity_id:
  status: written | verified | failed | dry_run
  files_written: []
  files_skipped: []
  verifier:
    status:
    missing_fields: []
    failed_checks: []
    acceptance_prompt:
  generated_at:
  side_effects: []
```

### 5.4 FOCUSA_PRELOAD_FAIL

A required fail-closed phrase used by external agents.

If the target session did not receive or cannot prove it received the required Focusa bootstrap context, it must respond:

```text
FOCUSA_PRELOAD_FAIL
```

It must not guess from transcript tail.

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

tests/
  adds Spec111 static and route/CLI/tool-contract audits
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
pub struct AgentBootstrapReceipt { /* schema above */ }
pub struct AgentBootstrapRenderedFiles { /* path + kind + content */ }
pub struct AgentBootstrapVerification { /* pass/fail + fields */ }

pub fn profile_for_target(target: AgentBootstrapTarget) -> AgentBootstrapProfile;

pub fn build_agent_bootstrap_packet(input: AgentBootstrapInput) -> AgentBootstrapPacket;

pub fn render_agent_bootstrap_markdown(packet: &AgentBootstrapPacket) -> String;

pub fn render_agent_bootstrap_json(packet: &AgentBootstrapPacket) -> serde_json::Value;

pub fn render_target_files(packet: &AgentBootstrapPacket, profile: &AgentBootstrapProfile) -> AgentBootstrapRenderedFiles;

pub fn verify_agent_bootstrap_packet(packet: &AgentBootstrapPacket) -> AgentBootstrapVerification;
```

Core rules:

- no filesystem writes in `focusa-core`,
- no HTTP calls in `focusa-core`,
- no UI logic in `focusa-core`,
- no reducer mutation,
- deterministic rendering for identical input,
- every output includes `schema_version`,
- every target profile includes `fail_phrase = "FOCUSA_PRELOAD_FAIL"`.

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
  "next_tools": ["focusa_preload_render", "focusa_preload_write", "focusa_preload_verify"]
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
  ]
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
  "write_hooks": false
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
  "next_tools": ["focusa_preload_doctor"]
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
```

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
```

JSON output returns full envelopes.

---

## 10. Pi extension integration

Pi currently has its own session_start bootstrap.

Spec 111 must not break it.

Required changes:

1. Keep existing Pi lifecycle behavior.
2. Add a Pi tool wrapper:

```text
focusa_preload_build
focusa_preload_write
focusa_preload_verify
focusa_preload_doctor
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
```

5. Awareness rendering may use the packet for reload/post-compaction/tool-guidance surfaces.

6. Pi must not write Cursor/Claude/Codex files unless the operator explicitly calls preload write.

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
  likely_next_tools: ["focusa_preload_render", "focusa_preload_write", "focusa_preload_verify"]

focusa_preload_render:
  family: preload
  side_effect_profile: read_state
  api_routes: ["POST /v1/preload/render"]
  likely_next_tools: ["focusa_preload_write", "focusa_preload_verify"]

focusa_preload_write:
  family: preload
  side_effect_profile: write_project_files
  api_routes: ["POST /v1/preload/write"]
  likely_next_tools: ["focusa_preload_verify", "focusa_preload_doctor"]

focusa_preload_verify:
  family: preload
  side_effect_profile: read_state
  api_routes: ["POST /v1/preload/verify"]
  likely_next_tools: ["focusa_preload_doctor", "focusa_workpoint_resume"]

focusa_preload_doctor:
  family: preload
  side_effect_profile: read_state
  api_routes: ["POST /v1/preload/doctor"]
  likely_next_tools: ["focusa_project_identity", "focusa_workpoint_resume", "focusa_context_cognition"]
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
- selected_context must fit target profile budgets.

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
```

The AwarenessPacket may summarize bootstrap status:

```text
- preload delivered
- preload degraded
- preload failed verification
- project scope missing
- Workpoint missing
- evidence gap declared
- recovery tool
```

Awareness must not duplicate the full packet.

Awareness is a visible/status layer, not the delivery artifact itself.

---

## 15. Session Transfer integration

Extend `focusa_session_transfer`.

New optional fields:

```json
{
  "write_preload": false,
  "preload_target": "cursor",
  "preload_mode": "session_transfer"
}
```

Behavior:

- `action="save"` may include a suggested preload command in `operator_handoff`.
- `action="continue"` may build a preload packet from the latest prior save.
- It must not write files unless `write_preload=true`.
- If no prior save exists, return degraded status with `focusa_preload_build` as next tool.

Operator handoff should become:

```json
{
  "command": "cd <project_root> && pi",
  "first_tool": "focusa_session_transfer action=\"continue\" ...",
  "preload": "focusa preload write --target cursor --project-root <root> --continuity-id <id>",
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
doctor_failed
```

Retry posture:

```text
validation/scope/write rejection -> do_not_retry_unchanged
daemon/resource transient -> safe_retry
missing Workpoint -> create_or_resume_workpoint
missing continuity -> bind_continuity_id
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
```

Minimum assertions:

### 19.1 Schema static test

Must verify:

```text
AgentBootstrapPacket
AgentBootstrapProfile
AgentBootstrapReceipt
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
```

### 19.3 CLI static test

Must verify:

```text
Commands::Preload
commands::preload::PreloadCmd
profiles/build/render/write/verify/doctor subcommands
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
```

### 19.6 Render snapshot test

For a synthetic packet, verify rendered markdown includes:

```text
FOCUSA_BOOTSTRAP_PACKET
ProjectIdentity
Workpoint
Trajectory
Evidence refs
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

---

## 20. Documentation

Add:

```text
docs/111-agent-context-bootstrap-and-delivery-spec.md
docs/focusa-tools/tools/focusa_preload_profiles.md
docs/focusa-tools/tools/focusa_preload_build.md
docs/focusa-tools/tools/focusa_preload_render.md
docs/focusa-tools/tools/focusa_preload_write.md
docs/focusa-tools/tools/focusa_preload_verify.md
docs/focusa-tools/tools/focusa_preload_doctor.md
docs/current/PRELOAD_TARGETS_CURRENT.md
```

Update:

```text
README.md
docs/current/API_REFERENCE_CURRENT.md
docs/current/CLI_REFERENCE_CURRENT.md
docs/current/focusa-tool-contracts.json
docs/current/focusa-tool-choreography.json
```

README addition should be short:

```text
Spec 111 — Agent Context Bootstrap & Delivery:
Focusa can now build, render, write, and verify compact startup packets for Cursor, Claude Code, Codex, Pi, OpenCode, and generic agents. Packets compose ProjectIdentity, Workpoint, Trajectory, Context Cognition, Utility Card, and Awareness without creating a new authority source.
```

---

## 21. Implementation order

### Slice 1 — spec and static contracts

- Add this spec.
- Add tool docs.
- Add static tests.
- Add placeholders in contracts/choreography.
- No runtime behavior yet.

### Slice 2 — core packet types/renderers

- Add `crates/focusa-core/src/preload.rs`.
- Add type exports.
- Add profile definitions.
- Add markdown/json renderers.
- Add unit tests.

### Slice 3 — API build/render/verify/doctor

- Add route file.
- Wire router.
- Implement build from existing state.
- Implement render.
- Implement verify.
- Implement doctor.
- No file writes yet.

### Slice 4 — safe write

- Implement `/v1/preload/write`.
- Add dry-run.
- Add write safety checks.
- Add generated headers.
- Add write receipt.

### Slice 5 — CLI

- Add `focusa preload`.
- Add human and JSON output.
- Add tests.

### Slice 6 — Pi/tool integration

- Add Pi tools.
- Add optional Pi session_start packet storage.
- Keep old session_start flow intact.
- Add Awareness handoff surface.

### Slice 7 — Session Transfer integration

- Add optional preload fields.
- Add operator_handoff preload command.
- Add continue-mode preload build.

---

## 22. Acceptance criteria

Spec 111 is complete when:

1. `cargo test --workspace` passes.
2. `cargo clippy --workspace -- -D warnings` passes.
3. Spec111 static tests pass.
4. `focusa preload profiles` lists cursor/claude/codex/pi/opencode/generic.
5. `focusa preload build --target cursor --project-root <root> --continuity-id <id> --json` returns `focusa.agent_bootstrap_packet.v1`.
6. `focusa preload render --target cursor ...` includes `FOCUSA_PRELOAD_FAIL`.
7. `focusa preload write --target cursor --dry-run ...` reports planned writes with zero side effects.
8. `focusa preload write --target cursor --overwrite ...` writes only under project_root.
9. `focusa preload verify --target cursor ...` passes for generated files.
10. `focusa preload doctor --target cursor ...` distinguishes delivery failures from content failures.
11. Pi session_start remains functional.
12. Session Transfer can suggest a preload handoff command.
13. Generated packet never claims canonical mutation authority.
14. Generated packet never treats transcript tail as authority.
15. Generated packet contains exact next action or explicitly returns degraded with recovery.

---

## 23. Agent implementation prompt

Implement Spec 111 exactly as written.

Do not collapse this into a Cursor-only feature.

Do not make Context Cognition or Preload a new authority source.

Keep `focusa-core` pure.

Keep API/CLI as facades.

Make file writing explicit, dry-run capable, and path-safe.

Preserve Pi’s existing session_start behavior while extracting reusable bootstrap policy.

Add static tests before runtime behavior.

End with proof:

```text
python tests/spec111_preload_schema_static_test.py
python tests/spec111_preload_routes_static_test.py
python tests/spec111_preload_cli_static_test.py
python tests/spec111_preload_tool_contract_static_test.py
python tests/spec111_preload_write_safety_static_test.py
python tests/spec111_preload_render_snapshot_test.py
cargo test --workspace
cargo clippy --workspace -- -D warnings
```
