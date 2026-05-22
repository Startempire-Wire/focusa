# 96 — Trajectory Projection and Daemon Stability Spec

**Status:** draft for review
**Priority:** critical
**Owner:** Focusa + Pi integration
**Scope:** preserve current Focusa model; add Trajectory Projection as a navigation view; harden daemon/tool/session behavior.

---

## 1) Why this spec exists

Focusa already has the right core primitives:

- Focus State for compact durable cognitive slots.
- Workpoint for typed short-term continuation.
- Work-loop for governed autonomous execution.
- Ontology/context slices for relevant object/action/evidence routing.
- Metacognition for reusable learning.
- Evidence/reference handles for proof discipline.
- Lineage/tree/snapshots for recovery and comparison.
- Prediction for bounded risk forecasting.

The issue is not that these should be replaced. The issue is that agents need a clearer, more stable projection layer that answers:

```text
Where are we trying to go long-term?
Where are we right now?
What short-term goal moves us from current state toward desired state?
What Workpoint candidate best preserves the next bounded step?
What evidence proves movement?
```

Today, these answers are spread across Focus State, Workpoint, Work-loop status, ontology context, metacognition, and local Pi fallback. During daemon flakiness, session switches, compaction, or project mismatch, the agent can lose the relationship between long-term goal, short-term action, and current project state.

This spec makes **per-project Trajectory Projection and Trajectory tools the north star of Focusa**: every supported agent should receive the project-scoped whole-picture orientation it needs before acting — ProjectIdentity, long-term goal, desired end state, current verified state, active gap, evidence, drift boundaries, and next Workpoint candidate.

Trajectory is the project-level product spine, not a side feature. It gives Focusa a clear role as a cross-agent companion framework: keep any agent oriented, continuous, evidence-grounded, and recoverable inside the correct project while existing primitives retain their authority boundaries.

---

## 2) Core thesis

Trajectory Projection is not a replacement for any existing Focusa primitive.

Trajectory Projection is a **derived cognitive navigation view** that continuously composes existing Focusa state into a current-to-desired-state model:

```text
Existing primitives → Trajectory Projection → Workpoint candidate → Tool/action/evidence → Existing primitives
```

In plain language:

> Keep every current Focusa model, but make those models feed a stable Trajectory Projection that lets an agent hold long-term and short-term goals simultaneously while seeing the current project state and the next gap-closing action.

This is an **evolutionary hardening spec**, not a radical rewrite. The goal is to build on the progress already implemented so every model using Focusa becomes more context-aware, cognitively grounded in the full work state, and capable of accomplishing exact goals with less outside assistance over long-running work.

### 2.1 Critical cognitive invariant: goal-state binding

The most important user-facing improvement is that every model using Focusa must be able to hold the following together at the same time:

1. stable highest long-term goal,
2. desired end state,
3. current verified state,
4. active short-term goal,
5. current gap between state and goal,
6. next bounded Workpoint candidate,
7. evidence proving progress or uncertainty.

Focusa must not inject a short-term task without the larger destination, and must not inject a long-term goal without current-state proof. The model should always see why the current step matters, what it is moving toward, what is already known, what is missing, and what evidence verifies the path.

The highest long-term goal is usually stable for a project/workstream and should not radically change inside normal progress. Short-term goals, milestones, gaps, and Workpoints may evolve frequently; the root long-term goal changes only through explicit operator steering or durable supersession evidence.

This binding is not decorative context. It is the core cognition contract for long-running work and must be clarified continuously during every work session: if any part is missing, stale, or conflicted, Trajectory Projection must mark `context_sufficiency` accordingly and tell the model whether to proceed, verify first, or request operator input.

---

## 3) Non-goals

This spec does not:

- delete or replace Focus State.
- delete or replace Workpoint.
- delete or replace Work-loop.
- rewrite working subsystems for architectural neatness.
- demote previous specs, evidence, tool contracts, or current working primitives.
- turn Pi into a parallel cognitive database.
- weaken Workpoint project/session scope guards.
- make daemon diagnostics part of every hot-path context call.
- allow Focusa outages to block normal coding work.
- make Trajectory Projection canonical ontology truth without reducer-approved events.

---

## 4) Source-aligned design laws

### 4.0 Original-runtime boundary

Focusa becomes a **per-project trajectory intelligence companion framework** for agents, not an agent replacement, scheduler, planner, or automation engine. Trajectory Projection improves project-scoped awareness and continuity, but it must not choose work, switch focus, execute actions, or become task authority. Beads remains task authority; Focus Stack remains attention authority; Focus State remains meaning authority; Workpoint remains immediate continuation authority.

### 4.1 Evolutionary improvement only

Changes must preserve and compound prior Focusa progress. New surfaces should wrap, feed, compose, or clarify existing primitives before introducing new storage or authority. Refactors must be incremental, evidence-backed, and reversible. Any proposal that removes a working primitive, bypasses the reducer, or replaces existing cognition instead of strengthening it is out of scope.

### 4.2 Existing model stays

All existing primitives remain authoritative for their current job:

| Primitive | Keeps responsibility |
|---|---|
| Focus State | compact durable slots: intent, current_focus, decisions, constraints, failures, etc. |
| Workpoint | exact short-term continuation contract: mission, action, targets, blockers, evidence, next slice |
| Work-loop | governed continuous execution, writer ownership, task traversal, stop conditions |
| Ontology/context | typed objects, links, valid next actions, blocked affordances, uncertainty |
| Metacognition | reusable learning, reflection, adjustments, evaluation |
| Evidence/reference handles | proof records and bounded raw output handles |
| Lineage/tree/snapshots | ancestry, recoverability, diff/restore reasoning |
| Prediction | bounded forecasts and calibration |

Trajectory Projection reads these signals; it does not decide, act, schedule, or supersede them.

### 4.3 Focusa remains single cognitive authority

Pi extension remains thin harness glue. Local Pi state may cache and fallback, but degraded local fallback is never canonical.

### 4.4 Hot path must not block on cold diagnostics

Agent context and tool readiness must use bounded hot-path surfaces. Replay, worktree scans, deep diagnostics, and large status reports are cold-path and opt-in.

### 4.5 Fail-safe passthrough

If Focusa daemon is down or partially degraded, the agent continues from:

1. latest operator instruction,
2. current repo/worktree/beads,
3. local noncanonical Focusa shadow,
4. last known Workpoint/Trajectory Projection packet if scope-compatible.

No non-safety Focusa tool failure should stop normal work.

### 4.6 Scope safety remains mandatory

Project/session mismatch must still prevent stale packet injection. The fix is better identity and checkpoint parity, not weaker guards.

### 4.7 Project identity must be verified from multiple sources

Focusa must accurately identify the active project without relying on a single signal such as `cwd`. Project identity must be derived from a multi-source ProjectIdentity record and verified by quorum/fingerprint before Workpoint, Trajectory Projection, Work-loop, Focus State projection, or tool scope is treated as canonical. Broad runtime directories such as `/root` are never sufficient canonical project roots; they must be quarantined unless replaced by an exact project/repo root. Focus Stack current-frame reads, Focus Slice injection, and Workpoint resume must not fall back from a scoped query to a global active frame.

---

## 5) Current conflicts to fix

### C1 — Full work-loop status mixes hot and cold paths

Current full `/v1/work-loop/status` assembles hot state plus worktree scans, alternate ready work, replay summaries, Workpoint replay, secondary-loop eval payloads, and governance details.

Conflict:

- Daemon docs require hot path not be blocked by workers/diagnostics.
- Tool proof and Pi tools treat status as a safe read, but full status can time out.

Requirement:

- Split work-loop status into bounded hot-path and explicit deep diagnostic routes.

### C2 — Safe proof harness probes unstable route

Spec91 safe fixtures must not regress to full `/v1/work-loop/status`. That route is too expensive for default proof.

Requirement:

- Safe fixtures must use bounded summary route.
- Deep proof must be separate and allowed to report diagnostic degradation without failing basic tool contract parity.

### C3 — Checkpoint/resume identity parity is incomplete

Some checkpoint paths include `session_id` but omit `project_root`; some omit both. Resume validates both.

Conflict:

- Scope guard expects consistent checkpoint/resume identity.
- Current code can create a packet that later rejects under its own resume rules.

Requirement:

- Every Workpoint/Trajectory Projection checkpoint and resume request must include one shared identity envelope.

### C4 — Tool failure taxonomy is too coarse

Current user-visible failures conflate:

- daemon unavailable,
- expensive endpoint timeout,
- writer conflict,
- approval gate,
- validation rejection,
- scope mismatch,
- noncanonical fallback.

Requirement:

- Every Focusa tool result must classify failure type and retry posture.

### C5 — Temporary operator steering vs durable constraints is ambiguous

`focusa_constraint` rejected temporary steering like “evaluate only.” That was correct validator behavior, but instructions are ambiguous.

Requirement:

- Distinguish temporary steering from durable constraints in skill docs, tool docs, and validation examples.

### C6 — Project identity currently depends too much on cwd/session hints

Current Pi/session flows use `ctx.cwd`, `S.sessionCwd`, `process.cwd()`, git/beads discovery, or persisted session data in different places. These are useful signals, but none is sufficient alone.

Conflict:

- Workpoint scope guards require accurate `project_root`.
- Trajectory Projection and Work-loop need stable project identity across compaction, session resume, daemon restart, and tool calls.
- A cwd-only or stale persisted-session identity can point Focusa at the wrong project.

Requirement:

- Add canonical ProjectIdentity discovery and verification with multiple independent signals and no single failure point.

### C7 — Compaction resume packets can lag current APIs and tool guidance

Compaction/resume output is one of the highest-leverage Focusa surfaces because it becomes the first context a model sees after memory pressure, model switch, fork, or session resume.

Conflict:

- A packet assembled from transcript tail can omit the newest Trajectory, ProjectIdentity, traversal, tool-result, and affordance APIs.
- A packet assembled from old tool call habits can over-fetch full lineage/work-loop data, miss bounded summaries, or advertise stale next tools.
- A packet can look canonical even when it came from local fallback, stale read models, or scope-mismatched data.
- A packet can preserve the immediate Workpoint but fail to explain the larger goal, current verified state, active gap, and why the next action matters.

Requirement:

- Define a versioned Workpoint Resume Packet rendering pipeline for compaction and model switch that composes the latest hot-path APIs and tools: ProjectIdentity, Trajectory view, Workpoint resume/current, `focusa_traverse` bounded slices, tool-result taxonomy, active object resolution, evidence refs, and tool affordance guidance.
- Treat `focusa_workpoint_resume` as the canonical continuation read after compaction, not transcript tail.
- Mark packets `canonical=false` whenever canonical APIs are unavailable, stale, scope-mismatched, or locally reconstructed.
- Include source/API provenance, freshness, failure classes, and corrected next-tool guidance in every rendered packet.

---

## 6) Project identity model

### 6.1 Definition

ProjectIdentity is a stable, explicit record for the project Focusa is operating on. It is used by SessionIdentity, Workpoint, Trajectory Projection, Work-loop, tool scopes, evidence, and lineage.

### 6.2 Root marker file

Each project should be able to define an optional root marker file:

```text
.focusa-project.json
```

Suggested schema:

```json
{
  "schema": "focusa.project.v1",
  "project_id": "focusa",
  "canonical_name": "Focusa",
  "project_root": "/home/wirebot/focusa",
  "repo_remote": "git@github.com:.../focusa.git",
  "beads_prefix": "focusa",
  "workspace_kind": "rust-monorepo",
  "aliases": ["focusa-daemon", "focusa-pi-extension"],
  "created_at": "2026-05-20T00:00:00Z"
}
```

The marker improves accuracy but must not be the only source.

### 6.3 Identity signals

Focusa must collect at least these signals when available:

| Signal | Example |
|---|---|
| root marker | `.focusa-project.json` |
| git root | `git rev-parse --show-toplevel` |
| git remote | `git remote get-url origin` |
| beads root | nearest `.beads/` root and issue prefix |
| package/workspace files | `Cargo.toml`, `package.json`, `go.mod`, etc. |
| current cwd | Pi `ctx.cwd` and process cwd |
| daemon working directory | systemd `WorkingDirectory`, daemon config |
| persisted session identity | prior session ProjectIdentity fingerprint |
| operator-supplied scope | explicit project root/name in prompt or Utility Card |

### 6.4 Fingerprint and quorum

Focusa should compute:

```ts
type ProjectIdentity = {
  project_id: string;
  canonical_name: string;
  project_root: string;
  fingerprint: string;
  confidence: "low" | "medium" | "high";
  signals: ProjectIdentitySignal[];
  mismatches: ProjectIdentityMismatch[];
  verified_at: string;
};
```

Rules:

- High confidence requires at least two independent matching signals, preferably marker + git or git + beads.
- If marker exists but conflicts with git/beads/cwd, identity is degraded and must not be silently canonical.
- If only cwd exists, identity is low-confidence and tools must say so.
- Cross-project Workpoint/Trajectory Projection resume requires matching fingerprint or explicit operator override.

### 6.5 Required tools/APIs

Add or expose:

- `GET /v1/project/identity?cwd=...`
- `POST /v1/project/verify`
- `focusa_project_identity`
- `focusa_project_verify`

These must be hot-path safe and bounded.

### 6.6 Integration points

ProjectIdentity must feed:

- `FocusaSessionIdentity.project_identity`
- Workpoint checkpoint/resume scope guard
- Trajectory Projection checkpoint/resume
- Work-loop root work selection
- Focus Slice `PROJECT_IDENTITY` section
- Evidence capture target refs
- Tool Doctor scope diagnostics
- Live proof harness project/session probes

---

## 7) Trajectory Projection model

### 7.1 Definition

A Trajectory Projection is a bounded view that ties together long-term goal, short-term goal, current state, desired end state, gaps, milestones, active Workpoint, evidence, and blockers.

It is computed from existing Focusa primitives and can be checkpointed as projection metadata via reducer-approved events.

### 7.2 Schema

```ts
type TrajectoryProjection = {
  trajectory_id: string;
  session_identity: FocusaSessionIdentity;

  root_long_term_goal: string;
  long_term_goal: string;
  desired_end_state: string;
  short_term_goal?: string;
  current_state?: string;
  root_goal_stability: "stable" | "clarifying" | "superseded";
  session_clarity_status: "clear" | "provisional" | "unclear" | "conflicted";

  gap_summary?: string;
  milestones: TrajectoryMilestone[];
  active_milestone_id?: string;
  active_workpoint_id?: string;

  source_refs: {
    focus_state_frame_id?: string;
    workpoint_id?: string;
    work_loop_task_id?: string;
    ontology_context_ref?: string;
    metacog_refs?: string[];
    evidence_refs?: string[];
    lineage_refs?: string[];
    prediction_refs?: string[];
  };

  blockers: string[];
  open_questions: string[];
  definition_status: "clear" | "provisional" | "unclear" | "conflicted";
  confidence: "low" | "medium" | "high";
  goal_provenance: Array<{
    field: "long_term_goal" | "desired_end_state" | "short_term_goal" | "current_state";
    source: "operator" | "focus_state" | "workpoint" | "beads" | "ontology" | "evidence" | "metacog" | "prediction" | "local_fallback";
    source_ref?: string;
    inferred: boolean;
    confidence: "low" | "medium" | "high";
  }>;
  supersedes_trajectory_id?: string;
  canonical: boolean;
  updated_at: string;
};
```

### 7.3 Milestone schema

```ts
type TrajectoryMilestone = {
  milestone_id: string;
  title: string;
  desired_state_delta: string;
  current_state_evidence_refs: string[];
  completion_evidence_refs: string[];
  status: "not_started" | "active" | "blocked" | "verified" | "superseded";
  next_workpoint_candidate?: WorkpointCheckpointPayload;
};
```

### 7.4 Session identity envelope

```ts
type FocusaSessionIdentity = {
  pi_session_id?: string;
  session_frame_key: string;
  session_incarnation_id: string;
  project_root: string;
  cwd: string;
  workspace_id: string;
  process_id?: number;
  started_at: string;
  resume_source: "session_start" | "session_switch" | "compaction" | "model_switch" | "fork" | "manual" | "unknown";
};
```

All checkpoint/resume/evidence/trajectory projection calls must use this envelope.

### 7.4.1 Workpoint Resume Packet v2 for compaction

A Workpoint Resume Packet is the canonical, bounded continuation object rendered before/after compaction, model switch, fork, session resume, or context overflow. High/mid/low trajectory grouping is advisory only: sessions with the same high-level trajectory may have distinct mid/low goals and must not merge unless `project_root + continuity_id` match.

Trajectory hierarchy contract: `high_level_goal` or its group key may cluster similar workstreams; `mid_level_goal`, `low_level_goal`, Workpoint identity, and `continuity_id` preserve fine distinctions. `session_id` is temporal metadata and must not be used as the authority boundary across compaction/model switch/fork.

The packet must be generated from canonical hot-path APIs when available, not from raw transcript tail. Transcript/summary fallback is allowed only as degraded local recovery and must set `canonical=false`.

```ts
type WorkpointResumePacketV2 = {
  schema_version: "focusa.workpoint_resume_packet.v2";
  packet_id: string;
  generated_at: string;
  resume_source: FocusaSessionIdentity["resume_source"];
  canonical: boolean;
  degraded: boolean;
  confidence: "high" | "medium" | "low";

  project_identity: ProjectIdentity;
  session_identity: FocusaSessionIdentity;
  workpoint_id?: string;
  work_item_id?: string;

  rendered_summary: string;       // compact prompt-facing summary, safe for direct injection
  resume_summary: {
    one_line: string;
    mission: string;
    current_action: string;
    short_term_goal?: string;
    long_term_goal?: string;
    desired_end_state?: string;
    current_verified_state?: string;
    current_state_delta?: string;
    gap?: string;
    why_this_next?: string;
    safest_next_action: string;
    context_sufficiency?: TrajectoryIntelligenceView["context_sufficiency"];
    warnings: string[];
    do_not_use: string[];
  };

  workpoint: {
    status: "active" | "stale" | "not_found" | "degraded";
    mission: string;
    action_intent?: ActionIntent;
    active_object_refs: string[];
    blockers: string[];
    drift_boundaries: string[];
    do_not_drift: string[];
    verification_hooks: string[];
    verified_evidence_refs: string[];
    next_action: string;
    next_slice?: string;
    updated_at?: string;
  };

  trajectory?: {
    trajectory_id?: string;
    root_goal_stability?: string;
    definition_status?: string;
    intelligence_view?: TrajectoryIntelligenceView;
  };

  traversal_slices: Array<{
    surface: FocusaTraverseInput["surface"];
    selector: FocusaTraverseInput["selector"];
    anchor?: string;
    returned: number;
    truncated: boolean;
    tags?: TraverseTagRef[];
    window_tag?: string;
    rehydrate_refs: string[];
  }>;

  tool_affordances: {
    best_next: string[];
    recovery: string[];
    do_not_use: string[];
  };

  api_provenance: Array<{
    tool_or_route: string;
    purpose: string;
    status: string;
    canonical: boolean;
    failure_class?: FocusaFailureClass;
    freshness?: "live" | "cached" | "stale" | "unknown";
    evidence_ref?: string;
  }>;

  next_tools: string[];
  failure_class?: FocusaFailureClass;
};
```

Packet rendering rules:

1. Include both `rendered_summary` for compact prompt injection and structured JSON for tool-aware agents.
2. The first line must state canonical/degraded status, Workpoint id, mission, current action, and next action.
3. Every packet must include ProjectIdentity/session identity and must reject or degrade on mismatch.
4. Prefer Trajectory current-state/gap/why-this-next fields when canonical; otherwise preserve Workpoint mission/action/next slice and mark missing trajectory fields.
5. Use `focusa_traverse` for bounded supporting slices such as lineage path, evidence windows, ontology neighborhood, tool registry affordances, and telemetry summaries; never fetch full stores by default.
6. Include `api_provenance` so another model can see which APIs/tools produced the packet and which failed or degraded.
7. Include `tool_affordances.best_next` so a resumed model knows whether to call `focusa_trajectory_view`, `focusa_workpoint_resume`, `focusa_traverse`, `focusa_active_object_resolve`, `focusa_tool_doctor`, or evidence tools next.
8. Include `do_not_use` when tools are unsafe, stale, authority-confusing, or blocked by operator constraints.
9. If daemon or canonical APIs are unavailable, render a minimal fallback packet from the operator-provided packet/current repo/beads with `canonical=false`, `degraded=true`, `failure_class=daemon_unavailable|noncanonical_fallback`, and next tools for recovery.
10. The packet must not claim a Focus State, Workpoint, Trajectory, evidence, or Work-loop write succeeded unless the corresponding tool result proves it.

### 7.5 Initial trajectory definition protocol

Trajectory must be defined before it is trusted. On project/session start, compaction resume, missing Workpoint, or `definition_status != clear`, Focusa must build a Trajectory candidate from ordered sources:

1. latest explicit operator goal or desired outcome in the current ask,
2. operator-approved project marker or durable project docs/specs,
3. Focus State `intent`, `current_focus`, decisions, constraints, and failures,
4. active Workpoint mission, action intent, blockers, evidence, and next action,
5. active Beads epic/task/current work item,
6. ontology active objects, valid next actions, and evidence links,
7. metacog lessons and prediction records relevant to the active gap,
8. local fallback only when canonical sources are unavailable.

Source precedence:

- Explicit operator goal wins over all inferred sources.
- Durable project/spec docs may define desired end state when cited as evidence.
- Workpoint/Beads may define short-term goal and next candidate, not the long-term goal by themselves.
- Ontology/evidence may define current state and gap, not operator intent by themselves.
- Local fallback is always noncanonical and must be marked degraded.

If no explicit or durable long-term goal/desired end state exists, Trajectory status is `unclear`; the model must not pretend clarity.

### 7.6 Inference policy

Inference is allowed, but the weight depends on the field:

| Field | Inference policy |
|---|---|
| root/highest long-term goal | minimal inference only; high confidence requires explicit operator or durable project/spec source, and radical change requires operator confirmation. |
| long-term goal | light inference only; explicit operator or durable spec source required for high confidence. |
| desired end state | light-to-moderate inference from operator/docs/specs/evidence; high confidence requires cited durable source. |
| short-term goal | moderate inference from Workpoint, Beads, current ask, and active milestone. |
| current verified state | heavier deterministic inference from ontology/evidence/current repo, but every claim needs evidence refs. |
| gap/current-state delta | inference from comparing desired end state to verified current state; must cite both sides. |
| Workpoint candidate | advisory inference from gap + valid next actions + blockers + evidence requirements. |
| approvals/destructive actions/scope changes | no inference; explicit operator approval or scope confirmation required. |

Every inferred field must carry provenance, confidence, and evidence or missing-fact rationale. Inference may propose a Trajectory; it must not silently make the Trajectory canonical.

### 7.7 Trajectory clarity gate

Before Focusa injects context, proposes a Workpoint, or lets a model proceed with a nontrivial action, it must run a clarity gate:

1. Verify ProjectIdentity and reject/suppress cross-project packets.
2. Read latest operator ask and detect steering or changed desired outcome.
3. Read active Focus State, Workpoint, Work-loop task, Beads state, ontology context, evidence refs, metacog, predictions, and lineage/snapshots.
4. Build or refresh the goal-state binding: long-term goal, desired end state, current verified state, short-term goal, gap, next candidate, evidence.
5. Score `context_sufficiency` and set `definition_status`.
6. Align all context refs to the active trajectory; put stale, mismatched, irrelevant, or unsafe refs into `do_not_use`.
7. Choose posture: `proceed`, `verify_first`, or `operator_required`.

Clarity statuses:

| Status | Meaning | Required posture |
|---|---|---|
| `clear` | goals, current state, gap, candidate, and evidence align. | proceed for safe non-destructive steps. |
| `provisional` | trajectory likely correct but missing local verification or evidence freshness. | verify_first. |
| `unclear` | long-term goal, desired end state, or current state is missing. | operator_required unless local verification can resolve. |
| `conflicted` | sources disagree, project/session mismatch exists, or operator steering supersedes prior path. | operator_required or verify_first before any candidate is trusted. |

### 7.8 Trajectory change over time

Trajectory is not static. It may change through explicit supersession or evidence-backed state updates.

Allowed change sources:

- operator changes or clarifies the long-term goal or desired end state,
- Workpoint completion/blocker/evidence changes the current state or active gap,
- Beads task status changes the active milestone/task relationship,
- ontology/evidence discovers that the current state is different than previously believed,
- metacog/prediction changes risk posture or verification burden,
- ProjectIdentity/session changes invalidate prior context,
- failed verification proves a Workpoint candidate or milestone was wrong.

Change rules:

- The root/highest long-term goal should remain stable across normal work sessions; do not radically reinterpret it from short-term context.
- Root/highest long-term goal changes require explicit operator steering or durable supersession evidence plus a supersession record.
- Desired end state changes require explicit source and supersession record.
- Current state and gap should update from verified evidence without asking the operator.
- Short-term goal may update from Workpoint/Beads as long as it remains linked to the long-term goal.
- Every change must record `supersedes_trajectory_id` or a state-delta evidence ref when applicable.
- If a change breaks goal-state binding, set `definition_status=provisional|unclear|conflicted` and run the clarity gate.

### 7.9 Operator definition path

The operator should be asked directly to define or confirm trajectory when:

- no explicit/durable long-term goal or desired end state exists,
- multiple plausible trajectories conflict,
- the current ask appears to supersede prior trajectory,
- action is destructive, high-risk, approval-gated, or cross-project,
- context sufficiency is below threshold after local verification,
- the model cannot explain why the next step serves the long-term goal.

The operator prompt should be short and editable:

```text
Trajectory unclear. Proposed: long-term=<...>; desired=<...>; current=<...>; short-term=<...>; next=<...>. Confirm or edit?
```

If the operator confirms, store provenance as `goal_source=operator` and update the Trajectory Projection through reducer-approved metadata.

### 7.10 Continuous per-session clarification requirement

Trajectory clarity is a mandatory work-session requirement, not a one-time setup task.

Every work session must have an active Trajectory clarity record that is refreshed:

- at session start and resume,
- after compaction/model switch/fork,
- when the operator gives new steering,
- before any nontrivial or multi-step implementation action,
- after Workpoint completion, blocker, or evidence link,
- after scope/project identity changes,
- after failed verification, tool failure, or daemon degradation,
- before handoff/decomposition/continuation to another model or agent.

The refresh does not mean repeatedly asking the operator. The default is to clarify from canonical sources and evidence. Ask the operator only when the clarity gate produces `unclear` or `conflicted`, or when root-goal supersession/approval is required.

Session clarity output must include:

```text
Root goal stability: stable|clarifying|superseded
Definition status: clear|provisional|unclear|conflicted
Current verified state: ...
Active short-term goal: ...
Gap: ...
Next candidate: ...
Do not use: ...
Posture: proceed|verify_first|operator_required
```

If no clear Trajectory exists, predetermined recovery steps are:

1. verify ProjectIdentity,
2. read operator ask and Focus State,
3. read Workpoint/Beads/ontology/evidence,
4. build a provisional goal-state binding with provenance,
5. suppress stale/mismatched context via `do_not_use`,
6. locally verify missing current-state facts when possible,
7. ask the operator to confirm/edit only if still unclear or conflicted.

---

## 8) Trajectory Projection inputs from existing primitives

| Input source | Trajectory Projection use |
|---|---|
| Focus State intent/current_focus | initial long-term/short-term goal candidates |
| Focus State decisions/constraints/failures | governing context and risk boundaries |
| Workpoint mission/next_slice/action_intent | active short-term execution point |
| Work-loop current_task/decision_context | task traversal and continuation policy |
| Ontology active objects/valid next actions | typed current state and possible next deltas |
| Evidence handles | proof of current state and milestone completion |
| Metacog reflections/adjustments | strategy and repeated failure learning |
| Prediction records | risk posture and expected next-action success |
| Lineage/tree/snapshots | recoverability, branch/session ancestry, before/after comparisons |
| Pi local fallback | noncanonical emergency continuity only |

### 8.1 Model intelligence uplift requirements

Trajectory Projection should make each model using Focusa smarter by giving it a bounded, evidence-backed awareness view before it acts.

The view must answer:

- **Goal hierarchy:** durable long-term goal, desired end state, active milestone, current ask, active gap, and bounded next Workpoint candidate.
- **Current-state proof:** what is known now, what changed since the last checkpoint, and which evidence handles prove it.
- **Relevant working set:** active objects, valid next actions, blocked affordances, and why each was included.
- **Context sufficiency:** whether the model has enough verified context to proceed without outside help.
- **Uncertainty register:** missing facts, stale references, conflicting signals, scope ambiguity, and confidence.
- **Prior learning:** metacog signals and prior failures relevant to the active gap.
- **Risk forecast:** prediction records that affect next-action confidence or verification burden.
- **Assistance boundary:** when to continue autonomously, when to verify first, and when operator input is truly required.

This is intelligence support, not execution authority. The model becomes smarter because Focusa gives it better grounded awareness, not because Trajectory chooses work for it.

### 8.2 Trajectory Intelligence View

Add a bounded derived view for model consumption:

```ts
type TrajectoryIntelligenceView = {
  trajectory_id: string;
  context_sufficiency: {
    score: number; // 0..1
    proceed_posture: "proceed" | "verify_first" | "operator_required";
    missing_facts: string[];
    stale_refs: string[];
    conflicting_signals: string[];
  };
  relevance_rationale: Array<{
    ref: string;
    why_included: string;
    confidence: "low" | "medium" | "high";
  }>;
  current_state_delta: {
    since_checkpoint?: string;
    changed_refs: string[];
    evidence_refs: string[];
  };
  learning_refs: string[];
  prediction_refs: string[];
  ask_operator_if: string[];
  do_not_use: string[];
};
```

Rules:

- `context_sufficiency.score` is advisory and must cite missing facts or evidence refs.
- `proceed_posture=operator_required` is allowed only for approval gates, destructive/risky ambiguity, scope conflict, or missing critical requirements.
- `do_not_use` should suppress stale packets, mismatched project state, and irrelevant carryover context.
- Relevance rationale must be short enough for prompt injection and never include raw transcript blobs.
- Negative context is first-class: stale packets, irrelevant carryover, mismatched project refs, superseded goals, and unsafe assumptions must appear in `do_not_use` rather than being silently omitted.

### 8.3 Assistance-minimization contract

Focusa should reduce outside assistance by helping the model continue safely from verified context.

Default behavior:

1. If context is sufficient and action is non-destructive, continue with the best verified Workpoint candidate.
2. If context is incomplete but verifiable locally, verify first instead of asking the operator.
3. If context is stale or conflicting, use ProjectIdentity, evidence refs, and current repo state to resolve before asking.
4. Ask the operator only for approval gates, scope conflicts, destructive actions, or requirements that cannot be inferred or verified.

Operator steering still wins immediately.

### 8.4 Smartness metrics

Track whether Focusa is actually making models more capable over long-running work:

- Workpoint resume success after compaction/model switch/session restart.
- Context sufficiency score before and after ProjectIdentity/Trajectory injection.
- Evidence-linked completion rate.
- Drift/scope-mismatch rate.
- Repeated-failure reduction from metacog retrieval.
- Prediction calibration improvement for next-action success.
- Operator-assistance requests per completed Workpoint.
- Time-to-recover from daemon degradation.
- Prompt relevance ratio: injected refs used vs ignored/stale refs.

These metrics are diagnostic feedback; they must not become task authority.

### 8.5 Definition-of-done contract

Every Trajectory Projection should carry an explicit desired-end-state proof contract:

```ts
type TrajectoryDefinitionOfDone = {
  desired_end_state: string;
  required_evidence_refs: string[];
  required_checks: string[];
  acceptance_risks: string[];
  not_done_if: string[];
};
```

Rules:

- A long-running trajectory is not complete because the model says it is complete; it is complete when required evidence and checks prove the desired end state.
- `not_done_if` should capture common false-completion traps such as unverified docs, stale daemon registry, missing restart approval, failing live proof, or unresolved scope mismatch.
- Workpoint completion evidence should roll up into Trajectory completion evidence.

### 8.6 Go-to-market mission frame

Mission: **Focusa is the per-project trajectory layer for AI agents.** It gives any agent a durable, evidence-backed sense of which project it is in, where that project is going, where the work currently stands, why the next move matters, what not to drift into, and which bounded Workpoint should happen next.

Message pillars:

1. **Orientation:** agents stop losing the plot across turns, tools, compactions, models, and handoffs.
2. **Continuity:** Workpoints, evidence, decisions, and state survive the session boundary.
3. **Trust:** every next move is tied to provenance, proof, freshness, and drift warnings.
4. **Portability:** Focusa wraps existing agents instead of replacing them.
5. **Reliability:** low memory keeps the core trajectory view alive; high memory enriches context opportunistically within budgets.

Positioning sentence:

> Focusa is the per-project trajectory intelligence runtime: it keeps AI agents aligned to the correct project, real goal, verified state, evidence, and next bounded move.

---

## 9) Required API/tool surface

### 9.0 Composition-first boundary

Trajectory tools are navigation/projection tools only. Their first implementation must compose existing Focusa primitives into bounded summaries and Workpoint candidates before adding any new persisted state.

They must not:

- select tasks or replace Beads.
- switch Focus Frames or replace Focus Stack.
- overwrite Focus State meaning slots without normal Focus State update rules.
- execute tools, schedule actions, or control Work-loop traversal.
- auto-promote a Workpoint candidate into canonical continuation without reducer-approved Workpoint checkpoint/resume semantics.

If a Trajectory tool returns a candidate, the candidate is advisory until the existing authoritative tool accepts it.

Tool naming should avoid authority/control verbs. Prefer:

- `view` for bounded read-only navigation state.
- `define_goal` for projected goal metadata, not task selection.
- `assess` for gap, sufficiency, and uncertainty analysis.
- `propose_workpoint` for advisory candidate generation.

Avoid names like `plan`, `execute`, `select`, `dispatch`, or bare `next` because they imply authority Focusa Trajectory does not own.

### 9.1 `focusa_trajectory_view`

Read active Trajectory Projection.

Must be hot-path safe and bounded.

Inputs:

```ts
{ session_identity?: FocusaSessionIdentity; mode?: "summary" | "full" }
```

Output:

```ts
{
  status: "completed" | "not_found" | "degraded";
  canonical: boolean;
  trajectory?: TrajectoryProjection;
  intelligence_view?: TrajectoryIntelligenceView;
  do_not_use: string[];
  next_tools: string[];
}
```

### 9.2 `focusa_trajectory_define_goal`

Create or update projected desired-state metadata for navigation. This does not change Focus Stack, Beads, Work-loop traversal, or the active Workpoint by itself.

Inputs:

```ts
{
  long_term_goal: string;
  desired_end_state: string;
  short_term_goal?: string;
  current_state?: string;
  goal_source?: "operator" | "focus_state" | "workpoint" | "beads" | "imported";
  supersedes_trajectory_id?: string;
  session_identity: FocusaSessionIdentity;
  idempotency_key?: string;
}
```

The tool must preserve goal provenance and supersession history so models know whether the goal came from the operator, a durable Focus State signal, a Beads item, or a Workpoint continuation packet.

### 9.3 `focusa_trajectory_assess`

Compare projected current state against desired end state using existing evidence and context signals. This returns a gap view, not an execution plan.

Inputs:

```ts
{
  observed_state?: string;
  evidence_refs?: string[];
  session_identity: FocusaSessionIdentity;
}
```

Outputs:

```ts
{
  current_state: string;
  desired_end_state: string;
  gaps: string[];
  blockers: string[];
  recommended_milestones: TrajectoryMilestone[];
  context_sufficiency: TrajectoryIntelligenceView["context_sufficiency"];
  uncertainty_register: string[];
  do_not_use: string[];
  next_workpoint_candidate?: WorkpointCheckpointPayload;
}
```

### 9.4 `focusa_trajectory_propose_workpoint`

Propose the next bounded Workpoint candidate from the Trajectory gap. This candidate is not canonical until accepted through existing Workpoint checkpoint/resume rules.

The proposal must include `why_this_next`, `goal_link`, `current_state_delta`, and `completion_evidence_required` so the model understands how the short-term step serves the long-term destination.

Outputs:

```ts
{
  trajectory_id: string;
  workpoint_checkpoint_payload: WorkpointCheckpointPayload;
  next_action: string;
  verification_required: string[];
  do_not_drift: string[];
}
```

### 9.5 `focusa_trajectory_checkpoint`

Persist Trajectory Projection progress as projection metadata before compaction/model switch/fork/risky continuation; do not persist task authority or execution state here.

### 9.6 `focusa_trajectory_resume`

Resume from active Trajectory Projection plus Workpoint. This resumes navigation awareness; it does not resume execution by itself.

Must distinguish:

- Trajectory Projection valid + Workpoint valid.
- Trajectory Projection valid + Workpoint stale.
- Trajectory Projection stale + operator steering wins.
- Scope mismatch; use current repo/operator fallback.

### 9.7 `focusa_traverse`

Read-only surgical traversal and parsing tool for any large Focusa structure. This is the low-level companion to Trajectory: Trajectory tells the agent **what matters for this project**, while `focusa_traverse` fetches **exactly the relevant slice** of the underlying structure without forcing full-tree/full-store payloads.

This tool is inspired by the Antirez article “Alternatives for the EDIT tool of LLM agents” (`https://antirez.com/news/166`). The relevant idea is not the edit operation itself; it is the token-efficient **check-and-set anchor** pattern:

- READ/SEARCH returns compact `line:tag` anchors instead of requiring the model to repeat old text verbatim.
- Follow-up operations cite the line/range plus checksum tag to prove they are acting on the same slice.
- Per-line/per-item tags allow unrelated changes elsewhere without invalidating the local operation.
- Whole-file CRC is cheaper but fails when unrelated changes happen; useful as an optional coarse mode, not the default.
- Multi-line operations can cite compact `line:tag` sequences for range safety.
- Tag length/collision/tokenization tradeoffs should be measured and configurable.

Focusa adaptation: every traversed item/window returns compact anchors and version tags so an agent can ask “is this slice still current?” or continue traversal without retransmitting the whole structure.

#### 9.7.1 Tool boundary

`focusa_traverse` must be read-only by default.

It must not:

- mutate Focus State, Workpoints, Trajectory metadata, Beads, ontology, or lineage.
- select tasks or execute actions.
- auto-promote Workpoint candidates.
- restore snapshots or perform rollback.
- fetch full stores by default.

It may:

- return bounded windows, paths, neighborhoods, summaries, search hits, and metadata.
- return item/window/surface tags for subsequent verification.
- return rehydrate refs for cold payloads.
- return `do_not_use` and stale/mismatch warnings.
- recommend more specific tools when the traversal result implies a better next tool.

#### 9.7.2 Inputs

```ts
type FocusaTraverseInput = {
  surface:
    | "trajectory"
    | "lineage"
    | "ontology"
    | "focus_stack"
    | "workpoints"
    | "evidence"
    | "ecs"
    | "references"
    | "metacognition"
    | "predictions"
    | "telemetry"
    | "commands"
    | "turns"
    | "snapshots"
    | "tool_registry"
    | "capabilities";

  selector:
    | "summary"
    | "head"
    | "current"
    | "path"
    | "parents"
    | "children"
    | "siblings"
    | "neighborhood"
    | "recent"
    | "search"
    | "by_id"
    | "diff"
    | "window"
    | "tags_verify";

  anchor?: string;              // node id, frame id, workpoint id, ref id, snapshot id, trajectory id
  query?: string;               // bounded text or semantic query
  cursor?: string | number;
  limit?: number;               // default small; hard cap per surface
  depth?: number;               // path/tree depth cap
  radius?: number;              // graph/tree neighborhood cap
  fields?: string[];            // projection fields; summary fields by default
  tags?: TraverseTagRef[];      // verify previously returned anchors/tags
  tag_mode?: "item" | "range" | "window" | "surface" | "mixed";
  include_payload?: boolean;    // false by default; cold opt-in only
  include_rehydrate_refs?: boolean;
  budget_tokens?: number;
  session_identity?: FocusaSessionIdentity;
};

type TraverseTagRef = {
  anchor: string;
  tag: string;
  ordinal?: number;
  range?: { start_anchor: string; end_anchor: string; tags?: string[] };
};
```

#### 9.7.3 Outputs

```ts
type FocusaTraverseOutput<T> = {
  status: "completed" | "degraded" | "blocked" | "validation_rejected";
  canonical: boolean;
  degraded: boolean;
  surface: FocusaTraverseInput["surface"];
  selector: FocusaTraverseInput["selector"];
  anchor?: string;
  project_identity?: ProjectIdentity;

  items: TraversedItem<T>[];
  summary?: string;
  do_not_use: string[];

  traversal: {
    returned: number;
    total_known?: number;
    cursor?: string | number | null;
    next_cursor?: string | number | null;
    truncated: boolean;
    caps: { limit: number; depth?: number; radius?: number; payload_bytes?: number; budget_tokens?: number };
    omitted: string[];
    rehydrate_refs: string[];
    stale_tags: TraverseTagRef[];
    verified_tags: TraverseTagRef[];
  };

  tag_scheme: TraverseTagScheme;
  failure_class?: FocusaFailureClass;
  next_tools: string[];
};

type TraversedItem<T> = {
  anchor: string;               // stable local ref for follow-up traversal/verification
  ordinal?: number;             // optional position within current window/path
  tag: string;                  // compact content/version tag
  surface_version?: string;     // coarse surface snapshot/version when available
  freshness?: "live" | "cached" | "stale" | "unknown";
  scope?: { project_root?: string; session_id?: string; frame_id?: string; workpoint_id?: string };
  kind?: string;
  label?: string;
  summary?: string;
  data?: T;                     // only summary/projection fields unless include_payload=true
};

type TraverseTagScheme = {
  algorithm: "crc32" | "xxhash64" | "sha1_64" | "opaque_version";
  length: number;               // default 6-10 chars depending collision budget
  includes_anchor: boolean;
  includes_surface_version: boolean;
  collision_policy: "retry_with_longer_tag" | "require_window_tag" | "require_surface_tag";
};
```

#### 9.7.4 Tag/CAS semantics from the article

Focusa should use Antirez-style local tags as a token-efficient alternative to sending full old payloads.

Required tag levels:

| Tag level | Use | Tradeoff |
|---|---|---|
| item tag | Verify one node/record/ref/line-like item. | Best locality; unrelated changes elsewhere do not fail. |
| range tag | Verify an ordered path/window/range with per-item tags. | Good for path/range traversal; slightly more tokens. |
| window tag | Verify the returned window as a whole. | Good for cursor continuation; invalidated by local reordering/window changes. |
| surface tag | Verify the whole structure version. | Cheapest but fails on unrelated changes; cold/coarse mode only. |
| opaque version | Use reducer/read-model version where content hashing is expensive. | Requires clear provenance; may be coarse. |

Rules:

1. Every traversed item should return `anchor` + `tag` unless the surface has no stable item identity; then return an opaque version and `freshness=unknown`.
2. Tags must be generated from normalized summary payload plus anchor and surface/version when possible.
3. Default tags should be short enough for models to use, but collision policy must allow longer tags on retry.
4. `tags_verify` selector checks whether anchors/tags are still current without returning the full payload.
5. If tag verification fails, return `status=degraded`, `failure_class=read_model_lag` or `scope_mismatch`, stale/verified tag lists, and next tool guidance.
6. Whole-surface tags are allowed only as an optimization, never as the only validation path for local traversal.
7. Surface implementations may choose `crc32` for speed, `xxhash64`/`sha1_64` for lower collision risk, or reducer versions for opaque state; the chosen scheme must be returned in `tag_scheme`.
8. Follow-up mutating tools may accept traversal tags as precondition refs, but `focusa_traverse` itself remains read-only.

#### 9.7.5 Surface mapping

| Surface | Required selectors | Default cap | Tag anchors |
|---|---|---:|---|
| `trajectory` | summary, current, by_id, tags_verify | 1-5 | trajectory_id, project fingerprint, Workpoint id |
| `lineage` | head, path, children, neighborhood, summaries, window, tags_verify | 25-50 | clt_node_id |
| `ontology` | working_set, adjacency/neighborhood, search, by_id, path, tags_verify | 20-100 | object id + link id |
| `focus_stack` | current, path, parents, children, window, tags_verify | 10-50 | frame_id |
| `workpoints` | current, by_id, recent, evidence window, blockers window, tags_verify | 10-50 | workpoint_id + verification ref |
| `evidence`/`ecs`/`references` | by_id, meta, search, recent, type/tag filter, tags_verify | 20-100 | handle/ref id |
| `metacognition` | recent, retrieve/search, by_id, top-k, tags_verify | 10-50 | signal/reflection/adjustment id |
| `predictions` | recent, by_id, stats summary, tags_verify | 10-50 | prediction id |
| `telemetry`/`commands`/`turns` | recent, time window, event type filter, cursor page, tags_verify | 20-100 | event id/log id/turn id |
| `snapshots` | recent, by_id metadata, diff summary, tags_verify | 5-20 | snapshot id |
| `tool_registry`/`capabilities` | family filter, tool by name, summary counts, tags_verify | 20-100 | tool name/capability id |

#### 9.7.6 Examples

Lineage path window:

```json
{
  "surface": "lineage",
  "selector": "path",
  "anchor": "clt:019...",
  "limit": 12,
  "fields": ["node_id", "node_type", "summary", "created_at"],
  "tag_mode": "range"
}
```

Tag verification:

```json
{
  "surface": "lineage",
  "selector": "tags_verify",
  "tags": [
    { "anchor": "clt:019...", "tag": "Q8fA2c", "ordinal": 0 },
    { "anchor": "clt:020...", "tag": "rA3_9b", "ordinal": 1 }
  ]
}
```

Ontology neighborhood:

```json
{
  "surface": "ontology",
  "selector": "neighborhood",
  "anchor": "object:focusa_trajectory_view",
  "radius": 1,
  "limit": 25,
  "fields": ["id", "object_type", "uncertainty", "links"]
}
```

Tool registry slice:

```json
{
  "surface": "tool_registry",
  "selector": "search",
  "query": "trajectory",
  "fields": ["name", "family", "side_effect_profile", "next_tools"]
}
```

#### 9.7.7 API/CLI/Tool surfaces

Required API routes:

- `POST /v1/traverse`
- `POST /v1/traverse/verify-tags`
- Optional surface aliases may exist later, but they must call the same traversal substrate.

Pi tool:

- `focusa_traverse`

CLI:

- `focusa traverse --surface <surface> --selector <selector> [--anchor ...] [--limit ...] [--cursor ...]`
- `focusa traverse verify-tags --surface <surface> --tags <json>`

#### 9.7.8 Safety and failure taxonomy

- `validation_rejected`: unsupported surface/selector, invalid anchor, limit above hard cap without cold opt-in.
- `scope_mismatch`: ProjectIdentity/session/workpoint tag does not match active scope.
- `read_model_lag`: anchor exists but tag/version is stale or not yet visible.
- `resource_exhausted`: traversal would exceed memory/byte/token budget.
- `cold_path_timeout`: requested full payload or deep selector exceeded budget.
- `noncanonical_fallback`: result came from cache/local fallback.

#### 9.7.9 Acceptance for `focusa_traverse`

- Models can retrieve a relevant partial slice from each major Focusa surface without reading full payloads.
- Every result carries traversal metadata and tag scheme details.
- Tag verification can distinguish unchanged local slices from stale/mismatched slices.
- Safe audit fails/warns if a hot traversal defaults to full tree/graph/log/store payload.
- Golden evals prove agents use `focusa_traverse` when they need a narrow slice and `focusa_trajectory_view` when they need project north-star orientation.
- Low-memory runs preserve hot traversal routes and degrade cold/full traversal explicitly.

### 9.8 `focusa_resource_mode`

Model-visible resource mode status/control tool. This tool lets an agent recognize resource pressure and activate/deactivate `LowMem` without waiting for hidden operator/system intervention.

Tool name:

- `focusa_resource_mode`

Accepted operator phrases should map to this tool:

- “Activate LowMem mode” -> `focusa_resource_mode(action="activate_lowmem")`
- “Turn on LowMem” -> `focusa_resource_mode(action="activate_lowmem")`
- “Deactivate LowMem mode” -> `focusa_resource_mode(action="deactivate_lowmem")`
- “Return Focusa to auto resource mode” -> `focusa_resource_mode(action="deactivate_lowmem")`
- “Force normal mode” -> `focusa_resource_mode(action="set_mode", mode="normal")`

Inputs:

```ts
type FocusaResourceModeToolInput = {
  action?:
    | "status"
    | "activate_lowmem"
    | "deactivate_lowmem"
    | "set_mode"
    | "set_normal"
    | "set_constrained"
    | "set_emergency";
  mode?: "auto" | "normal" | "constrained" | "lowmem" | "emergency";
  reason?: string;
  preflight?: boolean;
};
```

Behavior:

1. `status` reads current mode, pressure reason, budgets, pruning order, and deferred cold surfaces.
2. `activate_lowmem` sets a runtime LowMem override immediately; no daemon restart required.
3. `deactivate_lowmem` clears the runtime LowMem override and returns to automatic resource-mode detection.
4. `set_mode` can force `normal`, `constrained`, `lowmem`, or `emergency`; `mode="auto"` clears the runtime override.
5. `preflight=true` reports the intended change without mutation.
6. The tool must never hide other tools; it only changes their fidelity/budgets/degradation behavior.
7. Results include `tool_result_v1`, `side_effects`, `next_tools`, `resource_mode`, and failure taxonomy.

API routes:

- `GET /v1/resource/mode`
- `POST /v1/resource/mode`

Acceptance:

- The model can discover `focusa_resource_mode` through tool affordances when resource pressure is detected.
- Operator natural-language activation/deactivation commands map to the tool.
- Activation/deactivation changes runtime behavior without a rebuild/restart.
- Deactivation returns to `auto`; if auto detection still selects LowMem due pressure, the result explains why.
- The tool remains safe under LowMem and is itself a T0/T1 hot-path route.

---

## 10) Daemon/API stability requirements

### 10.1 Route tiers

| Tier | Purpose | Examples | SLA |
|---|---|---|---|
| Hot | agent context and readiness | health, session identity, trajectory projection summary, workpoint current, work-loop summary | bounded, fast, no replay |
| Warm | normal tool operations | checkpoint, evidence link, metacog capture, predictions | timeout + clear retry posture |
| Cold | diagnostics and proof | replay, deep work-loop status, worktree scans, release proof | opt-in, may be slower |

### 10.2 Status route split

Required routes:

- `GET /v1/status` — hot summary only; no persistence scans, `/proc` daemon enumeration, replay, or deep diagnostics.
- `GET /v1/status/deep` — explicit cold diagnostics with persisted event counts, daemon PID enumeration, duplicate-daemon detection, and other slow proof fields.
- `GET /v1/status?deep=true` — compatibility alias for the explicit cold route.
- `GET /v1/status?summary_only=true` — forces the hot summary even when a caller also sends `deep=true`.

Hot status responses must include `route_tier="hot"`, `summary_only=true`, `deep_status_route`, `cold_omitted`, `resource_mode`, runtime memory, worker/perf counters, and session/frame summary fields. Cold-only fields must be absent or `null` in the hot payload and listed in `cold_omitted`.

Deep status responses must include `route_tier="cold"`, `summary_only=false`, empty `cold_omitted`, and the cold diagnostics. Deep diagnostics are not allowed to hold `/v1/status` hot callers hostage.

### 10.3 Work-loop route split

Required routes:

- `GET /v1/work-loop/health`
- `GET /v1/work-loop/status?summary_only=true`
- `GET /v1/work-loop/status/deep`
- `GET /v1/work-loop/replay/closure-evidence`
- `GET /v1/work-loop/replay/closure-bundle`

`focusa_work_loop_status` defaults to summary. Deep diagnostics require explicit mode or separate tool.

### 10.4 Internal timeout rule

Cold subqueries must not hold hot route responses hostage. If replay/worktree/deep diagnostics exceed budget, return partial payload with:

```json
{
  "status": "degraded",
  "failure_class": "cold_path_timeout",
  "canonical": false,
  "next_tools": ["focusa_tool_doctor"]
}
```

### 10.5 Low-memory reliability mode

Focusa operating principle: **low memory = still reliable; high memory = opportunistic and performant without being a hog.** Core tool availability is more important than full-fidelity diagnostics. Rich cognition is budgeted enrichment, not an excuse to starve hot tools.

Core reliability set that must keep working with bounded memory:

- `/v1/health`
- ProjectIdentity verify/view
- `focusa_trajectory_view` summary
- Workpoint current/resume summary
- Focus State compact writes/reads
- evidence capture/link summary
- `focusa_tool_doctor` summary
- work-loop writer/status summary
- tool contract/static registry reads

Low-memory rules:

1. Hot routes must avoid cloning or serializing large stores, full Focus Stack paths, full telemetry, full ontology, full lineage tree, or replay bundles.
2. Cold routes are opportunistic: return full data only when memory/latency budgets allow; otherwise return degraded summaries with `failure_class=resource_exhausted` or `cold_path_timeout`.
3. Use last-known-good cached summaries for hot tool reads when daemon is restarting, under pressure, or temporarily unavailable. Cached data must include `cached_at`, age, source, and `canonical=false` when stale.
4. Core write tools should prefer compact reducer events and idempotency keys over large payloads.
5. Background compaction/replay/telemetry jobs must yield to hot tool routes and must not cause OOM risk.
6. Tool Doctor must surface memory pressure, RSS/peak RSS, store sizes, frame depth, and recent OOM/restart evidence when available.
7. Models should receive the best reliable data available: canonical live data first, then fresh bounded cache, then stale/degraded cache with warnings, then scratch/local fallback only as last resort.
8. Low-memory degradation must be explicit and actionable; never return bare `null`, `unknown`, or `daemon unavailable` without a failure class and recovery hint when any data is available.

### 10.5.1 `LowMem` extreme-constrained mode

`LowMem` is an explicit extreme-constrained operating mode for machines where RAM/CPU/I/O budgets are so tight that normal rich cognition would risk daemon freezes, healthcheck timeouts, OOM kills, swap thrash, or model-visible tool unavailability.

The goal of `LowMem` is **maximum useful Focusa value under hard constraints**, not minimal functionality. The same public `focusa_*` tool names remain available, but every tool must route through a mode-aware, bounded, summary-first implementation that preserves accuracy, provenance, and recovery posture.

#### 10.5.1.1 Mode taxonomy

```ts
type FocusaResourceMode =
  | "normal"       // full hot+warmed cognition inside normal budgets
  | "constrained"  // moderate pruning; rich context becomes budgeted top-k
  | "lowmem"       // extreme surgical mode; all tools remain available as bounded summaries
  | "emergency";   // daemon survival mode; hot core + cached/degraded responses only
```

`LowMem` may be entered by explicit config or automatically by pressure detection:

- `FOCUSA_RESOURCE_MODE=lowmem` or CLI/config equivalent.
- RSS/heap above soft budget.
- host `MemAvailable` below configured floor.
- repeated `hot_path_timeout` or healthcheck near-failures.
- cgroup memory pressure / OOM evidence.
- startup on known tiny environments.

Mode transitions must use hysteresis so Focusa does not flap between modes.

Daemon background auto-fallback requirements:

- ResourceMode detection is daemon-level and does not depend on an active Pi/agent session.
- When there is no active session, the daemon still monitors core RSS/peak RSS, host `MemAvailable`, cgroup/OOM/restart evidence, allocator pressure, and hot-route timeout counters.
- Automatic `normal -> constrained -> lowmem -> emergency` fallback may run as a background maintenance path to protect daemon liveness before any agent asks for context.
- Before applying any automatic fallback or recovery transition, the daemon must append a bounded `ResourceModeTransitionRecord` to the event log or hot in-memory transition ring; if durable persistence is temporarily unavailable, record the hot in-memory transition first and mark it `durability="pending"`.
- The transition record must include `transition_id`, `observed_at`, `from_mode`, `to_mode`, `reason`, `trigger="background_resource_monitor" | "operator_override" | "api_override"`, `active_session_id` (`null` allowed), `rss_kb`, `peak_rss_kb`, `host_mem_available_kb`, `budget`, `hysteresis_state`, `durability`, and `recovery_hint`.
- `focusa_resource_mode(status)` and Tool Doctor should expose the latest transition summary plus omitted history count; full transition history is cold/diagnostic and capped.

#### 10.5.1.2 Hard invariant: tools stay present

LowMem must not solve pressure by hiding or disabling normal Focusa tools from the agent. Instead:

- read tools return smaller live summaries, fresh caches, or explicit degraded envelopes.
- write tools accept compact reducer events/idempotency keys and reject only oversized/non-essential payloads with `validation_rejected` or `resource_exhausted`.
- cold tools stay callable but become `blocked`/`degraded` with rehydrate refs, next tools, and operator-safe recovery when budgets do not allow full execution.
- tool docs/affordances must advertise the LowMem behavior so the agent knows how to ask surgically.

#### 10.5.1.3 Core reliability ladder

When resources are scarce, Focusa prunes by cognitive importance, not by arbitrary tool family.

| Tier | Keep live first | LowMem behavior |
|---|---|---|
| T0 liveness | `/v1/health`, status summary, resource mode | zero/near-zero lock, no persistence scan, no cold fan-out |
| T1 continuation | Workpoint current/resume packet, Trajectory summary, ProjectIdentity | compact canonical summary, exact next action, drift boundaries, identity guard |
| T2 safety/scope | constraints, failures, decisions, writer status, approvals, `do_not_use` | bounded slot summaries and hard safety signals never pruned silently |
| T3 evidence | evidence handles, verification refs, active object refs | handles + proof summaries only; raw output via rehydrate refs |
| T4 surgical context | `focusa_traverse`, ontology working set, lineage path/neighborhood | small caps, cursor windows, tags/checksums, no full tree/graph |
| T5 learning/risk | metacog top-k, predictions, prior failures | top-k by active gap/failure class; stale entries omitted with count |
| T6 diagnostics/history | replay, telemetry logs, full status, full snapshots, full registry dumps | cold opt-in only; may return blocked/degraded under LowMem |

Pruning order under LowMem:

1. discard/defer full raw logs, replay bundles, telemetry bodies, full lineage trees, full ontology graphs, full snapshot bodies.
2. compress historical Focus Stack and CLT to active path + recent/head summaries.
3. keep only top-k metacog/prediction signals relevant to active gap/failure class.
4. retain evidence handles and checksums before raw payloads.
5. retain Workpoint, Trajectory, ProjectIdentity, durable constraints/decisions/failures, and safety/approval state last.

#### 10.5.1.4 LowMem budgets

Default LowMem budgets should be configurable per host class:

```ts
type LowMemBudget = {
  mode: "lowmem";
  rss_soft_mb: number;           // e.g. 256-512 on tiny hosts
  rss_hard_mb: number;           // hard cgroup/systemd guard
  hot_route_timeout_ms: number;  // e.g. 150-500
  warm_route_timeout_ms: number; // e.g. 500-1500
  cold_route_timeout_ms: number; // e.g. 1500-5000, may block/degrade
  hot_payload_bytes: number;     // small prompt-safe summary envelope
  max_items_default: number;     // e.g. 5-25 depending surface
  max_items_hard: number;        // e.g. 50-100
  max_rehydrate_refs: number;
  background_concurrency: 0 | 1;
};
```

Config/env examples:

- `FOCUSA_RESOURCE_MODE=auto|normal|constrained|lowmem|emergency`
- `FOCUSA_LOWMEM_RSS_SOFT_MB=384`
- `FOCUSA_LOWMEM_HOT_TIMEOUT_MS=250`
- `FOCUSA_LOWMEM_DEFAULT_LIMIT=10`
- `FOCUSA_LOWMEM_BACKGROUND_CONCURRENCY=0`
- `FOCUSA_ALLOCATOR_TRIM_INTERVAL_SECS=15`
- `FOCUSA_RESOURCE_MODE_MONITOR_INTERVAL_SECS=15`

#### 10.5.1.5 Mode-aware tool behavior

Every official tool must preserve its purpose while changing fidelity:

| Tool family | LowMem behavior |
|---|---|
| Trajectory | summary only; current goal/state/gap/why-next/context sufficiency; no broad source expansion |
| Workpoint | compact resume/current/checkpoint payload; evidence handles only; exact next action preserved |
| Focus State | compact slot reads/writes only; reject verbose/task-like payloads before allocation-heavy work |
| Work-loop | writer/status summary only; deep replay/worktree diagnostics cold-blocked by default |
| Tree/lineage | head/path/neighborhood/window via `focusa_traverse`; full tree cold-blocked by default |
| Ontology | active working set, affordances, `do_not_use`, top links; full graph omitted with counts/refs |
| Evidence/ECS | metadata/handles first; raw rehydrate only by explicit handle and byte cap |
| Metacog/prediction | active-gap top-k; no broad reflection/evaluation loops unless budget allows |
| Telemetry/commands | recent counters and failure classes; full logs cold-only |
| Tool doctor | resource-mode summary, pressure cause, active degraded surfaces, and exact recovery chain |
| Tool registry/docs | compact affordance catalog; full docs/registry by family/cursor only |

#### 10.5.1.6 Agent-facing LowMem Focus Slice

When LowMem is active, Focus Slice must make the constraint visible and useful:

```text
RESOURCE_MODE: lowmem
RESOURCE_REASON: rss_soft_exceeded|tiny_host|health_timeout_risk|operator_forced
LOWMEM_BUDGET: hot_timeout_ms=250 default_limit=10 payload_bytes=...
CONTEXT_POSTURE: surgical_summary_only
BEST_NEXT_TOOLS:
  - focusa_trajectory_view(mode="summary")
  - focusa_workpoint_resume(mode="compact_prompt")
  - focusa_traverse(limit=..., fields=[...])
DO_NOT_USE_BY_DEFAULT:
  - full lineage tree
  - full ontology graph
  - deep work-loop status
  - replay bundles
```

The agent should still be able to operate accurately: identify the project, understand the current goal and Workpoint, choose the next bounded action, verify evidence, and ask for specific rehydration only when worth the resource cost.

`focusa_resource_mode` should be advertised whenever pressure is detected or the operator asks to activate/deactivate LowMem.

#### 10.5.1.7 Adaptive value extraction algorithm

LowMem context assembly should follow this sequence:

1. verify ProjectIdentity and Workpoint scope.
2. include Trajectory summary if available; otherwise Workpoint mission/current action/next slice.
3. include durable constraints, decisions, failures, approvals, and `do_not_use` safety flags.
4. include active object refs and evidence handles.
5. use `focusa_traverse` to fetch only missing narrow slices by anchor/query.
6. include top-k metacog/prediction signals only if they directly improve the active gap or known failure class.
7. return omitted counts and rehydrate refs for everything pruned.
8. never perform cold expansion automatically inside prompt injection.

#### 10.5.1.8 Background work in LowMem

LowMem may pause or throttle background enrichment, but not public tool availability.

- compaction/replay/reflection/deep diagnostics run only when explicitly requested and budget permits.
- route handlers must yield to T0/T1 hot routes.
- one cold route may run at a time by default; additional cold requests return `blocked` with retry posture.
- allocator trimming and cache eviction are allowed maintenance, but must not block hot routes.

#### 10.5.1.9 LowMem acceptance

- All official `focusa_*` tools remain advertised and callable.
- Hot core routes respond within LowMem timeout budgets during cold-route pressure.
- No hot route performs unbounded clone/serialization/persistence scan.
- Full payload requests degrade explicitly with `resource_exhausted`, `cold_path_timeout`, `omitted`, `rehydrate_refs`, and next-tool guidance.
- Focus Slice and Workpoint Resume Packet v2 carry `RESOURCE_MODE`, pressure reason, and pruned context counts.
- Golden evals prove a fresh agent can complete a surgical task using LowMem summaries and targeted traversal without daemon freezes.
- Stress tests prove no healthcheck restart storm, no tool disappearance, and no OOM under configured tiny-host budgets.
- LowMem dependency proof validates that official tool registrations, contracts, docs, API route inventory, live ontology projection, and representative read dependencies stay available under forced LowMem.
- Background auto-fallback works with `active_session_id=null` and writes a `ResourceModeTransitionRecord` before changing mode.

### 10.6 Opportunistic context policy

Focusa should opportunistically enrich models with ontology, metacog, predictions, lineage, and telemetry only after the core reliability set is satisfied.

| Condition | Context behavior |
|---|---|
| healthy memory/latency | include richer Trajectory Intelligence View, ontology rationale, learning/prediction refs, and evidence summaries. |
| moderate pressure | include compact summaries, top refs, and `do_not_use`; defer deep lineage/replay/telemetry. |
| high pressure or recent OOM | include only core reliability set plus last-known-good summaries; mark rich context unavailable. |
| daemon restart/unavailable | use cached Workpoint/Trajectory/Focus Slice if scope-compatible; avoid claiming canonical state. |

This policy keeps Focusa powerful when resources allow and reliable when resources are constrained: low memory preserves the core reliability set; high memory enables richer context only inside explicit budgets.

### 10.7 Surgical traversal and parsing substrate

Focusa must not treat any large structure as all-or-nothing. Every large tree, graph, list, log, store, or registry must expose surgical traversal/parsing before full payload access.

This applies to all relevant Focusa structures, including but not limited to:

- CLT lineage/tree/path/children/summaries.
- Ontology world, links, communities, working sets, active context, affordances.
- Focus Stack/frame paths and historical frames.
- Workpoint records, evidence records, blockers, drift events, verification records.
- Reference/ECS handles, artifacts, semantic memory, evidence stores.
- Metacognition captures/reflections/adjustments and prediction records.
- Telemetry events, command logs, turn logs, replay bundles, work-loop diagnostics.
- Snapshot indexes/diffs/restore metadata.
- Tool contract registry, docs inventory, capabilities, and agent awareness cards.
- Trajectory Projection input sources and Focus Slice injected context.

Default hot-path traversal contract:

```ts
type SurgicalTraversalRequest = {
  surface: string;                // lineage|ontology|workpoint|metacog|telemetry|snapshot|...
  anchor?: string;                // node id, workpoint id, frame id, ref id, trajectory id
  selector?: string;              // path|children|parents|neighborhood|recent|summary|search|diff
  query?: string;                 // bounded text/semantic search term
  cursor?: string | number;
  limit?: number;                 // default small; hard cap enforced server side
  depth?: number;                 // default 1; hard cap enforced
  radius?: number;                // neighborhood radius; hard cap enforced
  fields?: string[];              // projection fields; default summary fields only
  include_payload?: false;        // true only on cold path with budget check
  budget_tokens?: number;
  project_identity?: ProjectIdentity;
};
```

Default response contract:

```ts
type SurgicalTraversalResponse<T> = {
  status: "completed" | "degraded" | "blocked";
  canonical: boolean;
  surface: string;
  selector: string;
  anchor?: string;
  items: T[];
  summary?: string;
  total_known?: number;
  returned: number;
  cursor?: string | number | null;
  next_cursor?: string | number | null;
  truncated: boolean;
  caps: { limit: number; depth?: number; radius?: number; payload_bytes?: number };
  omitted: string[];
  rehydrate_refs: string[];
  item_tags?: TraverseTagRef[];
  window_tag?: string;
  surface_tag?: string;
  tag_scheme?: TraverseTagScheme;
  failure_class?: "resource_exhausted" | "cold_path_timeout" | "scope_mismatch" | "read_model_lag" | "validation_rejected";
  next_tools: string[];
};
```

Rules:

1. Hot routes default to `summary`, `path`, `children`, `neighborhood`, `recent`, or `search` selectors with small caps.
2. Full tree/graph/log/store payloads are cold-path only and require explicit `include_payload=true` or `mode=full` plus budget checks.
3. Full payload fallback is never automatic. If a surgical selector cannot answer, return what is known with `truncated=true`, `omitted`, `rehydrate_refs`, and next tool guidance.
4. Any route that can grow with project/session history must expose `limit`, `cursor` or an anchor selector, and response metadata.
5. Partial parsing must preserve provenance and scope: ProjectIdentity, frame/workpoint/session identity, and freshness where available.
6. Focus Slice injection must use surgical summaries and selectors only. It must not fetch full lineage, full ontology, full telemetry, or full history to build normal prompt context.
7. Safe audits must fail or warn when a hot route serializes an unbounded full structure by default.
8. Cold routes may be rich and performative under high memory, but they must not be a hog: apply byte caps, token budgets, timeouts, and degradation envelopes.

### 10.8 Daemon flakiness hardening requirements

Live diagnosis on 2026-05-20 showed the daemon process can be active and listening while operator-visible tools intermittently report unavailable or healthcheck probes time out. The issue class is route/runtime flakiness, not only process death.

Observed signals:

- service active at `127.0.0.1:8787`, but stale probes to `127.0.0.1:3030` fail with connection refused.
- `/v1/health`, Workpoint current, bounded work-loop summary, and Trajectory view are normally sub-5ms hot paths.
- legacy `/v1/status` can take 3–10s and occasionally exceed a 10s timeout because it mixes session summary with persistence/process diagnostics.
- full `/v1/work-loop/status` is materially slower than `?summary_only=true` and should be treated as cold/deep by default.
- `reconcile_external_state` serializes full shared/local state on every daemon action when versions match; this creates CPU pressure proportional to state size.
- some API routes mutate shared `FocusaState` directly instead of using reducer events, requiring expensive daemon reconciliation.
- healthcheck fallback to `/v1/status` can turn a transient heavy route into a misleading daemon recovery signal.
- reducer errors for stale active frames should be classified and surfaced, not treated as generic daemon failure.

Required hardening:

1. `/v1/health` remains pure hot liveness: no state lock, no persistence, no process scan, no route fan-out.
2. `/v1/status` must become hot by default or support `summary_only=true`; persistence counts, latest event timestamp, duplicate-process scan, memory/process deep metrics, and replay/worktree data move to `/v1/status/deep` or explicit query.
3. The systemd/Pi healthcheck path must probe `/v1/health` first and may only use bounded hot fallback routes such as `/v1/status?summary_only=true`; it must not use cold/deep routes for restart decisions.
4. Healthcheck logs or telemetry must state which route failed and whether hot fallback recovery was attempted, so route pressure is distinguishable from daemon down.
5. Daemon reconciliation must not JSON-serialize full state on every action. Replace whole-state comparison with explicit mutation/version tracking for API direct writes, then adopt only when the counter/version changes.
6. Routes that directly mutate `state.focusa.write()` must either dispatch reducer actions or mark an external mutation counter and update a cheap version/freshness signal.
7. Long-running or blocking persistence/process reads must use `spawn_blocking`, cached counters, or cold endpoints so Tokio worker threads remain available for `/v1/health` and hot tool routes.
8. Hot route latency should be measured and exposed in bounded diagnostics: p50/p95/p99, timeout counts, recent route failures, restart count, last healthcheck failure route, and current RSS/host memory pressure.
9. Pi tools must classify route timeouts as `hot_path_timeout` or `cold_path_timeout` based on route tier; a timeout on cold `/v1/status/deep` must not mark all Focusa tools unavailable.
10. Restart remains a recovery action, not the primary fix; after code/config changes, restart only to load the new binary/script or recover a stuck listener.

Implementation acceptance:

- healthcheck no longer probes unbounded `/v1/status` by default.
- `/v1/status?summary_only=true` returns in the same order of magnitude as other hot summaries under normal load.
- deep status remains available but is explicitly cold and time/byte bounded.
- daemon logs no longer show repeated full-state adoption caused solely by same-version byte differences during normal Pi turns.
- safe audit includes latency guardrails for health, Workpoint current/resume, Trajectory view, bounded work-loop status, and status summary.
- stress evidence proves no restart storm during route pressure and no false daemon-unavailable result when hot routes are healthy.

---

## 11) Tool result taxonomy

Every Focusa tool must map failures into one of:

- `validation_rejected`
- `frame_unavailable`
- `daemon_unavailable`
- `stale_runtime_registry`
- `resource_exhausted`
- `null_response`
- `hot_path_timeout`
- `cold_path_timeout`
- `writer_conflict`
- `scope_mismatch`
- `approval_required`
- `permission_denied`
- `noncanonical_fallback`
- `read_model_lag`
- `unknown_ambiguous_completion`

Every result must include retry posture:

- `safe_retry`
- `retry_with_idempotency_key`
- `check_side_effects_first`
- `do_not_retry_unchanged`
- `operator_required`

---

## 12) Pi context injection changes

Pi currently injects a minimal Focus Slice. Keep that, but add Trajectory Projection as the top navigation view when available.

Recommended order:

```text
CURRENT_ASK
PROJECT_IDENTITY
QUERY_SCOPE
TRAJECTORY_SUMMARY
WORKPOINT
ACTIVE_OBJECT_SET
VALID_NEXT_ACTIONS
CONSTRAINTS / DECISIONS / FAILURES
EVIDENCE_HANDLES
```

Trajectory Projection summary must be bounded:

```text
TRAJECTORY:
  Root goal stability: stable|clarifying|superseded
  Definition status: clear|provisional|unclear|conflicted
  Highest long-term goal: ...
  Long-term goal: ...
  Desired end state: ...
  Current verified state: ...
  Short-term goal: ...
  Gap: ...
  Active milestone: ...
  Next Workpoint candidate: ...
  Why this next: ...
  Context sufficiency: score=... posture=...
  Missing/conflicting facts: ...
  Do not use: ...
  Relevant learning/predictions: ...
  Ask operator only if: ...
```

If Trajectory Projection is unavailable:

- continue with Workpoint and existing Focus Slice.
- mark `trajectory_status=unavailable`.
- do not block work.

### 12.1 Compaction resume rendering pipeline

After compaction, the model should receive the best bounded summary available for the exact resume point. The renderer must call the newest canonical APIs first, preserve source provenance, and fall back explicitly when a route/tool is unavailable.

Preferred pre-compaction chain:

1. `focusa_workpoint_checkpoint` for the immediate continuation contract, with mission, current action, targets, verified evidence, blockers, `do_not_drift`, and exact next action.
2. `focusa_trajectory_checkpoint` when Trajectory Projection has useful goal/gap/current-state metadata.
3. `focusa_evidence_capture` / `focusa_workpoint_link_evidence` for proof already collected, using handles instead of raw logs.
4. `focusa_traverse` for any bounded lineage/ontology/evidence/tool-registry slices needed by the next model.

Preferred post-compaction chain:

| Need | Correct call | Avoid |
|---|---|---|
| Canonical continuation packet | `focusa_workpoint_resume(mode="compact_prompt" | "full_json")` | relying on transcript tail as canonical |
| Whole-picture orientation | `focusa_trajectory_view(mode="summary")` | treating Workpoint alone as long-term goal truth |
| Narrow supporting context | `focusa_traverse(surface, selector, limit, fields, tags)` | full lineage tree/ontology/log/store reads by default |
| Scope safety | ProjectIdentity verify/session envelope | cwd-only identity or stale session root |
| Tool health/fallback | `focusa_tool_doctor(scope="workpoint")` | restarting daemon or assuming all Focusa is broken |
| Active target ambiguity | `focusa_active_object_resolve` | inventing canonical refs from names |
| Work-loop awareness | `focusa_work_loop_writer_status` or bounded summary status | full work-loop deep status on hot path |
| Evidence continuity | Workpoint evidence records and evidence refs; link only when adding proof | dumping raw command output into the packet |
| Tool choice | Tool affordance catalog / `TOOL_AFFORDANCES` | asking model to infer tool purpose from names |

Corrected call rules:

1. `focusa_workpoint_resume` is the first canonical resume call after compaction; `focusa_tool_doctor` is recovery/diagnostic, not a replacement packet.
2. `focusa_trajectory_view` should be read before acting when available so the Workpoint next action is tied to long-term goal, desired end state, current verified state, active gap, and `why_this_next`.
3. `focusa_traverse` should replace broad `focusa_lineage_tree`/full tree reads for default resume context; older tree/path tools remain compatibility fallbacks with strict caps.
4. Work-loop reads must use writer/status summary routes by default; deep replay/worktree diagnostics are cold opt-in.
5. Mutating tools after compaction require clear intent: do not call checkpoint/link/control tools just to inspect state.
6. Every tool result used in the packet must expose `tool_result_v1` fields or equivalent provenance: status, canonical/degraded, failure_class, retry posture, side effects, evidence refs, and next tools.
7. If a packet is built from cached or local fallback data, its rendered header must say `canonical=false` and name the recovery tool chain.

Minimum rendered packet shape:

```text
WORKPOINT_RESUME_PACKET v2 canonical=true|false degraded=true|false
PROJECT: <canonical_name> <fingerprint/confidence>
WORKPOINT: <id> mission=<...>; action=<...>; next=<...>
TRAJECTORY: goal=<...>; current=<...>; gap=<...>; why_next=<...>; sufficiency=<...>
ACTIVE_OBJECTS: <bounded refs>
EVIDENCE: <handles only>
TRAVERSAL: <bounded slices/tags/rehydrate refs>
TOOLS: best_next=<...>; recovery=<...>; do_not_use=<...>
FAILURES/WARNINGS: <failure_class + retry posture>
DO_NOT_DRIFT: <boundaries>
```

Quality gates:

- A compaction packet must be useful to a fresh model without reading the old transcript.
- The packet must explain both immediate Workpoint continuation and larger Trajectory relevance when available.
- The packet must never hide degraded/cached/local fallback status.
- The packet must not include raw giant logs; use evidence handles and traversal rehydrate refs.
- Golden evals must compare old transcript-tail resumes against v2 packet resumes for tool-choice accuracy, drift reduction, and context sufficiency.

---

## 13) Constraint/tool instruction fixes

Update tool docs and skills:

| User/agent input | Correct tool |
|---|---|
| temporary steering: “evaluate only” | `focusa_scratch` or current focus |
| durable architecture boundary | `focusa_constraint` |
| architectural choice | `focusa_decide` |
| debugging/investigation notes | `focusa_scratch` |
| specific failure + diagnosis | `focusa_failure` |

Example durable constraint:

```text
Focusa release/deployment workflows require explicit operator approval before build, restart, or deploy actions.
```

Example scratch only:

```text
Operator temporarily restricted this pass to evaluation only; no build/restart/deploy during current diagnostic pass.
```

### 13.1 Official tool remediation map

| Tool surface | Required alignment |
|---|---|
| `focusa_decide` | Enforce one crystallized architectural sentence and the canonical Focus State compact limit; no task/debug/process notes. |
| `focusa_constraint` | Accept durable operator/spec/environment boundaries, but route temporary steering and agent commitments to scratch/current focus. |
| Work-loop status/writer/doctor | Use bounded `status?summary_only=true` on default hot paths; replay/deep diagnostics are opt-in cold paths. |
| Work-loop control/context/select-next | Preserve writer/preflight semantics and classify writer conflicts as blocked, not daemon failures. |
| Workpoint checkpoint/resume/evidence | Include shared session/project identity envelope before treating a packet as canonical. |
| Prediction tools | Keep predictions advisory and calibration-bound; predictions never choose work or override operator steering. |
| Tree/lineage/metacog reads | Treat `pending`, stale, or lagged read-model output as recoverable `read_model_lag`, not success. |
| All tools | Return `tool_result_v1` with `failure_class`, retry posture, `canonical/degraded`, side effects, evidence refs, and next tools. |

### 13.2 Official tool evaluation and repair ledger

This spec must carry the official-tool evaluation results forward into decomposition. The ledger below is the minimum repair backlog; decomposition should create or update beads from it instead of rediscovering failures.

| Surface | Evaluation result | Suggested repair |
|---|---|---|
| `focusa_decide` | Public validator allowed longer text than canonical Focus State slot; multi-sentence decisions could survive first validator. | Keep one-sentence architectural choice limit aligned with Focus State compact slot; reject task/debug/process wording before write. |
| `focusa_constraint` | Durable operator boundaries and temporary steering can be confused; task-pattern validation can misclassify operator approval boundaries; no active Pi frame can prevent Focus State write even after validation. | Treat operator/spec/environment boundaries as durable constraints; route temporary steering to scratch/current focus; ensure operator-directive allowance runs before task-pattern rejection; classify missing active frame as `frame_unavailable` with scratch fallback. |
| Focus State slot tools (`intent`, `current_focus`, `next_step`, `open_question`, `recent_result`, `note`) | Validation protects compact state but result details can look generic. | Keep strict compact validators; return `tool_result_v1.failure_class` and retry posture on all rejected/offline writes. |
| `focusa_work_loop_status` | Full status mixed hot state with replay/worktree/deep diagnostics and could time out. | Default to bounded `GET /v1/work-loop/status?summary_only=true`; move replay/deep diagnostics to opt-in cold routes/tools. |
| `focusa_work_loop_writer_status` | Writer ownership reads depend on the same status surface. | Use bounded summary route; expose writer/preflight guidance without mutation. |
| `focusa_work_loop_control` | Writer conflicts can look like generic failure. | Preserve preflight mode; classify claimed-writer cases as `writer_conflict` blocked state with operator/owner guidance. |
| `focusa_work_loop_context` | Current ask/scope updates are authority-sensitive. | Keep writer identity, source turn, steering flags, and excluded-context labels explicit; classify approval/writer/scope failures. |
| `focusa_work_loop_checkpoint` | Continuous-loop checkpoint can be confused with Workpoint checkpoint. | Keep terminology distinct; return side-effect profile and retry posture. |
| `focusa_work_loop_select_next` | Could be mistaken for Trajectory selecting work. | Keep Beads/work-loop authority explicit; Trajectory may only propose candidates, while select-next remains governed Work-loop action. |
| State hygiene tools | Hygiene is diagnostic/proposal-only today. | Keep doctor/plan read-only; `apply` remains approval-gated non-destructive placeholder until reducer-backed hygiene exists. |
| `focusa_tool_doctor` | Diagnostic readiness must not hang on cold paths. | Use health, Workpoint current, and bounded work-loop summary by default; classify daemon down, timeout, writer conflict, scope mismatch, approval gate, stale live registry. |
| `focusa_active_object_resolve` | May return likely refs, not canonical truth. | Label results as candidates unless verified by Workpoint/ontology/evidence; include uncertainty in Trajectory Intelligence View. |
| `focusa_evidence_capture` / `focusa_workpoint_link_evidence` | Evidence links can be pending or invisible due read-model lag. | Return evidence refs and `read_model_lag` when accepted-but-not-visible; retry current/resume before relying on link. |
| `focusa_workpoint_checkpoint` | Checkpoint/resume identity parity incomplete across paths. | Add shared `FocusaSessionIdentity` with ProjectIdentity to all checkpoint/resume/evidence payloads. |
| `focusa_workpoint_resume` | Correctly rejects scope mismatch, but fallback can be noncanonical. | Keep strict project/session guard; provide degraded fallback guidance without overriding current operator/repo truth. |
| Tree/head/path/snapshot/diff/restore tools | Snapshot/lineage reads can lag or be confused with memory/focus authority. | Keep CLT as lineage/recovery only; classify stale or missing read model as `read_model_lag`; restore remains explicit recovery action only. |
| Metacog capture/retrieve/reflect/adjust/evaluate tools | Learning signals can be too weak or promoted without evidence. | Require reusable signal, confidence, rationale, evidence refs where possible; evaluate outcomes before promotion. |
| Metacog recent/doctor/loop tools | Composite tools can hide weak substeps. | Return substep statuses and quality gates; classify partial completion as degraded, not success. |
| Lineage intelligence tools | Extracted risks/decisions are candidates. | Treat extraction as advisory metacog input; cite lineage/evidence refs and avoid mutating Focus State directly. |
| Prediction tools | Predictions can be mistaken for decisions. | Keep prediction records bounded, evidence-calibrated, evaluated after outcomes, and advisory only. |
| Live proof harness | Safe fixtures previously probed expensive work-loop status; live registry can differ from static after code edits before restart. | Use bounded summary route; separate static validation, safe endpoint fixtures, and daemon-registry freshness failure (`read_model_lag`/stale daemon). |
| Tool contract registry/docs | Contracts/docs can drift from registered Pi tools. | Static validator remains required gate: registered tools = contracts = docs; registry query strings normalize to route inventory path. |
| All official tools | Failure classes were too coarse and prose-only recovery made tools hard to use. | Every tool result should expose `failure_class`, retry posture, canonical/degraded, side effects, evidence refs, and next tools. |
| Future `focusa_trajectory_*` tools | Risk of becoming a planner/orchestrator. | Implement composition-first advisory views only; no task selection, execution, Focus Stack switching, Work-loop control, or auto-promotion. |

### 13.3 Related tool improvements surfaced by Trajectory

Trajectory Projection increases value when surrounding tools expose the right cognitive fields:

| Tool/surface | Improvement |
|---|---|
| Workpoint checkpoint/resume | Add `why_this_next`, `goal_link`, `current_state_delta`, and `completion_evidence_required`. |
| Evidence capture/link | Add `proves`, `refutes`, `freshness`, `gap_ref`, and confidence so evidence can support or reject a Trajectory gap. |
| Active object resolve | Return rationale, confidence, blocked/unsafe refs, and `do_not_use` refs. |
| Metacog retrieve/doctor | Query automatically from active gap, repeated failure class, and current milestone. |
| Prediction tools | Forecast drift risk, context sufficiency, next-action success, and verification burden. |
| Tool doctor | Diagnose Trajectory health plus tool-suite reliability: missing long-term goal, missing current-state proof, stale Workpoint, weak evidence, ProjectIdentity mismatch, frame drift, stale runtime registry, OOM/resource pressure, and null responses. |
| Focus Slice injection | Inject Trajectory/Intelligence View automatically when available; model should not need to remember to call it. |
| Goal provenance | Version and supersede goals with source, timestamp, operator turn, and evidence refs. |
| Golden evals | Compare model behavior with and without Trajectory for goal retention, drift, recovery, and assistance requests. |

### 13.4 Decomposition rule for tools

Before implementation decomposition is complete, every row in the ledger must map to one of:

- an already-applied repair with evidence,
- an open bead,
- an explicit non-goal with rationale,
- or a blocked item with the required approval/dependency.

No official Focusa tool should be left as an unevaluated surface.

### 13.5 Tool error troubleshooting matrix

Focusa stability requires predictable recovery for every official tool failure class:

| Failure class | Typical cause | Tool behavior | Recovery |
|---|---|---|---|
| `validation_rejected` | Slot wording too verbose, task-like, temporary steering in durable slot, invalid enum/schema. | No canonical write; explain field and rule. | Rewrite compactly or use `focusa_scratch`; do not retry unchanged. |
| `frame_unavailable` | Pi has no active Focus Frame/session frame key; observed with `focusa_constraint` returning “No active Pi frame”. | Do not claim Focus State write; save scratch fallback. | Re-establish/ensure Pi frame, then retry durable write if still relevant. |
| `daemon_unavailable` | Daemon down, wrong port, connection refused, auth/base URL mismatch. | Degraded/noncanonical result only. | Check `/v1/health`, base URL, service state; continue from operator/repo fallback for non-safety work. |
| `stale_runtime_registry` | Static contracts/docs changed but running daemon still serves old registry before approved restart. | Safe fixtures may pass while live payload equality fails. | Treat as stale runtime/read-model lag; static validation remains source until approved rebuild/restart. |
| `resource_exhausted` | Kernel OOM, memory pressure, oversized in-memory store, or unbounded frame/path growth. | Degraded/unavailable tools may appear during kill/restart; hot routes may flap. | Use cached last-known-good data, cap stores/routes, audit memory, and avoid cold payloads until pressure resolves. |
| `null_response` | HTTP wrapper returns null/empty body or hides upstream status/reason. | Tool lacks actionable detail. | Preserve status/body/error in `tool_result_v1.raw`; classify retry posture instead of prose-only failure. |
| `hot_path_timeout` | Bounded readiness/status route exceeded budget. | Return degraded hot-path failure without cold replay. | Retry once; run tool doctor; avoid blocking normal work unless safety/scope depends on it. |
| `cold_path_timeout` | Replay, worktree scan, release proof, or deep diagnostic exceeded budget. | Mark diagnostic degraded, not tool-suite broken. | Use summary route; schedule/opt into cold diagnostic later. |
| `writer_conflict` | Work-loop controlled by another writer/session. | Block mutation; read-only tools still allowed. | Use writer-status/preflight; do not force ownership without policy/operator approval. |
| `scope_mismatch` | Project/session/Workpoint identity mismatch. | Reject stale packet; do not inject transcript tail as canonical. | Use current repo/operator ask; run ProjectIdentity verify; checkpoint fresh packet. |
| `approval_required` | Destructive/restart/deploy/governance action lacks approval. | Block mutation. | Ask operator only for explicit approval; do not infer approval. |
| `permission_denied` | Auth token, filesystem, cPanel user/root boundary, or API permission issue. | Block mutation/read as applicable. | Use correct identity/tool; preserve cPanel/user safety rules. |
| `noncanonical_fallback` | Local fallback, degraded Workpoint, stale packet, or missing daemon proof. | Expose as recovery hint only. | Verify canonical source before important continuation. |
| `read_model_lag` | Accepted write/evidence not visible yet; stale projection. | Return pending/degraded, not success. | Retry current/resume/read-model route with idempotency; avoid duplicate writes. |
| `unknown_ambiguous_completion` | Tool result cannot prove success or failure. | Mark ambiguous; include raw/details. | Check side effects first; do not blind retry mutating action. |

Tool docs must make the recovery path obvious enough that another model can pick the correct next tool without reading source code.

### 13.6 Model-facing tool affordance catalog

Focusa tools must be easy for any model to discover, choose, and chain. Tool availability is part of cognition, not hidden implementation detail.

Every official tool must be advertised to the model through a bounded affordance catalog with:

- `name` and short label,
- family and authority surface,
- when to use,
- when not to use,
- required inputs and common defaults,
- side-effect profile (`read_only`, `write_state`, `checkpoint`, `control_state`, etc.),
- safety/approval requirements,
- common failure classes and recovery posture,
- example invocation,
- expected evidence/result shape,
- likely `next_tools`,
- related tools and preferred workflow chains.

The model should not need to infer tool purpose from name alone. If a tool is relevant to the active Trajectory, Workpoint, failure class, or valid next action, Focusa should advertise it explicitly.

### 13.7 Tool advertisement in Focus Slice

Focus Slice injection should include a compact `TOOL_AFFORDANCES` section when helpful:

```text
TOOL_AFFORDANCES:
  best_next:
    - focusa_trajectory_view — see goal/state/gap/candidate before acting
    - focusa_active_object_resolve — resolve ambiguous target refs
    - focusa_evidence_capture — preserve proof after verification
  recovery:
    frame_unavailable -> focusa_tool_doctor, ensure frame, retry durable write
    scope_mismatch -> focusa_project_verify, focusa_workpoint_checkpoint
  do_not_use:
    - focusa_work_loop_control unless mutating continuous loop is intended
```

Rules:

- Advertise only the smallest useful subset by default; keep the full registry available on demand.
- Include `best_next` tools from Trajectory, Workpoint, active object set, and failure taxonomy.
- Include `do_not_use` tools when a tool would be unsafe, off-scope, or authority-confusing.
- Prefer workflow chains over isolated tool names.
- Never advertise a mutating/control tool as a default next step unless the intent and authority are clear.

### 13.8 Workflow-power chains

Tools make the model more powerful when they form predictable execution workflows. These chains should be documented, tested, and advertised:

| Workflow need | Preferred chain |
|---|---|
| Start or resume work | `focusa_trajectory_view` -> `focusa_workpoint_resume` -> `focusa_active_object_resolve` |
| Trajectory unclear | ProjectIdentity verify -> Trajectory clarity gate -> local evidence verification -> operator confirm only if still unclear |
| Before risky/long work | `focusa_workpoint_checkpoint` -> verify active objects -> capture expected evidence hooks |
| After verification | run local test/check -> `focusa_evidence_capture` -> `focusa_workpoint_link_evidence` -> update Trajectory current-state delta |
| Tool failure | inspect `failure_class` -> preserve raw status/body -> use matrix recovery -> retry only with correct posture |
| Compaction/model switch | `focusa_workpoint_checkpoint` + Trajectory summary + bounded `focusa_traverse` slices -> resume via `focusa_workpoint_resume`, `focusa_trajectory_view`, tool affordances, and `focusa_tool_doctor` fallback |
| Repeated failure | `focusa_metacog_retrieve` -> apply relevant lesson -> prediction risk update -> proceed/verify/operator posture |
| Scope ambiguity | ProjectIdentity verify -> `focusa_active_object_resolve` -> suppress mismatched refs in `do_not_use` |

Workflow chains are suggestions, not automation authority. The model remains responsible for choosing the correct tool under operator and safety constraints.

### 13.9 Tool usability acceptance criteria

Before decomposition completes:

- every official `focusa_*` tool has a contract, doc page, prompt-facing description, example, side-effect profile, failure classes, and recovery guidance.
- every tool family has a short model-facing decision guide.
- every tool result includes `next_tools` or an empty list with reason.
- tool doctor can report missing docs/contracts/schema/result-envelope coverage.
- Trajectory/Workpoint can advertise the top relevant tools for the current gap.
- golden tasks verify that a model can choose the right tool from the advertised affordances without reading source.
- mutating tools are clearly distinguished from read-only diagnostic tools.

---

## 14) Implementation sequence

0. Preserve current behavior behind existing tools and routes; add compatibility tests before changing defaults.
1. Document this spec and create bead decomposition.
2. Add ProjectIdentity marker schema, discovery, fingerprinting, and verification APIs/tools.
3. Add shared `FocusaSessionIdentity` builder in Pi extension using ProjectIdentity.
4. Add identity envelope to all Workpoint checkpoint/resume/evidence paths.
5. Split work-loop status hot/cold routes or make current summary route canonical default.
5a. Split daemon `/v1/status` hot/deep routes and remove cold status from healthcheck restart decisions.
5b. Replace daemon whole-state JSON reconciliation with explicit external mutation/version tracking.
5c. Add explicit ResourceMode/LowMem policy, budgets, mode detection, `focusa_resource_mode` activation/deactivation, and mode-aware tool envelopes.
6. Update Spec91 proof harness to use bounded status for safe fixtures.
7. Add Trajectory read/composition path over existing primitives first; prove it only projects, never selects or executes.
8. Add Trajectory Intelligence View with context sufficiency, relevance rationale, uncertainty register, negative context, current-state delta, learning refs, and prediction refs.
9. Add Definition-of-Done proof contract and goal provenance/supersession metadata.
10. Add Trajectory lifecycle logic: initial definition protocol, inference policy, continuous per-session clarity gate, root-goal stability, change/supersession rules, and operator confirmation path.
11. Add Trajectory core types and reducer events only for projection metadata and accepted checkpoints.
12. Add Trajectory API routes with hot-path summary defaults and cold-path full diagnostics.
13. Add Pi `focusa_trajectory_*` tools with advisory Workpoint candidates only; prefer `view`, `define_goal`, `assess`, and `propose_workpoint` naming over control verbs.
14. Add ProjectIdentity, Trajectory, and Intelligence View sections to Focus Slice injection.
15. Add `focusa_traverse` and `POST /v1/traverse` over the shared surgical traversal substrate, including item/range/window/surface tags and tag verification.
16. Add Workpoint Resume Packet v2 renderer/injector for compaction and model switch using ProjectIdentity, Trajectory, Workpoint resume, `focusa_traverse`, `tool_result_v1`, and tool affordances.
17. Add model-facing Tool Affordance Catalog and Focus Slice `TOOL_AFFORDANCES` injection.
18. Update tool docs/skills/result-envelope taxonomy and related tool fields from §13.3.
19. Add tests for degraded daemon behavior, low-memory/core-reliability mode, surgical traversal/parsing across large structures, project identity quorum, scope identity parity, no-authority drift, assistance minimization, Definition-of-Done proof, continuous trajectory clarity gate, root-goal stability, tool-choice usability, and Trajectory Projection→Workpoint handoff.

---

## 15) Acceptance criteria

### Surgical traversal

- `focusa_traverse` returns bounded slices with item/range/window/surface tags and tag verification support.
- Every large Focusa structure supports bounded selector/window/path/neighborhood traversal before full payload access.
- Hot routes do not serialize full lineage trees, ontology graphs, telemetry logs, metacog stores, snapshot bodies, or tool registries by default.
- Responses expose returned/total/truncated/cursor/caps/omitted/rehydrate_refs metadata.
- Safe audit detects and reports any all-or-nothing traversal surface on hot paths.

### Compaction/resume packets

- Workpoint Resume Packet v2 is generated from canonical hot-path APIs after compaction/model switch when available.
- Packet rendering uses `focusa_workpoint_resume`, `focusa_trajectory_view`, `focusa_traverse`, ProjectIdentity/session identity, evidence handles, and tool affordance guidance.
- Packet headers and JSON distinguish canonical live data from cached, stale, local, or noncanonical fallback data.
- Packets include API/tool provenance, failure classes, retry posture, `do_not_use`, best-next tools, and bounded traversal tags/rehydrate refs.
- A fresh model can resume from the packet without old transcript memory and choose the correct next Focusa tool chain.

### Stability

- LowMem mode keeps every official tool callable through bounded summaries, degraded envelopes, or rehydrate refs; public tool disappearance is not an acceptable pressure response.
- `focusa_resource_mode` activates/deactivates LowMem at runtime and is advertised to models under resource pressure or operator LowMem commands.
- `/v1/status` summary and healthcheck fallback are hot-path bounded; deep persistence/process diagnostics are explicit cold paths.
- Daemon state reconciliation uses explicit mutation/version signals instead of full-state JSON comparison on every action.
- `/v1/health` and bounded work-loop/trajectory/workpoint summary routes respond even when replay/deep diagnostics are slow.
- Pi tools never block normal work solely because Focusa daemon is unavailable.
- Tool Doctor distinguishes daemon down, OOM/resource pressure, RSS/store pressure, null response, cold-path timeout, writer conflict, scope mismatch, stale runtime registry, frame unavailable, and approval gate.
- Core reliability set routes keep returning bounded live or cached degraded summaries in low-memory environments.

### Project identity

- Focusa can identify a project from `.focusa-project.json`, git root/remote, beads root/prefix, workspace files, cwd, and persisted session metadata.
- Canonical project scope requires a high-confidence ProjectIdentity fingerprint derived from multiple matching signals.
- Cwd-only identity is allowed only as degraded fallback.
- Conflicting marker/git/beads/cwd signals produce explicit mismatch diagnostics, not silent scope selection.

### Session identity

- All Workpoint and Trajectory Projection checkpoint/resume paths include the same session identity envelope with ProjectIdentity.
- A checkpoint created by Pi can be resumed by the same Pi project/session without self-mismatch.
- Cross-project/session mismatch still rejects stale packets.

### Trajectory

- Agent can read one bounded Trajectory Projection summary containing long-term goal, desired end state, current state, short-term goal, active gap, and next Workpoint.
- Trajectory Projection can propose a Workpoint candidate without replacing Workpoint, Beads, Focus Stack, Work-loop traversal, or operator authority.
- Trajectory Projection degrades to Workpoint/Focus Slice/local fallback when unavailable.
- Initial Trajectory definition follows explicit source precedence, inference policy, provenance, clarity status, and operator confirmation rules.
- Trajectory clarity is refreshed continuously per work session at startup/resume, steering changes, Workpoint transitions, evidence updates, failures, degradation, and handoff.
- The highest/root long-term goal remains stable across normal work; radical changes require explicit operator or durable supersession evidence.
- Trajectory changes over time through explicit supersession or evidence-backed state deltas, with no silent long-term goal changes.

### Model intelligence

- Agent receives a bounded Trajectory Intelligence View that binds long-term goal, desired end state, current verified state, active short-term goal, active gap, next Workpoint candidate, evidence refs, context sufficiency, missing facts, stale refs, conflicting signals, relevance rationale, current-state delta, learning refs, prediction refs, and assistance boundary.
- Negative context is visible through `do_not_use` so stale packets, wrong-project refs, superseded goals, and unsafe assumptions do not silently poison the prompt.
- The model can tell whether to proceed, verify first, or request operator input based on evidence, scope, and `definition_status`, not transcript guesswork.
- Operator-assistance requests per completed Workpoint decrease without increasing drift, scope mismatch, or unsafe actions.
- Desired-end-state completion requires explicit Definition-of-Done evidence, not model confidence alone.

### Tool usability and workflow power

- Every official tool is advertised through a bounded affordance catalog with when-to-use, when-not-to-use, side effects, examples, failure recovery, and likely next tools.
- Focus Slice can expose `TOOL_AFFORDANCES` with best-next, recovery, and do-not-use tool guidance for the active Trajectory/Workpoint.
- Golden tasks prove models can choose correct tools and workflow chains from advertised affordances without source-code inspection.
- Read-only, mutating, checkpoint, and control tools are clearly distinguished.

### Current model preservation

- Focus State, Focus Stack, Workpoint, Work-loop, ontology, metacog, evidence, lineage, Beads, and predictions all remain active inputs.
- No existing primitive is demoted or removed.
- Trajectory Projection is documented as a projection/navigation view, not a planner, scheduler, or action authority.

---

## 16) Success condition

This spec is successful when Focusa lets agents see the long-term destination and short-term execution step simultaneously, while preserving all current cognitive primitives and making daemon/tool/session instability degrade into bounded, explainable fallback rather than blocked work.


## Spec96 trajectory multi-agent golden eval

`tests/spec96_trajectory_agent_golden_eval_test.sh` is the static golden eval harness for trajectory orientation across Pi, CLI/API, and generic agents. It compares enriched trajectory prompt surfaces against without-trajectory baselines and covers project mismatch, compaction, degraded daemon mode, drift avoidance, assistance reduction, and proof-based Definition of Done.


## Trajectory clarity gate contract

Trajectory view exposes a clarity gate with `clear`, `provisional`, `unclear`, and `conflicted` states. Missing long-term/desired goals returns `operator_input`; stale evidence or incomplete current-state proof returns `verify_first`; project or continuity conflicts return `verify_first`. Root goal supersession is valid only with operator confirmation or durable supersession evidence.


## Trajectory-to-Workpoint advisory handoff

`focusa_trajectory_propose_workpoint` returns an `advisory_workpoint_candidate_v1` with action intent, target refs, verification hooks, blockers, `do_not_drift`, and `checkpoint_required=true`. The proposal has `no_execution_side_effects=true`; canonical continuation requires `focusa_workpoint_checkpoint`, and work-loop selection/execution is forbidden from the trajectory proposal path.


## Traverse + Resume Packet v2 golden eval

`tests/spec96_traverse_resume_v2_golden_eval_test.sh` compares old transcript-tail resume against Workpoint Resume Packet v2 plus `focusa_traverse`. It requires narrow slices, tag verification, API provenance, failure taxonomy, tool-choice accuracy, drift reduction, and safe-audit guards for daemon unavailable, stale tag, scope mismatch, and cold-path timeout cases.


## Ontology active-context traversal posture

Ontology world/context hot paths expose `selector`, `field_projection`, `traversal_metadata`, `do_not_use`, and `rehydrate_refs`/routes. Active-context prompts use `surgical_summary_only` and forbid `full_ontology_graph`/broad object-link serialization by default.


## Ontology identity axes

Ontology may project bounded identity axes for orientation: `project_root`, `continuity_id`, daemon/runtime session id, adapter `session_id`, and Workpoint continuation card. This projection is advisory and includes rehydrate refs; the authority gate remains `project_root + continuity_id`. Daemon/session identifiers are runtime metadata and must not become resume authority.


## Store surface traversal posture

Large store/list surfaces—ECS/evidence refs, metacognition artifacts, telemetry trace/events, snapshots, Workpoints, and Focus Stack—expose bounded default windows with `limit`, `cursor`, `next_cursor`/metadata, and targeted rehydrate refs where applicable. Cold/full payloads require explicit opt-in or dedicated rehydrate routes; hot-path callers use summary windows.


## Traversal budget golden eval

`tests/spec96_traversal_budget_golden_eval_test.sh` proves that low-memory agents can request narrow `focusa_traverse` slices for partial lineage, ontology, evidence/ECS, metacognition, snapshots, and trajectory surfaces instead of dumping full history/graphs/logs. The safe audit fails missing surgical surfaces or missing budget controls.


## Stale active-frame validation

`POST /v1/focus/update` treats stale explicit `frame_id` writes as scoped validation failures, not daemon-wide flakiness. Responses include `target_frame_id`, `active_frame_id`, `failure_class=frame_unavailable|scope_mismatch`, `diagnostic_class=stale_active_frame_or_read_model_lag`, and recovery guidance. Pi Focus State tools refresh scoped frame identity, retry once, and mirror failed writes to scratchpad fallback.


## LowMem surgical-agent stress

`tests/spec96_lowmem_surgical_agent_stress_test.sh` forces LowMem and proves no tool disappearance, hot-route liveness after cold pressure, no restart storm/uptime reset, explicit degradation metadata for cold full-payload routes, and fresh-agent task completion using summaries plus `focusa_traverse`.
