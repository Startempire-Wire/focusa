# Spec 106 — Focusa Vision Tightening

Status: proposed-implementation-round
Source: operator steering, Focusa Vision tightening round
Date: 2026-06-14
Scope: vocabulary, authority, scoping, docs, tool surfaces, Context Cognition, Context Authority, Call Stack Design, pairing, public stream, release proof, security/trust, adapter contracts, evals, menubar, golden workflow.

## 0. Core Directive

Focusa MUST NOT simplify away its vocabulary, trajectory ladder, or cognitive architecture terms.

The goal is not to reduce terms. The goal is to make every term canonical, disciplined, enforced, and operationally useful.

Focusa should be tightened as:

> Focusa is a local cognitive runtime for systematic AI execution. It preserves meaning, scope, trajectory, evidence, and continuation across long-running agent work.

The strongest product phrasing remains:

> Kill the chat. Keep the mission.

Focusa’s core product identity should be:

> A local-first mission cohesion layer for AI coding agents.

Internally, and in serious docs/tooling, Focusa MUST preserve full systematic language:

```text
Focus State
Focus Stack
Workpoint
ProjectIdentity
Continuity ID
HLT
MLG
STG
Waypoints
Evidence Ref
Context Cognition
Context Authority
Call Stack Design
Focus Gate
Intuition Engine
Reference Store
Expression Engine
Canonical
Advisory
Degraded
```

The glossary is the vocabulary authority. Alternate simplified terms that weaken precision are not allowed.

## 1. Preserve the Full Trajectory Hierarchy

Focusa MUST preserve this canonical hierarchy:

```text
HLT → MLG → STG → Waypoints → Workpoint
```

These parts complement each other and are necessary for systematic execution.

### 1.1 Required framing

#### HLT — High-Level Trajectory

The ultimate project direction / north star. Describes what the project is trying to become.

#### MLG — Mid-Level Goal

An intermediate milestone derived from the HLT. Groups related STGs and keeps multi-step progress aligned.

#### STG — Short-Term Goal

The immediate bounded goal derived from the HLT through the current MLG/context. Guides the next bounded work slice.

#### Waypoint

A concrete progress marker/checkpoint along an MLG/STG path. Helps agents know where they are and what proof remains.

#### Workpoint

The canonical immediate continuation contract. Converts trajectory into exact executable next action.

### 1.2 Authority rule

- HLT, MLG, STG, and Waypoints steer orientation.
- Workpoint is canonical immediate continuation authority.
- Evidence proves waypoint/STG progress.
- Operator steering wins over all trajectory projections.
- Project/session merging is never allowed unless `project_root + continuity_id` match.

### 1.3 Correction to prior recommendation

Do NOT simplify the trajectory ladder into generic “mission/milestone/next step” language as a replacement.

Use:

> Progressive disclosure without vocabulary dilution.

Meaning:

- Beginner docs may introduce terms gradually.
- UI may add short helper text.
- Canonical terms remain intact.
- Agents, specs, tests, tool contracts, and API envelopes use canonical vocabulary.

## 2. Treat the Glossary as a Locked Operating Language

The glossary is not just documentation. It is the vocabulary authority for the whole system.

### 2.1 Required action

Add a Glossary Compliance Gate scanning:

- `README.md`
- `BENEFITS.md`
- `docs/current/*`
- `docs/focusa-tools/*`
- Pi tool descriptions
- CLI help text
- API envelope field names
- Menubar labels
- Tool choreography docs
- Agent instructions
- Code comments around authority-bearing surfaces

### 2.2 Flagged issues

- Duplicate definitions.
- Local redefinitions of glossary terms.
- Deprecated or conflicting names.
- Vague replacements for canonical terms.
- Authority contradictions.
- Tool docs using terms without glossary alignment.
- UI labels that imply wrong authority.
- “Simplified” names that obscure canonical meaning.

### 2.3 Invariant

```text
docs/00-glossary.md = vocabulary authority
All docs, tools, UI, API, CLI, and agent instructions conform to it.
```

## 3. Make the Authority Model Impossible to Misunderstand

Focusa has many cognitive and operational surfaces. They must not compete.

### 3.1 Required action

Create one canonical Authority Model doc/table and reference it from:

- README
- glossary
- Context Cognition docs
- Context Authority docs
- Workpoint docs
- Trajectory docs
- tool docs

### 3.2 Authority map

```text
Operator Ask        = current intent authority
ProjectIdentity     = project boundary / scope authority
Continuity ID       = logical workstream authority
Session ID          = temporal runtime metadata only
HLT                 = durable north-star trajectory authority within verified project_root + continuity_id
MLG                 = strategic milestone derived from HLT
STG                 = bounded current goal derived from HLT/MLG/current context
Waypoints           = proof-bearing progress markers
Workpoint           = canonical immediate continuation contract
Evidence Ref        = proof authority
Focus State         = bounded current cognitive state
Focus Stack         = nested attention structure
Context Cognition   = advisory bounded context packet
Context Authority   = mutation-boundary allow/block/ask gate
Project Card        = advisory bootstrap/re-bootstrap intelligence card
Call Stack Design   = advisory/evidence-linkable implementation blueprint
Metacognition       = reusable learning loop, advisory until evaluated/promoted
Prediction          = trackable forecast/calibration signal, advisory until evaluated
Work-loop           = governed execution state, writer-controlled
Operator Steering   = final override authority
```

### 3.3 Required rule

Every tool result that includes any of these terms MUST expose whether its output is:

```text
canonical | advisory | degraded | blocked | stale
```

No advisory/degraded output may be treated as canonical continuation truth.

## 4. Enforce Exact `project_root + continuity_id` Scoping Everywhere

### 4.1 Required invariant

```text
No canonical read/write without verified project_root + continuity_id.
```

### 4.2 Context Cognition hardening

Required:

- Select Workpoint by exact `project_root + continuity_id`.
- Select Trajectory by exact `project_root + continuity_id`.
- If `continuity_id` is missing, return degraded/blocked, not canonical-looking context.
- Prior-project/foreign trajectory can only appear as advisory with warning.

### 4.3 Project Card hardening

Required:

- Project Card only fuses signals matching `project_root + continuity_id`.
- Prediction/metacog/outcome signals are scoped.
- Prior-session context shows scope and confidence.
- Cross-project similarity is advisory only.

### 4.4 HLT Ledger hardening

Required:

- HLT changes remain append-only and scoped.
- HLT history retrieval must not silently become current authority.
- Generic bootstrap HLT remains degraded placeholder.
- Workpoint/current_focus may not populate MLG/STG when HLT is invalid/generic.

### 4.5 Menubar hardening

Required:

- Every visible active item shows project root / canonical project / continuity ID.
- Warnings display when scope is missing, degraded, stale, or mismatched.
- Menubar UI must not create authority by selection alone.

### 4.6 Pi Extension hardening

Required:

- Replace singleton-ish caches with scope-keyed maps.
- Cache key includes at minimum `project_root`, `continuity_id`, and canonical project id/name if available.
- Compaction/model-switch recovery rejects mismatched packets.

### 4.7 Work-loop hardening

Required:

- Writer ownership scoped per project/workstream/loop.
- Daemon-global “current task” is runtime telemetry only, not canonical authority.

### 4.8 Public Stream hardening

Every public card carries:

- project scope status
- continuity ID or redacted scoped identifier
- canonical/advisory/degraded label
- redaction status
- evidence refs if applicable

## 5. Fix Tool Count and Runtime Status Drift

### 5.1 Generated source of truth

Create:

```text
docs/current/generated/tool-surface-summary.md
```

Generated from:

```text
docs/current/focusa-tool-contracts.json
```

Include:

- tool count
- tool families
- API parity count
- CLI parity count
- Pi tool count
- docs coverage count
- current version
- generated timestamp
- source commit SHA

### 5.2 Replace manual counts

Remove or replace hardcoded counts in:

- README.md
- BENEFITS.md
- CURRENT_RUNTIME_STATUS.md
- docs/focusa-tools/README.md
- release docs
- marketing copy
- tool docs

Use:

```md
See docs/current/generated/tool-surface-summary.md for the current generated tool surface.
```

### 5.3 CI/static guard

Fail CI if:

- README contains stale hardcoded tool count.
- BENEFITS contains stale hardcoded tool count.
- CURRENT_RUNTIME_STATUS contradicts contract registry.
- Tool contract count and docs count diverge.

## 6. Clarify the Product Wedge Without Weakening Architecture

### 6.1 Primary wedge

```text
Focusa keeps AI coding agents on-mission through compaction, drift, and proof loss.
```

### 6.2 Product pillars

1. Preserve meaning — Focus State, Focus Stack, Expression Engine.
2. Preserve continuation — Workpoint, Session Transfer, Compaction recovery.
3. Preserve trajectory — HLT, MLG, STG, Waypoints, HLT Ledger.
4. Preserve proof — Evidence Ref, Reference Store, Workpoint evidence linking.
5. Preserve scope — ProjectIdentity, Continuity ID, Context Authority.
6. Preserve relevant context — Context Cognition, Context Curator, Cognition Optimizer.
7. Preserve implementation structure — Call Stack Design, future Call Stack Verify.
8. Preserve trust — canonical/advisory/degraded envelopes, preflight gates, writer ownership, audit trails.

### 6.3 Positioning statement

```text
Focusa is a local-first cognitive runtime that gives AI coding agents a durable operating language for systematic execution: scope, trajectory, Workpoints, evidence, context, and authority.
```

## 7. Make Context Cognition Clearly Advisory, Scoped, and Measurable

### 7.1 Required principles

Context Cognition:

- Is advisory by default.
- Is `canonical=false`.
- Never mutates Focus State.
- Never replaces Workpoint.
- Never overrides operator steering.
- Never marks inferred context as proof.
- Exposes selected and excluded context.
- Exposes stale/degraded/mismatch state.
- Requires `project_root + continuity_id` for canonical-scope matching.

### 7.2 Required implementation hardening

- Exact-match Workpoint by `project_root + continuity_id`.
- Exact-match Trajectory by `project_root + continuity_id`.
- Add stale context scoring.
- Add contradiction flags.
- Add missing evidence map.
- Add selected/excluded reasons in every render.
- Add explicit `scope_status`.
- Add `context_budget` and `tokens_used`.
- Add `do_not_drift` boundaries.
- Add `source_refs` and `rehydrate_refs`.

### 7.3 Required eval cases

Add/expand tests for:

- wrong project scope
- missing continuity_id
- stale Workpoint packet
- stale HLT
- missing evidence
- over-broad context selection
- under-selected critical file
- ontology relation error
- bad trajectory gap
- invalid next action
- transcript-tail authority attempt
- cross-project context bleed

### 7.4 Marketing boundary

Do not call Context Cognition “AI reasoning optimizer” yet.

Use:

```text
Bounded context curation with eval-backed promotion.
```

## 8. Make Context Authority Mandatory Before Risky Mutation

### 8.1 Required agent rule

Before risky mutation:

```text
1. classify prompt mode
2. inspect environment contract
3. inspect runtime inventory
4. inspect binary compatibility if relevant
5. run action preflight
6. mutate only if verdict allows
```

### 8.2 Required preflight triggers

- binary replacement
- daemon restart
- deploy
- release publish
- git push
- destructive file operation
- database migration
- broad refactor
- cross-project file edit
- generated code overwrite
- secret/config change
- live service action
- pairing/install/update ambiguity

### 8.3 Required verdicts

```text
allow
block
ask_operator
verify_first
diagnosis_only
planning_only
```

### 8.4 Required UX

- Pi shows why mutation is blocked.
- Menubar displays block reason and safe alternative.
- CLI returns machine-readable JSON plus human-readable summary.
- Tool envelopes provide next safe tools.

## 9. Promote Call Stack Design as a Core Execution Artifact

### 9.1 Core claim

```text
Before an agent writes a feature, it writes the call stack.
```

### 9.2 Required shape

```text
entry → handlers → services → adapters → storage → output
```

### 9.3 Required behavior

- Call Stack Design is advisory by default.
- It becomes evidence only when explicitly attached.
- It never silently mutates Workpoint or Trajectory.
- It links to active Workpoint when approved.
- It aligns to current STG.
- It is retrievable by design ID.
- It is visible in public stream when redaction allows.

### 9.4 Required next implementation

Implement `focusa_call_stack_verify`.

It reads Call Stack Design and checks implementation drift against actual surfaces:

- entry surface exists
- route/tool/CLI command exists
- handler exists
- service/adapters exist or are marked planned
- storage path exists or is planned
- output envelope matches `tool_result_v1` expectations
- evidence refs exist if claimed
- design still aligns with active STG/Workpoint

### 9.5 Required templates

Add templates for:

- Pi tool
- CLI command
- HTTP route
- background worker
- webhook
- auth flow
- database migration
- release workflow
- menubar action
- context-cognition route
- evidence capture flow

### 9.6 CLI parity

Finish CLI support if incomplete:

```bash
focusa call-stack design
focusa call-stack verify
focusa call-stack list
focusa call-stack show
```

## 10. Keep HLT / Trajectory Hardening Front and Center

### 10.1 Required rules

- Generic bootstrap HLT is degraded placeholder, not authority.
- HLT mutation requires explicit operator steering or durable supersession evidence.
- HLT is scoped by `project_root + continuity_id`.
- HLT ledger is append-only.
- MLG/STG/Waypoints derive from HLT.
- Workpoint is executable next slice.
- Operator steering wins.

### 10.2 Required visibility

Every agent-facing trajectory render shows:

```text
HLT:
MLG:
STG:
Waypoints:
Workpoint:
Scope:
Status:
Canonical/advisory/degraded:
Evidence refs:
```

### 10.3 Required alerts

Trigger visible warnings when:

- HLT changes.
- HLT is missing.
- HLT is generic placeholder.
- continuity mismatch occurs.
- current trajectory is stale.
- foreign/prior project trajectory appears.
- Workpoint and Trajectory disagree.
- current ask conflicts with saved trajectory.

## 11. Harden Device Pairing Before Public Release

Create:

```text
docs/current/DEVICE_PAIRING_THREAT_MODEL.md
```

### 11.1 Threat model coverage

- QR payload leakage
- connect_id guessing
- nonce replay
- token theft
- token leakage through status polling
- TLS assumptions
- public pairing URL exposure
- local callback abuse
- cross-device approval confusion
- revoked token reuse
- expired room cleanup
- rate limiting
- brute-force code attempts
- logs accidentally storing tokens

### 11.2 Required hardening

- Rate limit pairing endpoints.
- Enforce TLS for public Focusa URL.
- Bind nonce to connect session.
- Ensure token is only returned to intended device.
- Consider one-time token retrieval.
- Redact token from logs.
- Add token scopes.
- Add server-side token expiration validation.
- Add revocation proof.
- Add audit ledger view.
- Add connect room cleanup.
- Add pairing diagnostics that do not expose secrets.
- Ensure pairing failure never becomes install/update task substitution.

## 12. Create One Golden Workflow

### 12.1 Golden workflow

```text
1. Verify ProjectIdentity
2. Load or define HLT / Trajectory Hierarchy
3. Create or resume Workpoint
4. Generate Context Cognition packet
5. Create Call Stack Design
6. Run implementation
7. Capture Evidence Refs
8. Link evidence to Workpoint
9. Evaluate prediction/metacog outcomes
10. Save session transfer
11. Resume after compaction/model switch
12. Produce final report with proof
```

### 12.2 Required assets

Create/maintain:

```text
docs/current/GOLDEN_WORKFLOW.md
scripts/demo-golden-workflow.sh
tests/golden_workflow_static_test.sh
tests/golden_workflow_live_safe_test.sh
```

### 12.3 Required UI/agent support

- Pi tool choreography follows this route.
- Menubar shows golden workflow state.
- README quickstart demonstrates this flow.
- Public stream represents this flow cleanly.
- Every tool identifies where it sits in the workflow.

## 13. Use Progressive Disclosure, Not Term Deletion

### 13.1 Documentation layers

```text
README              = product and core terms
docs/00-glossary.md = authoritative vocabulary
docs/current/*      = current operational truth
docs/focusa-tools/* = one tool per doc, glossary-compliant
spec docs           = full systematic architecture
agent instructions  = operational usage and authority rules
UI                  = canonical labels with concise helper text
```

### 13.2 UI helper examples

```text
HLT — High-Level Trajectory: project north star
MLG — Mid-Level Goal: milestone derived from HLT
STG — Short-Term Goal: immediate bounded goal
Workpoint — current continuation contract
Evidence Ref — proof handle
Context Authority — mutation preflight gate
```

## 14. Build a Benchmark Story for “Makes Agents Smarter”

### 14.1 Benchmark categories

- Continuity: resume accuracy after compaction; Workpoint recovery after model switch; correct next action after long session.
- Scope: wrong-project mutation prevention; same project / different continuity rejection; broad root rejection.
- Evidence: evidence recall rate; claims with linked proof; missing evidence detection.
- Context: context selection precision/recall/F1; under-selected critical file rate; over-budget exclusion correctness.
- Execution: Call Stack Design adherence; implementation drift from blueprint; STG/Waypoint completion rate.
- Learning: prediction calibration improvement; metacog lesson reuse; repeated mistake reduction.
- Safety: risky mutation blocked correctly; planning prompt does not mutate; pairing does not become binary install.

### 14.2 Required output

Create:

```text
docs/current/FOCUSA_AGENT_INTELLIGENCE_EVALS.md
tests/evals/*
scripts/run-agent-intelligence-evals.sh
```

## 15. Make Menubar Boringly Useful

### 15.1 Main panel shows

```text
ProjectIdentity
Continuity ID
HLT
MLG
STG
Current Workpoint
Next action
Evidence count
Scope status
Context Authority status
Daemon/CLI version status
Pairing status
Warnings
Resume/copy button
```

### 15.2 Advanced panels

- Context Cognition packet
- Tool registry
- HLT history
- Workpoint history
- Evidence refs
- Pairing diagnostics
- Resource mode
- Public stream status

### 15.3 Rule

Menubar displays state and calls scoped API routes. UI selection alone does not create authority.

## 16. Define a Generic Agent Adapter Contract

Create:

```text
docs/current/AGENT_ADAPTER_CONTRACT.md
```

### 16.1 Minimal adapter contract

Every agent adapter supports:

1. Read awareness card.
2. Verify project identity.
3. Resume Workpoint.
4. Create Workpoint checkpoint.
5. Capture evidence.
6. Link evidence.
7. Run Context Authority preflight.
8. Render Context Cognition compact packet.
9. Surface `tool_result_v1` envelopes.
10. Respect canonical/advisory/degraded states.

### 16.2 Target adapters

- Pi
- Codex CLI
- Claude Code
- OpenCode
- OpenClaw
- generic shell agent
- MCP-compatible agents

### 16.3 Non-negotiable rule

Adapters stay thin. Focusa daemon/core remains cognitive authority.

## 17. Strengthen Public Stream and Redaction

Create:

```text
docs/current/PUBLIC_STREAM_REDACTION_POLICY.md
```

### 17.1 Every public card declares

```text
schema
project identity display name
redacted scope id
canonical/advisory/degraded status
tool family
evidence refs if public-safe
redaction status
secret scan status
publish_allowed
```

### 17.2 Never publish by default

- raw logs
- secrets
- tokens
- private file contents
- unredacted project paths if sensitive
- raw diffs unless explicitly allowed
- browser diagnostics with sensitive URLs
- environment contracts with host secrets

## 18. Tighten Release / Version / Proof Story

### 18.1 Release invariant

A release is not complete until:

1. release stamp is generated
2. CLI/daemon/core/menubar versions match
3. generated docs updated
4. tool contract summary updated
5. CI green
6. release workflow green
7. proof bundle captured
8. runtime status updated from generated source

### 18.2 Add/maintain

```text
scripts/stamp-release-version
scripts/generate-current-runtime-status
scripts/generate-tool-surface-summary
scripts/verify-doc-version-consistency
```

### 18.3 Avoid manual version drift

Version values should be stamped or generated, not hand-edited across many files.

## 19. Improve Security / Trust Docs

Add or harden:

```text
docs/current/SECURITY_MODEL.md
docs/current/DEVICE_PAIRING_THREAT_MODEL.md
docs/current/TOKEN_AND_SECRET_HANDLING.md
docs/current/PUBLIC_STREAM_REDACTION_POLICY.md
docs/current/LOCAL_FIRST_DATA_MODEL.md
docs/current/MULTI_AGENT_SCOPE_MODEL.md
```

Cover:

- local data storage
- token storage
- API auth
- pairing tokens
- public URL exposure
- redaction
- scope isolation
- agent mutation boundaries
- audit logs
- append-only ledgers
- destructive action policies

## 20. Keep the “Not” Boundaries Clear

### 20.1 Focusa is

- local cognitive runtime
- mission cohesion layer
- focus/context operating layer
- governance and continuity substrate
- evidence and trajectory system

### 20.2 Focusa is not

- model
- chatbot
- generic RAG system
- scheduler
- autonomous task authority
- hidden cloud memory service
- replacement for agent harnesses
- automatic silent memory mutation system

### 20.3 Agent instruction boundary

Agents must never describe Focusa as:

```text
an LLM
a chatbot
a RAG database
an autonomous task runner
a replacement for Codex/Claude/Pi/etc.
a cloud memory product
```

Use:

```text
Focusa gives agents durable mission state, scoped context, proof, and mutation authority.
```

## 21. Priority Plan

### 21.1 P0 — Immediate

1. Preserve full trajectory ladder and glossary vocabulary.
2. Add Glossary Compliance Gate.
3. Build canonical Authority Model doc/table.
4. Enforce exact `project_root + continuity_id` scoping everywhere.
5. Fix tool-count/runtime-status docs drift.
6. Make Context Authority mandatory before risky mutation.
7. Update Context Cognition to exact-match scope.
8. Align README/BENEFITS/current docs with generated tool surface.
9. Add Golden Workflow doc/script/test.
10. Keep generic HLT degraded-placeholder protections.

### 21.2 P1 — Next

11. Finish Context Cognition eval and optimizer hardening.
12. Add Call Stack Verify.
13. Add Call Stack CLI parity.
14. Add device pairing threat model and endpoint hardening.
15. Add public stream redaction policy.
16. Add Agent Adapter Contract.
17. Add benchmark/eval story for agent intelligence.
18. Make menubar mission-centered and warning-driven.
19. Add release/version/proof generation checks.
20. Add security/trust docs.

### 21.3 P2 — Productization

21. Build polished first-run flow.
22. Build public demo around Golden Workflow.
23. Add adapter examples for non-Pi agents.
24. Add commercial packaging docs.
25. Add installer/update policy.
26. Add migration and backup docs.
27. Add team/multi-agent federation plan.
28. Add public proof bundle viewer.
29. Add glossary-linked docs UI.
30. Add dashboard for eval metrics.

## 22. Final Updated Vision

Focusa should not become simpler by removing terms.

Focusa should become stronger by making its vocabulary a formal execution grammar.

### 22.1 Recommended final vision statement

> Focusa is a local-first cognitive runtime for systematic AI execution. It gives AI coding agents a durable operating language for scope, trajectory, Workpoints, evidence, context, and authority, so long-running work survives compaction, drift, model switches, and multi-agent handoffs.

### 22.2 Recommended short product line

> Kill the chat. Keep the mission.

### 22.3 Recommended technical line

> Meaning lives in Focus State, progress follows the Trajectory Hierarchy, proof lives in Evidence Refs, and execution resumes from Workpoints.

## 23. Current Implementation Reference Map

This section maps every Spec 106 requirement family to current repository anchors. Status values are implementation-audit snapshots, not final acceptance.

### 23.1 Trajectory Hierarchy / HLT Vocabulary

Status: partial; core trajectory primitives exist and need stricter glossary/visibility enforcement.

References:

- `docs/00-glossary.md`
- `docs/102-trajectory-ladder-consolidated-spec.md`
- `crates/focusa-api/src/routes/trajectory.rs`
- `crates/focusa-core/src/trajectory.rs`
- `apps/pi-extension/src/tools.ts`
- `apps/pi-extension/src/state.ts`
- `apps/menubar/src/lib/components/TrajectoryPeek.svelte`
- `docs/focusa-tools/tools/focusa_trajectory_view.md`
- `docs/focusa-tools/tools/focusa_trajectory_define_goal.md`

### 23.2 Glossary Compliance Gate

Status: gap; glossary exists, but no dedicated scanner/CI gate enforces vocabulary compliance across docs/tools/UI/API/CLI/comments.

References:

- `docs/00-glossary.md`
- `README.md`
- `BENEFITS.md`
- `docs/current/*`
- `docs/focusa-tools/*`
- `apps/pi-extension/src/tool-contracts.ts`
- `crates/focusa-cli/src/main.rs`
- `apps/menubar/src/lib/components/*`

### 23.3 Authority Model

Status: partial; authority docs/routes exist, but no single canonical authority model table is referenced everywhere.

References:

- `docs/current/CONTEXT_AUTHORITY_CURRENT.md`
- `docs/current/CONTEXT_AUTHORITY_ARCHITECTURE_WORKORDER_SPEC_2026-06-12.md`
- `docs/current/FOCUSA_AUTHORITY_SURFACE_REGISTRY.generated.json`
- `crates/focusa-api/src/routes/context_authority.rs`
- `apps/pi-extension/src/awareness.ts`
- `apps/pi-extension/src/state.ts`
- `tests/pi_extension_runtime_authority_test.mts`
- `tests/scope_arbitration_runtime_test.mts`

### 23.4 Exact `project_root + continuity_id` Scoping

Status: partial; multiple surfaces already gate by project scope, but Spec106 requires exact-match audit across every authority-bearing read/write.

References:

- `crates/focusa-api/src/routes/context_cognition.rs`
- `crates/focusa-api/src/routes/project.rs`
- `crates/focusa-api/src/routes/trajectory.rs`
- `crates/focusa-api/src/routes/workpoint.rs`
- `crates/focusa-api/src/routes/work_loop.rs`
- `apps/pi-extension/src/state.ts`
- `apps/pi-extension/src/session.ts`
- `apps/menubar/src/lib/api.ts`
- `apps/menubar/src/lib/components/WorkpointPeek.svelte`
- `tests/current_ask_project_override_runtime_test.mts`
- `tests/pi_project_root_inference_test.mts`
- `tests/pi_session_project_switch_ledger_runtime_test.mts`
- `tests/scope_routing_regression_eval.sh`

### 23.5 Tool Count / Runtime Status Drift

Status: gap; `focusa-tool-contracts.json` is a registry source, but no generated Markdown summary or stale-count CI guard exists.

Current registry snapshot:

```text
tool_count=79
api_parity_count=75
cli_parity_count=63
pi_tool_count=79
docs_coverage_count=79
```

References:

- `docs/current/focusa-tool-contracts.json`
- `docs/current/focusa-tool-choreography.json`
- `docs/current/CURRENT_RUNTIME_STATUS.md`
- `docs/focusa-tools/README.md`
- `scripts/validate-docs-runtime-parity.mjs`
- `README.md`
- `BENEFITS.md`

### 23.6 Product Wedge / Positioning

Status: partial; `README.md` and `BENEFITS.md` contain product language, but need Spec106 alignment and generated tool-summary references.

References:

- `README.md`
- `BENEFITS.md`
- `docs/current/FOCUSA_AGENT_UTILITY_CARD.md`
- `docs/current/AGENT_AWARENESS_QUICKSTART.md`

### 23.7 Context Cognition

Status: partial; routes/tools/eval machinery exist, but Spec106 requires exact scope enforcement, stale scoring, contradiction flags, missing-evidence map, and expanded eval cases.

References:

- `docs/100-context-cognition-spec.md`
- `crates/focusa-api/src/routes/context_cognition.rs`
- `apps/pi-extension/src/tools.ts`
- `docs/focusa-tools/tools/focusa_context_cognition.md`
- `docs/focusa-tools/tools/focusa_context_cognition_render.md`
- `docs/focusa-tools/tools/focusa_context_cognition_curate.md`
- `tests/spec100_eval_optimizer_static_test.py`
- `tests/scope_routing_regression_eval.sh`

### 23.8 Context Authority Risky Mutation Preflight

Status: partial; Context Authority exists and Pi has runtime authority tests, but Spec106 needs mandatory preflight triggers and consistent CLI/Menubar/Pi UX.

References:

- `docs/current/CONTEXT_AUTHORITY_CURRENT.md`
- `crates/focusa-api/src/routes/context_authority.rs`
- `crates/focusa-cli/src/main.rs`
- `apps/pi-extension/src/awareness.ts`
- `apps/menubar/src/lib/stores/gate.svelte.ts`
- `tests/pi_extension_runtime_authority_test.mts`
- `tests/context_authority_*` if present in future revisions

### 23.9 Call Stack Design / Verify

Status: partial; Spec103 v0 design route/tool/ledger shipped. Spec106 adds `focusa_call_stack_verify`, templates, retrieval, and CLI parity.

References:

- `docs/103-call-stack-architecture-blueprint-spec.md`
- `crates/focusa-api/src/routes/call_stack.rs`
- `crates/focusa-core/src/types.rs`
- `crates/focusa-core/src/persistence.rs`
- `apps/pi-extension/src/tools.ts`
- `docs/focusa-tools/tools/focusa_call_stack_design.md`
- `tests/spec103_post_compaction_recovery_surfaces_test.sh`

### 23.10 HLT / Trajectory Hardening

Status: partial; trajectory routes and HLT ledger exist, but generic placeholder and mismatch warnings need stronger universal enforcement.

References:

- `crates/focusa-api/src/routes/trajectory.rs`
- `crates/focusa-api/src/routes/project.rs`
- `crates/focusa-core/src/trajectory.rs`
- `hlt-ledger/`
- `apps/pi-extension/src/turns.ts`
- `apps/pi-extension/src/state.ts`
- `apps/menubar/src/lib/components/TrajectoryPeek.svelte`

### 23.11 Device Pairing

Status: hardened in this slice; pairing tools/routes/UI exist, threat model is consolidated in `docs/53-focusa-device-pairing-spec.md`, and endpoint hardening covers CSPRNG tokens, scope allowlist, URL validation, safe labels, unsafe host rejection, single-use codes, and append-only revoke/list audit behavior.

Required hardening proof:

- Static guard: `tests/device_pairing_threat_model_static_test.sh`
- Live-safe guard: `tests/device_pairing_endpoint_hardening_live_safe_test.sh`
- Compile guard: `cargo check -q -p focusa-api`

References:

- `crates/focusa-api/src/routes/device_pairing.rs`
- `crates/focusa-cli/src/commands/device_pairing.rs`
- `apps/pi-extension/src/tools.ts`
- `apps/menubar/src/lib/components/PairingPanel.svelte`
- `apps/menubar/src/lib/stores/pairing.svelte.ts`
- `docs/focusa-tools/tools/focusa_device_pair_start.md`
- `docs/focusa-tools/tools/focusa_device_pair_complete.md`
- `docs/focusa-tools/tools/focusa_device_pair_status.md`

### 23.12 Golden Workflow

Status: gap; related choreography exists, but no canonical Golden Workflow doc/script/static/live-safe tests exist.

References:

- `docs/current/focusa-tool-choreography.json`
- `docs/current/AGENT_COMMAND_COOKBOOK.md`
- `apps/pi-extension/src/tool-contracts.ts`
- `tests/golden_tasks_eval.sh`
- `tests/golden_tasks_comparative_eval.sh`

### 23.13 Progressive Disclosure

Status: partial; glossary/docs/tool pages exist, but no glossary-linked docs/UI compliance pass enforces canonical labels plus helper text.

References:

- `docs/00-glossary.md`
- `README.md`
- `docs/focusa-tools/*`
- `apps/menubar/src/lib/components/*`
- `apps/pi-extension/skills/focusa/SKILL.md`

### 23.14 Agent Intelligence Evals

Status: partial; scattered eval scripts exist, but no unified agent-intelligence eval story and runner exist.

References:

- `tests/golden_tasks_eval.sh`
- `tests/golden_tasks_comparative_eval.sh`
- `tests/scope_routing_regression_eval.sh`
- `tests/save_point_function_evaluation_test.py`
- `tests/eval*` when added

### 23.15 Menubar Operational Truth

Status: partial; Menubar components exist for trajectory/workpoint/context/pairing/gate, but main panel needs mission-centered warning-driven shape.

References:

- `apps/menubar/src/routes/+page.svelte`
- `apps/menubar/src/lib/components/TrajectoryPeek.svelte`
- `apps/menubar/src/lib/components/WorkpointPeek.svelte`
- `apps/menubar/src/lib/components/ContextCognitionPeek.svelte`
- `apps/menubar/src/lib/components/PairingPanel.svelte`
- `apps/menubar/src/lib/stores/gate.svelte.ts`
- `apps/menubar/src/lib/stores/diagnostics.svelte.ts`

### 23.16 Generic Agent Adapter Contract

Status: hardened in this slice; `docs/current/AGENT_ADAPTER_CONTRACT.md` defines the harness-agnostic minimum adapter contract, target adapter classes, authority boundaries, risky mutation preflight, and failure behavior. `NON_PI_AGENT_FOCUSA_USAGE.md` now points non-Pi agents to this contract.

Required adapter capabilities:

- read awareness card
- verify project identity
- resume Workpoint
- create Workpoint checkpoint
- capture evidence
- link evidence
- run Context Authority preflight
- render Context Cognition compact packet
- surface `tool_result_v1` envelopes
- respect canonical/advisory/degraded states

References:

- `docs/current/AGENT_ADAPTER_CONTRACT.md`
- `docs/current/NON_PI_AGENT_FOCUSA_USAGE.md`
- `apps/pi-extension/src/*`
- `apps/pi-extension/skills/focusa/SKILL.md`
- `docs/current/AGENT_AWARENESS_QUICKSTART.md`
- `docs/current/AGENT_COMMAND_COOKBOOK.md`
- `tests/agent_adapter_contract_static_test.sh`

### 23.17 Public Stream / Redaction

Status: hardened in this slice; `docs/current/PUBLIC_STREAM_REDACTION_POLICY.md` defines deny-by-default publication, required public card fields, redaction rules, and publish gates; `/v1/awareness/card` now includes `public_stream_policy` plus a rendered `PUBLIC_CARD` block.

Required public-card fields:

- `schema`
- `project_identity_display_name`
- `redacted_scope_id`
- `canonical_status`
- `tool_family`
- `evidence_refs_public_safe`
- `redaction_status`
- `secret_scan_status`
- `publish_allowed`

References:

- `docs/current/PUBLIC_STREAM_REDACTION_POLICY.md`
- `crates/focusa-api/src/routes/awareness.rs`
- `tests/public_stream_redaction_policy_static_test.sh`
- `tests/public_stream_redaction_policy_live_safe_test.sh`
- `apps/menubar/src/lib/components/ProofPeek.svelte`

### 23.18 Release / Version / Proof Story

Status: partial; some version/runtime parity scripts exist, but release stamp/tool summary/runtime status generation is incomplete.

References:

- `scripts/stamp-menubar-version.py`
- `scripts/validate-docs-runtime-parity.mjs`
- `docs/current/CURRENT_RUNTIME_STATUS.md`
- `docs/current/CLI_REFERENCE_CURRENT.md`
- `docs/current/API_REFERENCE_CURRENT.md`
- `.github/workflows/*`

### 23.19 Security / Trust Docs

Status: partial; some API/security docs exist, but Spec106 trust-doc set is incomplete.

References:

- `docs/current/API_ROUTE_PERMISSION_MATRIX.md`
- `docs/current/API_RESOURCE_LIMITS.md`
- `docs/current/DYNAMIC_API_SECURITY_SMOKE.md`
- `docs/current/DATA_RETENTION_BACKUP_DELETION_POLICY.md`
- `crates/focusa-api/src/routes/permissions.rs`
- `tests/security_api_resource_limits_static_test.sh`
- `tests/security_api_route_permission_matrix_static_test.py`

### 23.20 “Not” Boundaries

Status: partial; product docs mention local cognitive runtime, but agent instructions and docs need explicit forbidden descriptions.

References:

- `README.md`
- `BENEFITS.md`
- `apps/pi-extension/skills/focusa/SKILL.md`
- `docs/current/AGENT_AWARENESS_QUICKSTART.md`
