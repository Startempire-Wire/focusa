# Spec 139 — Distributed Presence Authority, Environment Identity, Execution Placement, and Multi-Daemon Coordination

**Status:** Normative draft; primitive-owning; implementation not implied  
**Owner:** Focusa core  
**Created:** 2026-07-22  
**Canonical label:** **Spec 139 — Distributed Presence Authority, Environment Identity, Execution Placement, and Multi-Daemon Coordination**  
**Primary implementation surfaces:** Focusa core, reducer, daemon, SQLite/CRDT persistence, API, Operation Registry, generated contracts, CLI, Pi extension, Agent Bootstrap, Awareness, Workpoint, Work Loop, Silent Sessions, Evidence, Receipts, Mission Deck/Canvas, menubar, generated UI, tests, conformance, and future Focusa.work relay  
**Depends on:** Specs 16, 23–26, 34, 40, 41, 43, 53, 72, 76, 88, 96, 97, 98, 100, 104, 108, 111, 112, 116, 119, 120, 125, 130, 130A, 131, 133, 135, 135A, 135B, 135F, 135G, 135I, 135J, 136, 137, and 138  
**Research basis:** OpenTelemetry Context/Baggage, SPIFFE/SPIRE workload identity, Git worktree identity, Kubernetes Lease semantics, NATS JetStream streams/KV, local-first event replication, and the current Focusa multi-device/CRDT/runtime implementation.

---

## 0. Executive requirement

Focusa MUST maintain a continuously refreshed, scope-safe, multi-daemon operational field that tells every attached agent:

- which project, repository, workspace, worktree, machine, daemon, profile, session, actor, Workpoint, and execution run it occupies;
- which other relevant actors and daemons are present, stale, partitioned, detached, or unknown;
- what those actors declare they are doing;
- which resources they may affect;
- which claims, leases, dependencies, handoffs, and conflicts exist;
- where an operation is authorized to execute;
- whether an equivalent expensive operation is already active or reusable;
- which facts are canonical, operational, observed, inferred, stale, or unknown;
- what exact action is safe next.

Presence awareness MUST permeate agent behavior with the same structural seriousness that Spec 137 gives temporal awareness. Presence is not an optional status card, dashboard ornament, or model reminder. It is a runtime-native constraint evaluated at every consequential decision boundary.

The central rule is:

> **THE AGENT NEVER WORKS ALONE BY ASSUMPTION.**
>
> **THE AGENT NEVER ASSUMES THAT THE CURRENT MACHINE, DAEMON, WORKTREE, OR TOOLCHAIN IS THE CORRECT EXECUTION VENUE.**

---

## 1. Problem statement

Focusa is designed for multiplexed work across projects, sessions, harnesses, worktrees, machines, and daemons. Existing specifications provide substantial pieces:

- Instances, Sessions, and Attachments;
- multi-device synchronization;
- ProjectIdentity and typed project/host scope;
- scoped CRDT events and deterministic reconciliation;
- governed sessions and work loops;
- Workpoints, Trajectory, Evidence, and Receipts;
- temporal authority and prediction/metacognitive governance.

The missing substrate is a unified operational model of **where every actor is, what it is doing, which resources are occupied, and which execution route is valid now**.

Without that substrate, agents can:

- execute in the wrong clone, branch, worktree, machine, or daemon;
- start duplicate builds or tests;
- consume the same CPU, memory, disk, locks, network, and external quota unnecessarily;
- interpret a missing heartbeat as absence;
- run local validation that belongs on a remote build host;
- bypass the canonical release or deployment route;
- conflict semantically even when their worktrees are physically isolated;
- overwrite shared resources after a stale lease;
- declare completion while another actor is still producing required integration work;
- make forecasts and learning claims without preserving the topology in which they were produced.

A concrete motivating incident occurred when two local Mac coding agents independently started the Focusa Rust build chain. The Mac became resource-constrained, the work was duplicated, and both agents ignored the intended remote full release process. The correct remote agent and release venue should have owned the operation and published binaries through the approved GitHub route.

This was not merely a documentation failure. It was an **Execution Placement Violation**, a **Duplicate Expensive Operation**, and a **Presence/Coordination Failure**. The system must prevent recurrence before process spawn.

---

## 2. Scope

### 2.1 In scope

This specification owns:

1. Environment identity and environment-resolution primitives.
2. Node, daemon, daemon-boot, repository, workspace, worktree, actor, session, execution-run, and coordination-realm identity.
3. System, architecture, filesystem, runtime, toolchain, network, service, and environment-variable observation.
4. Versioned environment profiles and profile bindings.
5. Presence records, heartbeats, freshness, expiry, partitions, orphans, and recovery.
6. Work intent, activity, progress, resource footprints, occupancy, claims, leases, fencing, dependencies, conflicts, and handoffs.
7. Multi-daemon topology, peer identity, signed events, topology epochs, synchronization lag, collision detection, and recovery.
8. Execution-placement policy, operation classification, venue selection, executor assignment, remote routing, fallback policy, and resource admission.
9. Deduplication and subscription for equivalent expensive operations.
10. Presence and environment awareness saturation in planning, tools, mutation, checkpointing, handoff, prediction, and completion.
11. Bounded agent-facing presence packets, guards, deltas, and interrupts.
12. APIs, CLI commands, Pi tools, generated UI, menubar, Mission Canvas, and future Focusa.work relay projections.
13. Persistence, event, CRDT, Receipt, migration, conformance, and testing requirements.

### 2.2 Out of scope

This specification does not own:

- project identity authority, which remains with ProjectIdentity and typed scope;
- Workpoint next-action authority;
- Trajectory goal/gap authority;
- temporal authority, deadlines, urgency, or forecast-time semantics;
- prediction/outcome/calibration or metacognitive promotion authority;
- agent role, stable system prompt, AGENTS.md compilation, or skill generation;
- general-purpose distributed consensus for arbitrary Focusa cognition;
- a cloud-only dependency;
- raw surveillance, keystroke capture, screen recording, or emotional monitoring;
- automatic abandonment of remote work because a peer is unreachable;
- using Git, license keys, or one daemon as universal identity authority;
- sharing one SQLite file between machines.

---

## 3. Cross-spec ownership and precedence

### 3.1 Project and host scope

Specs 96, 98, and 104 remain authoritative for `ScopeRef`, `ProjectRootKey`, `WorkstreamKey`, `AttachmentKey`, ProjectIdentity, host scope, scope confidence, and anti-singleton rules.

Spec 139 consumes verified scope and MUST NOT:

- treat `continuity_id` as project identity;
- treat session, actor, workspace, Git branch, daemon, or license as project identity;
- adopt an unverified root because another peer claimed it;
- recreate daemon-global `current_project`, `current_workpoint`, or `current_actor` authority.

### 3.2 Agent identity and role

Spec 72 owns `AgentIdentity`, `ActorInstance`, `RoleProfile`, `CapabilityProfile`, `PermissionProfile`, `Responsibility`, `HandoffBoundary`, and `SessionContinuity` as ontology concepts.

Spec 139 provides runtime identity bindings and presence states for those objects. It MUST NOT let a presence record grant role, capability, permission, or responsibility by implication.

### 3.3 Sessions and execution

Spec 40 owns Instance, Session, and Attachment lifecycle semantics. Spec 133 owns daemon-native governed sessions, runner adoption, durable checkpoints, and autonomous-session recovery.

Spec 139 extends them with environment coordinates, live presence, topology, placement, claims, leases, and resource occupancy.

### 3.4 Time

Spec 137 owns trusted clocks, clock domains, calendar intent, deadlines, urgency, estimates, lease-expiry time semantics, and temporal incidents.

Spec 139 owns which actor or daemon holds a lease and which resource it covers. Every lease references Spec 137 temporal authority for issuance, renewal, expiry, uncertainty, and breach determination.

### 3.5 Prediction and learning

Spec 138 owns prediction commitments, information sets, outcomes, scoring, calibration, metacognitive signals, learning applicability, transfer, drift, and promotion.

Spec 139 contributes environment, presence, contention, and topology references. It does not independently promote lessons or mutate predictive policy.

### 3.6 Proof and settlement

Spec 119 owns the canonical Receipt ledger. Spec 136 owns governed proposal-to-settlement truth when activated. Spec 139 emits evidence and Receipt requirements for presence, lease, execution-placement, coordination, partition-sensitive actions, and handoffs.

### 3.7 Stable agent constitution

Spec 140 owns the Project Agent Runtime Constitution, system-prompt compilation, AGENTS/rules/skill compilation, instruction delivery, and stable agent operating doctrine.

Spec 139 supplies dynamic environment and operational truth. Spec 140 may instruct agents to consult Spec 139, but static prompt text cannot replace Spec 139 runtime enforcement.

---

## 4. Constitutional directives

### 4.1 Presence primacy

Immediately below verified operator steering and explicit scope/authority, fresh presence and topology awareness MUST constrain every:

- plan;
- task selection;
- action decision;
- tool decision;
- mutation;
- command spawn;
- build/test/release/deploy request;
- checkpoint;
- handoff;
- prediction;
- completion claim;
- autonomous continuation;
- response that recommends consequential work.

### 4.2 Environment primacy

Every actor MUST know its resolved environment coordinate or carry an explicit unavailable/ambiguous posture. No actor may infer that its current machine, daemon, cwd, clone, worktree, branch, or toolchain is correct merely because it is locally accessible.

### 4.3 Placement primacy

Every costly, consequential, shared, or externally effective operation MUST be admitted against a Project Execution Policy before process spawn or remote dispatch.

### 4.4 Failure-to-observe law

> **Failure to observe presence is not evidence of absence.**

A stale, unreachable, or partitioned actor remains a coordination risk until an explicit detach, verified termination, valid expiry, operator revocation, or governed recovery path resolves it.

### 4.5 No singleton operational authority

No global `current_daemon`, `current_actor`, `current_session`, `current_worktree`, `current_build`, `current_release`, or `current_lease` may become canonical authority. All records are typed and scoped.

---

## 5. Core laws

1. **Presence is operational truth, not mutation authority.**
2. **Detection is not authority.** A discovered environment or process does not grant permission.
3. **One identifier is never enough.** Environment resolution uses a typed coordinate and signal quorum.
4. **Scope precedes actor and daemon.** Project/host scope is resolved before workstream, session, actor, or peer matching.
5. **A license realm is not a project.** License/account identity may define a coordination realm, never project scope.
6. **Git identity is not live presence.** Repository and worktree facts do not prove actor liveness or ownership.
7. **Branch is mutable state, not environment identity.**
8. **Every daemon has stable installation identity; every boot has unique process identity.**
9. **Machine ID and daemon ID are distinct.** One machine may run multiple daemons.
10. **Two daemon processes must never write the same Focusa data directory.**
11. **Copied daemon identity must be detected and rekeyed or explicitly adopted.**
12. **Each daemon owns local operational observations only.**
13. **Remote facts retain origin identity and freshness.**
14. **Presence expires; history does not disappear.** Expiry changes current posture but preserves the record.
15. **Corrections append.** Presence, environment, lease, claim, and topology corrections never rewrite history silently.
16. **Unknown remains unknown.** Missing probes or peers do not become neutral or safe defaults.
17. **Partition is a first-class state.** It is not equivalent to offline, detached, or terminated.
18. **An expired heartbeat is not an abandonment declaration.**
19. **Every consequential actor declares intent and expected resource footprint when feasible.**
20. **Shared or exclusive resources require typed claims or leases as policy specifies.**
21. **Exclusive leases use fencing tokens.** Stale holders cannot resume authority.
22. **Equivalent expensive operations deduplicate.** Later agents subscribe, reuse, wait, or intentionally supersede; they do not duplicate by default.
23. **Execution placement fails closed.** An unavailable authorized venue does not silently authorize a local substitute.
24. **Operation classification includes transitive effects.** Wrapper scripts cannot hide compilation, deployment, migration, or publication.
25. **Resource admission occurs before process spawn.**
26. **Hard conflicts interrupt execution.**
27. **Soft overlap informs planning but does not fabricate ownership.**
28. **Independent work may continue during partitions only when it cannot affect serialized shared resources.**
29. **Completion reconciles relevant active work, handoffs, and remote effects.**
30. **Presence packets are bounded projections, not raw event dumps.**
31. **Dynamic facts do not rewrite stable system prompts.**
32. **Every cross-daemon event is signed, scoped, versioned, replay-protected, and causally ordered.**
33. **Each daemon persists locally.** No multi-machine shared SQLite file is canonical.
34. **Cloud relay transports events; it does not become a parallel cognition authority.**
35. **Operator steering wins.** Operator authority may revoke, reassign, pause, or resolve coordination subject to safety and proof requirements.
36. **No surveillance.** Presence derives from declared sessions, daemon/runtime facts, operation events, and bounded resource observations—not hidden human monitoring.
37. **Time and presence cross-reference without duplicating ownership.**
38. **Predictions and learning preserve the environment/topology in which they arose.**
39. **Prompt guidance never substitutes for enforcement.**
40. **Absence of implementation cannot be reported as degraded-but-complete.**

---

## 6. The Focusa Operational Reality Field

Spec 139 defines the **Operational Reality Field**:

> A continuously materialized, event-backed, scope-typed graph of environments, actors, daemons, operations, resources, claims, leases, dependencies, conflicts, and freshness.

It has four planes:

```text
Environment Plane
  Where and how is this actor executing?

Presence Plane
  Which actors, sessions, daemons, and runs are active, stale, partitioned, or unknown?

Coordination Plane
  What are they doing and which resources, dependencies, claims, leases, and handoffs exist?

Admission Plane
  Is this proposed operation authorized here, now, by this actor, under this topology?
```

The field is not:

- a second Focus State;
- a shared transcript;
- a chat roster;
- a universal lock service for all cognition;
- a cloud-owned mission database;
- one global singleton object.

Materialized views are always keyed by typed scope and coordinate.

---

## 7. Environment coordinate and relationship model

### 7.1 Environment coordinate

```rust
pub struct EnvironmentCoordinate {
    pub coordination_realm_id: Option<String>,
    pub scope: Option<ScopeRef>,
    pub repository_id: Option<String>,
    pub repository_lineage_id: Option<String>,
    pub workspace_id: Option<String>,
    pub worktree_id: Option<String>,
    pub node_id: Option<String>,
    pub daemon_id: Option<String>,
    pub daemon_boot_id: Option<String>,
    pub agent_identity_id: Option<String>,
    pub actor_instance_id: String,
    pub surface_id: String,
    pub session_id: String,
    pub continuity_id: Option<String>,
    pub workpoint_id: Option<String>,
    pub execution_run_id: Option<String>,
}
```

Unknown components remain `None` with explicit missing-field reasons. A browser-connected actor may be project-aware and repository-aware while having no local node, daemon, workspace, or worktree.

### 7.2 Identity objects

#### CoordinationRealm

A pseudonymous trust/coordination boundary for paired nodes and daemons. It may derive from a Focusa.work account, license subject, self-hosted realm, or operator-created realm.

It MUST NOT contain raw license keys, customer email addresses, or secrets in replicated events.

#### NodeIdentity

Stable physical or virtual machine identity.

```rust
pub struct NodeIdentity {
    pub node_id: String,
    pub label: String,
    pub public_key: String,
    pub public_key_fingerprint: String,
    pub machine_kind: String,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}
```

#### DaemonIdentity

Stable Focusa daemon installation identity.

```rust
pub struct DaemonIdentity {
    pub daemon_id: String,
    pub node_id: String,
    pub installation_fingerprint: String,
    pub data_dir_id: String,
    pub public_key: String,
    pub supported_protocol_versions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub status: DaemonIdentityStatus,
}
```

#### DaemonBootIdentity

Unique identity for one daemon process lifetime.

```rust
pub struct DaemonBootIdentity {
    pub daemon_boot_id: String,
    pub daemon_id: String,
    pub process_id: u32,
    pub started_at: DateTime<Utc>,
    pub binary_version: String,
    pub config_hash: String,
    pub api_bind: String,
}
```

#### RepositoryIdentity

Portable repository and lineage identity derived from committed marker, normalized remote, repository format, and approved lineage facts.

#### WorkspaceIdentity

One clone, mounted checkout, or remote repository representation on one node.

#### WorktreeIdentity

One Git main or linked worktree, distinguished using `git rev-parse --git-dir`, `--git-common-dir`, worktree metadata, and a Focusa-local stable ID.

### 7.3 Relationship vocabulary

```text
same_coordination_realm
same_scope
same_project
same_repository
same_repository_lineage
same_workspace
same_git_common_directory
same_worktree
same_branch
same_head_commit
same_node
same_daemon
same_daemon_boot
same_agent_identity
same_actor_instance
same_session
same_workstream
same_workpoint
same_execution_run
same_resource_footprint
same_integration_lane
```

Relationships are computed facts with evidence and freshness, not booleans copied from callers.

---

## 8. Environment detection and resolution

### 8.1 Detection pipeline

```text
probe registration
→ bounded probe execution
→ observation normalization
→ sensitivity classification
→ evidence attachment
→ signal weighting
→ mismatch detection
→ candidate profile matching
→ environment resolution
→ immutable snapshot
→ bounded agent projection
```

### 8.2 Signal precedence

Default precedence, strongest first:

1. Explicit signed environment/profile binding.
2. Verified project/host scope marker.
3. Committed `.focusa/project.json` identity.
4. Previously verified ProjectIdentity exact match.
5. Normalized Git remote and repository lineage.
6. Git common directory and worktree metadata.
7. Focusa workspace/worktree registry.
8. Package, deployment, and service markers.
9. Node and daemon cryptographic identity.
10. Process/runtime observations.
11. Cwd and shell environment.
12. Operator declaration.
13. License/account coordination realm.

No single lower-priority signal may override a contradiction from a stronger verified signal.

### 8.3 SystemEnvironmentProfile

Required observations include, where available:

```text
operating_system
operating_system_version
kernel
kernel_version
cpu_architecture
cpu_vendor
cpu_features
logical_cpu_count
physical_cpu_count
memory_total
memory_available
gpu_or_accelerator
libc_or_runtime_abi
virtualization_kind
container_kind
host_or_guest
```

### 8.4 FilesystemProfile

```text
filesystem_type
case_sensitive
supports_file_locks
supports_atomic_rename
mount_identity
mount_kind
network_filesystem
free_space
path_separator
symlink_behavior
repository_mount_boundary
```

### 8.5 RuntimeProfile

```text
os_user
uid_or_sid
groups
home
shell
cwd
parent_process
process_id
service_manager
interactive_or_daemonized
container_id
terminal_kind
ssh_context
remote_origin
```

### 8.6 ToolchainProfile

```text
git_version
rustc_version
cargo_version
node_version
pnpm_version
bun_version
python_version
container_runtime
database_clients
focusa_version
api_schema_version
pi_version
harness_version
model_provider
model_name
thinking_level
```

### 8.7 NetworkAndServiceProfile

```text
network_zone
tailscale_identity
dns_context
proxy_context
reachable_daemons
local_services
service_versions
bound_ports
external_api_reachability
offline_status
```

Private addresses and sensitive endpoints remain local or redacted unless policy explicitly allows replication.

### 8.8 Environment-variable observations

```rust
pub struct EnvironmentVariableObservation {
    pub name: String,
    pub source_kind: VariableSourceKind,
    pub source_ref: Option<String>,
    pub classification: VariableClassification,
    pub present: bool,
    pub effective: bool,
    pub required: bool,
    pub representation: VariableRepresentation,
    pub value_hash: Option<String>,
    pub secret_ref: Option<String>,
    pub precedence_rank: u32,
    pub shadowed_by_ref: Option<String>,
    pub observed_at: DateTime<Utc>,
}
```

Source kinds:

```text
process
service_manager
shell
dotenv
container
workspace_profile
daemon_profile
operator_profile
secret_store
```

Classifications:

```text
public
internal
sensitive
secret
credential
token
private_endpoint
```

Replicated snapshots may carry name, presence, source, effective precedence, classification, and hash. They MUST NOT carry raw secret values.

### 8.9 EnvironmentSnapshot

```yaml
schema: focusa.environment_snapshot.v1
snapshot_id:
coordinate:
profile_refs: []
observed_at:
valid_until:
detector_version:
system:
runtime:
filesystem:
network:
clock:
repository:
workspace:
worktree:
daemon:
toolchain:
variables:
capabilities:
access:
deployment:
fingerprint:
confidence:
missing_fields: []
mismatches: []
evidence_refs: []
```

### 8.10 EnvironmentResolution

```yaml
schema: focusa.environment_resolution.v1
resolution_id:
snapshot_ref:
candidate_profile_refs: []
selected_profile_ref:
status: verified | ambiguous | mismatched | unknown | blocked
confidence:
matched_signals: []
mismatches: []
operator_confirmation_required:
exact_next_action:
```

---

## 9. Environment profiles

An `EnvironmentProfile` declares expected and permitted environment posture. It never overrides observed security or architecture facts.

```yaml
schema: focusa.environment_profile.v1
profile_id:
profile_kind:
name:
revision:
selectors: []
parent_profile_refs: []
expected:
  node:
  system:
  repository:
  workspace:
  worktree:
  daemon:
  toolchain:
  variables:
  capabilities:
policy:
  mutation_mode:
  allowed_branches: []
  forbidden_branches: []
  integration_lane:
  deployment_authority:
  required_daemon_role:
  required_lease_kinds: []
  offline_permissions: []
secret_refs: []
created_by:
approved_by:
status:
```

Recommended inheritance:

```text
CoordinationRealmProfile
→ ProjectEnvironmentPolicy
→ NodeProfile
→ DaemonProfile
→ WorkspaceProfile
→ WorktreeProfile
→ AgentSurfaceProfile
→ SessionExecutionProfile
```

A profile mismatch creates `ProfileDrift`; it does not coerce the snapshot.

---

## 10. Presence model

### 10.1 PresenceRecord

```rust
pub struct PresenceRecord {
    pub presence_id: String,
    pub coordinate: EnvironmentCoordinate,
    pub state: PresenceState,
    pub truth_class: PresenceTruthClass,
    pub authority_class: PresenceAuthorityClass,
    pub heartbeat_ref: Option<String>,
    pub work_intent_ref: Option<String>,
    pub activity_ref: Option<String>,
    pub resource_footprint_ref: Option<String>,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub origin_daemon_id: String,
    pub evidence_refs: Vec<String>,
}
```

### 10.2 PresenceState

```rust
pub enum PresenceState {
    Attaching,
    Orienting,
    Active,
    Executing,
    Waiting,
    Blocked,
    Paused,
    Idle,
    Completing,
    HandingOff,
    Detached,
    Stale,
    Unreachable,
    Partitioned,
    Orphaned,
    Recovered,
}
```

### 10.3 Truth classes

```text
canonical_reference
operational
observed
declared
inferred
stale
unknown
```

### 10.4 Authority classes

```text
none
advisory
claim
shared_lease
exclusive_lease
integration_authority
operator_authorized
```

Truth and authority are separate fields.

### 10.5 Freshness states

```text
fresh
aging
stale
unreachable
partitioned
expired
explicitly_detached
confirmed_terminated
```

### 10.6 Presence heartbeat

Heartbeats identify actor, session, daemon boot, topology epoch, local monotonic sequence, observed wall time, current state, and bounded activity/resource digests.

A heartbeat MUST NOT include raw prompt content, keystrokes, screen content, or secret values.

---

## 11. Presence Pulse

The daemon-owned `PresencePulse` is event-driven plus periodic.

```text
PresencePulse
├── refresh local actor/session liveness
├── inspect governed runners and silent sessions
├── reconcile peer advertisements
├── update sync lag and partition state
├── expire presence projections
├── detect orphaned claims and activities
├── renew or expire leases using Spec 137 time
├── recompute resource occupancy
├── recompute conflict/dependency indexes
├── invalidate affected execution guards
└── materialize bounded agent packets
```

Immediate recomputation triggers include:

- actor attach/detach;
- session start/end/restore;
- work intent revision;
- branch, HEAD, worktree, or dirty-state change;
- claim/lease acquisition or loss;
- footprint change;
- daemon peer appearance/disappearance;
- sync lag threshold crossing;
- operator steering;
- Workpoint revision;
- command/process spawn;
- checkpoint or handoff;
- profile or execution-policy revision.

---

## 12. Work intent, activity, and resource footprint

### 12.1 WorkIntent

```yaml
schema: focusa.work_intent.v1
intent_id:
actor_instance_id:
scope_ref:
continuity_id:
workpoint_id:
mission:
operation_classes: []
expected_resource_footprint_ref:
expected_artifact_refs: []
dependencies: []
started_at:
valid_until:
revision:
status: declared | revised | paused | completed | abandoned | superseded
```

### 12.2 ActivityRecord

Tracks current operational activity without claiming task completion.

```text
planned
preparing
executing
waiting_external
blocked
verifying
handing_off
completed_claimed
abandoned
```

### 12.3 ResourceFootprint

A footprint may include:

```text
files
paths
symbols
API contracts
database schemas
Git refs
worktrees
build targets
test suites
services
ports
external accounts
release tags
deployment environments
cloud resources
browser sessions
CPU/memory/disk budgets
```

Footprints may be exact, pattern-based, semantic, or unknown. Unknown footprints increase conflict uncertainty.

### 12.4 Resource occupancy

Occupancy is a materialized operational view from active intents, processes, claims, leases, and observations. Occupancy itself does not grant ownership.

---

## 13. Claims, leases, and fencing

### 13.1 ActivityClaim

Claims communicate intended ownership or coordination preference.

```yaml
schema: focusa.activity_claim.v1
claim_id:
claim_kind: advisory | shared | exclusive_candidate
resource_refs: []
holder_actor_ref:
holder_daemon_ref:
scope_ref:
issued_at:
expires_at:
status:
evidence_refs: []
```

### 13.2 ResourceLease

Leases are authority-bearing only when issued by the registered lease authority for the scoped resource.

```yaml
schema: focusa.resource_lease.v1
lease_id:
lease_kind:
resource_ref:
scope_ref:
authority_daemon_id:
authority_epoch:
holder_actor_id:
holder_daemon_id:
mode: shared | exclusive
issued_at:
renewed_at:
expires_at:
fencing_token:
status:
temporal_authority_ref:
evidence_refs: []
```

### 13.3 Fencing

Every exclusive or serialized resource mutation MUST submit the current fencing token. A higher token invalidates all lower holders even if their local process still believes the lease is active.

### 13.4 Standard lease kinds

```text
worktree_writer
branch_integration
main_branch
rust_build_pipeline
test_pipeline
release_pipeline
deployment
database_migration
schema_mutation
production_configuration
external_account
artifact_publication
```

### 13.5 Lease authority

Authority may be assigned per resource or operation class. A daemon assigned `main_branch_integration_authority` is not global cognition authority.

---

## 14. Conflicts and dependencies

### 14.1 Conflict classes

#### Hard conflict

- same physical worktree with multiple writers;
- same exclusive lease;
- stale fencing token;
- incompatible mutation of the same Workpoint/canonical target;
- duplicate release, deploy, migration, or publication;
- same branch integration lane without authority;
- same external account mutation.

Hard conflicts block or interrupt.

#### Structural overlap

- overlapping files;
- overlapping symbols;
- overlapping API/schema contracts;
- shared generated artifacts;
- shared dependencies.

Structural overlap normally requires coordination or partitioning.

#### Semantic overlap

- distinct files implementing conflicting architecture;
- parallel specs defining the same primitive;
- contradictory workflow changes;
- competing solutions to the same requirement.

Semantic overlap is advisory until verified but must be surfaced for high-impact work.

#### Complementary work

Work is related but mutually supportive. The system should recommend dependency links or handoffs rather than mark a conflict.

### 14.2 DependencyRelationship

```text
requires
blocks
produces_input_for
validates
reviews
integrates
supersedes
conflicts_with
complements
```

### 14.3 Handoff

```yaml
schema: focusa.presence_handoff.v1
handoff_id:
from_actor_ref:
to_actor_or_role_ref:
scope_ref:
workpoint_ref:
resource_refs: []
state_refs: []
required_acceptance: []
offered_at:
accepted_at:
status: offered | accepted | rejected | expired | superseded
evidence_refs: []
receipt_ref:
```

---

## 15. Multi-daemon topology

### 15.1 Required topology

```text
Agent/Harness
  → local Focusa daemon
      → local SQLite event ledger and snapshots
      → local PresencePulse and admission
      → transport adapter
          → paired direct daemon
          → Focusa.work encrypted relay
          → on-prem relay
          → offline queue
      → other Focusa daemons
```

### 15.2 Local truth ownership

Each daemon is authoritative for observations it can directly verify, including:

- local process and session liveness;
- local worktree state;
- local runner heartbeat;
- local operation spawn/exit;
- local resource pressure;
- locally issued actor attachment.

It is not automatically authoritative for:

- project-wide completion;
- another daemon's actor state;
- shared branch/release ownership;
- remote abandonment;
- settlement of external effects.

### 15.3 Data directory exclusivity

A daemon MUST acquire a local OS-level data-directory lock before opening writable persistence. A second process MUST refuse startup with a precise recovery envelope.

### 15.4 Identity collision

If the same `daemon_id` appears from two nodes or incompatible installation fingerprints:

```text
DaemonIdentityCollision
→ quarantine canonical-capable publication
→ preserve read-only diagnostics
→ require rekey or explicit operator adoption
→ emit security incident and Receipt
```

### 15.5 Peer authentication

The identity chain is:

```text
CoordinationRealm
→ Node public key
→ Daemon public key
→ DaemonBootIdentity
→ Actor attachment
→ signed events
```

Initial implementation may use Focusa-native asymmetric keys. Long-term identity SHOULD remain compatible with SPIFFE-style trust-domain/workload identity semantics without requiring SPIRE as a runtime dependency.

### 15.6 Signed event requirements

Cross-daemon events MUST include:

```text
event_id
schema_version
realm_id
scope_ref
repository/workspace/worktree refs when applicable
node_id
daemon_id
daemon_boot_id
actor_instance_id
session_id
execution_run_id
continuity_id
workpoint_id
vector_clock
Lamport timestamp
topology_epoch
origin sequence
nonce or replay window
payload hash
signature
```

### 15.7 Replication

- events are append-only and idempotent;
- original origin identity is preserved through relays;
- cursors are peer- and stream-scoped;
- compatible updates converge;
- conflicts remain explicit;
- foreign/unverified scope is quarantined;
- relay availability does not alter canonical ownership.

### 15.8 No shared SQLite

Each daemon has independent local persistence. SQLite WAL may support local concurrency, but the same database file MUST NOT be treated as multi-machine consensus or placed on a shared network filesystem for coordinated writes.

---

## 16. Topology snapshot

```yaml
schema: focusa.topology_snapshot.v1
topology_snapshot_id:
realm_id:
scope_ref:
epoch:
materialized_at:
nodes: []
daemons: []
actors: []
sessions: []
execution_runs: []
edges: []
peer_sync_heads: []
partitions: []
claims: []
leases: []
conflicts: []
dependencies: []
stale_presence: []
unknowns: []
evidence_refs: []
```

Topology epochs increase whenever a material coordination fact changes. A guard references one epoch and is invalidated when a relevant change occurs.

---

## 17. Execution placement

### 17.1 ProjectExecutionPolicy

```yaml
schema: focusa.project_execution_policy.v1
policy_id:
project_ref:
revision:
operation_routes:
  <operation_class>:
    execution_mode: local_only | remote_only | preferred_remote | any_verified | forbidden
    required_profiles: []
    allowed_profiles: []
    authority_daemon_ref:
    executor_role:
    fallback: block | ask_operator | alternate_verified_route
    singleton_scope:
    required_lease_kind:
    required_capabilities: []
    required_permissions: []
    validation_route_ref:
approved_by:
approved_at:
status:
```

### 17.2 OperationClass

Minimum registry:

```text
source_read
source_edit
static_noncompiling_check
format
rust_compile
rust_test
rust_clippy
rust_bench
frontend_build
package
cross_compile
artifact_sign
release_publish
deploy
database_migration
schema_mutation
large_index
browser_fleet
model_inference
external_account_mutation
```

### 17.3 Transitive effects

Every script, package command, Make target, task-runner command, and skill declares transitive operation classes. `bun run check` or a shell script that invokes Cargo cannot evade placement by using a wrapper name.

### 17.4 ExecutionAdmission

```yaml
schema: focusa.execution_admission.v1
admission_id:
operation_intent_ref:
actor_ref:
environment_resolution_ref:
project_execution_policy_ref:
presence_guard_ref:
temporal_guard_ref:
resource_snapshot_ref:
deduplication_key:
verdict: allow | redirect | delegate | subscribe | reuse | wait | block | ask_operator
required_venue_ref:
selected_executor_ref:
blocking_reasons: []
exact_next_action:
valid_until:
fencing_token:
```

### 17.5 Admission path

```text
proposed tool/command/action
→ operation classification
→ scope verification
→ environment/profile resolution
→ presence/topology refresh
→ execution-route resolution
→ equivalent-run lookup
→ lease/fencing check
→ resource admission
→ allow/redirect/delegate/subscribe/reuse/wait/block
```

---

## 18. Distributed expensive-operation deduplication

### 18.1 Deduplication key

```text
ExecutionDeduplicationKey =
  scope
+ source_commit
+ dirty_patch_hash
+ operation_class
+ target_matrix
+ feature_set
+ toolchain_profile
+ input_artifact_hashes
```

### 18.2 Required behavior

If an equivalent operation is:

- **running:** subscribe or wait;
- **successful and fresh:** reuse its Receipt;
- **failed:** inspect failure before retry;
- **stale:** request a new admitted run;
- **incompatible:** create a distinct key;
- **superseded:** cancel or mark obsolete through governed action.

### 18.3 ExecutionSubscription

Subscribers receive progress, logs through bounded handles, result, Evidence, and Receipt references without starting another process.

### 18.4 Resource reservation

Expensive operations declare expected CPU, memory, disk, process count, network, and external quota. Resource admission may queue or block even on the correct venue.

---

## 19. Presence and execution guards

### 19.1 PresenceExecutionGuard

```rust
pub struct PresenceExecutionGuard {
    pub guard_id: String,
    pub actor_instance_id: String,
    pub environment_coordinate_hash: String,
    pub topology_epoch: u64,
    pub generated_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub status: PresenceGuardStatus,
    pub related_actor_count: u32,
    pub hard_conflict_count: u32,
    pub soft_conflict_count: u32,
    pub dependency_count: u32,
    pub required_lease_refs: Vec<String>,
    pub held_lease_refs: Vec<String>,
    pub blocking_lease_refs: Vec<String>,
    pub partition_posture: String,
    pub synchronization_posture: String,
    pub exact_constraint: String,
    pub guard_hash: String,
}
```

Statuses:

```text
clear
clear_with_related_work
coordination_recommended
lease_required
blocked_by_conflict
blocked_by_partition
environment_stale
topology_unknown
daemon_unavailable
operator_resolution_required
```

### 19.2 EnvironmentExecutionGuard

Binds an action to resolved node, daemon, workspace, worktree, profile, toolchain, and policy revision.

### 19.3 ExecutionPlacementGuard

Binds an operation class to authorized venue, executor, deduplication key, lease, and fallback posture.

### 19.4 Guard validity

A guard is invalidated by relevant changes to:

- actor or session;
- scope or Workpoint;
- node, daemon, workspace, worktree, branch, or HEAD;
- environment/profile resolution;
- topology epoch;
- claim, lease, fencing token, or conflict;
- execution policy;
- temporal expiry.

---

## 20. Presence awareness packet

```yaml
schema: focusa.presence_awareness.v1
packet_id:
generated_at:
valid_until:
topology_epoch:
temporal_guard_ref:
current_actor:
current_environment:
related_presence:
  same_worktree: []
  same_branch: []
  same_workpoint: []
  same_resource_footprint: []
  same_project_other_workstreams: []
  relevant_remote_daemons: []
coordination:
  active_claims: []
  held_leases: []
  blocking_leases: []
  dependencies: []
  handoffs: []
  opportunities: []
conflicts:
  hard: []
  structural: []
  semantic: []
topology:
  peer_count:
  reachable_peers: []
  partitioned_peers: []
  sync_lag:
  stale_presence: []
  orphaned_activity: []
execution:
  current_policy_ref:
  allowed_operation_classes: []
  blocked_operation_classes: []
  active_equivalent_runs: []
constraints:
  must_not: []
  must_coordinate: []
  safe_independent_work: []
  exact_safe_next:
freshness:
  environment_snapshot_age:
  local_presence_age:
  remote_presence_age:
  stale:
  unknowns: []
guards:
  presence_guard_ref:
  environment_guard_ref:
  placement_guard_refs: []
```

---

## 21. Awareness saturation and context economy

Presence must saturate decisions without saturating prompt tokens.

### Level 1 — Guard

A compact signed summary referenced at every consequential boundary.

### Level 2 — Delta

Injected only when a relevant fact changes.

```text
PRESENCE_DELTA:
- KH daemon changed from reachable to partitioned.
- Main integration lease remains valid until its verified expiry.
- Local isolated editing remains allowed.
- Main integration and release are blocked.
```

### Level 3 — Standard packet

Used at startup, planning, checkpoint, compaction, task selection, and handoff.

### Level 4 — Rich topology

Fetched on demand for detailed coordination and conflict resolution.

Dynamic topology MUST remain outside stable system-prompt prefixes. Stable prompts state the obligation to consult guards; current facts arrive through bounded runtime context.

---

## 22. Required saturation boundaries

A fresh guard or explicit unavailable posture is required:

### Session

- bootstrap;
- attach;
- restore;
- daemon reconnect;
- post-compaction;
- model switch;
- workspace/worktree/branch switch.

### Reasoning

- plan creation;
- task selection;
- direction change;
- estimate creation;
- prediction commitment;
- learning application;
- blocker declaration.

### Execution

- broad repository read when overlap matters;
- file write;
- shared contract edit;
- command spawn;
- build/test/package;
- Git mutation;
- commit/push/merge;
- release/deploy;
- database/external-account mutation.

### Continuity and closure

- checkpoint;
- handoff;
- autonomous continuation;
- completion claim;
- Workpoint settlement;
- task closure.

---

## 23. Hard interrupts

The agent MUST be interrupted when:

- another writer enters the same worktree;
- an exclusive lease is lost;
- a fencing token is stale;
- branch/release/deployment authority changes;
- remote main HEAD changes before integration;
- another actor claims the same exclusive Workpoint item;
- daemon identity collision appears;
- a partition makes serialized shared state unknown;
- operator steering invalidates current work;
- resource footprint begins hard overlap;
- the selected venue becomes unauthorized;
- an equivalent expensive operation begins elsewhere.

Interrupts include exact facts, authority, and safe recovery—not generic warnings.

---

## 24. Command and process enforcement

### 24.1 Tool-level interception

Pi and other adapters classify tool calls before execution and obtain `ExecutionAdmission`.

### 24.2 Daemon admission

The daemon enforces scope, profile, placement, claims, leases, fencing, deduplication, and resource policy.

### 24.3 Shell wrapper

Focusa-managed sessions SHOULD execute through an admission-aware shell bridge for registered projects.

### 24.4 Process supervisor

For high-value policies, a node supervisor MAY detect forbidden processes rooted in a Focusa workspace and suspend or terminate them, preserving evidence and an incident. This is a backstop, not the primary control.

### 24.5 Absolute-path bypass

Enforcement cannot rely solely on PATH shims because agents may invoke absolute binaries. Process and daemon policy must use resolved executable, cwd/workspace, ancestry, and operation classification.

---

## 25. Focusa Rust build/release incident policy

A valid Focusa project policy may declare:

```yaml
rust_compile:
  execution_mode: remote_only
  required_profiles: [focusa-remote-build]
  fallback: block
  singleton_scope: project_commit_target_matrix
  required_lease_kind: rust_build_pipeline

rust_test:
  execution_mode: remote_only
  required_profiles: [focusa-remote-build]
  fallback: block

release_publish:
  execution_mode: remote_only
  required_profiles: [focusa-release]
  executor_role: release_agent
  required_lease_kind: release_pipeline
  fallback: block
```

A Mac implementation agent attempting `cargo test --workspace` receives:

```json
{
  "verdict": "subscribe",
  "operation_class": "rust_test",
  "current_profile": "focusa-mac-worktree",
  "required_profile": "focusa-remote-build",
  "active_equivalent_run": "buildrun_93",
  "reason": "Rust validation is remote-only and an equivalent run is active.",
  "exact_next_action": "Observe buildrun_93 or continue independent source work."
}
```

If the remote daemon is partitioned, local substitution remains blocked unless an approved policy revision or explicit operator-authorized alternate route is committed.

---

## 26. Event model

Minimum event families:

```text
environment.snapshot_observed
environment.resolved
environment.profile_bound
environment.profile_drifted
node.enrolled
node.revoked
daemon.registered
daemon.booted
daemon.heartbeat
daemon.partitioned
daemon.recovered
daemon.identity_collision
actor.attached
actor.detached
presence.heartbeat
presence.expired
presence.recovered
intent.declared
intent.revised
activity.started
activity.progressed
activity.blocked
activity.completed_claimed
resource.footprint_declared
resource.occupancy_changed
claim.acquired
claim.released
lease.acquired
lease.renewed
lease.expired
lease.revoked
fencing.rejected
handoff.offered
handoff.accepted
conflict.detected
conflict.resolved
operation.admission_requested
operation.admission_decided
operation.started
operation.subscribed
operation.reused
operation.completed
operation.failed
placement.violation
resource.contention_incident
```

Events are append-only, idempotent, typed, versioned, signed across daemons, and linked to Evidence/Receipts when consequential.

---

## 27. Persistence and projections

Required tables or equivalent stores:

```text
node_identities
daemon_identities
daemon_boots
environment_profiles
environment_snapshots
environment_resolutions
actor_attachments
presence_records
presence_heartbeats
work_intents
activity_records
resource_footprints
resource_occupancy
claims
leases
handoffs
conflicts
dependencies
topology_snapshots
peer_cursors
execution_policies
operation_intents
execution_admissions
execution_runs
execution_subscriptions
presence_incidents
```

Large logs and artifacts remain in ECS or external artifact storage with handles.

Materialized current projections are disposable and rebuildable from events plus signed peer state.

---

## 28. API surface

### Environment

```text
POST /v1/environment/detect
POST /v1/environment/resolve
GET  /v1/environment/current
GET  /v1/environment/snapshots/:id
GET  /v1/environment/diff
GET  /v1/environment/orientation
```

### Profiles

```text
GET  /v1/environment/profiles
GET  /v1/environment/profiles/:id
POST /v1/environment/profiles/preview
POST /v1/environment/profiles/commit
POST /v1/environment/profiles/bind
POST /v1/environment/profiles/verify
```

### Presence and topology

```text
POST /v1/presence/attach
POST /v1/presence/heartbeat
POST /v1/presence/detach
GET  /v1/presence/field
GET  /v1/presence/bootstrap
GET  /v1/presence/stream
GET  /v1/topology
GET  /v1/topology/snapshots/:id
```

### Coordination

```text
POST /v1/intents/declare
POST /v1/intents/revise
POST /v1/claims/acquire
POST /v1/claims/release
POST /v1/leases/acquire
POST /v1/leases/renew
POST /v1/leases/release
POST /v1/handoffs/create
POST /v1/handoffs/accept
GET  /v1/conflicts
POST /v1/conflicts/:id/resolve
```

### Execution

```text
GET  /v1/execution/policy
POST /v1/execution/classify
POST /v1/execution/admit
POST /v1/execution/delegate
POST /v1/execution/runs
GET  /v1/execution/runs/:id
POST /v1/execution/runs/:id/subscribe
POST /v1/execution/runs/:id/cancel
```

### Daemons

```text
GET  /v1/daemons
GET  /v1/daemons/:id
POST /v1/daemons/announce
POST /v1/daemons/heartbeat
POST /v1/daemons/rekey
GET  /v1/daemons/topology
```

Every consequential mutation uses generated typed contracts, idempotency, optimistic concurrency where applicable, and Receipt integration.

---

## 29. CLI surface

```text
focusa environment detect
focusa environment orient
focusa environment diff
focusa environment topology

focusa profile list
focusa profile show
focusa profile bind
focusa profile verify

focusa presence status
focusa presence field
focusa presence explain
focusa presence conflicts

focusa daemon identity
focusa daemon peers
focusa daemon topology
focusa daemon rekey
focusa daemon doctor

focusa intent declare
focusa claim acquire
focusa claim release
focusa lease acquire
focusa lease renew
focusa lease release
focusa handoff offer
focusa handoff accept

focusa execution classify
focusa execution preflight
focusa execution delegate
focusa execution runs
focusa execution subscribe
```

All machine-readable CLI output uses versioned JSON envelopes and exact recovery actions.

---

## 30. Pi and agent tool surface

Minimum Pi tools:

```text
focusa_environment_resolve
focusa_environment_orientation
focusa_presence_guard
focusa_presence_field
focusa_presence_explain
focusa_work_intent_declare
focusa_resource_footprint_declare
focusa_claim_acquire
focusa_claim_release
focusa_lease_status
focusa_handoff
focusa_execution_classify
focusa_execution_preflight
focusa_execution_delegate
focusa_execution_subscribe
focusa_topology_view
```

The Pi extension MUST:

- attach one full `ActorAttachment`, not only `InstanceKind`;
- preserve agent, instance, surface, session, continuity, profile, capability, and permission references;
- refresh compact guards at required boundaries;
- put volatile presence facts in cache-safe dynamic context;
- intercept consequential commands before spawn;
- show hard interrupts;
- preserve guard references across compaction and Workpoint checkpointing.

---

## 31. Generated UI and operator experience

Spec 135I A2UI surfaces SHOULD expose:

### Environment panel

- detected machine, OS, architecture, daemon, workspace, worktree, branch, HEAD, profile, and confidence;
- mismatches and missing facts;
- safe profile binding or confirmation.

### Presence field

- active actors grouped by project/workstream/worktree;
- local versus remote;
- fresh, stale, partitioned, orphaned;
- current intent and bounded activity;
- resource footprints and claims.

### Execution topology

- authorized build/test/integration/release/deploy venues;
- active expensive operations;
- subscribers;
- resource pressure;
- leases and fencing posture;
- sync lag and partitions.

### Conflict workbench

- hard, structural, semantic, complementary;
- evidence and affected resources;
- partition/split/coordinate/handoff/operator-resolution actions.

### Multi-daemon setup

- enroll node;
- pair daemon;
- label profiles;
- assign scoped authority roles;
- inspect identity collisions;
- rekey/revoke;
- configure direct/relay/offline transport.

Generated UI is a projection and action-binding layer, never a second topology store.

---

## 32. Spec 137 integration

Spec 139 records references for:

```text
heartbeat observed time
lease issuance/renewal/expiry
claim expiry
partition duration
sync lag age
orphan age
handoff age
resource-contention duration
operation queue/run duration
```

Cross-machine rules:

- monotonic clocks are local to one daemon boot;
- calendar deadlines use trusted wall time and uncertainty from Spec 137;
- causal order uses vector clocks/Lamport timestamps;
- host sleep and daemon downtime remain explicit;
- estimates widen or refuse when topology or placement is unknown;
- active parallel agents do not automatically double-count elapsed human time or progress.

Estimate provenance SHOULD include environment snapshot, topology snapshot, active actor count, contention, placement route, and sync lag.

---

## 33. Spec 138 integration

Prediction and learning records SHOULD reference:

```text
environment_snapshot_ref
topology_snapshot_ref
presence_snapshot_ref
topology_epoch
active_actor_refs
coordination_mode
contention_ref
partition_posture
resource_occupancy_ref
workspace_ref
worktree_ref
daemon_ref
```

Spec 138 can then evaluate cohorts such as:

- Mac versus remote Linux;
- ARM versus x86;
- browser reasoning versus direct execution;
- isolated worktree versus integration lane;
- one agent versus complementary agents versus conflicting agents;
- connected versus partitioned topology;
- local versus delegated validation.

New metacognitive signal candidates include:

```text
CoordinationFailure
DuplicateWorkFailure
PresenceStalenessFailure
PartitionPlanningFailure
LeaseViolation
HandoffFailure
ResourceContentionFailure
ExecutionPlacementFailure
TopologyAssumptionFailure
MultiAgentPositiveTransfer
MultiAgentNegativeTransfer
```

They remain candidate signals until Spec 138 governance evaluates and promotes learning.

---

## 34. Security, privacy, and threat model

### 34.1 Threats

- forged daemon or actor identity;
- replayed heartbeat or lease event;
- daemon identity copied with data directory;
- malicious peer claiming false work or authority;
- raw secret replication;
- topology metadata leakage;
- stale lease resumption;
- denial of service through claim/lease spam;
- malicious command misclassification;
- policy bypass through wrapper scripts or absolute paths;
- compromised relay altering events;
- cross-project presence bleed.

### 34.2 Required controls

- asymmetric node/daemon keys;
- short-lived peer credentials;
- signed events and payload hashes;
- replay windows/nonces and monotonic origin sequence;
- realm and scope verification;
- revocation lists;
- rate limits and bounded payloads;
- privacy classification and redaction;
- no raw license keys or secret values;
- secure local identity storage;
- data-directory locks;
- fencing tokens;
- least-privilege transport;
- quarantine for unverified peers/events;
- tamper-evident event chain integration;
- audit Receipts for authority changes and violations.

### 34.3 Privacy classes

```text
public_topology
internal_topology
sensitive_topology
secret_reference_only
local_only
```

Actor prompt content, raw shell history, keystrokes, screen content, and unrelated files are not presence telemetry.

---

## 35. Migration

### Phase 0 — Audit and freeze

- inventory current instance/session/machine/peer/environment fields;
- identify singleton operational pointers;
- record contradictory build/release instructions as Spec 140 inputs;
- classify existing sync/events by origin and scope;
- add no-op typed references without claiming implementation.

### Phase 1 — Identity and environment

- implement NodeIdentity, DaemonIdentity, DaemonBootIdentity;
- add data-dir lock and identity collision detection;
- implement repository/workspace/worktree identities;
- implement EnvironmentSnapshot/Resolution/Profile.

### Phase 2 — Local multiplexing

- support multiple sessions/worktrees on one machine/daemon;
- preserve complete ActorAttachment;
- add local PresencePulse, intents, footprints, conflicts, and guards.

### Phase 3 — Multi-daemon topology

- signed daemon advertisements and heartbeats;
- peer cursors, topology epochs, partitions, and recovery;
- multi-machine presence field.

### Phase 4 — Claims, leases, and fencing

- scoped lease authorities;
- worktree, branch, build, release, deploy, and migration leases;
- hard conflict interrupts.

### Phase 5 — Execution placement and deduplication

- Operation Registry transitive effects;
- pre-spawn admission;
- remote delegation/subscription/reuse;
- resource admission and process backstop.

### Phase 6 — Full saturation and UI

- guard integration at every boundary;
- Agent Bootstrap, compaction, Workpoint, prediction, and settlement integration;
- A2UI topology/configuration experience;
- Focusa.work encrypted relay.

---

## 36. Required tests

### Identity and scope

1. Same project on two machines resolves same project/repository lineage and different nodes/workspaces.
2. Two worktrees on one Mac resolve same Git common directory and different worktree IDs.
3. Branch change does not change worktree identity.
4. Browser actor remains project-aware but environment-partial.
5. Wrong project/host coercion blocks.

### Multi-daemon

6. Two daemons on separate data directories run independently.
7. Two daemons targeting one data directory: second refuses startup.
8. Same daemon identity on two nodes creates collision and quarantine.
9. Daemon restart preserves stable daemon ID and creates new boot ID.
10. Signed same-scope events converge causally.
11. Wrong-scope events quarantine.
12. Relay cannot rewrite origin identity.

### Presence

13. Two Pi sessions appear as distinct actors.
14. Explicit detach differs from stale heartbeat.
15. Peer partition does not imply remote absence.
16. Recovered peer preserves prior history and refreshes current state.
17. Orphaned activity is detected without auto-abandonment.
18. Presence packet remains bounded under many actors.

### Coordination

19. Same worktree two writers creates hard conflict.
20. Different worktrees same files create structural overlap.
21. Different files same architecture creates semantic overlap candidate.
22. Complementary work creates dependency/handoff suggestion.
23. Lease fencing rejects stale holder.
24. Handoff acceptance changes coordination state and emits Receipt.

### Execution placement

25. Mac profile blocks Rust compilation before spawn.
26. Wrapper script invoking Cargo is classified as Rust compilation.
27. Absolute Cargo path cannot bypass policy.
28. Authorized remote agent receives delegated operation.
29. Equivalent active build causes subscription, not duplicate process.
30. Fresh successful run is reused with Receipt.
31. Failed run is inspected before retry.
32. Remote partition blocks local fallback when policy says block.
33. Explicit approved alternate route works with new admission/lease.
34. Resource pressure queues or blocks expensive run.

### Awareness saturation

35. Planning uses fresh guard.
36. File mutation invalidates/rechecks relevant topology.
37. Branch switch invalidates environment guard.
38. Lease loss produces hard interrupt.
39. Compaction preserves guard refs and reloads fresh packet.
40. Completion claim checks outstanding related work and handoffs.

### Temporal/prediction

41. Lease expiry uses Spec 137 trusted time and uncertainty.
42. Cross-machine monotonic times are never directly compared.
43. Prediction retains original environment/topology refs.
44. Learning transfer from Mac to Linux records differences.
45. Duplicate-work incident creates Spec 137/138 evidence inputs.

### Privacy/security

46. Secret environment values never replicate.
47. Forged/replayed heartbeat is rejected.
48. Revoked node/daemon cannot publish canonical-capable events.
49. Cross-project presence bleed test remains empty.
50. Bounded payload and rate-limit tests prevent heartbeat/claim spam.

---

## 37. Acceptance criteria

Spec 139 is accepted only when:

1. Every active actor has a typed EnvironmentCoordinate or explicit unresolved posture.
2. Multiple nodes, daemons, workspaces, worktrees, sessions, and actors coexist without singleton authority.
3. Environment detection covers system, architecture, runtime, filesystem, toolchain, variables, network, repository, workspace, worktree, daemon, and profile facts.
4. Unknown and mismatch states fail visibly.
5. PresencePulse operates independently of model memory.
6. Presence is refreshed and applied at every consequential decision boundary.
7. Multi-daemon identity, boot identity, signing, replay protection, topology epochs, partition, and recovery work.
8. Daemons cannot share a writable data directory.
9. Daemon identity collisions are quarantined.
10. Work intents, resource footprints, claims, leases, fencing, dependencies, conflicts, and handoffs are first-class.
11. Hard conflicts interrupt execution.
12. Execution placement is checked before process spawn.
13. Operation classification includes transitive wrapper effects.
14. Equivalent expensive operations deduplicate and support subscription/reuse.
15. Remote-only policy never silently falls back locally.
16. Resource admission accounts for CPU, memory, disk, process, network, and external quotas.
17. Agent packets are bounded and cache-safe.
18. Stable prompts contain obligations, not volatile topology facts.
19. Spec 137 and Spec 138 references are preserved.
20. Evidence and Receipts prove consequential coordination and placement decisions.
21. Generated UI supports configuration, explanation, recovery, and multi-machine topology.
22. Privacy and security requirements pass adversarial tests.
23. The dual-local-Rust-build incident is reproduced and deterministically prevented.
24. No mandatory requirement is represented only as documentation or prompt advice.

---

## 38. Machine-readable requirement ledger

Every normative statement MUST map to `docs/contracts/spec139-complete-feature-ledger.v1.yaml` before implementation closure.

Required row shape:

```yaml
requirement_id:
spec_section:
requirement_text:
requirement_class: must | shall | should | may
applicability: required | conditional | optional | not_applicable
applicability_condition_ref:
primitive_owner:
implementation_slice:
blocking_dependencies: []
core_types: []
reducer_events: []
persistence: []
api_operations: []
cli_commands: []
pi_tools: []
ui_surfaces: []
operation_registry_changes: []
generated_contracts: []
migrations: []
positive_tests: []
negative_tests: []
restart_recovery_tests: []
security_tests: []
performance_tests: []
evidence_refs: []
receipt_refs: []
status: not_started | active | blocked | implemented_unverified | verified | variance_approved | not_applicable_verified
```

A new normative clause without a ledger mapping fails the completeness gate.

---

## 39. Canonical summary

Spec 139 makes Focusa agents continuously situated in distributed reality.

```text
ProjectIdentity tells the agent what project it belongs to.
EnvironmentResolution tells it exactly where and how it is operating.
Presence tells it who and what else is active.
Coordination tells it what resources, claims, leases, dependencies, and conflicts exist.
Execution Placement tells it where work is authorized to run.
Admission prevents unsafe or duplicate execution before process spawn.
```

The resulting system does not depend on an agent remembering that other agents may exist. It makes operational awareness a daemon-owned, event-backed, scope-safe, multi-machine runtime invariant.