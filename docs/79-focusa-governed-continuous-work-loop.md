# 79 — Focusa-Governed Continuous Work Loop (FGCWL)

Status: Draft for implementation  
Owner: Focusa runtime / Pi integration  
Depends on: `docs/core-reducer.md`, `docs/G1-detail-03-runtime-daemon.md`, `docs/44-pi-focusa-integration-spec.md`, `docs/52-pi-extension-contract.md`, `docs/41-proposal-resolution-engine.md`, `docs/46-ontology-core-primitives.md`, `docs/54a-operator-priority-and-subject-preservation.md`, `docs/54b-context-injection-and-attention-routing.md`, `docs/56-trace-checkpoints-recovery.md`, `docs/57-golden-tasks-and-evals.md`, `docs/61-domain-general-cognition-core.md`, `docs/67-query-scope-and-relevance-control.md`, `docs/68-current-ask-and-scope-integration.md`, `docs/70-shared-interfaces-statuses-and-lifecycle.md`, `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`, `apps/pi-extension/src/state.ts`, `apps/pi-extension/src/tools.ts`, Pi `docs/rpc.md`, Pi `docs/extensions.md`

---

## 0. Source Basis

This spec is intentionally source-backed. Every major design claim below is derived from one or more existing Pi or Focusa sources.

It also inherits the project operating workflow already in force for this environment:
- **Docs-first**: specifications and authoritative docs define what correct execution means.
- **BD-first**: ordered `bd` work items are decomposed from the supreme spec and serve as the executable work graph for decomposition, progress, blocking, and completion.

The continuous executor must therefore treat docs as completion authority and `bd` as the primary traversal substrate for multi-task and multi-tranche runs. The authoritative spec is the supreme normative source for execution, interpretation, completion, and conflict resolution. `bd` is a reliable guide because it is spec-derived, but `bd` never outranks spec; if there is any conflict, ambiguity, or tension, the authoritative spec decides.

### Pi sources
- Pi RPC mode exposes a headless JSON protocol over stdin/stdout, including `prompt` and `abort` commands and streamed events such as `agent_start`, `agent_end`, `turn_start`, `turn_end`, and `message_update` (`/opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/docs/rpc.md`).
- Pi extensions can register commands, tools, UI surfaces, and lifecycle hooks such as `session_start`, `session_shutdown`, and `tool_call`, but are still extension logic running inside Pi (`/opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/docs/extensions.md`; `/opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/examples/extensions/system-prompt-header.ts`).

### Focusa sources
- Focusa must remain the single cognitive authority while Pi stays extension-thin and does not maintain parallel cognition (`docs/44-pi-focusa-integration-spec.md`, `docs/52-pi-extension-contract.md`).
- The reducer is the sole canonical mutation surface and must remain pure, replayable, and side-effect free (`docs/core-reducer.md`).
- The daemon owns authoritative state, persistence, API surface, background scheduling, and event emission/logging (`docs/G1-detail-03-runtime-daemon.md`).
- Continuous/autonomous work must remain bounded by operator priority, ontology authority, scope control, verification, and explicit stop conditions (`docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`, `docs/54a-operator-priority-and-subject-preservation.md`, `docs/54b-context-injection-and-attention-routing.md`, `docs/67-query-scope-and-relevance-control.md`, `docs/68-current-ask-and-scope-integration.md`).
- Recovery requires traceability, checkpoints, and resumability (`docs/56-trace-checkpoints-recovery.md`).
- Focusa's value must be shown through measurable evals and realistic continuity/recovery tasks (`docs/57-golden-tasks-and-evals.md`).
- Domain-general cognition requires decomposition, constraint tracking, blocker handling, verification, recovery, and finishing loops rather than drifting (`docs/61-domain-general-cognition-core.md`).
- Existing code already contains the relevant harness-edge concepts: `active_turn`, `TurnStarted`, `TurnCompleted`, `PromptAssembled`, and daemon `Action::EmitEvent`, plus Pi-extension state for `PiCurrentAsk`, `PiQueryScope`, `PiExcludedContext`, and scratchpad-vs-Focus-State separation (`crates/focusa-core/src/types.rs`, `apps/pi-extension/src/state.ts`, `apps/pi-extension/src/tools.ts`).

---

## 1. Purpose

Define a **continuous work mode** where Focusa can drive Pi or another supported harness/model across turn boundaries, task boundaries, and tranche boundaries **without manual user prodding**, while preserving Focusa's core laws:

- Focusa remains the **single cognitive authority**.
- Canonical state remains **reducer-expressed, replayable, auditable**.
- Continuous execution remains **bounded, scope-aware, operator-prioritized, and stoppable**.
- Pi or any other supported harness/model remains a **worker transport**, not the owner of cognition or continuation policy.
- A properly scoped and decomposed project may run from one task through many tranches until completion without requiring reprompt-to-continue behavior.

This document exists because normal chat turns end when the assistant replies. If the operator wants uninterrupted progress, the system needs an explicit outer execution loop rather than etiquette or repeated manual `continue` prompts.  
**Sources:** Pi RPC turn/event model in `docs/rpc.md`; Focusa single-authority rule in `docs/44-pi-focusa-integration-spec.md`; bounded autonomy rules in `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`; plan/decomposition continuity requirements in `docs/61-domain-general-cognition-core.md` and `docs/57-golden-tasks-and-evals.md`.

---

## 2. Executive Summary

The correct architecture is:

- **Reducer** decides canonical cognitive state transitions only.
- **Daemon** owns continuation policy, project-execution policy, and loop authority.
- **RPC/SDK transport** provides the mechanism to re-enter Pi or another supported harness after each turn.
- **Harness extension** remains thin UX glue, observability, and operator controls where applicable.
- **Ordered project decomposition** (for example `bd` task/tranche graphs) becomes executable input to the daemon rather than a passive note system.

Therefore:

> Continuous work is a **daemon-governed project executor** running over harness RPC/SDK transport, not an extension-only behavior and not reducer logic.

This follows directly from: reducer purity (`docs/core-reducer.md`), daemon runtime ownership (`docs/G1-detail-03-runtime-daemon.md`), Pi-extension-thin authority rules (`docs/44-pi-focusa-integration-spec.md`, `docs/52-pi-extension-contract.md`), and Focusa's emphasis on decomposition, blocker handling, recovery, and loop completion (`docs/61-domain-general-cognition-core.md`).

### 2.1 Highest-Value Kernel
To avoid overengineering, the implementation should start with the smallest set of features that delivers the operator outcome.

Highest-value kernel:
1. authoritative spec supremacy
2. `bd`-driven ready-work traversal
3. daemon-owned auto-continuation across turns/tasks/tranches
4. blocker recovery/defer/alternate-work behavior
5. verification-before-close
6. visible status + checkpoint/resume
7. no destructive or governance-breaching actions without approval

If a feature does not materially improve one of those seven outcomes, it should be deferred from the first implementation.

### 2.2 Deferred-by-Default Complexity
The following are useful but lower priority than the kernel above and should be treated as deferred unless they unlock an immediate operator outcome:
- sophisticated worker/model routing heuristics beyond basic fallback
- rich UI surfaces beyond essential status/control
- advanced trust-level delegation mechanics beyond explicit manual delegation
- fine-grained budget optimization beyond sane defaults
- exhaustive ontology embellishment beyond what is needed for mission/constraint/provenance/verification integration
- any abstraction that duplicates daemon policy or spec authority

---

## 3. Non-Negotiable Design Laws

### 3.0 Docs-First + BD-First
The executor MUST treat:
- the operator/spec author, or an explicitly trusted delegated spec author, as the only authority above the current spec
- authoritative docs/specs as the supreme authority for correctness, interpretation, and closure below that authoring authority
- ordered `bd` work items as the default executable work graph derived from those specs

Therefore:
- work selection comes from ready/dependency-satisfied `bd` items unless explicit override says otherwise
- `bd` is the default operational guide for decomposition and traversal
- task completion is determined by doc/spec acceptance plus verification, not by the worker's subjective sense of done
- auto-advancement means traversing the `bd` graph, not merely sending another prompt
- `bd` NEVER outranks spec
- if `bd` guidance is ever ambiguous, incomplete, or apparently inconsistent with the linked authoritative spec, the executor must return to the spec
- if the worker's preferred implementation conflicts with the linked spec, the spec wins and the implementation path must change
- all conflicts below the operator/spec author are resolved by the authoritative spec
- no worker judgment, convenience, momentum, local heuristic, or self-granted decision power may supersede a clearly defined spec requirement

**Sources:** project workflow requirements; `docs/61-domain-general-cognition-core.md`; `docs/57-golden-tasks-and-evals.md`; `docs/56-trace-checkpoints-recovery.md`.


### 3.1 Single Cognitive Authority
Focusa is the only canonical authority for:
- active mission
- focus stack
- scope state
- blocker state
- tranche progression
- autonomy posture
- durable memory
- proposal/governance outcomes

Pi MUST NOT create a parallel continuation planner or parallel persistent memory model.  
**Sources:** `docs/44-pi-focusa-integration-spec.md`, `docs/52-pi-extension-contract.md`.

### 3.2 No Decision Authority for the LLM
The LLM/worker has execution capability and direct communication with the operator/spec author, but not governing authority by default.

It may:
- interpret the current task/spec for implementation purposes
- propose actions, changes, or resolutions
- observe mismatches, blockers, and verification outcomes
- execute within the active authority chain
- ask the operator/spec author for clarification, confirmation, or amendment when needed

It may NOT by default:
- overrule the operator/spec author
- overrule the authoritative spec
- overrule the canonical decomposition when it is consistent with spec
- decide that deviation is acceptable when the spec says otherwise
- redefine completion criteria by preference or convenience

The LLM is a subordinate executor and proposal source, not a decision authority. Direct communication upward does not confer authority upward.

### 3.3 Delegated Spec Authorship via Trust Levels
Focusa trust levels may eventually permit delegated spec-independent authorship, but only when all are true:
- delegation is explicitly granted by the operator or current spec author
- the delegated authority scope is explicit
- the trust level/policy permits authorship at that scope
- resulting spec amendments are recorded as authoritative changes

Without explicit delegation, the LLM remains a non-authoring executor/proposal source.

Delegated authorship is not implied by competence, convenience, or long execution duration; it is a governed authority state.  
**Sources:** `docs/41-proposal-resolution-engine.md`; `docs/70-shared-interfaces-statuses-and-lifecycle.md`; `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`.

### 3.4 Reducer Purity
Per `docs/core-reducer.md`:
- reducer is pure
- no IO
- no async
- no process spawning
- no timers
- no RPC orchestration

Therefore the reducer may express:
- whether continuation is allowed
- whether a blocker exists
- whether clarification is required
- whether a tranche is complete

But the reducer MUST NOT:
- launch Pi
- send another prompt
- retry a turn
- wait for agent events  
**Sources:** `docs/core-reducer.md`.

### 3.5 Daemon Runtime Ownership
Per `docs/G1-detail-03-runtime-daemon.md`, the daemon owns:
- authoritative state
- persistence
- API surface
- background worker scheduling
- event emission/logging

Continuous work loop control belongs here.  
**Sources:** `docs/G1-detail-03-runtime-daemon.md`.

### 3.6 Operator Priority
Continuous work MUST yield immediately to:
- explicit operator interruption
- destructive/high-risk confirmation requirements
- clarification-required states
- policy stop conditions

The operator's newest explicit input remains the primary driver of action selection.  
**Sources:** `docs/54a-operator-priority-and-subject-preservation.md`, `docs/52-pi-extension-contract.md`.

### 3.7 Bounded Secondary Cognition
Per `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`, secondary cognition loops are allowed only when bounded by:
- mission
- scope
- verification discipline
- ontology authority
- stop conditions
- auditability

Continuous work without bounds is forbidden.  
**Sources:** `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`, `docs/61-domain-general-cognition-core.md`.

### 3.8 Minimal Applicable Slice
Continuous execution must not keep reinjecting broad stale state. Every next turn must be built from the minimal relevant slice for the current ask, not from a monolithic memory block.  
**Sources:** `docs/54b-context-injection-and-attention-routing.md`, `docs/67-query-scope-and-relevance-control.md`, `docs/68-current-ask-and-scope-integration.md`.

---

### 3.9 Authority Cascade
The authoritative cascade for continuous execution is:

1. operator / spec author, or trusted delegated spec author when explicitly authorized
2. authoritative spec
3. `bd` decomposition
4. code composition
5. app/runtime functionality

The LLM has a direct communication path to the operator/spec author for clarification and amendment requests, but that path is conversational, not authoritative unless explicit delegated authorship is active.

If the operator/spec author or a trusted delegated author amends the spec, that change MUST cascade downward:
- spec updates first
- `bd` decomposition is reconciled to the updated spec
- implementation/code plans are reconciled to the updated decomposition/spec
- application behavior is reconciled to the updated implementation target

The executor MUST NOT preserve stale beads, stale plans, or stale code behavior against an updated authoritative spec.  
**Sources:** operator workflow requirement; `docs/61-domain-general-cognition-core.md`; `docs/56-trace-checkpoints-recovery.md`; `docs/57-golden-tasks-and-evals.md`; `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`.

## 4. Problem Statement

Today, Pi turn execution is turn-bounded:
1. operator or controller sends prompt
2. Pi works
3. Pi replies
4. turn ends
5. no more work occurs until another prompt arrives

Pi RPC explicitly models this as discrete prompts/commands and end-of-turn / end-of-agent events, which is good transport but not by itself a continuity policy. Manual re-prodding is therefore a missing control-loop problem, not merely a prompt-style problem.  
**Sources:** Pi `docs/rpc.md`; user-observed harness behavior; `docs/56-trace-checkpoints-recovery.md` recovery/resume framing.

---

## 5. Scope

This spec covers:
- continuous multi-turn work execution
- automatic advancement across tasks, beads, and tranches
- project-level execution over ordered work graphs such as `bd` task/dependency structures
- single-tranche and multi-tranche execution
- model/harness-agnostic supervision with Pi RPC/SDK as the first transport
- continuation policy
- stop/pause/escalation conditions
- RPC/SDK orchestration shape
- daemon/extension/reducer boundaries
- canonical event requirements
- checkpoint, audit, and telemetry requirements
- operator controls and resume semantics
- eval criteria for proving the loop actually helps

This spec does **not** define:
- model-specific prompting details
- the entire worker ontology
- arbitrary background agents unrelated to an active mission
- unlimited autonomous self-directed work detached from a scoped plan or task graph

**Sources:** `docs/57-golden-tasks-and-evals.md`, `docs/61-domain-general-cognition-core.md`, `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`.

---

## 6. Architecture

### 6.1 Components

#### A. Focusa Reducer
Owns canonical state transitions only.  
**Sources:** `docs/core-reducer.md`.

#### B. Focusa Daemon
Owns:
- loop policy
- runtime supervision
- persistence
- event logging
- continuation eligibility evaluation
- tranche lifecycle state
- recovery/resume coordination

**Sources:** `docs/G1-detail-03-runtime-daemon.md`, `docs/56-trace-checkpoints-recovery.md`.

#### C. Pi RPC/SDK Driver
Owns transport mechanics only:
- spawn/connect to Pi
- send `prompt`
- send `abort` when needed
- receive `agent_start`, `turn_start`, `message_update`, `turn_end`, `agent_end`
- return raw outputs/status to daemon

The driver SHOULD be thin and stupid.  
**Sources:** Pi `docs/rpc.md`.

#### D. Pi Extension
Owns:
- UX glue
- operator commands
- status surface
- lightweight bridge helpers
- observability inside Pi
- scratchpad / Focus-State discipline reinforcement

The extension MUST NOT become a second cognitive runtime.  
**Sources:** Pi `docs/extensions.md`; `docs/44-pi-focusa-integration-spec.md`; `docs/52-pi-extension-contract.md`; `apps/pi-extension/src/tools.ts`.

---

### 6.2 Authority Split

| Concern | Reducer | Daemon | RPC Driver | Pi Extension |
|---|---|---|---|---|
| Canonical cognitive state | Yes | Reads/applies | No | No |
| Continuation policy | State-expressible only | Yes | No | No |
| Spawn/manage Pi session | No | Yes / delegates | Yes | No |
| Re-prompt after reply | No | Yes / delegates | Yes | No |
| UI controls | No | Optional API | No | Yes |
| Session-local helper state | No | Optional | Minimal | Minimal |
| Durable memory authority | Yes | Yes | No | No |
| Operator pause/stop handling | State/result only | Yes | Relay only | Yes |
| Scratchpad discipline | No | Optional policy | No | Yes |

**Sources:** `docs/core-reducer.md`, `docs/G1-detail-03-runtime-daemon.md`, Pi `docs/rpc.md`, Pi `docs/extensions.md`, `apps/pi-extension/src/tools.ts`.

---

## 7. Core Concept: Focusa-Governed Continuous Project Execution

A continuous work loop is a runtime mode in which:
- a supported harness/model completes a turn,
- the daemon evaluates Focusa state plus turn outcome,
- the daemon decides whether to automatically initiate the next turn,
- and, when the current task is complete or deferred, the daemon advances to the next ready task/tranche in the scoped project graph.

The loop continues only while Focusa says the work remains:
- within mission
- within scope
- within autonomy bounds
- within verification policy
- not globally blocked
- not waiting for operator clarification
- not project-complete

This is an instance of bounded secondary cognition: sustained execution is allowed, but only inside an explicit mission/scoping/verification shell. It is also an execution model for decomposition: properly ordered tasks and tranches are meant to be traversed automatically rather than waiting for manual reprompting between units of work.  
**Sources:** `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`, `docs/67-query-scope-and-relevance-control.md`, `docs/68-current-ask-and-scope-integration.md`, `docs/61-domain-general-cognition-core.md`, `docs/57-golden-tasks-and-evals.md`.

---

## 8. Why Extension-Only Is Insufficient

An extension is useful but not sufficient as the primary mechanism because:

1. the problem happens **after** turn completion
2. turn re-entry is an outer-loop orchestration task
3. extension logic runs inside Pi's process/lifecycle
4. the reliable transport for re-prompting is Pi RPC/SDK
5. making the extension own continuation would over-fatten Pi-side control and risk parallel cognition/planning

Extensions do provide commands, status UI, lifecycle hooks, and tool interception, which are valuable for operator controls and observability, but those capabilities do not by themselves replace a daemon-owned continuation loop.  
**Sources:** Pi `docs/extensions.md`; `docs/44-pi-focusa-integration-spec.md`; `docs/52-pi-extension-contract.md`.

Therefore:

> Extension-only work mode is insufficient for reliable unattended continuity.

---

## 9. Why Reducer-Only Is Incorrect

Reducer-only implementation is forbidden because:
- reducer must remain pure
- reducer must remain replayable
- reducer cannot wait on external agent events
- reducer cannot spawn processes or send prompts
- reducer cannot own timers/retries/backoff/session control

The reducer can express predicates like:
- `continuation_allowed = true`
- `operator_input_required = false`
- `tranche_complete = false`

But runtime orchestration belongs to daemon + transport.  
**Sources:** `docs/core-reducer.md`, `docs/G1-detail-03-runtime-daemon.md`.

---

## 10. Hierarchical Execution Model

Continuous execution must operate at multiple levels, not only the next-turn level.

### 10.1 Execution Levels
- `turn_loop`: continue within the current task
- `task_loop`: continue until the current task is complete, deferred, or blocked
- `tranche_loop`: continue across ordered tasks within a tranche
- `project_loop`: continue across all ready tranches/tasks in the scoped project graph until the project is complete or genuinely blocked

### 10.2 No-Reprompt Guarantee
Once autonomous execution is enabled for a properly scoped plan, the system MUST NOT require operator reprompts merely to continue:
- from turn to turn
- from task to task
- from tranche to tranche
- across a whole decomposed project
- across ready `bd` items in dependency order

Operator input is required only for genuine blockers, approvals, governance boundaries, scope changes, or explicit stop/pause.

### 10.3 Effective Infinity
The operator requirement of "infinite turns until completed" is implemented as:
- no artificial stop purely because another turn is needed
- no artificial stop purely because another task or tranche is next
- arbitrarily many turns permitted while valid project work remains
- actual stopping only on completion, real blocker, policy boundary, or operator intervention

This is intentionally **not** unbounded drift; it is bounded-by-project, bounded-by-scope, and bounded-by-policy continuous execution.

**Sources:** `docs/61-domain-general-cognition-core.md`; `docs/57-golden-tasks-and-evals.md`; `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`; `docs/54a-operator-priority-and-subject-preservation.md`.

## 11. Loop Lifecycle

### 11.1 States
Suggested runtime loop states:
- `Idle`
- `SelectingReadyWork`
- `PreparingTurn`
- `AwaitingHarnessTurn`
- `EvaluatingOutcome`
- `AdvancingTask`
- `Paused`
- `Blocked`
- `Completed`
- `Aborted`
- `TransportDegraded`

These states should align with Focusa's broader lifecycle/status normalization rather than inventing one-off semantics.  
**Sources:** `docs/70-shared-interfaces-statuses-and-lifecycle.md`.

### 11.2 High-Level Flow
1. operator enables continuous execution mode for a scoped plan/project
2. daemon validates mission/scope/autonomy prerequisites
3. daemon reads the ordered ready-work graph (for example `bd` tasks/tranches/dependencies)
4. daemon selects the next ready task/tranche
5. daemon starts or attaches to harness RPC/SDK session
6. daemon constructs next-turn prompt from Focusa state plus current task/tranche context
7. daemon sends prompt through transport driver
8. harness executes turn
9. driver streams events back to daemon
10. daemon records observations/events/checkpoints/telemetry
11. on turn completion, daemon evaluates outcome
12. if current task still has valid next work, daemon sends next prompt automatically
13. if current task is complete, deferred, or blocked, daemon advances intelligently to the next ready task/tranche if one exists
14. if no valid work remains, daemon completes, pauses, or escalates

### 11.3 Event-Aware Flow Detail
The driver should treat RPC as a turn/event stream, not just a final blob. In particular:
- `agent_start` and `turn_start` mark supervised execution beginning
- `message_update` can feed progress/UI/telemetry
- `turn_end` captures turn-local completion and tool surface
- `agent_end` is the final continuation decision point for either continuing current work or advancing to the next ready work item

**Sources:** Pi `docs/rpc.md`; `docs/56-trace-checkpoints-recovery.md`.

---

## 11. Required Inputs to Continuation Decisions

The daemon MUST evaluate continuation from canonical Focusa state plus runtime turn result.

Required inputs include:
- active mission
- current ask
- ask kind (`question` / `instruction` / `correction` / `meta` / `unknown`)
- scope kind and carryover policy
- focus selection / excluded context reason
- active blockers/failures
- tranche plan / tranche completion criteria
- current ordered `bd` work item and ready-next candidates
- linked authoritative docs/specs for the selected work item
- current autonomy level
- pending proposals requiring resolution
- verification status for relevant claims/actions
- risk class of next work
- budget caps (turn count, wall clock, token budget, retry budget)
- operator overrides / pause flags
- recent checkpoint state sufficient for recovery

Optional inputs:
- recent telemetry
- turn productivity score
- model/provider health
- tool failure frequency
- stale-session heuristics
- anticipated context for pre-turn enrichment, if retained in policy-safe form

This requirement is already reflected in the extension bridge state types `PiCurrentAsk`, `PiQueryScope`, `PiExcludedContext`, and `PiFocusSelection`, and in the daemon/core emphasis on mission, blockers, verification, and working-set discipline.  
**Sources:** `apps/pi-extension/src/state.ts`; `docs/61-domain-general-cognition-core.md`; `docs/67-query-scope-and-relevance-control.md`; `docs/68-current-ask-and-scope-integration.md`; `docs/56-trace-checkpoints-recovery.md`; `crates/focusa-core/src/types.rs` (`anticipated_context`, `active_turn`).

---

## 12. Work-Graph Traversal and Advancement

Continuous execution must understand ordered work, not just repeated prompting.

### 12.1 BD-First Work Graph Input
The daemon SHOULD accept an ordered/dependency-aware work graph, with `bd`-managed tasks, beads, and tranches treated as the default execution substrate where available.

Minimum work-item fields:
- id
- title
- status
- dependency state
- tranche/parent grouping
- acceptance/completion criteria
- blocker state
- relevant specs/constraints
- linked authoritative docs/specs

The executor should derive ready work from the `bd` graph rather than inventing ad hoc task order.  
**Sources:** project BD-first workflow requirement; `docs/61-domain-general-cognition-core.md`; `docs/56-trace-checkpoints-recovery.md`.

### 12.2 BD Execution Contract
When operating on a `bd`-managed project, the daemon SHOULD:
1. discover ready work from `bd`
2. claim/select the next dependency-satisfied item
3. move it to active/in-progress state when execution begins
4. write progress/checkpoints/blocker notes back to the work item or linked run record
5. mark complete/deferred/blocked only after policy-valid outcome evaluation
6. advance automatically to the next ready item without operator reprompt

`bd` is therefore not passive metadata; it is the traversal substrate for autonomous project execution. Because `bd` is decomposed from the supreme spec, it is normally a reliable operational guide. But `bd` never outranks spec: when in doubt, the executor must resolve uncertainty by returning to the linked authoritative spec, and all bead/spec conflicts are decided by the spec rather than by improvisation from the bead alone.  
**Sources:** project BD-first workflow requirement; `docs/56-trace-checkpoints-recovery.md`; `docs/61-domain-general-cognition-core.md`.

### 12.2a Spec-Linked Task Packet
Each executable work item SHOULD be materialized as a spec-linked task packet rather than a bare title.

Minimum fields:
- work-item id
- linked spec refs
- acceptance criteria
- required verification tier
- allowed scope/files/surfaces
- dependencies
- tranche/project context
- current blocker status
- last checkpoint summary

This prevents the executor from drifting because a bead title was too terse or momentum-biased.  
**Sources:** project Docs-first + BD-first workflow requirement; `docs/56-trace-checkpoints-recovery.md`; `docs/67-query-scope-and-relevance-control.md`.

### 12.3 Advancement Rules
When current work is complete, deferred, or locally blocked, the daemon SHOULD:
1. checkpoint current state
2. mark or propose the appropriate work-item transition
3. select the next dependency-satisfied ready item
4. rebuild the minimal relevant slice for that item
5. continue execution without requiring operator reprompt

A blocked bead must not stall unrelated ready beads. Advancement should follow dependency order, not emotional attachment to the current task.  
**Sources:** `docs/61-domain-general-cognition-core.md`; `docs/57-golden-tasks-and-evals.md`.

### 12.4 Docs-First Completion Semantics
The loop must distinguish:
- `turn_complete`
- `task_complete`
- `tranche_complete`
- `project_complete`
- `task_blocked`
- `project_blocked`

Task completion is not merely “agent thinks done.” A task is complete only when:
- doc/spec-derived acceptance criteria are satisfied
- implementation behavior matches the linked authoritative spec
- required verification has run successfully for that task class
- required work-item updates have been recorded

If the implementation still deviates from the defined spec, the task remains incomplete and requires more work or explicit re-specification.

A blocked task is not the same as a blocked project. The project is blocked only when no valid ready work remains or policy forbids further advancement.

**Sources:** project Docs-first workflow requirement; `docs/57-golden-tasks-and-evals.md`; `docs/61-domain-general-cognition-core.md`; `docs/56-trace-checkpoints-recovery.md`.

## 13. Verification and Close Conditions

### 13.1 Verification Before Close
Before a `bd` work item is closed or advanced past, the daemon SHOULD ensure the appropriate verification tier has succeeded for that task type.

Examples:
- code task: relevant tests/lint/typecheck/spec gate as required by docs/policy
- doc task: spec/doc consistency and required cross-reference updates
- architecture task: required spec alignment and unresolved-conflict check
- integration task: endpoint/flow verification relevant to the claimed behavior

### 13.2 Close Authority
The executor may mark work complete only when docs/specs and verification support that transition. Subjective confidence is insufficient.

### 13.2a Verification Matrix by Task Class
Verification SHOULD be task-class aware rather than one-size-fits-all.

Examples:
- code task
  - relevant tests
  - lint/typecheck/spec gate as required
- refactor task
  - regression checks on touched surfaces
  - unchanged behavior checks where spec requires preservation
- doc/spec task
  - cross-reference integrity
  - behavior/doc consistency
- architecture task
  - spec alignment
  - unresolved-conflict scan
- integration task
  - endpoint/flow validation
  - contract conformance

The linked spec or project policy may strengthen these requirements, but the executor must not weaken them by convenience.  
**Sources:** project Docs-first workflow requirement; `docs/57-golden-tasks-and-evals.md`; `docs/61-domain-general-cognition-core.md`.

### 13.3 Replan Trigger
If docs/specs change such that current or queued work is no longer valid, the executor SHOULD replan the remaining `bd` graph rather than blindly continue stale assumptions.

If the change comes from the operator/spec author, the executor MUST treat it as authoritative and cascade the update through:
- current spec interpretation
- `bd` decomposition and ready-work selection
- implementation plan/code targets
- verification and completion criteria

### 13.4 Anti-Drift Rule
The executor MUST NOT substitute its own preferred implementation plan for a clearly defined linked spec.

When implementation and spec diverge, the executor must do one of:
1. change implementation to match the spec
2. continue because the current state is already spec-conformant
3. pause/escalate because the spec is ambiguous, contradictory, or incomplete

It MUST NOT silently invent a different target and declare success against that invented target. The spec is the final authority on what counts as correct, complete, and still-required work.

**Sources:** project Docs-first workflow requirement; `docs/57-golden-tasks-and-evals.md`; `docs/61-domain-general-cognition-core.md`; `docs/54a-operator-priority-and-subject-preservation.md`.

## 14. Continue Conditions

The daemon MAY auto-continue only if all are true:

1. active mission exists
2. current work remains within scope
3. no operator clarification is required
4. no governance decision is waiting on operator choice
5. no high-risk/destructive action awaits consent
6. no hard blocker exists for the selected work item
7. autonomy policy permits another step
8. verification policy permits another step
9. wall-clock and retry/backoff policy are still valid
10. transport is healthy enough to continue
11. the next-turn slice is still minimally relevant to the current ask/current work item
12. operator's newest steering has not superseded the prior continuation path
13. either the current task has valid next work or another dependency-satisfied ready task exists
14. the project is not already complete

These conditions are the runtime embodiment of operator priority, scope purity, and bounded cognition.  
**Sources:** `docs/54a-operator-priority-and-subject-preservation.md`; `docs/54b-context-injection-and-attention-routing.md`; `docs/67-query-scope-and-relevance-control.md`; `docs/68-current-ask-and-scope-integration.md`; `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`.

---

## 15. Stop Conditions

The loop MUST stop or pause on any of the following:

### 13.1 Hard Stop
- operator stop request
- destructive confirmation required
- explicit mission completion
- explicit tranche completion
- autonomy boundary reached
- verification gate failure requiring operator judgment
- repeated transport/session failure beyond retry budget
- explicit subject-reset / task-reset from operator
- no further ready tasks remain and at least one unresolved operator-required blocker is active

### 13.2 Pause
- clarification required
- pending proposal/governance decision
- temporary upstream/model outage
- recoverable tool environment issue
- cool-down/backoff policy triggered
- resume checkpoint emitted and waiting
- blocked current task with viable later retry, while executor awaits policy/backoff window

### 13.3 Escalate
- repeated subject drift attempts
- repeated invalid self-directed replanning
- contradiction accumulation beyond threshold
- unsafe persistence or replay anomaly
- repeated low-productivity loops suggesting bounded autonomy is no longer helping
- blocker package requires operator judgment because fallback, deferral, and alternate-ready work are exhausted

**Sources:** `docs/41-proposal-resolution-engine.md`; `docs/54a-operator-priority-and-subject-preservation.md`; `docs/56-trace-checkpoints-recovery.md`; `docs/57-golden-tasks-and-evals.md`; `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`.

---

## 16. Output Contract for Worker Turns

Continuous work mode SHOULD standardize turn-final output classes so the daemon can interpret outcomes with low ambiguity.

Preferred explicit markers:
- `BLOCKER:` human/operator input required
- `TRANCHE COMPLETE:` meaningful bounded milestone complete
- `PAUSE:` temporary stop, safe to resume later
- `ESCALATE:` policy/authority conflict
- `DONE:` mission complete

Absence of a stop marker does **not** automatically imply continuation; the daemon must still apply policy.

Markers are an interpretation aid, not the source of truth. The source of truth remains Focusa canonical state plus runtime policy evaluation.  
**Sources:** `docs/56-trace-checkpoints-recovery.md`; `docs/61-domain-general-cognition-core.md`; `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`.

---

## 17. Prompt Construction for Next Turn

The daemon SHOULD construct next-turn prompts from Focusa state, not by naive `continue` repetition.

Each generated continuation prompt SHOULD include:
- active mission reminder
- current tranche focus
- blocker policy reminder
- scope preservation reminder
- relevant verification/risk constraints
- latest checkpoint or next unresolved subproblem
- no-reply instruction unless blocked or milestone-complete

The next prompt MUST NOT:
- invent a new mission
- silently widen scope
- foreground stale daemon metadata over operator subject
- dump a giant always-on state block into every turn

Prompt construction should follow the minimal-applicable-slice and current-ask rules rather than monolithic state injection.  
**Sources:** `docs/54a-operator-priority-and-subject-preservation.md`; `docs/54b-context-injection-and-attention-routing.md`; `docs/67-query-scope-and-relevance-control.md`; `docs/68-current-ask-and-scope-integration.md`; `docs/44-pi-focusa-integration-spec.md`.

### 17.1 Task-Switch Context Reset
When advancing to a different `bd` item, the executor SHOULD:
1. checkpoint the previous task state
2. suppress stale task-local assumptions by default
3. reload the linked spec slice for the next item
4. rebuild the working set from spec + bead + checkpoint
5. verify that carryover context is still relevant before reuse

Task switching must feel intentional, not like one long muddled context stream.  
**Sources:** `docs/67-query-scope-and-relevance-control.md`; `docs/68-current-ask-and-scope-integration.md`; `docs/54b-context-injection-and-attention-routing.md`.

---

## 18. Canonical Events and Reducer Surface

To preserve replayability and auditability, the loop's canonical consequences SHOULD be expressed via reducer events.

Suggested event families:
- `ContinuousWorkModeEnabled`
- `ContinuousWorkModeDisabled`
- `ContinuousTurnRequested`
- `ContinuousTurnStarted`
- `ContinuousTurnObserved`
- `ContinuousTurnCompleted`
- `ContinuousTurnPaused`
- `ContinuousTurnBlocked`
- `ContinuousTurnEscalated`
- `ContinuousTrancheCompleted`
- `ContinuousLoopBudgetExhausted`
- `ContinuousLoopTransportDegraded`
- `ContinuousLoopResumed`
- `ContinuousLoopRecoveryCheckpointed`

Notes:
- these events express canonical facts and audit records
- they do **not** cause side effects by themselves
- the daemon interprets them and drives transport accordingly
- they should compose with existing turn events such as `TurnStarted`, `TurnCompleted`, and `PromptAssembled`, rather than replacing them

**Sources:** `docs/core-reducer.md`; `crates/focusa-core/src/types.rs` (`TurnStarted`, `TurnCompleted`, `PromptAssembled`, `Action::EmitEvent`); `docs/56-trace-checkpoints-recovery.md`; `docs/70-shared-interfaces-statuses-and-lifecycle.md`.

### 18.1 Focusa-Layer Integration Contract
Continuous execution must integrate coherently across every Focusa layer.

- **Reducer layer** owns canonical loop facts only.
- **Daemon layer** owns supervision, orchestration, retry, fallback, and policy application.
- **API layer** exposes control/observability surfaces without becoming a second policy engine.
- **Ontology/governance layer** shapes execution through mission, constraints, provenance, verification, trust, and proposal discipline.
- **Harness/extension layer** surfaces UX/status/bridge behavior without becoming a parallel cognitive authority.

An implementation is incorrect if any layer starts doing another layer's job by convenience.  
**Sources:** `docs/core-reducer.md`; `docs/G1-detail-03-runtime-daemon.md`; `docs/44-pi-focusa-integration-spec.md`; `docs/41-proposal-resolution-engine.md`; `docs/70-shared-interfaces-statuses-and-lifecycle.md`.

### 18.2 Shared Lifecycle and Status Vocabulary
Continuous execution status values SHOULD align with Focusa's shared lifecycle/status discipline rather than inventing ad hoc meanings per subsystem.

At minimum, loop/task/tranche/project status should remain mappable to consistent lifecycle concepts such as:
- active
- blocked
- verified
- stale
- canonical
- speculative
- completed
- suspended

This keeps execution state interoperable with broader ontology/governance surfaces.  
**Sources:** `docs/70-shared-interfaces-statuses-and-lifecycle.md`; `docs/46-ontology-core-primitives.md`.

---

## 19. Runtime State vs Canonical State

Not every loop runtime field belongs in canonical `FocusaState`.

### 17.1 Canonical / Replay-Worthy
- loop enabled/disabled status
- active mission/tranche association
- current blocker/escalation/completion status
- key turn checkpoints
- governance-relevant pauses/stops
- operator overrides
- turn outcome class
- last resume point identity

### 17.2 Runtime-Only / Ephemeral
- Pi process handle
- active RPC connection object
- stdout token stream buffer
- retry timers/backoff timers
- transient session heartbeat timestamps
- local transport reconnection bookkeeping
- UI widget state

Rule:
- reducer owns canonical facts
- daemon owns ephemeral transport mechanics

This mirrors existing Focusa treatment of canonical turn events versus runtime adapter/session plumbing.  
**Sources:** `docs/core-reducer.md`; `docs/G1-detail-03-runtime-daemon.md`; `crates/focusa-core/src/types.rs` (`active_turn` as runtime harness reality); Pi `docs/rpc.md`.

---

## 20. PRE / Governance Integration

Per `docs/41-proposal-resolution-engine.md`, decisions must remain auditable and conflict-safe.

Therefore continuous work loops MUST NOT silently auto-resolve governance decisions that require proposal handling.

If a turn produces:
- focus change proposals
- autonomy changes
- thesis revisions
- constitution-affecting choices
- operator-priority conflicts

then the daemon MUST either:
- route through normal proposal/reducer flow, or
- pause for operator/governance resolution

Loop speed does not override PRE discipline.  
**Sources:** `docs/41-proposal-resolution-engine.md`; `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`.

### 20.0 Proposal-Aware Continuation Rule
If a turn produces proposal-relevant outcomes, continuation policy must evaluate whether execution may proceed in parallel with pending resolution or must pause for governance. The default should favor explicit governance over silent optimistic assumption where spec or policy is unclear.  
**Sources:** `docs/41-proposal-resolution-engine.md`; `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`.

---

## 21. Ontology Discipline

Per `docs/46-ontology-core-primitives.md`, long-lived truth requires provenance, boundedness, and replayability.

Therefore continuous work mode MUST preserve:
- provenance of why a turn was continued
- provenance of why a pause/stop/escalation happened
- bounded mission/object-set selection
- verification records where claims become durable

Continuous work MUST NOT become an unbounded freeform stream detached from ontology objects, constraints, or mission context.  
**Sources:** `docs/46-ontology-core-primitives.md`; `docs/70-shared-interfaces-statuses-and-lifecycle.md`.

### 21.1 Execution as Ontology-Shaped Activity
The executor should treat project execution as ontology-shaped work, not raw prompt churn.

That means each meaningful run should stay anchored to:
- mission/objective context
- active constraints
- relevant decisions
- verification obligations
- provenance-bearing outputs
- explicit status transitions

This keeps continuous execution integrated with Focusa's domain-general cognition model instead of bolting on an isolated loop subsystem.  
**Sources:** `docs/61-domain-general-cognition-core.md`; `docs/46-ontology-core-primitives.md`; `docs/70-shared-interfaces-statuses-and-lifecycle.md`.

---

## 22. Trace, Checkpoints, and Recovery

Continuous work mode must be inspectable and resumable.

### 22.0 Run Identity Model
Every autonomous execution run SHOULD carry stable identities sufficient for audit and resume.

Suggested identities:
- `project_run_id`
- `tranche_run_id`
- `task_run_id`
- `turn_id`
- `worker_session_id`
- `checkpoint_id`

This prevents ambiguous recovery and makes long-running unattended work explainable.  
**Sources:** `docs/56-trace-checkpoints-recovery.md`; `docs/70-shared-interfaces-statuses-and-lifecycle.md`.

### 20.1 Required Trace Dimensions
At minimum, loop traces should capture:
- mission/frame context
- working set used
- constraints consulted
- decisions consulted
- action intents proposed
- tools invoked
- verification results
- blockers/failures emitted
- final state transition
- operator subject / active subject after routing
- steering detected or not
- prior mission reused or suppressed
- focus-slice size/relevance

### 20.2 Checkpoint Triggers
Create checkpoints on:
- loop enable
- session start/restore
- session compact
- high-impact action completion
- verification completion
- blocker/failure emergence
- blocked-task deferral or alternate-task switch
- explicit resume/fork points
- pre-shutdown
- pause/stop/escalation

### 20.3 Resume Semantics
On resume, Focusa should restore:
- active mission/frame
- working set identity
- relevant decisions/constraints
- recent blockers/open loops
- recent verified deltas
- last loop state
- last continuation reason
- last safe re-entry prompt basis

**Sources:** `docs/56-trace-checkpoints-recovery.md`; `docs/44-pi-focusa-integration-spec.md`; Pi `docs/rpc.md`.

---

## 23. Harness Extension Responsibilities

The existing Pi extension remains valuable.

Recommended responsibilities:
- expose operator commands such as `/focus-work on|off|pause|resume|status`
- display loop status in UI
- surface current mission/tranche/focus summary
- expose small bridge tools for loop status/control/context/checkpoints/select-next (`focusa_work_loop_status`, `focusa_work_loop_control`, `focusa_work_loop_context`, `focusa_work_loop_checkpoint`, `focusa_work_loop_select_next`)
- show whether current session is daemon-supervised
- decorate status/system-prompt/header surfaces when useful
- preserve the two-file model: scratchpad for working notes, Focus State for operator-curated cognition

Forbidden responsibilities:
- owning canonical continuation policy
- maintaining parallel long-term planning state
- reimplementing Focusa memory/compaction authority
- becoming a second autonomy engine
- turning scratchpad notes into canonical state by convenience

This section should explicitly inherit the current extension's two-file model and validation posture.  
**Sources:** Pi `docs/extensions.md`; `/opt/cpanel/ea-nodejs20/lib/node_modules/@mariozechner/pi-coding-agent/examples/extensions/system-prompt-header.ts`; `apps/pi-extension/src/tools.ts`; `docs/44-pi-focusa-integration-spec.md`; `docs/52-pi-extension-contract.md`.

---

## 24. RPC / SDK Driver Responsibilities

The driver SHOULD be thin and mechanical.

Responsibilities:
- spawn/attach to Pi in RPC mode, or use `AgentSession`/SDK equivalent where embedding is preferable
- send prompts
- forward abort requests
- stream events back to daemon
- report session exit, crash, timeout, or retry state
- preserve ordered event delivery to daemon supervisors

Non-responsibilities:
- deciding mission
- deciding scope
- deciding whether continuation is semantically valid
- deciding tranche completion from its own heuristics alone
- storing parallel durable cognition

The driver is transport, not cognition.  
**Sources:** Pi `docs/rpc.md` (subprocess RPC mode) and SDK guidance inside that doc; `docs/44-pi-focusa-integration-spec.md`; `docs/52-pi-extension-contract.md`.

### 24.1 Worker Capability Profile and Model Routing
Because continuous execution should work with different models/harnesses, the daemon SHOULD reason over a worker capability profile.

Suggested capability fields:
- tool-use support
- edit reliability
- context-window class
- structured-output reliability
- code-generation strength
- latency class
- cost tier
- fallback availability

Routing policy SHOULD prefer spec-fidelity and completion likelihood over mere speed or novelty. Stronger or different workers may be selected when repeated failures indicate the current worker is no longer fit for the task class.  
**Sources:** project requirement for model flexibility; Pi `docs/rpc.md`; `docs/57-golden-tasks-and-evals.md`; `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`.

---

## 25. Daemon APIs

Suggested daemon-facing APIs.

### 25.0 API/Daemon Boundary Rule
Simple rule: UI and tools can ask the loop to act, but the daemon is the only place that decides whether it can continue.

In plain language:
- controls send requests
- daemon approves or blocks
- daemon always returns a clear reason when blocked

**Sources:** `docs/G1-detail-03-runtime-daemon.md`; `docs/core-reducer.md`; `docs/44-pi-focusa-integration-spec.md`.

### 23.1 Control
- `POST /v1/work-loop/enable`
- `POST /v1/work-loop/pause`
- `POST /v1/work-loop/resume`
- `POST /v1/work-loop/stop`
- `GET /v1/work-loop/status`
- `GET /v1/work-loop/checkpoints`

### 23.2 Internal Supervisor Actions
- `Action::EnableContinuousWork { mission_id, tranche_id?, harness, policy }`
- `Action::PauseContinuousWork { reason }`
- `Action::ResumeContinuousWork`
- `Action::StopContinuousWork { reason }`
- `Action::RequestNextContinuousTurn`
- `Action::ObserveContinuousTurnOutcome { outcome }`
- `Action::CheckpointContinuousLoop { reason }`

### 23.3 Status Payload
Suggested status fields:
- loop state
- active mission id
- active tranche id
- current Pi session id
- turn count
- retry count
- last turn result class
- last blocker reason
- time budget remaining
- autonomy level
- paused/stopped reason
- last checkpoint id
- current ask summary
- scope kind / carryover policy

**Sources:** `docs/G1-detail-03-runtime-daemon.md`; `docs/56-trace-checkpoints-recovery.md`; `apps/pi-extension/src/state.ts`.

---

## 26. Policies

Each enabled loop SHOULD carry an explicit policy bundle.

Suggested policy fields:
- `max_turns`
- `max_wall_clock_ms`
- `max_retries`
- `cooldown_ms`
- `allow_destructive_actions` (default false)
- `require_operator_for_governance` (default true)
- `require_operator_for_scope_change` (default true)
- `require_verification_before_persist` (default true)
- `allow_background_tooling_only` (optional)
- `max_consecutive_low_productivity_turns`
- `max_consecutive_failures`
- `max_irrelevance_score`
- `checkpoint_frequency`
- `resume_requires_operator_ack` (optional)
- `auto_pause_on_operator_message` (default false; steering redirects trajectory and does not imply stop)
- `require_explainable_continue_reason` (default true)
- `max_same-subproblem_retries`
- `max_silent_runtime_ms`
- `status_heartbeat_ms`

Policies SHOULD be explicit, inspectable, event-logged, and safe by default.  
**Sources:** `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`; `docs/57-golden-tasks-and-evals.md`; `docs/67-query-scope-and-relevance-control.md`; `docs/56-trace-checkpoints-recovery.md`.

### 24.1 Preset Modes
To keep configuration intuitive, the first implementation SHOULD ship with named presets rather than forcing operators to tune raw knobs.

Suggested presets:
- `conservative`
  - low turn budget
  - aggressive pause on ambiguity
  - governance/operator review favored
- `balanced`
  - default preset
  - moderate autonomy with checkpointing
- `push`
  - longer uninterrupted execution
  - still bounded by scope/governance/safety laws
- `audit`
  - verbose checkpoints
  - stronger explainability and verification before continuation

Operators may override preset fields, but presets should establish sane starting behavior.  
**Sources:** `docs/61-domain-general-cognition-core.md`; `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`; `docs/57-golden-tasks-and-evals.md`.

### 24.2 Sensible Defaults
The implementation MUST prefer safe, intelligent defaults over mandatory operator micromanagement.

Required default posture:
- destructive actions off
- governance-sensitive continuation paused by default
- operator steering immediately supersedes stale continuation
- checkpoints enabled
- explainability enabled
- bounded retry behavior enabled
- low-productivity loop detection enabled

A user should be able to say the equivalent of "keep working until blocked" without first configuring ten unrelated fields.  
**Sources:** `docs/54a-operator-priority-and-subject-preservation.md`; `docs/56-trace-checkpoints-recovery.md`; `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`.

### 26.3 Budget Hierarchy
"Infinite turns until completed" means continuity, not unmetered waste.

Budgets SHOULD be expressible at multiple levels:
- per turn
- per task
- per tranche
- per project
- per worker/model class

Exhausting a local budget should trigger fallback, pause, escalation, or alternate-work selection rather than blind continuation.  
**Sources:** `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`; `docs/57-golden-tasks-and-evals.md`.

---

## 27. Intelligence Requirements

The loop must be more than persistent; it must be intelligently bounded.

### 25.1 No Blind Auto-Continue
The loop MUST NOT treat the absence of a stop marker as sufficient reason to continue. It must evaluate whether another turn is still likely to be productive, in-scope, operator-aligned, and still directed at the defined spec target rather than a drifted substitute.  
**Sources:** `docs/61-domain-general-cognition-core.md`; `docs/67-query-scope-and-relevance-control.md`; `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`.

### 25.2 Low-Productivity Detection
The daemon SHOULD detect repeated unproductive cycles, including:
- repeated restatement without net progress
- repeated failures on the same subproblem
- repeated tool calls with no state advance
- repeated subject drift suppression with no useful work

Crossing threshold should pause or escalate rather than grind.  
**Sources:** `docs/57-golden-tasks-and-evals.md`; `docs/61-domain-general-cognition-core.md`; `docs/56-trace-checkpoints-recovery.md`.

### 25.3 Intelligent Blocker Handling
Blockers must be treated as a first-class reasoning surface, not a generic stop bucket.

The system SHOULD classify blockers into at least:
- `tooling_blocker`
- `environment_blocker`
- `dependency_blocker`
- `spec_gap`
- `verification_blocker`
- `governance_blocker`
- `permission_blocker`
- `transport_blocker`
- `model_quality_blocker`
- `unknown_blocker`

For each blocker, the loop SHOULD determine:
- whether it is retryable
- whether it is bypassable with an allowed fallback
- whether it requires operator input
- whether work can continue on sibling/next-ready tasks
- whether a checkpoint and tranche/task handoff should be emitted before pause

The system MUST prefer intelligent blocker handling over immediate full-stop behavior. If the blocked task cannot proceed, the executor should attempt one of the following, in order permitted by policy:
1. self-recovery on the same task
2. fallback tool/model/path
3. defer blocked task and advance to another ready task/tranche
4. emit a high-quality blocker package for the operator

A blocker should stop the entire project only when policy, dependency structure, or safety rules make further progress invalid.  
**Sources:** `docs/61-domain-general-cognition-core.md`; `docs/56-trace-checkpoints-recovery.md`; `docs/57-golden-tasks-and-evals.md`; `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`.

### 25.3a Blocker Package Quality
When escalation is required, the blocker package SHOULD include:
- blocker class
- affected work-item id
- linked spec requirement at issue
- recovery attempts made
- fallback attempts made
- whether alternate ready work exists
- exact operator decision needed
- recommended next action

A blocker message should reduce operator thinking load, not increase it.  
**Sources:** `docs/56-trace-checkpoints-recovery.md`; `docs/61-domain-general-cognition-core.md`.

### 25.4 Adaptive Pacing
The loop SHOULD adapt pacing based on context:
- continue quickly when progress is steady and low risk
- checkpoint more often when work becomes risky or long-running
- cool down after repeated transport/tool faults
- prefer pause over thrash under uncertainty

**Sources:** `docs/G1-detail-03-runtime-daemon.md`; `docs/56-trace-checkpoints-recovery.md`; `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`.

### 25.4a Degraded-Mode Integration
When transport, worker quality, tools, or context quality degrade, the executor SHOULD degrade intelligently rather than simply stop or thrash.

Degraded-mode behavior may include:
- narrowing task scope
- switching to stronger verification/checkpoint cadence
- selecting a better-fit worker/model
- deferring risky work while continuing safe ready work
- escalating only after policy-valid recovery paths are exhausted

Degraded mode must remain spec-faithful and auditably bounded.  
**Sources:** `docs/52-pi-extension-contract.md`; `docs/57-golden-tasks-and-evals.md`; `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`.

### 25.5 Explainable Continuation
Every auto-continued turn SHOULD have an intelligible reason, e.g.:
- active tranche still incomplete
- no blocker present
- next unresolved subproblem available
- scope unchanged
- budget remaining
- blocked task deferred and alternate ready work selected

The system should be able to answer: "why did it keep going?"  
**Sources:** `docs/56-trace-checkpoints-recovery.md`; `docs/61-domain-general-cognition-core.md`.

### 25.6 No Busy Loops
The loop MUST NOT hammer Pi or the environment with zero-think re-prompts. If nothing meaningful changed and no next productive action is available, it must pause rather than spin.  
**Sources:** `docs/61-domain-general-cognition-core.md`; `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`.

---

## 28. Intuitive Operator Experience

The system should feel natural, not like operating a scheduler panel.

### 26.0 Simple Operator Model
Default mental model:
- one operator
- one active loop
- steering changes direction, not stop
- daemon auto-prompts next ready work without requiring operator re-prompt
- stop only when truly blocked, paused, or explicitly stopped

If control cannot proceed, status must say why in plain language (for example: "loop controlled by another session").

Worktree state should remain visible in status, but uncommitted changes should not hard-stop continuation by themselves.

### 26.1 Command Shape
The common UX should be simple:
- `keep working until blocked`
- `/focus-work on [root_work_item_id]` (if omitted, daemon/tooling should infer from active task or `bd ready`)
- `/focus-work pause`
- `/focus-work resume`
- `/focus-work stop`
- `/focus-work status`

Complex policy tuning should remain available but optional.  
**Sources:** Pi `docs/extensions.md`; `docs/52-pi-extension-contract.md`.

### 26.2 Status Visibility
At any time, the operator should be able to see:
- whether loop mode is on
- current mission/tranche
- why it is continuing, paused, or stopped
- what it is waiting on
- budget remaining
- last meaningful checkpoint

**Sources:** Pi `docs/extensions.md`; `docs/56-trace-checkpoints-recovery.md`.

### 26.3 No Hidden Work
The loop MUST NOT continue in a surprising or opaque way. Status and stop reasons must remain visible through daemon APIs and, where available, Pi UI surfaces.  
**Sources:** `docs/56-trace-checkpoints-recovery.md`; Pi `docs/extensions.md`.

### 26.4 Graceful Operator Interrupts
Any new operator message should either:
- pause the loop automatically, or
- be interpreted as new steering that supersedes stale continuation

The system must not force the operator to fight an already-running work loop for attention.  
**Sources:** `docs/54a-operator-priority-and-subject-preservation.md`; `docs/68-current-ask-and-scope-integration.md`.

---

## 29. Safety Rules

Continuous work mode MUST NOT:
- continue after destructive confirmation is required
- silently widen mission/scope
- auto-accept governance decisions requiring operator judgment
- treat transport success as cognitive success
- persist unverified durable claims by convenience
- create parallel memory authority in Pi extension or driver
- bypass reducer for canonical state mutations
- prefer daemon metadata over the operator's newest explicit request
- ignore authoritative docs/specs when selecting, executing, or closing work
- let the LLM behave as if it has decision authority over operator/spec/spec-derived decomposition
- treat a bead as outranking the spec it was derived from
- resolve any bead/spec conflict in favor of the bead
- fail to propagate an operator/spec-author amendment down through spec interpretation, `bd`, implementation targets, and runtime behavior
- substitute a self-invented implementation target for a clearly defined spec
- treat `bd` as optional decoration when the project is explicitly running a BD-first workflow
- declare the whole project blocked when only the current task is blocked and alternate ready work exists
- emit low-quality blocker reports that omit recovery attempts, fallback attempts, and the next recommended human action

**Sources:** `docs/core-reducer.md`; `docs/52-pi-extension-contract.md`; `docs/54a-operator-priority-and-subject-preservation.md`; `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`; `docs/56-trace-checkpoints-recovery.md`.

---

### 29.1 Git and Worktree Discipline
Autonomous execution MUST respect source-control and workspace hygiene.

The executor SHOULD:
- inspect `git status` / `git diff` before meaningful work units
- preserve unrelated worktree changes
- avoid touching files outside current scoped work unless spec requires it
- ship small logical checkpoints/commits where policy permits
- treat unrecognized concurrent changes as potentially belonging to another actor

The executor MUST NOT:
- hard reset/clean/restore without explicit approval
- erase unrelated changes for convenience
- use dirty-tree cleanup as a substitute for proper scoping

**Sources:** project workflow requirements; `docs/54a-operator-priority-and-subject-preservation.md`; `docs/56-trace-checkpoints-recovery.md`.

## 30. Recovery and Resumption

The daemon MUST support safe pause/resume semantics.

### 26.1 Resume Requirements
On resume, daemon SHOULD reconstruct:
- active mission/tranche
- last known blocker/pause reason
- last completed turn summary
- continuation eligibility
- current transport health
- last checkpoint identity
- current ask and scope posture

### 26.2 Crash/Restart Behavior
If daemon or driver restarts:
- canonical loop status must be recoverable from events/state
- ephemeral transport/session handles may be discarded and rebuilt
- resumption MUST be explicit or policy-authorized
- no hidden zombie continuation loops

**Sources:** `docs/56-trace-checkpoints-recovery.md`; `docs/G1-detail-03-runtime-daemon.md`; Pi `docs/rpc.md`.

---

## 31. Telemetry, Audit, and Eval

Every loop run SHOULD produce audit-grade records for:
- enable/disable
- every requested turn
- every completed turn
- every pause/stop/escalation
- every retry/backoff
- why continuation happened
- why continuation stopped
- scope reset vs carryover behavior
- blocker rate
- recovery success

Evaluation should include explicit golden tasks such as:
- resume interrupted refactor
- recover after wrong turn
- operator steering under active Focusa state
- correction handling
- continuity across context windows

Suggested loop-specific metrics:
- auto-continue success rate
- manual-prod elimination rate
- subject-drift rate under loop mode
- tranche completion rate
- low-productivity loop detection precision
- blocker auto-recovery rate
- blocker deferral success rate
- project-level completion rate despite partial task blockers
- recovery success after pause/crash
- token/latency cost of loop supervision

**Sources:** `docs/56-trace-checkpoints-recovery.md`; `docs/57-golden-tasks-and-evals.md`; `docs/67-query-scope-and-relevance-control.md`.

---

## 32. Readiness to Decompose

This spec is ready to decompose for implementation only if the kernel can be translated into small work slices without re-arguing authority or architecture.

### 32.1 Decomposition Readiness Criteria
The spec is decomposition-ready when all of the following are true:
- authority chain is unambiguous
- kernel scope is smaller than the full sophistication set
- reducer/daemon/API/transport boundaries are explicit
- Phase-1 success can be tested end-to-end
- completion and blocker semantics are defined tightly enough to code against
- lower-priority complexity is explicitly deferrable

This document now satisfies those conditions.

### 32.2 Phase-1 Decomposition Units
The highest-value kernel can be decomposed into the following implementation slices:

1. **Core types + policy skeleton**
   - loop mode types
   - loop policy defaults/presets
   - run identity types
   - minimal canonical event family

2. **Daemon supervisor kernel**
   - enable/pause/resume/stop state machine
   - next-ready-work selection from `bd`
   - turn/task/tranche/project progression logic
   - blocker defer/alternate-work handling

3. **Transport adapter kernel**
   - Pi RPC/SDK session control
   - prompt send / event receive / abort
   - minimal health/error reporting

4. **API kernel**
   - work-loop control/status endpoints
   - checkpoint/status payloads
   - no second-writer behavior

5. **Verification/close kernel**
   - spec-linked task packet ingestion
   - verification-before-close enforcement
   - close/defer/block transitions

6. **Recovery/observability kernel**
   - checkpoint creation
   - resume from checkpoint
   - visible current status / last blocker / current work item

### 32.3 Decomposition Guardrails
Implementation decomposition MUST preserve these guardrails:
- do not split authority across layers
- do not build rich UI before the kernel works
- do not add speculative routing/trust automation before kernel autonomy works
- do not weaken spec supremacy to simplify implementation
- do not treat deferred sophistication as a prerequisite for proving the kernel

### 32.4 First End-to-End Proof Target
The first proof target should be narrow and concrete:
- one scoped project
- one ordered `bd` graph
- one supported harness transport (Pi RPC/SDK)
- continuous execution from first ready item through completion or genuine blocker
- no manual reprompt required

If that proof target works, the architecture is validated and richer features can be layered afterward.

---

## 33. Recommended Implementation Phases

### Phase 1 — Highest-Value Kernel
- daemon-owned auto-continuation over RPC/SDK transport
- `bd` ready-work traversal
- spec-linked task packets
- verification-before-close
- blocker recovery/defer/alternate-ready-work
- basic pause/resume/status/checkpoint surfaces
- strict safety/spec/governance boundaries

### Phase 2 — Canonical Eventing + Recovery Hardening
- add continuous-work event family
- persist loop checkpoints canonically where warranted
- unify with existing `TurnStarted` / `TurnCompleted` / `PromptAssembled` event stream
- strengthen resume/recovery semantics

### Phase 3 — Practical Quality Improvements
- stronger context-reset behavior on task switches
- fidelity-first worker fallback/routing
- degraded-mode behavior
- better blocker package quality

### Phase 4 — Nice-to-Have Sophistication
- richer UI/status experiences
- broader delegation/trust automation
- more advanced budget/model optimization
- additional ergonomic abstractions that do not dilute spec supremacy

**Sources:** `crates/focusa-core/src/types.rs`; `apps/pi-extension/src/tools.ts`; `apps/pi-extension/src/state.ts`; Pi `docs/rpc.md`; Pi `docs/extensions.md`; `docs/56-trace-checkpoints-recovery.md`; `docs/57-golden-tasks-and-evals.md`.

---

## 34. Recommended Immediate File/Module Shape

Suggested initial implementation shape:

- `docs/79-focusa-governed-continuous-work-loop.md`
- `crates/focusa-core/src/types.rs`
  - loop events and loop policy types
- `crates/focusa-core/src/reducer.rs`
  - canonical loop state transitions only
- `crates/focusa-core/src/runtime/daemon.rs`
  - loop supervisor
- `crates/focusa-api/src/routes/work_loop.rs`
  - enable/pause/resume/status endpoints
- transport adapter
  - either embedded daemon-side SDK session use, or a thin helper wrapping Pi RPC subprocess transport

Keep the first cut minimal:
- do not require rich extension UI to prove the kernel
- do not require advanced routing heuristics to prove the kernel
- do not require trust/delegation automation to prove the kernel
- prove end-to-end autonomous completion first, then layer sophistication on top

**Sources:** Pi `docs/rpc.md`; Pi `docs/extensions.md`; `docs/G1-detail-03-runtime-daemon.md`; `docs/core-reducer.md`.

---

## 35. Reference Decision

The core architectural decision is:

> Focusa continuous work must be implemented as a daemon-governed, policy-bounded RPC/SDK execution loop, with reducer-expressed canonical loop facts and a thin Pi extension, rather than as extension-only automation or reducer-side orchestration.

**Sources:** `docs/core-reducer.md`; `docs/G1-detail-03-runtime-daemon.md`; `docs/44-pi-focusa-integration-spec.md`; Pi `docs/rpc.md`; Pi `docs/extensions.md`.

---

## 36. Open Questions

1. Should Pi transport live inside daemon process space via SDK or as a helper subprocess wrapping RPC?
2. Which loop facts must be canonical vs runtime-only in tranche 1?
3. Should tranche completion be explicit-only or permit policy-guided inference?
4. How much next-turn prompt construction should be templated vs generated from policy + state?
5. Should governance-sensitive outputs always force pause, or can some proposal classes continue optimistically while awaiting resolution?
6. Should `active_turn` remain purely runtime harness reality, or should loop mode add a separate runtime supervisor structure to avoid overloading adapter turn state?
7. Should anticipated-context prediction participate in next-turn planning, and if so, under what scope/verification safeguards?

---

## 37. Acceptance Criteria

This spec is satisfied only when:
- the operator can enable continuous work once and stop babysitting
- turns auto-continue after replies without manual prodding
- tasks and tranches auto-advance without manual reprompt when valid ordered work remains
- a properly decomposed project can run end-to-end until completion or a genuine blocker
- the operator/spec author, or an explicitly trusted delegated spec author, is the only authority above the current spec
- docs/specs remain the supreme authority for correctness, interpretation, and closure below the active authoring authority
- the LLM has direct communication with the operator/spec author but no decision authority unless explicit delegated authorship is active
- ordered `bd` items function as the execution graph in BD-first workflow mode
- `bd` items are treated as spec-derived operational guides, and authoritative specs decide all doubt or conflict
- operator/spec-author amendments cascade correctly through spec, `bd`, code, and resulting functionality
- Focusa remains single cognitive authority
- reducer purity is preserved
- canonical state mutations stay reducer-expressed
- the loop halts correctly on blocker/pause/complete conditions
- continuation decisions are auditable and explainable
- scope/missions do not silently drift
- implementation does not drift from clearly defined specs into self-invented targets
- operator subject changes correctly suppress stale carryover
- recovery after pause/crash is checkpoint-backed and intelligible
- the default experience is usable without heavy manual tuning
- the loop pauses instead of spinning when it is no longer productive
- the operator can always tell what the system is doing and why
- task-level blockers are intelligently recovered, deferred, or routed around when valid alternate work exists
- operator escalation includes a high-quality blocker package instead of a bare stop
- no artificial reprompt-to-continue behavior is required for mere progression through valid project work
- work items are not marked complete without doc-derived acceptance and required verification
- task switches reload the right spec context instead of leaking stale task-local assumptions
- model/worker selection remains fidelity-first rather than novelty-first
- autonomous execution preserves unrelated worktree state and avoids destructive git behavior without approval

**Sources:** `docs/54a-operator-priority-and-subject-preservation.md`; `docs/56-trace-checkpoints-recovery.md`; `docs/57-golden-tasks-and-evals.md`; `docs/67-query-scope-and-relevance-control.md`; `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`.

---

## 38. One-Sentence Summary

Focusa continuous work mode should be a daemon-owned, policy-bounded multi-turn execution loop over Pi RPC/SDK transport, with canonical continuation facts expressed through reducer events, scope/current-ask/checkpoint discipline enforced by Focusa policy, and the Pi extension kept thin as UX glue rather than control authority.
