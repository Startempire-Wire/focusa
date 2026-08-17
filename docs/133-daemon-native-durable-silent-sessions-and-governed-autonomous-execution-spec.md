# Spec 133 — Daemon-Native Durable Silent Sessions and Governed Autonomous Execution

**Status:** Draft / proposed implementation-ready specification
**Owner:** Focusa / Verious Smith
**Created:** 2026-07-12
**Scope:** Focusa core, reducer, daemon, persistence, API, CLI, Pi extension, harness adapters, local process runner, menubar, local dashboard, Workpoint, Trajectory, Evidence, Context Cognition, Context Authority, Work Loop, Agent Bootstrap, Session Transfer, Receipts, resource governance, worktree isolation, retention, protocol evolution, and runtime testing
**Supersedes:** The current Pi-local tmux implementation of `focusa_silent_sessions` as the canonical Silent Session control plane
**Preserves:** The existing `focusa_silent_sessions` tool name as a compatibility facade over daemon APIs
**Proposed path:** `docs/133-daemon-native-durable-silent-sessions-and-governed-autonomous-execution-spec.md`

---

## 0. One-line definition

A Focusa Silent Session is a durable, observable, model-pinned, scope-bound, daemon-supervised autonomous agent execution that remains governed by ProjectIdentity, Trajectory, Workpoint, Context Authority, Evidence, Receipts, resource policy, and operator control.

---

## 1. Normative basis

This specification extends and preserves:

* `docs/G1-detail-03-runtime-daemon.md`
* `docs/core-reducer.md`
* `docs/44-pi-focusa-integration-spec.md`
* `docs/66-affordance-and-execution-environment-ontology.md`
* `docs/70-shared-interfaces-statuses-and-lifecycle.md`
* `docs/72-agent-identity-role-and-self-model-ontology.md`
* `docs/76-retention-forgetting-and-decay-policy.md`
* `docs/77-ontology-governance-versioning-and-migration.md`
* `docs/78-bounded-secondary-cognition-and-persistent-autonomy.md`
* `docs/79-focusa-governed-continuous-work-loop.md`
* `docs/83-pi-focusa-rpc-efficiency-spec.md`
* `docs/88-ontology-backed-workpoint-continuity.md`
* `docs/96-trajectory-projection-and-daemon-stability-spec.md`
* Specs 98 and 99 project-root and authority corrections
* `docs/100-context-cognition-spec.md`
* Spec 101 Bloatgaurd and resource-context posture
* Spec 104 route-scope authorization
* Spec 106 authority-model tightening
* Spec 107 spec-first lifecycle and claim discipline
* `docs/111-agent-context-bootstrap-and-delivery-spec.md`
* Spec 116 provider-neutral work-item closure authority
* `docs/119-verifiable-agent-work-receipts-and-governed-execution-ledger-spec.md`
* `docs/120-adversarial-spec-workbench-and-operator-approval-gates.md`
* `docs/current/AUTHORITY_MODEL.md`
* `docs/current/CONTEXT_AUTHORITY_CURRENT.md`
* `docs/current/TAMPER_EVIDENT_EVENT_CHAIN.md`

Any mismatch between this specification and the current Pi-local tmux implementation is an implementation gap, not a reason to weaken this specification.

---

## 2. Current-state finding

The existing feature is a useful prototype but is not a complete Silent Session subsystem.

It currently:

* lives inside the Pi extension;
* directly invokes tmux;
* stores metadata, registry state, and logs beneath `/tmp`;
* uses terminal activity and pane state as rough health signals;
* captures output through tmux pane snapshots and `pipe-pane`;
* launches a shell-composed Pi command;
* does not require an explicit provider, model, thinking level, or fallback policy;
* does not expose daemon-owned API, CLI, dashboard, stream, receipt, or recovery contracts;
* does not isolate foreground and background writers;
* does not produce sufficient runtime proof that an exited agent completed its Workpoint.

The replacement must not merely add more tmux commands. It must correct the ownership, persistence, execution, and authority model.

---

## 3. Core directive

```text
Every Silent Session must be:

daemon-owned,
durably identifiable,
explicitly configured,
model-pinned,
scope-verified,
continuously observable,
process-supervised,
resource-bounded,
writer-isolated,
checkpoint-backed,
evidence-producing,
receipt-producing,
recoverable after controller loss,
and controllable without a foreground Pi session.
```

A Silent Session is not a terminal pane.

A terminal pane, Pi process, Herdr agent, tmux session, Windows ConPTY process, or harness RPC session is a replaceable runtime attachment beneath the canonical Focusa execution object.

---

## 4. Product promise

Before a Silent Session starts, Focusa can prove:

1. who authorized it;
2. which project and continuity scope it belongs to;
3. which mission, Trajectory, Workpoint, and work item it is executing;
4. which workspace or isolated worktree it may modify;
5. which harness, provider, model, thinking level, and fallback policy will be used;
6. whether provider authentication and model entitlement passed;
7. which resource, cost, output, time, and concurrency budgets apply;
8. which Context Authority verdict allows launch and later risky mutations;
9. where its live event stream can be watched;
10. how it will checkpoint, pause, recover, complete, and prove its result.

While it runs, Focusa can show:

* what the agent is doing;
* what tools it is using;
* what output it is producing;
* whether it is working, waiting, blocked, paused, degraded, completing, or orphaned;
* which Workpoint and work item remain active;
* what changed since the last checkpoint;
* what resources and tokens have been consumed;
* what operator action is required;
* why execution continued, paused, failed, or changed direction.

After it exits, Focusa can determine whether the work was actually completed rather than equating process exit with task completion.

---

## 5. Non-goals

Spec 133 is not:

* a replacement for Workpoint;
* a replacement for Trajectory;
* a replacement for Work Loop;
* a replacement for Context Cognition;
* a replacement for Context Authority;
* a second Focus State;
* a second receipt ledger;
* a Pi-only automation system;
* a tmux or Herdr product wrapper;
* a guarantee that every harness supports every control capability;
* a cloud-first remote execution service;
* unlimited autonomous execution;
* permission for foreground and background agents to edit the same dirty worktree;
* permission to silently change models;
* permission to infer completion from exit code alone;
* permission to store secrets in output or effective-configuration records;
* permission for a terminal transcript to become canonical project truth.

---

## 6. Hard design laws

### 6.1 Daemon ownership

The Focusa daemon owns:

* canonical Silent Session identity;
* configuration revisions;
* lifecycle state;
* authorization records;
* supervision policy;
* event sequencing;
* persistence;
* checkpoint coordination;
* resource admission;
* lease ownership;
* notification state;
* completion evaluation;
* receipt coordination;
* recovery and reconciliation.

The Pi extension must not own canonical Silent Session state.

### 6.2 Reducer purity

The reducer may express canonical facts such as:

* session authorized;
* session admitted;
* session started;
* session paused;
* session blocked;
* session completed;
* operator steering received;
* model binding verified;
* Workpoint checkpoint linked;
* completion receipt committed.

The reducer must not:

* spawn processes;
* open PTYs;
* send signals;
* read stdout;
* write stream chunks;
* wait on timers;
* call providers;
* create worktrees;
* perform retries.

Those side effects belong to daemon supervision and local runner components.

### 6.3 Harness neutrality

Pi is the first structured harness adapter, not the architecture.

The subsystem must support a common adapter contract for:

* Pi RPC;
* Codex;
* Claude Code;
* OpenCode or compatible agents;
* generic JSONL/RPC harnesses;
* generic PTY-only harnesses.

### 6.4 Explicit execution configuration

Autonomous sessions must not inherit a model or provider implicitly from:

* the foreground Pi process;
* a previous session;
* an ambient OpenRouter default;
* an unrelated shell configuration;
* the last model selected in a different continuity scope.

The effective provider and model must be visible and verified before mutation begins.

### 6.5 No invisible background work

A detached start must return:

* durable session ID;
* run ID;
* effective model;
* effective workspace;
* current state;
* event cursor;
* watch command;
* dashboard route;
* notification posture.

A session that cannot expose a durable observation stream must not be reported as fully started.

### 6.6 Process exit is not completion

Exit code `0`, a final assistant message, an idle terminal, or a closed pane does not prove Workpoint completion.

Process exit transitions the session into `completing`.

Only evidence, verification, and receipt rules may transition it to `completed`.

### 6.7 Single-writer and workspace isolation

Two writing agents must not concurrently edit one dirty worktree unless an explicit shared-writer policy exists and the operator has approved it.

Background mutation should default to an isolated worktree.

### 6.8 Focusa scope remains authoritative

```text
project_root + continuity_id = authority boundary
session_id = runtime metadata
terminal pane id = runtime metadata
process id = runtime metadata
transcript/output = observation
```

### 6.9 Evidence and receipts are mandatory for completion

Meaningful Silent Sessions must produce a Spec119-compatible `work_session` receipt with `execution_mode=silent_session`.

Risky operations additionally require `risky_mutation` receipts.

Blocked completion claims require `blocked_claim` receipts or equivalent receipt projections.

### 6.10 Operator steering wins

New operator steering immediately supersedes stale autonomous direction.

The session may redirect, pause, or stop according to policy, but must not continue an obsolete mission merely because a process is already running.

### 6.11 No silent model fallback

Fallback is disabled unless an explicit policy enables an ordered allowlist.

Every fallback attempt and effective model change must be visible, evented, and policy-authorized.

### 6.12 Durability before convenience

Canonical session identity, lifecycle, configuration, stream indexes, checkpoints, and receipts must not depend on `/tmp`, tmux server state, plugin memory, or transcript history.

### 6.13 Resource-bounded autonomy

“Continue until complete” means no artificial reprompt requirement. It does not mean unlimited compute, output, tokens, storage, time, or money.

### 6.14 Output is externalized evidence, not prompt memory

Full output belongs in durable stream storage and the Reference Store.

Only bounded, purpose-selected projections enter agent or operator context.

### 6.15 Versioned evolution

Config, event, runner, adapter, and API protocols must declare versions and compatibility. Compatibility must never be assumed.

---

## 7. Target architecture

```text
Operator / CLI / Dashboard / Menubar / Pi Tool / Remote Authorized Client
                              │
                              ▼
                       Focusa Daemon
              ┌────────────────────────────────┐
              │ Silent Session Control Plane   │
              │                                │
              │ • identity and scope           │
              │ • authorization and approvals  │
              │ • config revisions             │
              │ • lifecycle reducer events     │
              │ • Workpoint/Trajectory links   │
              │ • resource admission           │
              │ • leases and scheduling        │
              │ • stream indexes and cursors   │
              │ • checkpoints and receipts     │
              │ • notifications and recovery   │
              └───────────────┬────────────────┘
                              │ protected local protocol
                              ▼
                  Focusa Session Runner
              ┌────────────────────────────────┐
              │ OS process supervision         │
              │ process groups / Job Objects   │
              │ stdout/stderr capture           │
              │ resource enforcement            │
              │ signal and input delivery       │
              │ runner heartbeat and adoption   │
              └───────────────┬────────────────┘
                              │
                 ┌────────────┴────────────┐
                 ▼                         ▼
           Harness Adapter           Process Backend
       Pi RPC / Codex / Claude       direct / PTY / ConPTY
       generic JSONL / PTY           Herdr / tmux compatibility
```

### 7.1 Why a session runner exists

The daemon may run under a service account or elevated system context, while project files belong to a specific OS user.

The runner provides:

* execution under the project owner;
* OS-user isolation;
* protected local communication;
* process-tree ownership;
* portable process supervision;
* survival across client and plugin restarts;
* optional survival across daemon restarts;
* reduced need for shell-based `as-user` command composition.

The runner may be embedded when the daemon and project owner are the same user. Cross-user execution must use a per-user runner or another explicitly supported isolation mechanism.

### 7.2 Layer authority table

| Concern                     |          Reducer |          Daemon |                    Runner |           Harness adapter |          Terminal backend | Pi extension |
| --------------------------- | ---------------: | --------------: | ------------------------: | ------------------------: | ------------------------: | -----------: |
| Canonical session facts     |              Yes |   Applies/reads |                        No |                        No |                        No |           No |
| Scope and Workpoint binding |              Yes |             Yes |                        No |              Reads packet |                        No |     Displays |
| Configuration revisions     | Canonical events |             Yes | Receives effective config | Interprets harness fields | Interprets process fields |  Client only |
| Process lifecycle           |       Facts only |      Supervises |           Owns OS process |             Owns protocol |       Owns PTY attachment |           No |
| Output persistence          |  Event refs only |         Indexes |                  Captures |    Parses semantic events |        Provides raw bytes |     Displays |
| Model verification          |       Fact/event |        Enforces |                  Launches |     Resolves and observes |                        No |     Displays |
| Resource admission          |       Fact/event | Enforces policy |        Enforces OS limits |             Reports usage |                        No |     Displays |
| Operator control            |       Fact/event |      Authorizes |                  Executes |                    Relays |                    Relays |  Client only |
| Completion authority        |       Fact/event |       Evaluates |              Reports exit |            Reports result |              Reports exit |           No |

---

## 8. Core domain objects

### 8.1 `SilentSession`

The durable logical execution object.

```yaml
SilentSession:
  schema: focusa.silent_session.v1
  session_id: uuidv7
  display_name:
  created_at:
  created_by_actor_ref:
  project_root:
  project_identity_ref:
  continuity_id:
  trajectory_ref:
  workpoint_ref:
  work_item_ref:
  mission:
  lifecycle_state:
  active_run_id:
  config_revision_id:
  writer_lease_ref:
  retention_policy_ref:
  receipt_refs: []
```

A `SilentSession` may contain multiple runs after restart, model change, transport recovery, or explicit relaunch.

### 8.2 `SilentSessionRun`

One supervised execution generation.

```yaml
SilentSessionRun:
  run_id: uuidv7
  session_id:
  generation: 1
  runner_id:
  adapter_id:
  process_backend_id:
  requested_model_binding:
  effective_model_binding:
  observed_model_binding:
  workspace_binding:
  process_identity:
  started_at:
  ended_at:
  exit_status:
  current_event_seq:
  output_stream_refs: []
  runtime_checkpoint_refs: []
  workpoint_checkpoint_refs: []
```

Delayed controls must include both `session_id` and `run_id`. A command targeting an old generation must not affect a newer generation.

### 8.3 `SilentSessionConfig`

A typed, versioned, reproducible configuration object.

### 8.4 `SilentSessionConfigRevision`

An immutable configuration revision with:

* parent revision;
* requested changes;
* effective diff;
* field provenance;
* policy-lock results;
* operator approval reference;
* validation result;
* applied timestamp;
* rollback target.

### 8.5 `SilentSessionEvent`

A sequenced, durable observation or canonical fact reference.

### 8.6 `SilentSessionCheckpoint`

Two checkpoint classes are required:

1. **Runtime checkpoint**

   * process and protocol position;
   * stream cursor;
   * current harness session reference;
   * resource counters;
   * retry state;
   * safe for frequent persistence.

2. **Canonical Workpoint checkpoint**

   * mission;
   * action intent;
   * active objects;
   * blockers;
   * verified evidence;
   * next slice;
   * do-not-drift boundaries.

Runtime checkpoints must not impersonate Workpoint authority.

### 8.7 `SilentSessionLease`

A durable lease over:

* project and continuity scope;
* work item;
* workspace or worktree;
* optional path-intent set;
* writer role;
* owner actor instance;
* expiration and heartbeat;
* adoption policy.

### 8.8 `SilentSessionCompletionEvaluation`

A structured completion decision containing:

* process result;
* Workpoint status;
* work-item acceptance criteria;
* evidence classes;
* test results;
* diff and commit refs;
* unresolved blockers;
* adversarial verifier verdict when required;
* receipt readiness;
* completion decision.

---

## 9. Identity and scope

### 9.1 Required identities

Every session must carry:

* `session_id`: stable UUIDv7 for the logical Silent Session;
* `run_id`: UUIDv7 for one process generation;
* `project_root`;
* verified `project_identity_ref`;
* `continuity_id`;
* `workpoint_id` and revision when available;
* `work_item_ref` when work is task-bound;
* `actor_instance_ref`;
* authenticated operator or service principal;
* OS execution user;
* workspace identity;
* harness-native session reference when available.

### 9.2 Identity laws

* Display names are not identifiers.
* tmux names, Herdr pane IDs, PIDs, terminal IDs, and Pi session paths are backend references.
* A restarted run keeps the same `session_id` and receives a new `run_id`.
* Cross-project adoption is forbidden.
* A process found after daemon restart may be adopted only when its signed runner record matches the expected session, run, user, executable, project, and workspace.
* A session cannot change `project_root` or `continuity_id` through a config edit. Such a change requires a new Silent Session.

---

## 10. Lifecycle and semantic state

One vague `running/stale/dead` status is insufficient.

### 10.1 Operational state machine

```text
draft
  → validating
  → queued
  → launching
  → initializing
  → running

running
  → waiting_input
  → blocked
  → pausing
  → completing
  → recovering
  → orphaned
  → cancelling

waiting_input
  → running
  → paused
  → cancelling

blocked
  → running
  → paused
  → completing
  → cancelling

pausing
  → paused

paused
  → resuming
  → cancelling

resuming
  → running
  → blocked

recovering
  → running
  → waiting_input
  → blocked
  → orphaned
  → failed

orphaned
  → recovering
  → cancelled
  → failed

cancelling
  → cancelled

completing
  → completed
  → blocked
  → failed
```

### 10.2 Required states

* `draft`
* `validating`
* `queued`
* `launching`
* `initializing`
* `running`
* `waiting_input`
* `blocked`
* `pausing`
* `paused`
* `resuming`
* `recovering`
* `orphaned`
* `completing`
* `completed`
* `failed`
* `cancelling`
* `cancelled`

### 10.3 Orthogonal health axis

Operational state must be supplemented by health:

```text
healthy
degraded
stale
unresponsive
process_exited
transport_lost
runner_lost
unknown
```

`stale` and `dead` must not substitute for lifecycle meaning.

### 10.4 Semantic activity axis

```text
working
tool_running
thinking
waiting_for_operator
waiting_for_provider
waiting_for_dependency
idle_between_turns
verifying
checkpointing
integrating
unknown
```

### 10.5 Truthful-state rules

* `waiting_input` requires an explicit harness prompt/input event or a high-confidence adapter observation.
* Terminal silence alone cannot prove `waiting_input`.
* `blocked` requires a typed blocker or a policy-classified failure.
* `completed` requires completion evaluation and receipt readiness.
* `process_exited` is a health fact, not a completion state.
* Heuristic observations must include:

  * source;
  * confidence;
  * freshness;
  * whether they are model-inferred, terminal-inferred, runtime-observed, or verification-confirmed.

---

## 11. Persistence and storage

### 11.1 Canonical persistence

Use the existing Focusa SQLite persistence and event-chain architecture.

Minimum tables or equivalent projections:

```text
silent_sessions
silent_session_runs
silent_session_config_revisions
silent_session_events
silent_session_stream_indexes
silent_session_checkpoints
silent_session_leases
silent_session_notifications
silent_session_completion_evaluations
silent_session_backend_bindings
```

Canonical session and lifecycle mutations must flow through daemon/reducer events.

### 11.2 Stream storage

Raw and structured stream data should not inflate canonical event rows.

Recommended storage:

```text
~/.focusa/silent-sessions/<session-id>/<run-id>/
  manifest.json
  streams/
    stdout-000001.jsonl.zst
    stderr-000001.jsonl.zst
    semantic-000001.jsonl.zst
  artifacts/
  recovery/
```

The exact root follows Focusa’s configured data directory.

### 11.3 Storage requirements

* directory permissions default to `0700`;
* file permissions default to `0600`;
* no predictable shared `/tmp` metadata paths;
* no symlink following;
* use `O_NOFOLLOW` or platform-equivalent protections;
* write temporary files in the same secured directory;
* flush and atomically rename;
* checksum each closed chunk;
* index chunks transactionally;
* recover or quarantine partial chunks after crash;
* persist session UUID, run UUID, config hash, cursor range, byte count, and content hash;
* never persist raw provider credentials;
* redact configured secrets before durable output storage;
* permit explicitly enabled sealed local forensic artifacts only under a separate high-security policy.

### 11.4 Corruption recovery

On startup:

1. verify database migrations;
2. verify stream indexes;
3. scan only registered session directories;
4. validate manifests and chunk hashes;
5. quarantine malformed or unindexed chunks;
6. rebuild indexes where safe;
7. emit recovery events;
8. never silently discard unknown data;
9. mark affected sessions `degraded` until reconciliation completes.

### 11.5 Legacy import

A one-time migration may inspect the existing `/tmp/focusa-silent-registry.json` and registered log paths.

Legacy import must:

* treat all records as untrusted;
* validate ownership and path safety;
* assign stable UUIDs;
* copy recoverable logs into secured session storage;
* preserve original names as aliases;
* mark imported metadata `legacy_unverified`;
* never execute a stored legacy shell command automatically.

---

## 12. Canonical event and output protocol

### 12.1 Event envelope

```json
{
  "schema": "focusa.silent_session_event.v1",
  "event_id": "uuidv7",
  "session_id": "uuidv7",
  "run_id": "uuidv7",
  "seq": 1842,
  "occurred_at": "2026-07-12T20:00:00Z",
  "observed_at": "2026-07-12T20:00:00Z",
  "kind": "tool.started",
  "source": "pi_rpc",
  "provenance": "runtime_observed",
  "canonical": false,
  "payload": {},
  "artifact_refs": [],
  "correlation_id": "uuidv7",
  "redaction": {
    "applied": true,
    "classes": []
  }
}
```

### 12.2 Sequencing and cursors

* Every run has a monotonically increasing `seq`.
* Cursor shape is opaque to clients but must encode run and sequence safely.
* Reconnect must support resume from the last acknowledged cursor.
* Event delivery may be at-least-once.
* Clients deduplicate using `event_id`.
* A cursor must remain usable after process exit and plugin restart.
* Slow subscribers must not block process output capture.
* Slow clients may be disconnected with a resumable cursor.

### 12.3 Required event families

#### Session lifecycle

```text
session.created
session.validation_started
session.validation_failed
session.admitted
session.queued
session.launching
session.initializing
session.started
session.pausing
session.paused
session.resuming
session.recovering
session.orphaned
session.completing
session.completed
session.failed
session.cancelling
session.cancelled
```

#### Configuration and model

```text
config.resolved
config.revision_proposed
config.revision_applied
config.revision_rolled_back
model.preflight_started
model.preflight_passed
model.preflight_failed
model.requested
model.effective
model.observed
model.mismatch
model.fallback_proposed
model.fallback_applied
```

#### Harness and agent

```text
harness.connected
harness.disconnected
agent.started
agent.working
agent.waiting_input
agent.blocked
agent.idle
agent.turn_started
agent.turn_ended
agent.settled
agent.error
```

#### Output and tools

```text
stream.stdout
stream.stderr
assistant.text_delta
assistant.thinking_delta
tool.started
tool.output
tool.completed
tool.failed
prompt.detected
input.sent
key.sent
interrupt.sent
```

#### Focusa governance

```text
project_identity.verified
trajectory.bound
workpoint.bound
workpoint.checkpoint_requested
workpoint.checkpoint_linked
context_cognition.packet_bound
context_authority.preflight
evidence.captured
receipt.previewed
receipt.committed
writer_lease.acquired
writer_lease.renewed
writer_lease.released
writer_lease.conflict
```

#### Resources and supervision

```text
resource.admitted
resource.sample
resource.pressure
resource.limit_approaching
resource.limit_exceeded
retry.scheduled
retry.exhausted
backpressure.applied
process.spawned
process.exited
process.signal_sent
process_group.terminated
child_leak.detected
```

### 12.4 Output channels

The subsystem must distinguish:

* stdout;
* stderr;
* structured harness events;
* assistant text;
* thinking text when available and policy permits;
* tool calls;
* tool output;
* Focusa control events;
* operator input;
* system diagnostics.

### 12.5 Continuous rotation

Rotation must occur while the session is running.

Rotation triggers may include:

* maximum uncompressed bytes;
* maximum compressed bytes;
* maximum event count;
* maximum chunk age;
* explicit checkpoint;
* session completion.

A long-running process must not wait for restart before rotation occurs.

### 12.6 Durable completion artifacts

At completion or terminal failure, generate immutable artifacts for:

* final bounded transcript projection;
* full redacted stream manifest;
* stdout/stderr chunk index;
* effective configuration;
* requested/effective/observed model binding;
* Workpoint checkpoint history;
* git diff/status summary;
* test and verification results;
* blocker summary;
* completion evaluation;
* receipt references.

Large artifacts are stored through the Reference Store and addressed by stable handles.

---

## 13. Process supervision

### 13.1 Ownership

Each run must have one canonical runner and one owned process tree.

On POSIX systems, the runner must use:

* a dedicated process group or session;
* process-group signaling;
* child-process tracking;
* optional cgroup v2 scope where available;
* safe UID/GID execution;
* explicit working directory;
* explicit environment map.

On Windows, the declared backend must use:

* Job Objects for process-tree ownership;
* ConPTY when interactive terminal semantics are required;
* platform-native control behavior;
* declared capability limitations.

### 13.2 Launch manifest

Processes must launch from a typed manifest, not a shell-composed command string.

```yaml
LaunchManifest:
  executable:
  argv: []
  cwd:
  env:
    SAFE_KEY: value
  secret_env_refs: []
  stdin_mode:
  stdout_mode:
  stderr_mode:
  process_backend:
  os_user:
  resource_limits:
  trust_policy:
  adapter_config:
```

The effective launch manifest is hashed and stored in redacted form.

### 13.3 Launcher defect prevention

The implementation must specifically prevent the confirmed defects:

#### Mission truncation

* Do not interpolate a mission into nested shell quoting.
* Deliver large mission/bootstrap data through:

  * RPC request;
  * stdin;
  * secured prompt file;
  * typed argument when safely supported.
* Persist the exact mission artifact hash.

#### Trust prompt

* Harness adapters must declare required noninteractive and trust-preflight capabilities.
* The Pi adapter must apply the required approved noninteractive/trust flag used by the deployment, including `-a` where that remains Pi’s required interface.
* Trust must be granted only after project, workspace, operator, and Context Authority validation.
* An unexpected trust prompt transitions to `waiting_input` or `blocked`; it must not remain silently hung.

#### LowMem HTTP failure

* Do not concatenate a `curl` command into the harness launch string.
* Invoke ResourceMode through an internal typed daemon function or API client.
* Use the correct content type and validated body.
* Resolve LowMem before process spawn.
* If LowMem is required and activation fails, block before launch.
* If LowMem is advisory, record degraded posture and continue according to policy.
* A LowMem activation failure must never accidentally terminate the harness command through shell chaining.

#### Reproducibility

Persist:

* executable;
* argv;
* safe environment;
* secret references;
* mission artifact;
* bootstrap packet;
* model binding;
* thinking level;
* config revision;
* project/workspace;
* adapter and backend versions;
* trust decision;
* resource policy.

### 13.4 Pause and resume

Support two pause levels:

1. **Soft pause**

   * stop dispatching new prompts or work items;
   * allow the current bounded tool or turn to settle;
   * preferred portable behavior.

2. **Hard pause**

   * suspend the process tree where supported;
   * capability-gated;
   * operator-visible;
   * not claimed on unsupported backends.

A backend that cannot hard-pause must report that limitation rather than pretend success.

### 13.5 Termination escalation

Default controlled stop:

1. harness-native abort;
2. grace period;
3. process-group graceful termination;
4. second grace period;
5. force termination of the owned tree;
6. child-leak verification;
7. terminal event and receipt projection.

Every stage is evented.

### 13.6 Retry budgets

Retries must be typed by class:

* provider retry;
* transport reconnect;
* harness restart;
* tool/environment recovery;
* model fallback;
* runner reconnect;
* work-item retry.

Budgets must be independent. A provider retry must not silently consume a full session-restart budget.

### 13.7 Orphan adoption

After daemon restart:

* query active runners;
* verify runner identity and protocol;
* compare process manifest hash;
* compare session/run IDs;
* compare OS user and workspace;
* restore stream cursor;
* reconcile lifecycle state;
* adopt only on a complete match.

Unknown processes are not adopted.

### 13.8 Reboot recovery

A machine reboot normally destroys active processes.

After reboot:

* restore durable session state;
* classify unfinished active runs as `orphaned`;
* load the latest runtime and Workpoint checkpoints;
* evaluate whether policy permits a new run generation;
* require operator acknowledgment when policy says so;
* never claim that the original process survived.

---

## 14. Harness adapter and backend capability contract

### 14.1 Harness adapter interface

```rust
trait HarnessAdapter {
    fn capabilities(&self) -> HarnessCapabilities;
    fn preflight(&self, config: &EffectiveConfig) -> PreflightResult;
    fn build_launch_manifest(&self, config: &EffectiveConfig) -> LaunchManifest;
    fn parse_event(&self, frame: &[u8]) -> Vec<SilentSessionEvent>;
    fn send_prompt(&self, run: RunRef, prompt: PromptPayload) -> Result<()>;
    fn send_input(&self, run: RunRef, input: InputPayload) -> Result<()>;
    fn abort(&self, run: RunRef) -> Result<()>;
    fn query_state(&self, run: RunRef) -> Result<HarnessState>;
    fn query_model(&self, run: RunRef) -> Result<ObservedModelBinding>;
    fn resume_native_session(&self, native_ref: &str) -> Result<()>;
}
```

### 14.2 Required capability declaration

```yaml
HarnessCapabilities:
  structured_events:
  stdout_stderr_split:
  semantic_agent_state:
  model_preflight:
  model_observation:
  model_switch:
  thinking_control:
  native_session_resume:
  prompt_delivery:
  steering:
  followup_queue:
  special_keys:
  native_abort:
  hard_pause:
  token_usage:
  cost_usage:
  subscription_entitlement_probe:
```

### 14.3 Process backend interface

Process backends declare:

```yaml
ProcessBackendCapabilities:
  platform:
  detached_execution:
  reconnect_after_client_exit:
  survive_daemon_restart:
  survive_machine_reboot:
  stdout_stderr_capture:
  pty:
  attach:
  send_text:
  send_keys:
  process_tree_kill:
  hard_pause:
  cpu_limit:
  memory_limit:
  pid_limit:
  disk_limit:
```

### 14.4 Initial adapters and backends

#### Pi RPC adapter

Preferred first structured harness because it can expose:

* turn and agent lifecycle;
* streamed messages;
* tool execution events;
* model state;
* steering;
* abort;
* session references;
* token and usage data.

#### Generic PTY adapter

Fallback for harnesses without structured RPC.

It must label prompt and blocker detection as heuristic unless verified.

#### Direct process backend

Canonical initial backend for RPC-capable agents.

#### Herdr backend

Optional interactive workspace and pane backend.

Herdr may improve attachment and semantic status, but Focusa remains canonical.

#### tmux backend

Compatibility fallback and migration bridge.

tmux metadata must not become canonical session identity.

#### Windows backend

A declared `windows_job_conpty` backend must exist before Windows is advertised as supported. Until it passes runtime tests, capability negotiation must return `unsupported`, not silently fall back to an untracked process.

---

## 15. Typed configuration

### 15.1 Schema

```yaml
SilentSessionConfig:
  schema: focusa.silent_session_config.v1

  identity:
    display_name:
    project_root:
    continuity_id:
    work_item_ref:
    mission:
    agent_identity_ref:
    role_profile_ref:

  harness:
    kind: pi | codex | claude | opencode | generic_rpc | generic_pty
    adapter_version:
    native_resume_policy: prefer | require | disable

  model:
    provider:
    model:
    thinking:
    selection_policy: exact | allow_list | adaptive
    fallback_policy: disabled | explicit_allow_list
    allowed_fallbacks: []
    auth_profile_ref:
    require_entitlement_preflight: true
    require_runtime_model_confirmation: true

  workspace:
    strategy: isolated_worktree | exclusive_existing | read_only_shared | explicit_shared
    source_root:
    worktree_root:
    base_ref:
    branch_name:
    integration_policy: manual | verified_fast_forward | governed_merge

  bootstrap:
    target_profile:
    packet_mode: session_start
    verification_required: true

  supervision:
    restart_policy:
    max_process_restarts:
    max_transport_retries:
    retry_backoff:
    soft_pause_timeout:
    graceful_stop_timeout:
    checkpoint_interval_seconds:
    checkpoint_event_interval:
    waiting_input_timeout:
    silent_output_warning_seconds:

  resources:
    priority:
    max_wall_clock_seconds:
    max_cpu_percent:
    max_memory_bytes:
    max_pids:
    max_disk_bytes:
    max_output_bytes:
    max_tokens:
    max_cost_usd:
    max_turns:

  output:
    persist_stdout: true
    persist_stderr: true
    persist_semantic_events: true
    chunk_max_bytes:
    chunk_max_seconds:
    redaction_profile_ref:
    operator_projection_budget:
    raw_retention_policy_ref:

  governance:
    context_authority_required: true
    risky_mutation_preflight_required: true
    destructive_actions_allowed: false
    writer_lease_required: true
    completion_receipt_required: true
    evidence_policy_ref:
    policy_locks: []

  notifications:
    waiting_input: true
    blocked: true
    failed: true
    completed: true
    model_mismatch: true
    budget_pressure: true
    channels: []

  retention:
    policy_ref:
    evidence_hold:
```

### 15.2 Configuration precedence

Lowest to highest:

```text
1. compiled safe defaults
2. named global execution profile
3. project-level Silent Session policy
4. selected named preset
5. Workpoint/Trajectory/Context Authority constraints
6. explicit session request overrides
7. operator-approved running-session revision
8. non-overridable constitutional and security policy locks
```

Every effective field must retain provenance showing which layer supplied it.

### 15.3 Profile and preset distinction

A **profile** defines reusable execution identity and environment choices, such as:

* harness;
* provider;
* model;
* auth profile;
* workspace strategy;
* runner backend.

A **preset** defines behavioral policy, such as:

* conservative;
* balanced;
* push;
* audit.

### 15.4 Policy locks

Policies may lock:

* exact provider and model;
* fallback disabled;
* isolated worktree required;
* maximum cost;
* maximum concurrency;
* destructive actions disabled;
* receipt required;
* specific auth profile;
* specific OS execution user.

A lower layer cannot override a lock.

### 15.5 Effective config

Before launch, return:

* requested config;
* resolved effective config;
* field provenance;
* policy locks;
* restart-required fields;
* warnings;
* validation result;
* redacted config hash.

### 15.6 Hot-mutable fields

May be changed transactionally without process restart when supported:

* notification channels;
* operator rendering verbosity;
* follow filters;
* output projection budget;
* checkpoint cadence;
* retry budgets not already exceeded;
* resource limits that are tightened;
* priority;
* soft-pause behavior;
* completion notification policy.

### 15.7 Restart-required fields

Require a new run generation:

* harness;
* provider;
* model;
* thinking level;
* fallback policy;
* auth profile;
* executable or argv;
* project root;
* workspace strategy;
* worktree;
* OS execution user;
* process backend;
* trust mode;
* secret environment references.

### 15.8 Immutable session fields

Cannot be revised in place:

* `session_id`;
* original project identity;
* continuity boundary;
* creator principal;
* retention evidence-hold provenance.

### 15.9 Transactional revision flow

```text
preview
  → validate
  → show effective diff
  → Context Authority / operator gate
  → persist revision
  → apply hot fields or create restart plan
  → verify
  → commit
```

Failure rolls back to the prior effective revision.

---

## 16. Provider and model safety

### 16.1 Required binding fields

```yaml
ModelBinding:
  provider:
  model:
  thinking:
  auth_profile_ref:
  selection_policy:
  fallback_policy:
  requested_at:
```

### 16.2 Three-stage model truth

Every run must record:

1. `requested_model_binding`
2. `effective_model_binding`
3. `observed_model_binding`

All three must match when `selection_policy=exact`.

### 16.3 Strict model example

```yaml
model:
  provider: openai-codex
  model: gpt-5.6-luna
  thinking: xhigh
  selection_policy: exact
  fallback_policy: disabled
  allowed_fallbacks: []
  require_entitlement_preflight: true
  require_runtime_model_confirmation: true
```

This prevents silent inheritance of OpenRouter, Kimi, or another ambient default.

### 16.4 Provider preflight

The adapter must check, where supported:

* provider configured;
* authentication available;
* authentication type;
* subscription or API entitlement;
* exact model availability;
* thinking-level support;
* context-window compatibility;
* rate-limit posture;
* billing or usage budget posture;
* model catalog freshness.

Result fields:

```text
passed
blocked
degraded
unknown
```

A strict profile blocks on `unknown` entitlement when entitlement verification is required.

### 16.5 Runtime confirmation barrier

A session may launch into `initializing`, but it must not begin project mutation until:

* harness connection is established;
* observed model is read;
* model matches policy;
* bootstrap verification passes;
* writer lease remains valid;
* Context Authority launch verdict remains fresh.

A mismatch triggers:

* `model.mismatch`;
* immediate mutation barrier;
* controlled abort;
* blocked state;
* operator notification.

### 16.6 Model changes

A model change requires:

* Workpoint checkpoint with reason `model_switch`;
* configuration revision;
* new preflight;
* new run generation unless the adapter proves safe in-place switching;
* refreshed bootstrap packet;
* runtime model confirmation;
* event and receipt linkage.

### 16.7 Fallback

Fallback is disabled by default.

When enabled:

* only listed models are eligible;
* trigger classes are explicit;
* cost and capability constraints are re-evaluated;
* operator is notified;
* Workpoint remains unchanged unless explicitly updated;
* effective and observed bindings are persisted;
* completion receipt records each model used.

---

## 17. Authorization and security

### 17.1 `approved=true` is insufficient

The legacy boolean may remain as a compatibility input, but the daemon must require durable authorization context.

A control request must resolve:

* authenticated principal;
* actor and role;
* OS user;
* route scopes;
* project permission;
* continuity scope;
* Workpoint/work-item permission;
* writer ownership;
* Context Authority verdict where required;
* operator approval receipt where required;
* target session and run generation.

### 17.2 Route scopes

Add:

```text
silent_sessions:read
silent_sessions:stream
silent_sessions:create
silent_sessions:control
silent_sessions:config
silent_sessions:admin
silent_sessions:forensics
```

Suggested mapping:

| Action                                   | Scope                       |
| ---------------------------------------- | --------------------------- |
| list/show                                | `silent_sessions:read`      |
| events/output follow                     | `silent_sessions:stream`    |
| create/preflight/start                   | `silent_sessions:create`    |
| send/input/pause/resume/interrupt/cancel | `silent_sessions:control`   |
| config revisions/rollback                | `silent_sessions:config`    |
| cross-user view/adoption/force kill      | `silent_sessions:admin`     |
| sealed raw forensic access               | `silent_sessions:forensics` |

### 17.3 Durable approval record

Approval records should include:

* approval ID;
* operator actor;
* action;
* project and continuity scope;
* session and run;
* config hash;
* command or mutation digest;
* model binding;
* workspace;
* risk class;
* expiration;
* permitted side effects.

### 17.4 Cross-user isolation

Default behavior:

* users see only their own sessions;
* project metadata is redacted across OS-user boundaries;
* stream access is denied across users;
* admin views default to summaries;
* raw output requires additional forensic permission;
* runner sockets are user-scoped;
* daemon commands to runners are authenticated.

### 17.5 Audit redaction

Persist a complete control audit without storing:

* raw bearer tokens;
* provider credentials;
* secret environment values;
* private key material;
* auth headers;
* unredacted connector secrets.

Store secret references and redaction classifications instead.

### 17.6 Remote access

* loopback remains the default;
* non-loopback requires Focusa authentication;
* Tailscale or other private networking does not replace route authorization;
* stream endpoints require authenticated scopes;
* cloud or remote controllers may request actions, but the local node performs policy and Context Authority decisions.

---

## 18. Concurrency, worktrees, and writer ownership

### 18.1 Admission law

A writing Silent Session cannot be admitted until Focusa determines:

* existing Work Loop writer;
* foreground actor;
* active Silent Session writers;
* target work item;
* workspace cleanliness;
* worktree identity;
* overlapping path intent;
* lease conflicts.

### 18.2 Workspace strategies

#### `isolated_worktree`

Default for background mutation.

Creates:

* dedicated git worktree;
* dedicated branch;
* session-to-worktree binding;
* integration policy.

#### `exclusive_existing`

May use an existing workspace only when:

* no competing writer exists;
* writer lease is acquired;
* operator explicitly selected it or policy allows it.

A dirty worktree is not inherently invalid for its sole owner.

#### `read_only_shared`

Allows inspection without mutation.

#### `explicit_shared`

High-risk mode requiring:

* explicit operator approval;
* compatible writer policy;
* path leases;
* visible conflict warnings.

Not an MVP default.

### 18.3 Writer leases

Minimum lease scopes:

* `project_root + continuity_id`;
* work item;
* workspace/worktree;
* mutation mode.

Optional path-intent leases may improve parallelism but do not replace work-item ownership.

### 18.4 Important reconciliation with existing dirty-tree behavior

A dirty worktree must not hard-block its current sole writer merely because it is dirty.

However, dirty state must block admission of a second writer into the same workspace unless:

* the second session is read-only;
* an isolated worktree is created;
* or explicit shared-writer policy is approved.

### 18.5 Default worktree naming

```text
branch: focusa/silent/<session-short-id>/<work-item>
path:   <focusa-worktree-root>/<project>/<session-short-id>
```

Names must be sanitized and collision-safe.

### 18.6 Scheduler

The daemon scheduler must consider:

* global concurrency quota;
* per-user quota;
* per-project quota;
* per-provider quota;
* resource pressure;
* work-item dependencies;
* writer leases;
* session priority;
* blocked sibling work;
* alternate ready work.

### 18.7 Safe integration protocol

A background session does not silently merge into the operator’s primary workspace.

Required flow:

```text
isolated implementation
  → tests and verification
  → final Workpoint checkpoint
  → diff and commit evidence
  → integration preview
  → Context Authority preflight
  → governed merge/rebase/cherry-pick
  → conflict detection
  → integration receipt
```

Conflicts transition to `blocked`, not destructive cleanup.

---

## 19. Integration with core Focusa primitives

### 19.1 Operator Ask and Steering

* Creation must capture the exact operator ask.
* New steering is evented and bound to the session.
* Steering may update current direction only through the correct Workpoint/Work Loop path.
* Stale session prompts must not outrank newer operator input.

### 19.2 ProjectIdentity

ProjectIdentity is the scope gate.

Start is blocked when:

* project root is missing;
* project identity mismatches;
* root ownership is unsafe;
* continuity scope is missing for canonical work;
* target work item belongs to another project.

### 19.3 Continuity ID

The Silent Session binds to the logical workstream, not the foreground terminal session.

Pi/plugin restarts do not change the Continuity ID.

### 19.4 Trajectory

Bind:

* HLT;
* MLG;
* STG;
* relevant Waypoints;
* active gap;
* destination and completion posture.

A generic placeholder HLT must remain visibly degraded.

The session cannot silently redefine Trajectory.

### 19.5 Workpoint

Workpoint supplies:

* immediate mission;
* current ActionIntent;
* active objects;
* verification hooks;
* blockers;
* next slice;
* do-not-drift constraints.

Required checkpoints:

* before launch;
* after bootstrap verification;
* before risky mutation;
* after meaningful evidence;
* before work-item transition;
* before model switch;
* on pause;
* before controlled shutdown;
* after recovery;
* before completion evaluation.

Checkpoint frequency must remain meaningful. Runtime heartbeats do not require canonical Workpoint spam.

### 19.6 Focus State and Focus Stack

Silent Sessions may read bounded Focus State and active Focus Stack frames.

They must not:

* store full output in Focus State;
* create parallel long-lived state in the adapter;
* treat terminal history as Focus State;
* write verbose process logs into cognitive slots.

Useful durable decisions, constraints, failures, and results must use normal validated Focus State paths.

### 19.7 Context Cognition

Before launch and major work-item transitions, Context Cognition may build a bounded advisory packet containing:

* relevant files;
* relevant specs;
* diffs;
* active objects;
* evidence gaps;
* risks;
* excluded context;
* valid next tools;
* do-not-drift guidance.

Context Cognition remains advisory and cannot authorize execution.

### 19.8 Context Authority

Context Authority is required for:

* session launch when mutation is planned;
* daemon or service restart;
* git integration;
* deploy;
* release;
* database migration;
* destructive file operations;
* secret/config changes;
* cross-project edits;
* generated-code overwrite;
* model or trust-policy changes with execution implications.

A preflight verdict must be fresh and action-specific.

### 19.9 Ontology and affordance model

Each session should bind ontology refs for:

* `AgentIdentity`;
* `ActorInstance`;
* `RoleProfile`;
* `CapabilityProfile`;
* `PermissionProfile`;
* `Responsibility`;
* `HandoffBoundary`;
* `ExecutionContext`;
* `ToolSurface`;
* `Affordance`;
* `Resource`;
* `CostModel`;
* `ReliabilityProfile`;
* `ReversibilityProfile`;
* `WorkItem`;
* `ActionIntent`;
* `Blocker`;
* `VerificationRecord`;
* `EvidenceArtifact`.

This allows Focusa to distinguish what the agent wants to do from what it can safely and practically do.

### 19.10 Work Loop

Silent Sessions are an execution substrate beneath governed continuous work.

Work Loop owns:

* ordered work selection;
* writer ownership;
* continuation decisions;
* blocker deferral;
* alternate-ready-work selection;
* pause/stop policy;
* work-item progression.

Silent Session owns:

* one supervised agent execution;
* runtime streams;
* process control;
* model binding;
* session-local checkpoints;
* operator interaction transport.

The two must not become parallel schedulers.

### 19.11 Evidence and Reference Store

Store large outputs, diffs, test logs, manifests, and final reports as Reference Store artifacts.

Evidence linking must identify:

* target Workpoint;
* target work item;
* target claim;
* source run and event cursor;
* verification class;
* freshness;
* hash.

### 19.12 Agent Bootstrap

Before mutation, create and verify an `AgentBootstrapPacket`.

The packet must include:

* project identity;
* continuity;
* Workpoint;
* Trajectory;
* selected context;
* evidence refs;
* next action;
* blockers;
* do-not-drift constraints;
* model and role information;
* completion expectations.

Failure to verify bootstrap blocks autonomous mutation.

### 19.13 Session Transfer

Pause, orphan recovery, handoff, and model-switch flows should use Session Transfer projections.

A foreground Pi session may take over from a Silent Session only through an explicit transfer or writer handoff.

### 19.14 Receipts and execution ledger

Do not create a second durable audit ledger.

Use Spec119:

```text
receipt_type = work_session
execution_mode = silent_session
```

Additional receipt use:

* `risky_mutation`;
* `blocked_claim`;
* `handoff`;
* `bootstrap_delivery`;
* `work_item_closure`;
* `final_report`.

Receipt commits must flow through the existing event and event-hash-chain path.

### 19.15 Work-item closure authority

The session may propose closure.

It must not directly treat its own final message as closure truth.

Closure follows:

```text
prepare
  → validate
  → authorize
  → provider submit
  → reconcile
  → audit
```

### 19.16 Prediction

Record predictions before uncertain actions such as:

* model fallback;
* broad refactor;
* flaky test repair;
* dependency upgrade;
* risky integration;
* recovery strategy.

Evaluate them afterward.

### 19.17 Metacognition

After completion or failure:

* capture reusable workflow lessons;
* evaluate whether the selected model and strategy worked;
* assess blocker handling;
* assess retry waste;
* assess checkpoint quality;
* propose policy changes.

Metacognition remains advisory until outcome-evaluated and promoted.

### 19.18 ResourceMode and Bloatgaurd

ResourceMode governs execution fidelity and budgets.

Bloatgaurd governs operator and agent projections, not canonical storage.

Under pressure:

* retain complete durable streams according to storage policy;
* reduce live projection verbosity;
* summarize repeated output;
* preserve errors, prompts, Workpoint changes, blockers, and evidence;
* provide handles for rehydration;
* never silently truncate canonical stream indexes.

### 19.19 Proposal Resolution and governance

A session that encounters a governance-relevant choice must:

* propose;
* pause or continue only if policy allows unrelated safe work;
* wait for operator/governance resolution;
* never self-approve.

### 19.20 Awareness, Utility Cards, menubar, and dashboard

These are projections.

They may display and invoke scoped daemon actions but cannot mint canonical authority.

### 19.21 UIAI Engine

UIAI may provide browser and product-reality evidence.

UIAI findings remain proposal-only until captured and linked through Focusa Evidence.

---

## 20. Checkpoint, evidence, and completion protocol

### 20.1 Periodic runtime checkpoints

Default triggers:

* every configured time interval;
* every configured number of semantic events;
* after tool completion when durable project change occurred;
* before and after retry escalation;
* before pause;
* before process restart;
* before daemon upgrade;
* on runner disconnect.

### 20.2 Canonical Workpoint checkpoint triggers

Create only when meaning changes:

* mission or current ActionIntent changes;
* active object set changes;
* blocker changes;
* evidence changes;
* next slice changes;
* work item advances;
* operator steering changes direction;
* model switch occurs;
* completion evaluation begins.

### 20.3 Completion evidence bundle

Minimum for code-changing sessions:

* exact project and worktree;
* starting and ending git status;
* bounded diff summary;
* full diff artifact handle;
* files changed;
* tests run;
* test outputs and exit codes;
* lint/typecheck/spec gates where required;
* commit ref when commit policy requires;
* Workpoint final checkpoint;
* unresolved blockers;
* Context Authority refs for risky actions;
* model usage;
* resource usage;
* stream manifest;
* completion verifier result;
* receipt preview.

### 20.4 Completion decision

```text
process exits
  → state becomes completing
  → gather artifacts
  → refresh ProjectIdentity and Workpoint
  → run required verification
  → evaluate work-item acceptance
  → run adversarial closure verifier where policy requires
  → build receipt preview
  → commit receipt
  → transition to completed
```

When evidence is missing:

```text
completing
  → blocked
  reason = completion_evidence_missing
```

When verification fails:

```text
completing
  → failed or blocked
  reason = verification_failed
```

### 20.5 Durable reconstruction

After context loss, an operator or agent must be able to reconstruct:

* original ask;
* effective config;
* project/workspace;
* model;
* Workpoint history;
* output cursor;
* events;
* changes;
* evidence;
* verification;
* completion result;
* next safe action.

No reconstruction may depend on the original foreground Pi transcript.

---

## 21. Resources and admission control

### 21.1 Admission checks

Before launch:

* global session quota;
* user quota;
* project quota;
* provider quota;
* available CPU and memory;
* disk and stream-spool capacity;
* output budget;
* token and cost budget;
* writer lease;
* worktree availability;
* runner availability;
* model entitlement;
* current ResourceMode;
* Context Authority;
* Workpoint readiness.

### 21.2 Required budget levels

* per turn;
* per run;
* per session;
* per work item;
* per project;
* per user;
* per provider/model;
* global host.

### 21.3 OS resource enforcement

Where available:

* CPU quota or scheduling weight;
* memory high/max;
* process count;
* open-file limit;
* I/O priority;
* disk quota;
* wall-clock timeout.

Unsupported enforcement must be declared.

### 21.4 Token and cost controls

Track:

* input tokens;
* output tokens;
* cached tokens when available;
* estimated cost;
* provider-reported cost;
* subscription usage when available;
* context-window pressure;
* retry waste.

Approaching limits produces warnings. Exceeding hard limits pauses or cancels according to policy.

### 21.5 Output backpressure

* process capture writes to a durable spool;
* subscribers read independently;
* output flood may reduce live rendering;
* repeated lines may be summarized in projections;
* full retained output remains cursor-addressable;
* disk pressure triggers policy before storage exhaustion;
* a slow UI cannot block the agent process.

### 21.6 Priority

Sessions may declare:

```text
interactive
high
normal
background
low
maintenance
```

Foreground operator work should normally outrank unattended background work.

---

## 22. Operator experience

### 22.1 Start behavior

Interactive CLI start should watch by default:

```text
focusa silent start ...
```

Use `--detach` to return immediately.

Detached start must still print:

```text
Session:
Run:
State:
Project:
Worktree:
Work item:
Provider/model:
Thinking:
Fallback:
Budget:
Watch:
Dashboard:
```

### 22.2 Persistent dashboard

The daemon should serve a local dashboard showing:

* all visible sessions;
* state and health;
* project and work item;
* model;
* elapsed time;
* current activity;
* recent output;
* resource usage;
* last checkpoint;
* blocker/input request;
* evidence and completion state;
* controls.

The dashboard survives Pi and plugin restarts.

### 22.3 Live view modes

```text
Summary
Agent text
Tools
stdout
stderr
All structured events
Raw terminal
Evidence/checkpoints
```

### 22.4 Cursor-follow UX

```bash
focusa silent watch <session>
focusa silent watch <session> --after <cursor>
focusa silent watch <session> --tools
focusa silent watch <session> --stderr
focusa silent output <session> --after <cursor> --limit 200
```

### 22.5 Controls

```text
send text
send follow-up
send steering
send special key
soft pause
hard pause when supported
resume
interrupt
controlled stop
force cancel
restart
adopt
handoff
open worktree
open evidence
open receipt
```

### 22.6 Proactive notifications

Default notification triggers:

* waiting for operator input;
* blocker requiring judgment;
* model mismatch;
* auth or entitlement failure;
* repeated provider failure;
* resource pressure;
* checkpoint failure;
* process failure;
* orphaned run;
* completion blocked by missing evidence;
* verified completion.

Notifications must deduplicate repeated identical states.

### 22.7 Creation wizard

Suggested steps:

1. Select project and verify ProjectIdentity.
2. Select Continuity ID and Workpoint.
3. Select work item and mission.
4. Choose workspace strategy.
5. Choose harness profile.
6. Choose exact provider, model, and thinking level.
7. Verify authentication and entitlement.
8. Select policy preset.
9. Review resource and cost budgets.
10. Review Context Authority and writer lease.
11. Preview effective configuration.
12. Approve and launch.
13. Open live watch automatically.

### 22.8 Context-flood protection

Operator-facing summaries should show:

* meaningful deltas;
* current action;
* errors;
* blockers;
* tool boundaries;
* evidence;
* checkpoints.

Full output remains available by cursor or artifact handle rather than being dumped into operator context.

---

## 23. API contract

### 23.1 Session lifecycle

```text
POST   /v1/silent-sessions/preflight
POST   /v1/silent-sessions
GET    /v1/silent-sessions
GET    /v1/silent-sessions/{session_id}
POST   /v1/silent-sessions/{session_id}/start
POST   /v1/silent-sessions/{session_id}/pause
POST   /v1/silent-sessions/{session_id}/resume
POST   /v1/silent-sessions/{session_id}/interrupt
POST   /v1/silent-sessions/{session_id}/cancel
POST   /v1/silent-sessions/{session_id}/restart
POST   /v1/silent-sessions/{session_id}/adopt
```

### 23.2 Observation

```text
GET /v1/silent-sessions/{session_id}/events
GET /v1/silent-sessions/{session_id}/output
GET /v1/silent-sessions/{session_id}/status
GET /v1/silent-sessions/{session_id}/usage
GET /v1/silent-sessions/{session_id}/checkpoints
GET /v1/silent-sessions/{session_id}/artifacts
GET /v1/silent-sessions/{session_id}/receipts
```

`events` must support SSE or an equivalent resumable streaming protocol with `Last-Event-ID`.

### 23.3 Input

```text
POST /v1/silent-sessions/{session_id}/input
POST /v1/silent-sessions/{session_id}/steer
POST /v1/silent-sessions/{session_id}/follow-up
POST /v1/silent-sessions/{session_id}/keys
```

### 23.4 Configuration

```text
GET  /v1/silent-sessions/profiles
GET  /v1/silent-sessions/presets
POST /v1/silent-sessions/config/resolve
POST /v1/silent-sessions/{session_id}/config/preview
POST /v1/silent-sessions/{session_id}/config/revisions
POST /v1/silent-sessions/{session_id}/config/rollback
```

### 23.5 Capability and models

```text
GET  /v1/silent-sessions/capabilities
GET  /v1/harnesses
GET  /v1/harnesses/{harness}/capabilities
POST /v1/harnesses/{harness}/preflight
GET  /v1/providers
GET  /v1/providers/{provider}/models
POST /v1/providers/{provider}/models/preflight
```

### 23.6 Retention and export

```text
POST   /v1/silent-sessions/{session_id}/export
POST   /v1/silent-sessions/{session_id}/evidence-hold
DELETE /v1/silent-sessions/{session_id}
POST   /v1/silent-sessions/{session_id}/purge
```

Delete and purge are separate. Purge requires stronger authority.

### 23.7 Result envelopes

All routes use Focusa’s shared envelope:

```text
ok
status
canonical
advisory
degraded
stale
failure_class
retry
side_effects
evidence_refs
receipt_refs
next_tools
recovery_hint
misuse_hint
```

---

## 24. CLI contract

```bash
focusa silent preflight
focusa silent create
focusa silent start
focusa silent list
focusa silent show
focusa silent watch
focusa silent output
focusa silent send
focusa silent steer
focusa silent follow-up
focusa silent key
focusa silent pause
focusa silent resume
focusa silent interrupt
focusa silent cancel
focusa silent restart
focusa silent adopt

focusa silent config resolve
focusa silent config diff
focusa silent config apply
focusa silent config rollback
focusa silent profile list
focusa silent preset list

focusa silent checkpoints
focusa silent evidence
focusa silent receipt
focusa silent export
focusa silent hold
focusa silent delete
focusa silent purge
focusa silent doctor
```

Requirements:

* human and stable JSON output;
* cursor-based follow;
* no raw secret output;
* explicit side effects;
* exact session and run IDs;
* no ambiguous retries;
* completion status separate from process status.

---

## 25. Pi extension and tool migration

### 25.1 New role

The Pi extension becomes:

* a daemon API client;
* an in-Pi status and notification surface;
* a compatibility wrapper;
* a source of foreground steering;
* a consumer of Workpoint and bootstrap packets.

It stops:

* writing `/tmp` registries;
* directly launching tmux;
* owning model selection;
* owning process lifecycle;
* owning canonical health;
* owning recovery state.

### 25.2 Compatibility tool

Retain:

```text
focusa_silent_sessions
```

Map actions to daemon routes.

New actions may include:

```text
preflight
watch
pause
resume
config
receipt
```

The tool contract changes from:

```text
parity: pi_only
```

to:

```text
parity: full
```

with API and CLI surfaces.

### 25.3 Legacy action mapping

| Legacy action | Daemon action                             |
| ------------- | ----------------------------------------- |
| `list`        | list sessions                             |
| `start`       | preflight/create/start                    |
| `reopen`      | return dashboard/watch/attach projections |
| `tail`        | cursor-based output query                 |
| `health`      | status projection                         |
| `send`        | input/steer                               |
| `interrupt`   | controlled interrupt                      |
| `restart`     | new run generation                        |
| `kill`        | cancel with authority                     |

### 25.4 Attach behavior

`reopen` should no longer return only a tmux command.

It should return:

* watch command;
* dashboard route;
* native harness session reference;
* terminal attach option when available;
* backend capability details.

---

## 26. Retention, export, deletion, and forensics

### 26.1 Default retention classes

Suggested defaults:

* canonical lifecycle events and receipts: governed by Focusa canonical event policy;
* raw redacted output: 30 days after terminal state;
* completion artifacts: 180 days;
* active blocker and recovery artifacts: retained until resolved plus policy window;
* pinned Evidence artifacts: no automatic deletion;
* sealed forensic artifacts: explicit policy only.

### 26.2 Active relevance versus historical truth

A completed session may decay out of active cards while remaining historically verifiable.

Archived sessions must not keep influencing active Workpoint or Context Cognition packets merely because their output exists.

### 26.3 Evidence hold

Evidence hold prevents pruning of:

* stream chunks;
* manifests;
* checkpoints;
* receipts;
* referenced diffs;
* test artifacts;
* configuration revisions.

### 26.4 Export

Export package may include:

* redacted manifest;
* events;
* stream chunks;
* checkpoints;
* artifacts;
* receipts;
* integrity hashes;
* schema versions;
* verification instructions.

### 26.5 Deletion

Ordinary delete:

* removes active projections;
* schedules eligible raw artifacts for retention-policy cleanup;
* creates an audit event;
* does not silently break receipt or event integrity.

### 26.6 Purge

Purge requires:

* explicit operator authorization;
* Context Authority;
* evidence-hold check;
* impact preview;
* tombstone event;
* retained hashes and deletion metadata where policy permits;
* clear warning that forensic reconstruction may become impossible.

---

## 27. Versioning and rolling upgrades

### 27.1 Version fields

Declare independently:

```text
silent_session_schema_version
config_schema_version
event_schema_version
daemon_runner_protocol_version
harness_adapter_protocol_version
process_backend_protocol_version
stream_chunk_format_version
receipt_mapping_version
```

### 27.2 Capability handshake

Runner and daemon handshake must exchange:

* supported versions;
* optional capabilities;
* required capabilities;
* platform;
* user identity;
* resource-control support;
* active run references.

### 27.3 Compatibility rules

* additive event fields are tolerated;
* unknown event kinds are preserved;
* breaking config changes require migration;
* one previous major protocol may be supported during rolling upgrade;
* unsupported required capability blocks start;
* an active session is not silently transferred to an incompatible runner.

### 27.4 Rolling upgrade behavior

Before upgrade:

1. checkpoint active sessions;
2. mark runner drain posture;
3. stop admitting incompatible runs;
4. preserve event cursors;
5. upgrade daemon or runner;
6. reconnect and reconcile;
7. adopt compatible processes;
8. create new run generations when necessary;
9. report any orphaned sessions.

### 27.5 Migration

Every breaking schema change requires:

* migration plan;
* backup;
* dry-run;
* compatibility report;
* post-migration conformance verification;
* rollback path;
* governance decision record.

---

## 28. Failure taxonomy

Required failure classes include:

```text
scope_mismatch
project_identity_unverified
continuity_missing
workpoint_unavailable
writer_conflict
workspace_conflict
authorization_required
permission_denied
approval_expired
context_authority_blocked
config_invalid
config_locked
model_not_found
model_entitlement_unverified
model_mismatch
fallback_disallowed
harness_unsupported
backend_unsupported
capability_missing
runner_unavailable
runner_lost
process_spawn_failed
process_control_failed
process_exited
child_leak_detected
transport_degraded
transport_lost
waiting_input
provider_failure
retry_exhausted
resource_admission_denied
resource_limit_exceeded
output_storage_pressure
stream_corruption
checkpoint_failed
evidence_missing
verification_failed
completion_evidence_missing
receipt_commit_failed
orphan_adoption_rejected
protocol_incompatible
retention_blocked_by_hold
```

Every failure must expose:

* why;
* current lifecycle;
* canonical versus runtime posture;
* safe retry posture;
* side effects already performed;
* exact recovery tools;
* whether operator action is required.

---

## 29. Testing and proof requirements

Static string tests are insufficient.

### 29.1 Test harnesses

Build:

1. deterministic fake harness adapter;
2. real subprocess fixture;
3. child-leak fixture;
4. prompt/wait fixture;
5. output-flood fixture;
6. model-mismatch fixture;
7. retry/failure fixture;
8. isolated git repository fixture;
9. fake provider entitlement service;
10. runner disconnect/reconnect fixture.

### 29.2 Mandatory runtime E2E tests

#### Durability

* create and start;
* restart Pi plugin;
* restart client;
* restart daemon;
* reconnect runner;
* verify state, cursor, and output remain available;
* simulate corrupted final stream chunk;
* verify quarantine and recovery;
* simulate machine reboot and verify orphan/relaunch policy.

#### Output

* stdout/stderr distinction;
* semantic-event ordering;
* cursor resume;
* duplicate delivery deduplication;
* continuous rotation;
* slow subscriber;
* output flood;
* completion artifact generation;
* dead-session output retrieval.

#### Lifecycle

* working;
* waiting input;
* blocked;
* paused;
* resumed;
* completing;
* completed;
* failed;
* cancelled;
* orphaned;
* transport degraded;
* no false completion from exit code.

#### Process supervision

* graceful abort;
* force escalation;
* process-tree kill;
* child leak;
* hard-pause capability;
* soft-pause fallback;
* retry budget;
* runner loss;
* orphan adoption;
* no zombie continuation after recovery.

#### Launcher

* mission containing quotes, apostrophes, newlines, code blocks, and shell symbols;
* exact mission hash;
* required Pi trust flag;
* unexpected trust prompt;
* LowMem success;
* LowMem HTTP/content-type failure;
* LowMem required versus advisory policy;
* exact argv and effective config persistence.

#### Authorization

* legacy `approved=true` without principal is rejected;
* scoped token create/read/stream/control separation;
* approval expiration;
* wrong project;
* wrong Workpoint;
* wrong OS user;
* cross-user list redaction;
* cross-user stream denial;
* symlink attack;
* permissions on session files;
* redacted control/config audit.

#### Configuration

* precedence;
* policy locks;
* exact effective config;
* hot revision;
* restart-required revision;
* transaction rollback;
* import/export;
* schema migration;
* revision rollback.

#### Provider/model safety

* explicit exact model launch;
* missing model;
* missing entitlement;
* unknown entitlement under strict policy;
* ambient provider ignored;
* observed model mismatch;
* fallback disabled;
* allowed fallback;
* model switch checkpoint;
* model survives daemon/client/plugin restart.

#### Operator experience

* start returns watch information;
* detached session appears in dashboard;
* cursor follow after reconnect;
* notifications for waiting input, blocker, failure, completion;
* special-key control;
* output projection remains bounded;
* full output remains rehydratable.

#### Foreground independence

* start through CLI with no Pi open;
* control through a second Pi instance;
* control after original Pi exits;
* dashboard control;
* daemon restart;
* backend capability negotiation;
* unsupported Windows backend returns explicit failure.

#### Concurrency

* foreground and background target same dirty worktree;
* second writer is blocked or isolated;
* two isolated worktrees proceed;
* work-item lease conflict;
* lease expiration and safe adoption;
* overlapping path-intent warning;
* safe integration;
* merge conflict blocks without destructive cleanup;
* unrelated changes preserved.

#### Governance and evidence

* periodic runtime checkpoint;
* Workpoint checkpoint triggers;
* process exit enters `completing`;
* missing tests block completion;
* failed tests block completion;
* actual evidence allows receipt;
* risky mutation requires Context Authority;
* final receipt reconstructs the run after transcript loss.

#### Resources

* concurrency admission;
* CPU pressure;
* memory limit;
* disk pressure;
* output limit;
* wall timeout;
* token limit;
* cost limit;
* provider quota;
* backpressure;
* priority scheduling.

#### Evolution and retention

* protocol negotiation;
* previous-version runner;
* incompatible runner;
* rolling upgrade;
* evidence hold;
* ordinary delete;
* purge preview;
* export and re-import verification;
* stream retention without canonical event corruption.

### 29.3 Cross-platform matrix

At minimum:

| Platform           | Required proof                                         |
| ------------------ | ------------------------------------------------------ |
| Linux              | Full MVP runtime suite                                 |
| macOS              | Runner, process-tree, streams, pause/control, recovery |
| Windows            | Job Object/ConPTY suite before support claim           |
| tmux compatibility | Legacy migration and fallback suite                    |
| Herdr integration  | Attach, stream, state, reconnect, fallback suite       |

### 29.4 Real Pi proof

A release cannot claim the Pi adapter complete until a real Pi session proves:

```text
preflight
→ exact model resolution
→ bootstrap verification
→ isolated worktree start
→ live stream
→ tool execution
→ operator steering
→ pause/resume
→ Workpoint checkpoint
→ tests/evidence
→ process exit
→ completion evaluation
→ receipt commit
→ post-plugin-restart rehydration
```

### 29.5 Test classification

Each requirement must have one or more:

* unit test;
* state-machine property test;
* persistence test;
* route test;
* runner integration test;
* fault-injection test;
* real harness proof;
* cross-platform proof;
* security test.

A static grep may guard a contract but cannot serve as the primary proof of runtime behavior.

---

## 30. Migration strategy

### Phase 0 — Freeze and instrument the legacy wrapper

* stop expanding tmux-specific behavior;
* label it legacy;
* add telemetry for current usage;
* add safe import tooling;
* document that it is not durable.

### Phase 1 — Core domain and persistence

* add types;
* add state machine;
* add SQLite migrations;
* add event families;
* add secured stream storage;
* add fake harness and process runner;
* prove cursor replay and durability.

### Phase 2 — Daemon control plane and CLI

* add API routes;
* add route scopes;
* add CLI;
* add config resolution;
* add authorization and approval records;
* make start observable.

### Phase 3 — Pi RPC adapter and model safety

* add Pi RPC adapter;
* add provider/model preflight;
* exact model binding;
* bootstrap verification;
* live structured events;
* steering and abort;
* remove direct tmux launch from the Pi tool.

### Phase 4 — Supervision and recovery

* process groups;
* retries;
* pause/resume;
* child leak prevention;
* daemon restart adoption;
* reboot recovery;
* resource limits.

### Phase 5 — Concurrency and worktrees

* writer leases;
* isolated worktree default;
* scheduler;
* safe integration;
* foreground/background conflict policy.

### Phase 6 — Evidence and receipts

* periodic checkpoints;
* completion evaluation;
* Workpoint/Evidence linkage;
* Spec119 receipt integration;
* work-item closure integration.

### Phase 7 — Operator surfaces

* daemon dashboard;
* menubar status and notifications;
* creation wizard;
* output projections;
* cross-client rehydration.

### Phase 8 — Optional backends

* Herdr;
* tmux compatibility;
* generic PTY;
* Windows Job Object/ConPTY;
* capability negotiation matrix.

### Phase 9 — Retention and rolling upgrades

* retention policies;
* evidence hold;
* export/delete/purge;
* protocol migration;
* rolling-upgrade proof.

---

## 31. Recommended module shape

```text
crates/focusa-core/src/silent_sessions/
  mod.rs
  types.rs
  config.rs
  policy.rs
  state_machine.rs
  events.rs
  completion.rs
  leases.rs
  retention.rs

crates/focusa-core/src/reducer.rs
  canonical Silent Session event transitions

crates/focusa-core/src/runtime/
  silent_session_supervisor.rs
  silent_session_scheduler.rs
  silent_session_recovery.rs

crates/focusa-api/src/routes/
  silent_sessions.rs
  silent_session_stream.rs
  harnesses.rs
  providers.rs

crates/focusa-api/src/middleware/route_scope.rs
  silent_sessions scopes

crates/focusa-cli/src/commands/
  silent.rs

crates/focusa-session-runner/
  protocol.rs
  supervisor.rs
  streams.rs
  process_posix.rs
  process_windows.rs
  resource_linux.rs
  resource_macos.rs
  resource_windows.rs

crates/focusa-harness-adapters/
  pi_rpc.rs
  generic_rpc.rs
  generic_pty.rs
  herdr.rs
  tmux_legacy.rs

apps/pi-extension/src/
  silent-sessions-client.ts
  tools.ts
  awareness.ts

apps/menubar/
  Silent Session cards and notifications

apps/local-dashboard/
  session list, watch, config, controls, evidence

migrations/
  Silent Session tables and indexes

tests/
  spec133 unit, route, runner, E2E, fault, security,
  real-Pi, worktree, retention, upgrade, and platform suites
```

The exact crate split may be simplified initially, but policy, persistence, process supervision, and harness adaptation must not collapse back into `apps/pi-extension/src/tools.ts`.

---

## 32. Acceptance criteria

Spec 133 is satisfied only when all of the following are true:

### Architecture

* canonical control belongs to the daemon;
* the Pi extension contains no canonical registry or direct lifecycle ownership;
* Pi is one harness adapter;
* tmux and Herdr are optional backends;
* reducer purity is preserved;
* the session can be created and controlled without Pi running.

### Durability

* stable UUID identity survives plugin and daemon restart;
* config, lifecycle, events, cursors, checkpoints, and artifacts are durable;
* output remains available after process death;
* no canonical state depends on `/tmp`;
* corruption recovery is tested.

### Output and observation

* live cursor-based follow works;
* stdout and stderr are distinct;
* structured events are persisted;
* rotation is continuous;
* reconnect resumes from cursor;
* start always returns an observation surface;
* operator projections remain bounded.

### Lifecycle and supervision

* all required states exist;
* waiting, blocked, paused, completing, cancelled, failed, and orphaned are truthful;
* process trees are owned and cleaned;
* pause, resume, retry, adoption, and escalation are tested;
* exit cannot directly produce `completed`.

### Configuration and models

* `SilentSessionConfig` is typed and versioned;
* precedence and policy locks are implemented;
* revisions are transactional;
* hot and restart-required fields are classified;
* provider, model, thinking, and fallback are explicit;
* requested/effective/observed model bindings are visible;
* exact model mismatch blocks mutation;
* no silent fallback occurs.

### Authorization

* `approved=true` alone cannot authorize daemon-native mutation;
* actor, role, route scope, project, Workpoint, lease, and Context Authority are enforced;
* cross-user output and metadata are isolated;
* storage paths resist symlink and permission attacks;
* audits are complete and redacted.

### Concurrency

* foreground and background writers cannot unknowingly share a dirty worktree;
* isolated worktrees are supported;
* leases and scheduler exist;
* unrelated changes are preserved;
* integration is governed and evidence-backed.

### Governance and evidence

* canonical Workpoint checkpoints occur at meaningful boundaries;
* completion requires verification and receipts;
* tests, diffs, commits, blockers, and model/resource usage are reconstructable;
* Spec119 receipts use the existing event hash chain;
* work-item closure follows provider-neutral closure authority.

### Resources

* admission control exists;
* concurrency, CPU, memory, disk, output, time, token, and cost budgets exist;
* resource pressure is visible;
* backpressure does not block process capture;
* ResourceMode affects execution policy;
* Bloatgaurd affects projections rather than truth.

### Evolution

* protocol and schema versions are negotiated;
* rolling upgrade behavior is defined and tested;
* retention, hold, export, deletion, and forensic policies exist;
* Windows support is not claimed before a real backend passes its suite.

### Testing

* runtime E2E tests replace static string checks as primary proof;
* disconnect/reconnect, quoting, trust, LowMem, reboot, rotation, authorization, model binding, process leaks, resource pressure, plugin restart, worktree conflicts, and completion evidence are all covered;
* a real Pi proof demonstrates the complete lifecycle.

---

## 33. Gap closure matrix

| Confirmed gap               | Required closure                                                                                                              |
| --------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| 1. Durability               | SQLite-backed identities and state, secured stream storage, atomic writes, recovery, UUIDv7, legacy import                    |
| 2. Output                   | Canonical sequenced streams, stdout/stderr split, cursors, SSE/follow, continuous rotation, completion artifacts              |
| 3. Lifecycle                | Full state machine, health/activity axes, typed blockers, semantic harness events, no silence-based truth                     |
| 4. Process supervision      | Runner ownership, process groups/Job Objects, pause/resume, retries, adoption, reboot recovery, leak prevention               |
| 5. Launcher defects         | Typed argv manifest, prompt artifact delivery, trust preflight and required flags, typed LowMem call, reproducible config     |
| 6. Authorization/security   | Durable principals and approvals, route scopes, project/Workpoint authorization, user isolation, secure paths, redacted audit |
| 7. Configuration            | `SilentSessionConfig`, precedence, locks, profiles, presets, revisions, hot/restart classification, rollback                  |
| 8. Provider/model safety    | Exact requested/effective/observed binding, entitlement preflight, runtime confirmation barrier, fallback disabled by default |
| 9. Operator experience      | Persistent dashboard, watch/follow, proactive notifications, controls, wizard, rehydration, bounded projections               |
| 10. Foreground independence | Daemon API and CLI, runner, capability negotiation, Pi as client, declared Windows backend                                    |
| 11. Concurrency safety      | Writer leases, isolated worktrees, scheduler, path intent, safe integration protocol                                          |
| 12. Governance/evidence     | Workpoint checkpoints, evidence bundle, completion evaluation, receipts, no exit-equals-done                                  |
| 13. Resources               | Admission, quotas, OS limits, priority, timeout, output/backpressure, token/cost controls                                     |
| 14. Evolution/retention     | Version negotiation, migrations, rolling upgrades, retention, export, hold, delete/purge, forensics                           |
| 15. Testing                 | Runtime E2E, fault injection, real Pi proof, cross-user, cross-client, reboot, model, resources, worktrees, receipts          |

---

## 34. Reference decision

The governing architectural decision is:

> Focusa Silent Sessions must become a daemon-native, durable, harness-neutral autonomous execution subsystem with structured streams, exact model binding, process supervision, writer isolation, Focusa-wide governance integration, and receipt-backed completion. Pi, Herdr, tmux, and terminal processes remain replaceable execution adapters beneath that subsystem.

---

## 35. One-sentence summary

Focusa Silent Sessions should let an authorized operator launch, observe, steer, pause, recover, verify, and prove autonomous agent work from the daemon—using an exact model and isolated workspace—without depending on a foreground Pi process, a tmux pane, or transcript memory.
