# Spec 103 — Call Stack Architecture Blueprint

Status: iterable-spec-v0
Scope: design first; bounded artifact; advisory by default; promotion requires existing Focusa Workpoint, Evidence, Trajectory, or reducer-backed paths
Canonical label: Call Stack Architecture Blueprint (CSAB)

## 0. Normative basis

This spec references Spec 100 (Context Cognition, `status=planning`) and Spec 101 (Focusa Bloatgaurd, `status=iterable-spec-v0`, "design first; no implementation authority") as forward-looking design lineage, **not as build-time dependencies**. Spec 103's implementation is complete today on existing Focusa primitives; the references to 100/101 are commitments that future revisions of this spec will defer to 100/101's surface contracts and leanness invariants once those specs ship their implementation phases.

### 0.1 Implementation status (as of v0)

- **103 implementation: shipped.** Daemon route `POST /v1/call-stack/design`, `CallStackDesign` type, append-only JSONL ledger, `focusa_call_stack_design` Pi tool, contract entry, choreography entry, tool doc page, and `BENEFITS.md` narrative all live. Audits: 65/65 contracts, 198 edges, 0 failures, 0 warnings.
- **103's actual runtime dependencies (all implemented):** `project_identity_payload_for_scope` (in `routes/project.rs`), `AppState` + axum routing (pre-existing), `spec80Result` + `piToolText` template (Spec 103 references its own round 1 here, not 100/101), `SqlitePersistence::append_call_stack_design` (new in 103), `focusa_project_identity` / `focusa_project_verify` for recovery (pre-existing).
- **100 implementation: not started.** Status `planning`; no code touches Spec 100's `ContextCognitionPacket` or eval harness.
- **101 implementation: not started.** Status `iterable-spec-v0`; no code implements the Bloatgaurd budget domains or adaptive router.

### 0.2 Design lineage from 100/101 (forward-looking, not load-bearing)

The following 100/101 concepts are referenced for design coherence only. None of them is enforced by the v0 implementation; 103 works without them.

- project scope is bounded by `project_root + continuity_id` (Spec 100 §2, §4) — already enforced by existing `project_identity_payload_for_scope`.
- Workpoint remains immediate action authority (Spec 100 §4) — already enforced by existing workpoint route handlers.
- Trajectory supplies project/workstream goal and gap context (Spec 100 §4) — already enforced by trajectory route handlers.
- HLT is durable north-star context while MLG/STG/waypoints are adaptive advisory context (Spec 100 §2) — already enforced by HLT append-only ledger (`/v1/hlt/history`).
- Ontology is semantic structure, not proof by itself (Spec 100 §5) — already enforced by ontology route.
- Evidence refs are the proof boundary (Spec 100 §5) — already enforced by `focusa_evidence_capture` and `focusa_workpoint_link_evidence`.
- Bounded artifacts beat context dumps (Spec 101 §1, §5.5) — design goal for 103; the `CallStackDesign` envelope is bounded to ≤ 1KB JSON.
- Tool-call history is elided; structured rehydration is the contract (Spec 101 §5.10) — already enforced by `focusa_traverse` and the ledger pattern.

When 100 and 101 land their implementation phases, this spec (103) will be revised to:
- (100) defer the `focusa_call_stack_design` surface contract to whatever 100's `Pi extension contract` (§12) and `Focusa tool wrappers` (§13) prescribe.
- (101) defer the `CallStackDesign` envelope bounds to whatever 101's `Tool-call compression` (§5.2) and `Output firewall` (§5.1) prescribe.

Until then, 103 ships as a thin, self-contained tool that uses existing Focusa primitives and existing design lineage from 100/101 as documented commitments only.

## 1. Purpose

Specify a typed, append-only, evidence-linkable **Call Stack Design** that an agent (or operator) writes *before* implementing a feature, and that Focusa can later verify against the actual call surface.

A Call Stack Design is the answer to the question: "given this feature, what is the exact end-to-end call flow from operator/agent input all the way to storage and back, and how does each layer compose with the next?"

## 2. Core thesis

> A well-typed call stack blueprint written before implementation is the single highest-leverage artifact an agent can be given. It bridges high-level requirements and concrete code by spelling out the exact call flow: entry → handlers → services → adapters → storage → output. LLMs follow structured patterns well and wander less when the structure is explicit.

This spec turns the call stack from informal prose into a typed, idempotent, cacheable, evidence-linkable artifact.

## 3. Non-goals

- Generating call stacks from arbitrary natural language. The tool returns a *typed scaffold* the operator/agent fills in for the specific feature. Generation is human/LLM-driven, structure is enforced by the schema.
- Replacing the Workpoint or Trajectory ladder. Call stacks are *artifacts attached to* a Workpoint's STG, not a replacement.
- Replacing code review. Call stack verification is a coarse smoke check, not a substitute for diff review.
- Covering every call surface. The schema covers CLI, Pi tool, and HTTP route entry points — internal-only helpers are out of scope.

## 4. Authority boundaries

- **Advisory by default.** `focusa_call_stack_design` is non-canonical. The design never mutates Workpoint or Trajectory state.
- **Promotion to evidence.** When the operator opts in via `attach_to_workpoint=true`, the design becomes `focusa_evidence` linked to the active Workpoint.
- **Promotion to trajectory.** When the operator opts in via `attach_to_stg=true`, the design becomes the `STG` of the active Trajectory. The ladder is otherwise unaffected.
- **No daemon mutation.** The daemon may persist the design as `focusa_evidence`; it never changes runtime state.

## 5. Inputs

Required:

- `project_root` — bounded project scope, mandatory.
- `mission` — short description of the feature this design covers (≤ 200 chars).

Optional:

- `continuity_id` — workstream filter.
- `workpoint_id` — Workpoint to attach the design to.
- `entry_surface` — `pi_tool` | `cli_command` | `http_route` (default: `pi_tool`).
- `entry_name` — proposed tool/command/route name; falls back to a deterministic stub when omitted.
- `notes` — bounded free-form notes for the design.
- `attach_to_workpoint` — default `false`; when `true`, links as `focusa_evidence`.
- `attach_to_stg` — default `false`; when `true`, sets the active STG to the design's mission.
- `parent_design_id` — chain an incremental refinement onto an existing design.

## 6. Primary output

The primary artifact is a bounded `CallStackDesign` envelope.

```yaml
CallStackDesign:
  schema_version: focusa.call_stack_design.v1
  status: completed | degraded | stale | blocked
  advisory: true
  canonical: false
  scope_status: matched | missing | partial | mismatch
  freshness:
    generated_at:
    stale:
    source_design_id: null | <uuid>
  scope:
    project_root:
    continuity_id:
    workpoint_id:
    session_id:
  authority:
    action_authority: workpoint | none
    goal_context: trajectory | none
    semantic_context: ontology
    proof_context: evidence | none
    canonical_mutation_allowed: false
  design:
    design_id: <uuid>
    mission: <text>
    entry:
      surface: pi_tool | cli_command | http_route
      name: <text>
      parameters: <bounded schema stub>
    handlers:
      - name: validation
        purpose: input schema check
      - name: scope_binding
        purpose: project_root + continuity_id
      - name: workpoint_link
        purpose: attach evidence to active Workpoint
    services:
      - name: spec80_envelope
        purpose: tool_result_v1 wrapper
      - name: trajectory_assess
        purpose: short-term-goal alignment
    adapters:
      - name: focusa_fetch
        purpose: HTTP/JSON to daemon
      - name: persistence_jsonl
        purpose: append-only ledger
    storage:
      - kind: jsonl | sqlite | evidence
        path: <deterministic>
    output:
      - envelope: tool_result_v1
        evidence_refs: []
        next_tools: []
  evidence_refs: []
  next_tools: [focusa_call_stack_verify, focusa_workpoint_link_evidence]
  rehydrate_id: <design_id>
```

The scaffold above is the *standard Focusa call stack shape*. The operator/agent is expected to fill in feature-specific details (entry name, parameter schema, storage path, evidence refs). The tool does not invent those.

## 7. Ontology as semantic spine

A Call Stack Design is indexed in the ontology as `OntologyClass.CallStackDesign` with these affordances:

- `trajectory_ladder.stg_alignment` — verifies the design's mission is consistent with the active STG.
- `workpoint.evidence_handle` — the design itself, when promoted, is a handle of class `evidence.call_stack_design`.
- `ontology.call_stack_design_index` — recent designs are surfaced by `focusa_traverse` on the `call_stack_designs` surface.
- `side_effects.classification` — designs never mutate Workpoint or Trajectory unless `attach_to_workpoint` / `attach_to_stg` are explicitly true.

## 8. Surface interaction model

The design is reachable from three surfaces, each with the same authority boundary:

- **Pi tool**: `focusa_call_stack_design` (Pi extension, calls `POST /v1/call-stack/design`).
- **CLI**: `focusa call-stack design` (planned, not in v1; deferred to a follow-up slice).
- **HTTP**: `POST /v1/call-stack/design` with the same JSON body.

A future `focusa_call_stack_verify` (next slice) reads the design and reports drift against the actual route / tool surface.

## 9. Daemon behavior

The daemon stores the design as an append-only JSONL line in `data/call-stack-designs/{project_root_hash}/designs.jsonl`. The design is *advisory* and *non-canonical*; it never replaces Workpoint or Trajectory state.

Storage guarantees:

- `project_root` is required; `project_identity_unverified` failure class when missing.
- The ledger is append-only; existing entries are never modified or deleted.
- Entries are ordered by timestamp (oldest first, most recent last).
- File path is deterministic: `{data_dir}/call-stack-designs/{project_root_hash}/designs.jsonl`.
- When `attach_to_workpoint=true`, a second ledger appends to the Workpoint's evidence store via `focusa_evidence_capture`.
- When `attach_to_stg=true`, the active Trajectory's STG is updated via `focusa_trajectory_define_goal` (separate write path; user-confirmed).

## 10. API contract

`POST /v1/call-stack/design`

```json
{
  "project_root": "/home/wirebot/focusa",
  "continuity_id": "focusa-cont-…",
  "mission": "Add focusa_call_stack_design tool",
  "entry_surface": "pi_tool",
  "entry_name": "focusa_call_stack_design",
  "workpoint_id": "019ea…",
  "attach_to_workpoint": false,
  "attach_to_stg": false,
  "parent_design_id": null,
  "notes": null
}
```

Response (success):

```json
{
  "status": "completed",
  "canonical": false,
  "advisory": true,
  "failure_class": null,
  "design": { /* CallStackDesign envelope */ },
  "evidence_refs": [],
  "next_tools": ["focusa_call_stack_verify", "focusa_workpoint_link_evidence", "focusa_trajectory_assess"]
}
```

Failure classes (non-exhaustive):

- `project_root_missing` — body lacks `project_root`.
- `project_root_unverified` — `project_identity` says unsafe/unknown.
- `daemon_unavailable` — daemon health probe failed.
- `workpoint_unavailable` — `attach_to_workpoint=true` but no active Workpoint and no `workpoint_id` provided.
- `trajectory_unclear` — `attach_to_stg=true` but no active Trajectory.
- `validation_rejected` — input failed validation.

## 11. CLI contract

`focusa call-stack design` (planned v1 follow-up, not in v0 of this spec) takes the same JSON body via `--input` or stdin and prints the design. The CLI is a thin wrapper over the HTTP route.

## 12. Pi extension contract

`focusa_call_stack_design` parameters:

- `project_root` — optional, defaults to Pi session cwd.
- `continuity_id` — optional.
- `mission` — required, ≤ 200 chars.
- `entry_surface` — optional enum.
- `entry_name` — optional.
- `workpoint_id` — optional.
- `attach_to_workpoint` — optional bool.
- `attach_to_stg` — optional bool.
- `parent_design_id` — optional.
- `notes` — optional.

Return: `tool_result_v1` envelope with `design.rehydrate_id` set to the `design_id` for later rehydration.

## 13. Focusa tool wrappers

The tool is exposed as `focusa_call_stack_design` in the standard 64-tool registry. It is part of the `workpoint` family and is choreographed as:

- **Best next tools:** `focusa_call_stack_verify` (next slice), `focusa_workpoint_link_evidence`, `focusa_trajectory_assess`.
- **Recovery:** on `project_root_unverified`, run `focusa_project_verify` first. On `workpoint_unavailable`, run `focusa_workpoint_resume` first.

## 14. Curator and optimizer roles

A future `Call Stack Curator` (analogous to Spec 100's Context Curator) will:

- prune stale designs (>30 days, no recent verification);
- dedupe by `(project_root, mission, entry_name)`;
- surface the most-recently-verified design per `entry_name` for re-use.

A future `Call Stack Optimizer` (analogous to Spec 100's Cognition Optimizer) will:

- compare `design.adapters` against the actual `focusa_ontology.tool_contracts` surface;
- recommend adapter simplifications (e.g., drop a redundant handler);
- feed deltas into `focusa_metacog_capture` as `strategy_class=call_stack_optimization`.

These are not in v0.

## 15. UIAI interaction

If `attach_to_workpoint=true`, the design becomes `focusa_evidence` linked to the active Workpoint. Future UIAI diagnostic intakes (e.g., `focusa_browser_diagnostics_intake`) can reference the design via `evidence_refs` to anchor the diagnostics in the work plan.

## 16. Menubar interaction

The macOS menubar app surfaces the most recent design for the active Workpoint in the Workpoint Peek card. Tapping the card opens a focused view with the design's mission, entry, handlers, services, adapters, storage, and output.

## 17. Work-loop interaction

The work-loop treats the design as an advisory artifact. The work-loop status field `next_design_id` (read-only) is set to the most recent design's `design_id` when one exists, otherwise null. The work-loop never mutates the design.

## 18. Eval requirements

`focusa_call_stack_design` is auditable by:

- `audit-focusa-tool-suite-safe.mjs` — must report `status=passed` with 65 contracts.
- `audit-focusa-tool-implementation-spec-gaps.mjs` — must report no spec gaps.
- `validate-focusa-tool-contracts.mjs` — must report `passed` with `tools=65 contracts=65`.
- A future `audit-focusa-call-stack-designs.mjs` (Spec 103 §19) checks that recent designs are bounded (≤ 1KB JSON), have valid `project_root`, and have a non-empty `entry.name`.

## 19. Performance constraints

- Latency: `focusa_call_stack_design` must return within 200ms hot-path (idempotent template + jsonl append).
- Caching: identical `(project_root, mission, entry_name)` re-uses the most recent design; cache hit returns in <50ms.
- Budget: the design envelope is ≤ 1KB JSON; deeper content lives in the optional `notes` field (≤ 2KB) and the workpoint's evidence store (which is bounded per Spec 100/101).
- Pressure-aware: under `resource_mode=lowmem`, the tool refuses new design writes and returns `failure_class=resource_pressure`; it always serves the cached `recent` view.

## 20. Proof and release requirements

- A new audit script `scripts/audit-focusa-call-stack-designs.mjs` is added in v0.
- The Pi tool contract is added to `docs/current/focusa-tool-contracts.json` and `docs/current/focusa-tool-choreography.json`.
- A new doc page `docs/focusa-tools/tools/focusa_call_stack_design.md` is added.
- The README is updated to mention the 65-tool surface and the new spec.
- The `BENEFITS.md` public doc gains a section on call stack design.

## 21. Safety rules

- Designs never carry secrets. The `notes` field is bounded and redacted by the daemon's standard PII redaction pass.
- The design does not influence Workpoint or Trajectory state unless `attach_to_workpoint` / `attach_to_stg` are explicitly `true`.
- The standard agent-runtime path blocklist applies: a design attached to a `/root/pi-mono` or similar agent runtime path is rejected with `scope_mismatch`.

## 22. Implementation phases

### Phase 1 — Schema + daemon route

- Add `CallStackDesign` envelope in `crates/focusa-core/src/types.rs`.
- Add `routes/call_stack.rs` with `POST /v1/call-stack/design`.
- Add `data/call-stack-designs/{project_root_hash}/designs.jsonl` append-only persistence.
- Wire into `server.rs` and `routes/mod.rs`.

### Phase 2 — Pi extension tool

- Add `focusa_call_stack_design` in `apps/pi-extension/src/tools.ts` using the `spec80Result` template (with the new `piToolText` human-readable format).
- Add the contract entry in `apps/pi-extension/src/tool-contracts.ts`.
- Add the choreography entry in `docs/current/focusa-tool-choreography.json`.

### Phase 3 — Docs + audits

- Add `docs/focusa-tools/tools/focusa_call_stack_design.md` (Purpose, When to use, Example, Expected result, Failure recovery, Contract summary, Next tools).
- Add `scripts/audit-focusa-call-stack-designs.mjs` for v0.5.
- Update `README.md` to 65 tools and reference this spec.

### Phase 4 — Verification tool (deferred to next slice)

- `focusa_call_stack_verify` compares a `CallStackDesign` against the actual route/tool surface and emits `focusa_evidence` for the drift.
- Choreography: `focusa_call_stack_design` → `focusa_call_stack_verify` → `focusa_evidence_capture`.
